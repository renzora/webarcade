use crate::core::plugin::{Plugin, PluginMetadata};
use crate::core::plugin_context::PluginContext;
use async_trait::async_trait;
use std::sync::Arc;
use anyhow::Result;

mod router;

pub struct DiscordPlugin;

#[async_trait]
impl Plugin for DiscordPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            id: "discord".to_string(),
            name: "Discord Integration".to_string(),
            version: "1.0.0".to_string(),
            description: "Discord bot integration for song requests and commands".to_string(),
            author: "WebArcade Team".to_string(),
            dependencies: vec![],
        }
    }

    async fn init(&self, ctx: &PluginContext) -> Result<()> {
        log::info!("[Discord] Initializing plugin...");

        // Register HTTP routes
        router::register_routes(ctx).await?;

        log::info!("[Discord] Plugin initialized successfully");
        Ok(())
    }

    async fn start(&self, _ctx: Arc<PluginContext>) -> Result<()> {
        log::info!("[Discord] Starting plugin...");
        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        log::info!("[Discord] Stopping plugin...");
        Ok(())
    }
}
