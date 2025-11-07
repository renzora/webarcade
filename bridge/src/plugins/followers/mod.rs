use crate::core::plugin::{Plugin, PluginMetadata};
use crate::core::plugin_context::PluginContext;
use async_trait::async_trait;
use std::sync::Arc;
use anyhow::Result;
use rusqlite::OptionalExtension;

pub struct FollowersPlugin;

#[async_trait]
impl Plugin for FollowersPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            id: "followers".to_string(),
            name: "Followers Manager".to_string(),
            version: "1.0.0".to_string(),
            description: "Manages follower events and auto-thanks new followers".to_string(),
            author: "WebArcade Team".to_string(),
            dependencies: vec!["twitch".to_string()],
        }
    }

    async fn init(&self, ctx: &PluginContext) -> Result<()> {
        log::info!("[Followers] Initializing plugin...");

        ctx.migrate(&[
            r#"
            CREATE TABLE IF NOT EXISTS followers (
                user_id TEXT PRIMARY KEY,
                username TEXT NOT NULL,
                followed_at INTEGER NOT NULL,
                thanked INTEGER NOT NULL DEFAULT 0,
                created_at INTEGER NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_followers_followed_at ON followers(followed_at DESC);
            CREATE INDEX IF NOT EXISTS idx_followers_thanked ON followers(thanked);
            "#,
        ])?;

        // Service: Get follower list
        ctx.provide_service("get_followers", |input| async move {
            let limit: i64 = serde_json::from_value(input["limit"].clone()).unwrap_or(100);

            let conn = crate::core::database::get_database_path();
            let conn = rusqlite::Connection::open(conn)?;

            let mut stmt = conn.prepare(
                "SELECT user_id, username, followed_at, thanked
                 FROM followers ORDER BY followed_at DESC LIMIT ?1"
            )?;

            let followers: Vec<serde_json::Value> = stmt.query_map(
                rusqlite::params![limit],
                |row| {
                    Ok(serde_json::json!({
                        "user_id": row.get::<_, String>(0)?,
                        "username": row.get::<_, String>(1)?,
                        "followed_at": row.get::<_, i64>(2)?,
                        "thanked": row.get::<_, i64>(3)? != 0,
                    }))
                }
            )?.collect::<rusqlite::Result<Vec<_>>>()?;

            Ok(serde_json::json!({ "followers": followers }))
        }).await;

        // Service: Get follower count
        ctx.provide_service("get_follower_count", |_input| async move {
            let conn = crate::core::database::get_database_path();
            let conn = rusqlite::Connection::open(conn)?;

            let count: i64 = conn.query_row(
                "SELECT COUNT(*) FROM followers",
                [],
                |row| row.get(0)
            )?;

            Ok(serde_json::json!({ "count": count }))
        }).await;

        log::info!("[Followers] Plugin initialized successfully");
        Ok(())
    }

    async fn start(&self, ctx: Arc<PluginContext>) -> Result<()> {
        log::info!("[Followers] Starting plugin...");

        // Get the broadcaster's channel from the database
        let conn = crate::core::database::get_database_path();
        let conn_result = rusqlite::Connection::open(&conn);

        let channel = if let Ok(conn) = conn_result {
            let channel_result: Option<String> = conn.query_row(
                "SELECT username FROM twitch_auth LIMIT 1",
                [],
                |row| row.get(0)
            ).optional().unwrap_or(None);

            channel_result
        } else {
            None
        };

        // Listen for follow events
        let ctx_follow = ctx.clone();
        tokio::spawn(async move {
            let mut events = ctx_follow.subscribe_to("twitch.follow").await;

            while let Ok(event) = events.recv().await {
                let user_id = match serde_json::from_value::<String>(event.payload["user_id"].clone()) {
                    Ok(id) => id,
                    Err(_) => {
                        log::error!("[Followers] Failed to parse user_id from follow event");
                        continue;
                    }
                };

                let username = match serde_json::from_value::<String>(event.payload["username"].clone()) {
                    Ok(name) => name,
                    Err(_) => {
                        log::error!("[Followers] Failed to parse username from follow event");
                        continue;
                    }
                };

                let followed_at = match serde_json::from_value::<i64>(event.payload["followed_at"].clone()) {
                    Ok(ts) => ts,
                    Err(_) => chrono::Utc::now().timestamp(),
                };

                log::info!("[Followers] New follower: {} ({})", username, user_id);

                // Store in database
                let conn = crate::core::database::get_database_path();
                if let Ok(conn) = rusqlite::Connection::open(&conn) {
                    let now = chrono::Utc::now().timestamp();
                    if let Err(e) = conn.execute(
                        "INSERT OR REPLACE INTO followers (user_id, username, followed_at, thanked, created_at)
                         VALUES (?1, ?2, ?3, 1, ?4)",
                        rusqlite::params![user_id, username, followed_at, now],
                    ) {
                        log::error!("[Followers] Failed to store follower: {}", e);
                    }
                }

                // Get channel to send thank you message to
                let conn = crate::core::database::get_database_path();
                let target_channel = if let Ok(conn) = rusqlite::Connection::open(&conn) {
                    conn.query_row(
                        "SELECT username FROM twitch_auth LIMIT 1",
                        [],
                        |row| row.get::<_, String>(0)
                    ).ok()
                } else {
                    None
                };

                if let Some(channel) = target_channel {
                    // Send thank you message to chat
                    let thank_you_message = format!("Thank you for the follow, @{}! Welcome to the community!", username);

                    ctx_follow.emit("twitch.send_message", &serde_json::json!({
                        "channel": channel,
                        "message": thank_you_message
                    }));

                    log::info!("[Followers] Sent thank you message to {} in {}", username, channel);
                } else {
                    log::warn!("[Followers] Could not determine channel to send thank you message");
                }
            }
        });

        log::info!("[Followers] Plugin started successfully");
        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        log::info!("[Followers] Stopping plugin...");
        Ok(())
    }
}
