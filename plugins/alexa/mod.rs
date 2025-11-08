use crate::core::plugin::{Plugin, PluginMetadata};
use crate::core::plugin_context::PluginContext;
use async_trait::async_trait;
use std::sync::Arc;
use anyhow::Result;

mod router;

pub struct AlexaPlugin;

#[async_trait]
impl Plugin for AlexaPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            id: "alexa".to_string(),
            name: "Alexa Integration".to_string(),
            version: "1.0.0".to_string(),
            description: "Amazon Alexa integration for voice commands".to_string(),
            author: "WebArcade Team".to_string(),
            dependencies: vec![],
        }
    }

    async fn init(&self, ctx: &PluginContext) -> Result<()> {
        log::info!("[Alexa] Initializing plugin...");

        // Register HTTP routes
        router::register_routes(ctx).await?;

        log::info!("[Alexa] Plugin initialized successfully");
        Ok(())
    }

    async fn start(&self, _ctx: Arc<PluginContext>) -> Result<()> {
        log::info!("[Alexa] Starting plugin...");
        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        log::info!("[Alexa] Stopping plugin...");
        Ok(())
    }
}
