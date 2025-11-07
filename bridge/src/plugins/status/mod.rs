use crate::core::plugin::{Plugin, PluginMetadata};
use crate::core::plugin_context::PluginContext;
use async_trait::async_trait;
use std::sync::Arc;
use anyhow::Result;

mod router;

pub struct StatusPlugin;

#[async_trait]
impl Plugin for StatusPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            id: "status".to_string(),
            name: "Status System".to_string(),
            version: "1.0.0".to_string(),
            description: "Stream status configuration (start date, ticker speed, etc.)".to_string(),
            author: "WebArcade Team".to_string(),
            dependencies: vec![],
        }
    }

    async fn init(&self, ctx: &PluginContext) -> Result<()> {
        log::info!("[Status] Initializing plugin...");

        ctx.migrate(&[
            r#"
            CREATE TABLE IF NOT EXISTS status_config (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                updated_at INTEGER NOT NULL
            );
            "#,
        ])?;

        // Register HTTP routes
        router::register_routes(ctx).await?;

        log::info!("[Status] Plugin initialized");
        Ok(())
    }

    async fn start(&self, _ctx: Arc<PluginContext>) -> Result<()> {
        log::info!("[Status] Plugin started");
        Ok(())
    }
}

fn current_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}
