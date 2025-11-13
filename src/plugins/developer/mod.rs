mod router;
pub mod builder;

use crate::core::plugin::Plugin;
use crate::core::plugin_context::PluginContext;
use crate::plugin_metadata;
use async_trait::async_trait;
use std::sync::Arc;
use anyhow::Result;

pub struct DeveloperPlugin;

#[async_trait]
impl Plugin for DeveloperPlugin {
    plugin_metadata!("developer", "Developer", "1.0.0", "Developer tools for building WebArcade plugins", author: "WebArcade");

    async fn init(&self, ctx: &PluginContext) -> Result<()> {
        log::info!("[Developer] Initializing Developer plugin...");

        // No database tables needed

        // Register HTTP routes
        router::register_routes(ctx).await?;

        log::info!("[Developer] Initialized successfully");
        Ok(())
    }

    async fn start(&self, _ctx: Arc<PluginContext>) -> Result<()> {
        log::info!("[Developer] Started");
        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        log::info!("[Developer] Stopped");
        Ok(())
    }
}
