mod router;

use crate::core::plugin::{Plugin, PluginMetadata};
use crate::core::plugin_context::PluginContext;
use async_trait::async_trait;
use std::sync::Arc;
use anyhow::Result;

pub struct FilesPlugin;

const BLACKLISTED_EXTENSIONS: &[&str] = &[
    "exe", "com", "scr", "pif", "bat", "cmd", "ps1", "vbs", "vbe", "js", "jse", "jar", "msi",
    "dll", "sys", "drv", "ocx", "cpl", "inf", "reg", "scf", "lnk", "url", "desktop", "app",
    "deb", "rpm", "dmg", "pkg", "apk", "ipa", "bin", "run", "out", "sh", "bash", "zsh",
    "fish", "csh", "tcsh", "py", "pl", "rb", "php", "asp", "aspx", "jsp", "cgi"
];

#[async_trait]
impl Plugin for FilesPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            id: "files".to_string(),
            name: "Files System".to_string(),
            version: "1.0.0".to_string(),
            description: "File operations with security validation".to_string(),
            author: "WebArcade Team".to_string(),
            dependencies: vec![],
        }
    }

    async fn init(&self, ctx: &PluginContext) -> Result<()> {
        log::info!("[Files] Initializing plugin...");

        // Register services
        ctx.provide_service("read_file", |input| async move {
            let file_path: String = serde_json::from_value(input["file_path"].clone())?;
            validate_file_path(&file_path)?;

            let content = std::fs::read_to_string(&file_path)?;
            Ok(serde_json::json!({ "content": content }))
        }).await;

        ctx.provide_service("write_file", |input| async move {
            let file_path: String = serde_json::from_value(input["file_path"].clone())?;
            let content: String = serde_json::from_value(input["content"].clone())?;

            validate_file_path(&file_path)?;
            validate_file_extension(&file_path)?;

            // Ensure parent directory exists
            if let Some(parent) = std::path::Path::new(&file_path).parent() {
                std::fs::create_dir_all(parent)?;
            }

            std::fs::write(&file_path, content)?;
            Ok(serde_json::json!({ "success": true }))
        }).await;

        ctx.provide_service("delete_file", |input| async move {
            let file_path: String = serde_json::from_value(input["file_path"].clone())?;
            validate_file_path(&file_path)?;

            std::fs::remove_file(&file_path)?;
            Ok(serde_json::json!({ "success": true }))
        }).await;

        ctx.provide_service("list_files", |input| async move {
            let dir_path: String = serde_json::from_value(input["dir_path"].clone())?;
            validate_file_path(&dir_path)?;

            let entries: Vec<serde_json::Value> = std::fs::read_dir(&dir_path)?
                .filter_map(|entry| {
                    let entry = entry.ok()?;
                    let path = entry.path();
                    let metadata = entry.metadata().ok()?;

                    Some(serde_json::json!({
                        "name": path.file_name()?.to_string_lossy(),
                        "path": path.to_string_lossy(),
                        "is_dir": metadata.is_dir(),
                        "size": metadata.len(),
                    }))
                })
                .collect();

            Ok(serde_json::json!({ "files": entries }))
        }).await;

        // Register HTTP routes
        router::register_routes(ctx).await?;

        log::info!("[Files] Plugin initialized successfully");
        Ok(())
    }

    async fn start(&self, _ctx: Arc<PluginContext>) -> Result<()> {
        log::info!("[Files] Starting plugin...");
        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        log::info!("[Files] Stopping plugin...");
        Ok(())
    }
}

pub(crate) fn validate_file_path(path: &str) -> Result<()> {
    // Prevent path traversal
    if path.contains("..") {
        return Err(anyhow::anyhow!("Path traversal not allowed"));
    }

    // Ensure path is within projects directory
    let base_path = std::path::PathBuf::from("projects");
    let full_path = base_path.join(path);

    if !full_path.starts_with(&base_path) {
        return Err(anyhow::anyhow!("Access outside projects directory not allowed"));
    }

    Ok(())
}

pub(crate) fn validate_file_extension(path: &str) -> Result<()> {
    let path = std::path::Path::new(path);

    if let Some(extension) = path.extension().and_then(|s| s.to_str()) {
        let ext_lower = extension.to_lowercase();
        if BLACKLISTED_EXTENSIONS.contains(&ext_lower.as_str()) {
            return Err(anyhow::anyhow!("File extension '{}' is not allowed for security reasons", extension));
        }
    }

    Ok(())
}
