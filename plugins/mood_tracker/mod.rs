use crate::core::plugin::{Plugin, PluginMetadata};
use crate::core::plugin_context::PluginContext;
use async_trait::async_trait;
use std::sync::Arc;
use anyhow::Result;

mod router;

pub struct MoodTrackerPlugin;

#[async_trait]
impl Plugin for MoodTrackerPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            id: "mood-tracker".to_string(),
            name: "Mood Tracker".to_string(),
            version: "1.0.0".to_string(),
            description: "Track daily mood, weight, sleep, and water intake".to_string(),
            author: "WebArcade Team".to_string(),
            dependencies: vec![],
        }
    }

    async fn init(&self, ctx: &PluginContext) -> Result<()> {
        log::info!("[Mood Tracker] Initializing plugin...");

        // Create mood_ticker_data table if it doesn't exist
        ctx.migrate(&[
            r#"
            CREATE TABLE IF NOT EXISTS mood_ticker_data (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                mood INTEGER NOT NULL DEFAULT 5,
                show_background INTEGER NOT NULL DEFAULT 1,
                sleep REAL,
                updated_at INTEGER NOT NULL,
                water INTEGER NOT NULL DEFAULT 0,
                weight REAL
            );

            -- Insert default row if not exists
            INSERT OR IGNORE INTO mood_ticker_data (id, mood, show_background, sleep, updated_at, water, weight)
            VALUES (1, 5, 1, NULL, 0, 0, NULL);
            "#,
        ])?;

        // Register HTTP routes
        router::register_routes(ctx).await?;

        log::info!("[Mood Tracker] Plugin initialized successfully");
        Ok(())
    }

    async fn start(&self, _ctx: Arc<PluginContext>) -> Result<()> {
        log::info!("[Mood Tracker] Starting plugin...");
        log::info!("[Mood Tracker] Plugin started successfully");
        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        log::info!("[Mood Tracker] Stopping plugin...");
        Ok(())
    }
}
