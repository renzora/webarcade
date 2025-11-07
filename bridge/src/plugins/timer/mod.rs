use crate::core::plugin::{Plugin, PluginMetadata};
use crate::core::plugin_context::PluginContext;
use async_trait::async_trait;
use std::sync::Arc;
use anyhow::Result;

mod router;

pub struct TimerPlugin;

#[async_trait]
impl Plugin for TimerPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            id: "timer".to_string(),
            name: "Timer".to_string(),
            version: "1.0.0".to_string(),
            description: "Countdown timer for overlays and stream".to_string(),
            author: "WebArcade Team".to_string(),
            dependencies: vec![],
        }
    }

    async fn init(&self, ctx: &PluginContext) -> Result<()> {
        log::info!("[Timer] Initializing plugin...");

        // Register HTTP routes
        router::register_routes(ctx).await?;

        log::info!("[Timer] Plugin initialized successfully");
        Ok(())
    }

    async fn start(&self, _ctx: Arc<PluginContext>) -> Result<()> {
        log::info!("[Timer] Starting plugin...");
        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        log::info!("[Timer] Stopping plugin...");
        Ok(())
    }
}
