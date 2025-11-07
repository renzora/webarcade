use crate::core::plugin::{Plugin, PluginMetadata};
use crate::core::plugin_context::PluginContext;
use async_trait::async_trait;
use std::sync::Arc;
use anyhow::Result;

mod router;

pub struct OverlayManagerPlugin;

#[async_trait]
impl Plugin for OverlayManagerPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            id: "overlay-manager".to_string(),
            name: "Overlay Manager".to_string(),
            version: "1.0.0".to_string(),
            description: "Manage and edit overlay files".to_string(),
            author: "WebArcade Team".to_string(),
            dependencies: vec![],
        }
    }

    async fn init(&self, ctx: &PluginContext) -> Result<()> {
        log::info!("[OverlayManager] Initializing plugin...");

        // Register HTTP routes
        router::register_routes(ctx).await?;

        log::info!("[OverlayManager] Plugin initialized successfully");
        Ok(())
    }

    async fn start(&self, _ctx: Arc<PluginContext>) -> Result<()> {
        log::info!("[OverlayManager] Starting plugin...");
        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        log::info!("[OverlayManager] Stopping plugin...");
        Ok(())
    }
}
