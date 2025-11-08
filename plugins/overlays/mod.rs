use crate::core::plugin::{Plugin, PluginMetadata};
use crate::core::plugin_context::PluginContext;
use async_trait::async_trait;
use std::sync::Arc;
use anyhow::Result;

mod router;

pub struct OverlaysPlugin;

#[async_trait]
impl Plugin for OverlaysPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            id: "overlay".to_string(),
            name: "Overlay Layout Server".to_string(),
            version: "1.0.0".to_string(),
            description: "Serves dynamic layout HTML (overlays served by rspack)".to_string(),
            author: "WebArcade Team".to_string(),
            dependencies: vec![],
        }
    }

    async fn init(&self, ctx: &PluginContext) -> Result<()> {
        log::info!("[Overlays] Initializing plugin...");

        // Register HTTP routes
        router::register_routes(ctx).await?;

        log::info!("[Overlays] Plugin initialized successfully");
        Ok(())
    }

    async fn start(&self, _ctx: Arc<PluginContext>) -> Result<()> {
        log::info!("[Overlays] Starting plugin...");
        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        log::info!("[Overlays] Stopping plugin...");
        Ok(())
    }
}
