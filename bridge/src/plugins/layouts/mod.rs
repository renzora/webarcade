use crate::core::plugin::{Plugin, PluginMetadata};
use crate::core::plugin_context::PluginContext;
use async_trait::async_trait;
use std::sync::Arc;
use anyhow::Result;

mod router;

pub struct LayoutsPlugin;

#[async_trait]
impl Plugin for LayoutsPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            id: "layouts".to_string(),
            name: "Layout Manager".to_string(),
            version: "1.0.0".to_string(),
            description: "Manage dashboard layouts".to_string(),
            author: "WebArcade Team".to_string(),
            dependencies: vec![],
        }
    }

    async fn init(&self, ctx: &PluginContext) -> Result<()> {
        log::info!("[Layouts] Initializing plugin...");

        // Create layouts table
        ctx.migrate(&[
            r#"
            CREATE TABLE IF NOT EXISTS layouts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                layout_data TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )
            "#,
        ])?;

        // Register HTTP routes
        router::register_routes(ctx).await?;

        log::info!("[Layouts] Plugin initialized successfully");
        Ok(())
    }

    async fn start(&self, _ctx: Arc<PluginContext>) -> Result<()> {
        log::info!("[Layouts] Starting plugin...");
        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        log::info!("[Layouts] Stopping plugin...");
        Ok(())
    }
}
