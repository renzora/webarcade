use hyper::{Response, StatusCode};
use http_body_util::{Full, combinators::BoxBody};
use hyper::body::Bytes;
use std::convert::Infallible;
use std::path::PathBuf;

use crate::bridge::core::dynamic_plugin_loader::DynamicPluginLoader;

/// Get the plugins directory based on environment
/// - Development: {repo_root}/plugins (detected by checking if exe is in target/debug)
/// - Production: {exe_dir}/plugins (next to the executable)
pub fn get_plugins_dir() -> PathBuf {
    let exe_path = std::env::current_exe().ok();
    let exe_dir = exe_path.as_ref()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()));

    // Check if we're in development mode by looking for "target\debug" in path
    // Note: target\release is NOT dev mode - it's a production build being tested
    let is_dev = exe_path.as_ref()
        .and_then(|p| p.to_str())
        .map(|s| s.contains("target\\debug") || s.contains("target/debug"))
        .unwrap_or(false);

    if is_dev {
        // Development: use repo root's plugins/ directory
        // Navigate up from src-tauri/target/debug to repo root
        if let Some(exe) = &exe_path {
            if let Some(target_dir) = exe.parent() { // debug or release
                if let Some(target) = target_dir.parent() { // target
                    if let Some(src_tauri) = target.parent() { // src-tauri
                        if let Some(repo_root) = src_tauri.parent() { // repo root
                            let plugins_dir = repo_root.join("plugins");
                            if plugins_dir.exists() || std::fs::create_dir_all(&plugins_dir).is_ok() {
                                return plugins_dir;
                            }
                        }
                    }
                }
            }
        }
        // Fallback: try current directory
        std::env::current_dir()
            .unwrap_or_default()
            .join("plugins")
    } else {
        // Production: try multiple locations
        // 1. First check next to executable (Windows MSI installs here)
        if let Some(ref dir) = exe_dir {
            let plugins_dir = dir.join("plugins");
            if plugins_dir.exists() {
                return plugins_dir;
            }
        }

        // 2. Check in Resources folder (macOS .app bundle)
        if let Some(ref dir) = exe_dir {
            let resources_plugins = dir.join("../Resources/plugins");
            if resources_plugins.exists() {
                return resources_plugins;
            }
        }

        // 3. Fallback to exe directory even if plugins folder doesn't exist yet
        exe_dir
            .unwrap_or_default()
            .join("plugins")
    }
}

/// Handle /api/plugins/list - list runtime plugins
/// Now reads plugin info from the global loaded plugins state
pub fn handle_list_plugins() -> Response<BoxBody<Bytes, Infallible>> {
    // Get the loaded plugins from the global state
    let loaded_plugins = crate::bridge::LOADED_PLUGINS.lock().unwrap();

    let mut plugins = Vec::new();

    for plugin_info in loaded_plugins.iter() {
        let plugin_metadata = serde_json::json!({
            "id": plugin_info.id,
            "name": plugin_info.id, // Could be enhanced to read from manifest
            "version": "1.0.0",
            "description": "",
            "author": "Unknown",
            "routes": plugin_info.routes,
            "has_plugin_js": plugin_info.has_frontend,
            "has_dll": plugin_info.has_backend
        });

        plugins.push(plugin_metadata);
    }

    let json = serde_json::json!({
        "plugins": plugins
    }).to_string();

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .header("Access-Control-Allow-Origin", "*")
        .body(full_body(&json))
        .unwrap()
}

/// Handle /api/plugins/{plugin_id}/{file} - serve plugin files
/// For plugin.js, retrieves from the embedded DLL content
pub fn handle_serve_plugin_file(plugin_id: &str, file_path: &str) -> Response<BoxBody<Bytes, Infallible>> {
    // For plugin.js, serve from the embedded DLL
    if file_path == "plugin.js" {
        match DynamicPluginLoader::get_frontend_js(plugin_id) {
            Ok(js_content) => {
                return Response::builder()
                    .status(StatusCode::OK)
                    .header("Content-Type", "application/javascript")
                    .header("Access-Control-Allow-Origin", "*")
                    .body(BoxBody::new(Full::new(Bytes::from(js_content))))
                    .unwrap();
            }
            Err(e) => {
                log::warn!("Failed to get frontend for plugin {}: {}", plugin_id, e);
                return error_response(StatusCode::NOT_FOUND, "Plugin frontend not found");
            }
        }
    }

    // For other files, return not found (plugins are now self-contained in DLLs)
    error_response(StatusCode::NOT_FOUND, "File not found - plugins are now self-contained in DLLs")
}

fn full_body(s: &str) -> BoxBody<Bytes, Infallible> {
    use http_body_util::combinators::BoxBody;
    use http_body_util::BodyExt;
    BoxBody::new(Full::new(Bytes::from(s.to_string())).map_err(|err: Infallible| match err {}))
}

fn error_response(status: StatusCode, message: &str) -> Response<BoxBody<Bytes, Infallible>> {
    let json = serde_json::json!({"error": message}).to_string();
    Response::builder()
        .status(status)
        .header("Content-Type", "application/json")
        .header("Access-Control-Allow-Origin", "*")
        .body(full_body(&json))
        .unwrap()
}
