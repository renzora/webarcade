mod router;

use crate::core::plugin::Plugin;
use crate::core::plugin_context::PluginContext;
use crate::plugin_metadata;
use async_trait::async_trait;
use std::sync::Arc;
use anyhow::Result;

pub struct TestPlugin;

#[async_trait]
impl Plugin for TestPlugin {
    plugin_metadata!("test", "test", "1.0.0", "nothing", author: "james");

    async fn init(&self, ctx: &PluginContext) -> Result<()> {
        log::info!("[test] Initializing");

        // Database migrations
        ctx.migrate(&[
            r"
            CREATE TABLE IF NOT EXISTS test_data (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                created_at INTEGER NOT NULL
            )
            ",
        ])?;

        // Register routes
        router::register_routes(ctx).await?;

        log::info!("[test] Initialized successfully");
        Ok(())
    }

    async fn start(&self, _ctx: Arc<PluginContext>) -> Result<()> {
        log::info!("[test] Started");
        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        log::info!("[test] Stopped");
        Ok(())
    }
}
