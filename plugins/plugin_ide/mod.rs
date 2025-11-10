mod router;

use crate::core::plugin::Plugin;
use crate::core::plugin_context::PluginContext;
use crate::plugin_metadata;
use async_trait::async_trait;
use std::sync::Arc;
use anyhow::Result;

pub struct PluginIdePlugin;

#[async_trait]
impl Plugin for PluginIdePlugin {
    plugin_metadata!("plugin_ide", "Plugin IDE", "1.0.0", "Integrated development environment for building WebArcade plugins", author: "WebArcade");

    async fn init(&self, ctx: &PluginContext) -> Result<()> {
        log::info!("[plugin_ide] Initializing Plugin IDE v2.0 - UPDATED CODE");

        // No database tables needed for the IDE

        // Register HTTP routes
        router::register_routes(ctx).await?;

        log::info!("[plugin_ide] Initialized successfully");
        Ok(())
    }

    async fn start(&self, _ctx: Arc<PluginContext>) -> Result<()> {
        log::info!("[plugin_ide] Started");
        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        log::info!("[plugin_ide] Stopped");
        Ok(())
    }
}
