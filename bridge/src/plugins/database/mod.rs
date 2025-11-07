mod router;

use crate::core::plugin::{Plugin, PluginMetadata};
use crate::core::plugin_context::PluginContext;
use async_trait::async_trait;
use std::sync::Arc;
use anyhow::Result;

pub struct DatabasePlugin;

#[async_trait]
impl Plugin for DatabasePlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            id: "database".to_string(),
            name: "Database Manager".to_string(),
            version: "1.0.0".to_string(),
            description: "SQL query interface for the database".to_string(),
            author: "WebArcade Team".to_string(),
            dependencies: vec![],
        }
    }

    async fn init(&self, ctx: &PluginContext) -> Result<()> {
        log::info!("[Database] Initializing plugin...");

        // Register the router
        router::register_routes(ctx).await?;

        log::info!("[Database] Plugin initialized successfully");
        Ok(())
    }

    async fn start(&self, _ctx: Arc<PluginContext>) -> Result<()> {
        log::info!("[Database] Starting plugin...");
        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        log::info!("[Database] Stopping plugin...");
        Ok(())
    }
}
