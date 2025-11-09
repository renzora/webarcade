mod router;

use crate::core::plugin::Plugin;
use crate::core::plugin_context::PluginContext;
use crate::plugin_metadata;
use async_trait::async_trait;
use std::sync::Arc;
use anyhow::Result;

pub struct DatabasePlugin;

#[async_trait]
impl Plugin for DatabasePlugin {
    plugin_metadata!("database", "Database", "1.0.0", "SQL query interface for the database");

    async fn init(&self, ctx: &PluginContext) -> Result<()> {
        log::info!("[Database] Initializing plugin...");
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
