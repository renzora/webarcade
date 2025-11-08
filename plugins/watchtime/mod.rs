use crate::core::plugin::{Plugin, PluginMetadata};
use crate::core::plugin_context::PluginContext;
use async_trait::async_trait;
use std::sync::Arc;
use anyhow::Result;
use serde_json::json;

mod router;

pub struct WatchtimePlugin;

#[async_trait]
impl Plugin for WatchtimePlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            id: "watchtime".to_string(),
            name: "Watchtime Tracking".to_string(),
            version: "1.0.0".to_string(),
            description: "Track viewer watchtime and last_seen using the users table".to_string(),
            author: "WebArcade Team".to_string(),
            dependencies: vec![],
        }
    }

    async fn init(&self, ctx: &PluginContext) -> Result<()> {
        log::info!("[Watchtime] Initializing plugin...");

        // No table creation - we use the existing users table
        // Users table schema (already exists):
        // - total_minutes: watchtime in minutes
        // - last_seen: last activity timestamp
        // - xp, level, coins: managed by other plugins

        // Get all watchtime records with pagination
        ctx.provide_service("get_all", |input| async move {
            let channel: String = serde_json::from_value(input["channel"].clone())?;
            let limit: i64 = serde_json::from_value(input["limit"].clone()).unwrap_or(50);
            let offset: i64 = serde_json::from_value(input["offset"].clone()).unwrap_or(0);

            let conn = crate::core::database::get_database_path();
            let conn = rusqlite::Connection::open(conn)?;

            // Get total count
            let total: i64 = conn.query_row(
                "SELECT COUNT(*) FROM users WHERE channel = ?1",
                [&channel],
                |row| row.get(0)
            )?;

            // Get watchers
            let mut stmt = conn.prepare(
                "SELECT username, total_minutes, last_seen
                 FROM users
                 WHERE channel = ?1
                 ORDER BY total_minutes DESC
                 LIMIT ?2 OFFSET ?3"
            )?;

            let watchers: Vec<serde_json::Value> = stmt.query_map(
                rusqlite::params![&channel, limit, offset],
                |row| {
                    Ok(json!({
                        "username": row.get::<_, String>(0)?,
                        "total_minutes": row.get::<_, i64>(1)?,
                        "last_seen": row.get::<_, i64>(2)?
                    }))
                }
            )?.collect::<rusqlite::Result<Vec<_>>>()?;

            Ok(json!({
                "watchers": watchers,
                "total": total
            }))
        }).await;

        // Search watchtime by username
        ctx.provide_service("search", |input| async move {
            let channel: String = serde_json::from_value(input["channel"].clone())?;
            let search: String = serde_json::from_value(input["search"].clone())?;

            let conn = crate::core::database::get_database_path();
            let conn = rusqlite::Connection::open(conn)?;

            let search_pattern = format!("%{}%", search);

            let mut stmt = conn.prepare(
                "SELECT username, total_minutes, last_seen
                 FROM users
                 WHERE channel = ?1 AND username LIKE ?2
                 ORDER BY total_minutes DESC
                 LIMIT 100"
            )?;

            let watchers: Vec<serde_json::Value> = stmt.query_map(
                rusqlite::params![&channel, &search_pattern],
                |row| {
                    Ok(json!({
                        "username": row.get::<_, String>(0)?,
                        "total_minutes": row.get::<_, i64>(1)?,
                        "last_seen": row.get::<_, i64>(2)?
                    }))
                }
            )?.collect::<rusqlite::Result<Vec<_>>>()?;

            Ok(json!(watchers))
        }).await;

        // Get watchtime by period (for stats)
        ctx.provide_service("get_by_period", |input| async move {
            let channel: String = serde_json::from_value(input["channel"].clone())?;
            let period: String = serde_json::from_value(input["period"].clone()).unwrap_or_else(|_| "all".to_string());
            let limit: i64 = serde_json::from_value(input["limit"].clone()).unwrap_or(50);
            let offset: i64 = serde_json::from_value(input["offset"].clone()).unwrap_or(0);

            let conn = crate::core::database::get_database_path();
            let conn = rusqlite::Connection::open(conn)?;

            let now = current_timestamp();
            let cutoff_time = match period.as_str() {
                "day" => now - 86400,
                "week" => now - 604800,
                "month" => now - 2592000,
                _ => 0, // "all" or anything else
            };

            // Get total count for period
            let total: i64 = if cutoff_time > 0 {
                conn.query_row(
                    "SELECT COUNT(*) FROM users WHERE channel = ?1 AND last_seen >= ?2",
                    rusqlite::params![&channel, cutoff_time],
                    |row| row.get(0)
                )?
            } else {
                conn.query_row(
                    "SELECT COUNT(*) FROM users WHERE channel = ?1",
                    [&channel],
                    |row| row.get(0)
                )?
            };

            // Get watchers for period
            let watchers: Vec<serde_json::Value> = if cutoff_time > 0 {
                let mut stmt = conn.prepare(
                    "SELECT username, total_minutes, last_seen
                     FROM users
                     WHERE channel = ?1 AND last_seen >= ?2
                     ORDER BY total_minutes DESC
                     LIMIT ?3 OFFSET ?4"
                )?;

                let result = stmt.query_map(
                    rusqlite::params![&channel, cutoff_time, limit, offset],
                    |row| {
                        Ok(json!({
                            "username": row.get::<_, String>(0)?,
                            "total_minutes": row.get::<_, i64>(1)?,
                            "last_seen": row.get::<_, i64>(2)?
                        }))
                    }
                )?.collect::<rusqlite::Result<Vec<_>>>()?;
                result
            } else {
                let mut stmt = conn.prepare(
                    "SELECT username, total_minutes, last_seen
                     FROM users
                     WHERE channel = ?1
                     ORDER BY total_minutes DESC
                     LIMIT ?2 OFFSET ?3"
                )?;

                let result = stmt.query_map(
                    rusqlite::params![&channel, limit, offset],
                    |row| {
                        Ok(json!({
                            "username": row.get::<_, String>(0)?,
                            "total_minutes": row.get::<_, i64>(1)?,
                            "last_seen": row.get::<_, i64>(2)?
                        }))
                    }
                )?.collect::<rusqlite::Result<Vec<_>>>()?;
                result
            };

            Ok(json!({
                "watchers": watchers,
                "total": total
            }))
        }).await;

        // Add watchtime (internal use only - updates total_minutes)
        ctx.provide_service("add_time", |input| async move {
            let channel: String = serde_json::from_value(input["channel"].clone())?;
            let username: String = serde_json::from_value(input["username"].clone())?;
            let minutes: i64 = serde_json::from_value(input["minutes"].clone()).unwrap_or(1);

            let conn = crate::core::database::get_database_path();
            let conn = rusqlite::Connection::open(conn)?;

            let now = current_timestamp();

            // Update total_minutes and last_seen in users table
            conn.execute(
                "INSERT INTO users (channel, username, total_minutes, last_seen, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?4)
                 ON CONFLICT(channel, username) DO UPDATE SET
                   total_minutes = total_minutes + ?3,
                   last_seen = ?4",
                rusqlite::params![channel, username, minutes, now],
            )?;

            Ok(json!({ "success": true }))
        }).await;

        // Register HTTP routes
        router::register_routes(ctx).await?;

        log::info!("[Watchtime] Plugin initialized successfully");
        Ok(())
    }

    async fn start(&self, ctx: Arc<PluginContext>) -> Result<()> {
        log::info!("[Watchtime] Starting plugin...");

        // Track active viewers - award 1 minute of watchtime + rewards every minute
        use std::collections::HashMap;
        use std::sync::Arc as StdArc;
        use tokio::sync::Mutex;

        // Map of (channel, username) -> last_activity_time
        let active_viewers: StdArc<Mutex<HashMap<(String, String), i64>>> =
            StdArc::new(Mutex::new(HashMap::new()));

        // Subscribe to chat messages to track active users and update last_seen
        let ctx_chat = ctx.clone();
        let active_viewers_chat = active_viewers.clone();
        tokio::spawn(async move {
            let mut events = ctx_chat.subscribe_to("twitch.chat_message").await;

            while let Ok(event) = events.recv().await {
                if let (Ok(channel), Ok(username)) = (
                    serde_json::from_value::<String>(event.payload["channel"].clone()),
                    serde_json::from_value::<String>(event.payload["username"].clone()),
                ) {
                    let now = current_timestamp();

                    // Mark user as active in memory
                    let mut viewers = active_viewers_chat.lock().await;
                    viewers.insert((channel.clone(), username.clone()), now);
                    drop(viewers);

                    // Immediately update last_seen in users table (without adding watchtime)
                    let conn = crate::core::database::get_database_path();
                    if let Ok(conn) = rusqlite::Connection::open(conn) {
                        let _ = conn.execute(
                            "INSERT INTO users (channel, username, last_seen, created_at)
                             VALUES (?1, ?2, ?3, ?3)
                             ON CONFLICT(channel, username) DO UPDATE SET last_seen = ?3",
                            rusqlite::params![channel, username, now],
                        );
                    }
                }
            }
        });

        // Every minute, award watchtime and rewards to active viewers
        let ctx_tracking = ctx.clone();
        let active_viewers_tracking = active_viewers.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60));
            loop {
                interval.tick().await;

                let now = current_timestamp();
                let five_minutes_ago = now - 300; // 5 minutes of inactivity = not watching

                let mut viewers = active_viewers_tracking.lock().await;

                // Remove inactive viewers and award time to active ones
                viewers.retain(|(channel, username), last_activity| {
                    if *last_activity < five_minutes_ago {
                        // User inactive for 5+ minutes, remove them
                        false
                    } else {
                        // User is active, award rewards
                        let ctx_clone = ctx_tracking.clone();
                        let channel = channel.clone();
                        let username = username.clone();

                        tokio::spawn(async move {
                            // Award 1 minute of watchtime (updates total_minutes and last_seen in users table)
                            let _ = ctx_clone.call_service("watchtime", "add_time", serde_json::json!({
                                "channel": channel,
                                "username": username,
                                "minutes": 1
                            })).await;

                            // Award 5 XP for watching (every minute)
                            let _ = ctx_clone.call_service("levels", "add_xp", serde_json::json!({
                                "channel": channel,
                                "username": username,
                                "amount": 5,
                                "reason": "Watching stream"
                            })).await;

                            // Award 10 coins for watching (every minute)
                            let _ = ctx_clone.call_service("currency", "add_currency", serde_json::json!({
                                "user_id": username.clone(), // TODO: Get real user_id
                                "username": username,
                                "amount": 10,
                                "reason": "Watching stream"
                            })).await;
                        });

                        true // Keep this viewer
                    }
                });

                log::debug!("[Watchtime] Active viewers: {}", viewers.len());
            }
        });

        // Handle !watchtime command
        let ctx_command = ctx.clone();
        tokio::spawn(async move {
            let mut events = ctx_command.subscribe_to("twitch.chat_message").await;
            log::info!("[Watchtime] Command handler started, waiting for chat messages...");

            while let Ok(event) = events.recv().await {
                if let (Ok(channel), Ok(username), Ok(message)) = (
                    serde_json::from_value::<String>(event.payload["channel"].clone()),
                    serde_json::from_value::<String>(event.payload["username"].clone()),
                    serde_json::from_value::<String>(event.payload["message"].clone()),
                ) {
                    // Check if message is !watchtime or !watchtime @username
                    if message.starts_with("!watchtime") {
                        log::info!("[Watchtime] Detected !watchtime command from {} in {}", username, channel);
                        let parts: Vec<&str> = message.split_whitespace().collect();
                        let target_user = if parts.len() > 1 {
                            // Remove @ if present
                            parts[1].trim_start_matches('@').to_string()
                        } else {
                            username.clone()
                        };

                        // Query watchtime from users table
                        let conn = crate::core::database::get_database_path();
                        if let Ok(conn) = rusqlite::Connection::open(conn) {
                            let result: Result<(i64, i64), _> = conn.query_row(
                                "SELECT total_minutes, last_seen FROM users WHERE channel = ?1 AND username = ?2",
                                rusqlite::params![&channel, &target_user],
                                |row| Ok((row.get(0)?, row.get(1)?))
                            );

                            let response = match result {
                                Ok((total_minutes, last_seen)) => {
                                    let hours = total_minutes / 60;
                                    let minutes = total_minutes % 60;

                                    if target_user.to_lowercase() == username.to_lowercase() {
                                        format!("@{} You have watched for {} hours and {} minutes! ⏱️", username, hours, minutes)
                                    } else {
                                        format!("@{} has watched for {} hours and {} minutes! ⏱️", target_user, hours, minutes)
                                    }
                                }
                                Err(_) => {
                                    if target_user.to_lowercase() == username.to_lowercase() {
                                        format!("@{} You haven't watched yet! Start watching to track your time. 👀", username)
                                    } else {
                                        format!("@{} hasn't watched yet! 👀", target_user)
                                    }
                                }
                            };

                            // Send response to chat via twitch.send_message event
                            ctx_command.emit("twitch.send_message", &serde_json::json!({
                                "channel": channel,
                                "message": response
                            }));
                        }
                    }
                }
            }
        });

        log::info!("[Watchtime] Plugin started successfully");
        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        log::info!("[Watchtime] Stopping plugin...");
        Ok(())
    }
}

fn current_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}
