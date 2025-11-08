use crate::core::plugin::{Plugin, PluginMetadata};
use crate::core::plugin_context::PluginContext;
use async_trait::async_trait;
use std::sync::Arc;
use anyhow::Result;

mod auth;
mod api;
mod router;
mod events;

pub use auth::*;
pub use api::*;
pub use events::*;

pub struct YoutubePlugin;

#[async_trait]
impl Plugin for YoutubePlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            id: "youtube".to_string(),
            name: "YouTube Integration".to_string(),
            version: "1.0.0".to_string(),
            description: "YouTube integration with channel analytics and OAuth authentication".to_string(),
            author: "WebArcade Team".to_string(),
            dependencies: vec![],
        }
    }

    async fn init(&self, ctx: &PluginContext) -> Result<()> {
        log::info!("[YouTube] Initializing plugin...");

        ctx.migrate(&[
            r#"
            CREATE TABLE IF NOT EXISTS youtube_auth (
                user_id TEXT PRIMARY KEY,
                access_token TEXT NOT NULL,
                refresh_token TEXT NOT NULL,
                expires_at INTEGER NOT NULL,
                scopes TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS youtube_channels (
                channel_id TEXT PRIMARY KEY,
                channel_title TEXT NOT NULL,
                description TEXT,
                custom_url TEXT,
                thumbnail_url TEXT,
                subscriber_count INTEGER,
                video_count INTEGER,
                view_count INTEGER,
                updated_at INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS youtube_analytics_cache (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                channel_id TEXT NOT NULL,
                metric TEXT NOT NULL,
                start_date TEXT NOT NULL,
                end_date TEXT NOT NULL,
                value TEXT NOT NULL,
                cached_at INTEGER NOT NULL,
                UNIQUE(channel_id, metric, start_date, end_date)
            );
            "#,
        ])?;

        // Register routes
        router::register_routes(ctx).await?;

        log::info!("[YouTube] Plugin initialized successfully");
        Ok(())
    }

    async fn start(&self, _ctx: Arc<PluginContext>) -> Result<()> {
        log::info!("[YouTube] Starting plugin...");
        log::info!("[YouTube] Plugin started successfully");
        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        log::info!("[YouTube] Stopping plugin...");
        Ok(())
    }
}
