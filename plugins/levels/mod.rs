use crate::core::plugin::{Plugin, PluginMetadata};
use crate::core::plugin_context::PluginContext;
use async_trait::async_trait;
use std::sync::Arc;
use anyhow::Result;

mod database;
mod events;
mod router;

pub use database::*;
pub use events::*;

pub struct LevelsPlugin;

#[async_trait]
impl Plugin for LevelsPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            id: "levels".to_string(),
            name: "Levels System".to_string(),
            version: "1.0.0".to_string(),
            description: "User XP and leveling system".to_string(),
            author: "WebArcade Team".to_string(),
            dependencies: vec![],
        }
    }

    async fn init(&self, ctx: &PluginContext) -> Result<()> {
        log::info!("[Levels] Initializing plugin...");

        // Note: Table already exists with (channel, username) as unique key
        // Schema: id, channel, username, xp, level, total_messages, last_xp_gain
        ctx.migrate(&[
            r#"
            CREATE TABLE IF NOT EXISTS user_levels (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                channel TEXT NOT NULL,
                username TEXT NOT NULL,
                xp INTEGER NOT NULL DEFAULT 0,
                level INTEGER NOT NULL DEFAULT 1,
                total_messages INTEGER NOT NULL DEFAULT 0,
                last_xp_gain INTEGER NOT NULL DEFAULT 0,
                UNIQUE(channel, username)
            );

            CREATE TABLE IF NOT EXISTS xp_transactions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id TEXT NOT NULL,
                amount INTEGER NOT NULL,
                reason TEXT,
                created_at INTEGER NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_user_levels ON user_levels(channel, username);
            CREATE INDEX IF NOT EXISTS idx_user_levels_leaderboard ON user_levels(channel, level DESC, xp DESC);
            CREATE INDEX IF NOT EXISTS idx_user_levels_level ON user_levels(level DESC);
            CREATE INDEX IF NOT EXISTS idx_xp_transactions_user ON xp_transactions(user_id);
            "#,
        ])?;

        // Register services
        ctx.provide_service("add_xp", |input| async move {
            let channel: String = serde_json::from_value(input["channel"].clone())?;
            let username: String = serde_json::from_value(input["username"].clone())?;
            let amount: i64 = serde_json::from_value(input["amount"].clone())?;
            let reason: Option<String> = serde_json::from_value(input["reason"].clone()).ok();

            let conn = crate::core::database::get_database_path();
            let conn = rusqlite::Connection::open(conn)?;

            let (old_level, new_level) = database::add_xp(&conn, &channel, &username, amount, reason.as_deref())?;
            Ok(serde_json::json!({
                "old_level": old_level,
                "new_level": new_level,
                "leveled_up": new_level > old_level
            }))
        }).await;

        ctx.provide_service("get_level", |input| async move {
            let channel: String = serde_json::from_value(input["channel"].clone())?;
            let username: String = serde_json::from_value(input["username"].clone())?;

            let conn = crate::core::database::get_database_path();
            let conn = rusqlite::Connection::open(conn)?;

            let user_level = database::get_user_level(&conn, &channel, &username)?;
            Ok(serde_json::to_value(user_level)?)
        }).await;

        ctx.provide_service("get_leaderboard", |input| async move {
            let channel: String = serde_json::from_value(input["channel"].clone())?;
            let limit: usize = serde_json::from_value(input["limit"].clone()).unwrap_or(10);

            let conn = crate::core::database::get_database_path();
            let conn = rusqlite::Connection::open(conn)?;

            let leaderboard = database::get_leaderboard(&conn, &channel, limit)?;
            Ok(serde_json::to_value(leaderboard)?)
        }).await;

        // Register HTTP routes
        router::register_routes(ctx).await?;

        log::info!("[Levels] Plugin initialized successfully");
        Ok(())
    }

    async fn start(&self, ctx: Arc<PluginContext>) -> Result<()> {
        log::info!("[Levels] Starting plugin...");

        // Subscribe to events that grant XP
        let ctx_clone = ctx.clone();
        tokio::spawn(async move {
            let mut events = ctx_clone.subscribe_to("twitch.chat_message").await;

            while let Ok(event) = events.recv().await {
                if let (Ok(channel), Ok(username)) = (
                    serde_json::from_value::<String>(event.payload["channel"].clone()),
                    serde_json::from_value::<String>(event.payload["username"].clone()),
                ) {
                    // Award 1 XP per message (with cooldown handled elsewhere)
                    let _ = ctx_clone.call_service("levels", "add_xp", serde_json::json!({
                        "channel": channel,
                        "username": username,
                        "amount": 1,
                        "reason": "Chat message"
                    })).await;
                }
            }
        });

        log::info!("[Levels] Plugin started successfully");
        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        log::info!("[Levels] Stopping plugin...");
        Ok(())
    }
}
