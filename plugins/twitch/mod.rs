use crate::core::plugin::{Plugin, PluginMetadata};
use crate::core::plugin_context::PluginContext;
use async_trait::async_trait;
use std::sync::Arc;
use anyhow::Result;
use rusqlite::OptionalExtension;
use std::sync::atomic::{AtomicBool, Ordering};

mod irc;
mod api;
mod auth;
mod eventsub;
mod eventsub_webhook;
mod commands;
// mod database; // Removed - database operations moved to plugin services
mod events;
mod router;
mod ngrok;

pub use irc::*;
pub use api::*;
pub use auth::*;
pub use eventsub::*;
pub use eventsub_webhook::*;
pub use commands::*;
// pub use database::*; // Removed
pub use events::*;
pub use ngrok::*;

// Global flag to prevent duplicate event listener registration
static LISTENERS_INITIALIZED: AtomicBool = AtomicBool::new(false);

pub struct TwitchPlugin;

#[async_trait]
impl Plugin for TwitchPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            id: "twitch".to_string(),
            name: "Twitch Integration".to_string(),
            version: "1.0.0".to_string(),
            description: "Complete Twitch integration with IRC, API, EventSub, and commands".to_string(),
            author: "WebArcade Team".to_string(),
            dependencies: vec![],
        }
    }

    async fn init(&self, ctx: &PluginContext) -> Result<()> {
        log::info!("[Twitch] Initializing plugin...");

        ctx.migrate(&[
            r#"
            CREATE TABLE IF NOT EXISTS twitch_config (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                updated_at INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS twitch_auth (
                user_id TEXT PRIMARY KEY,
                username TEXT NOT NULL,
                access_token TEXT NOT NULL,
                refresh_token TEXT NOT NULL,
                expires_at INTEGER NOT NULL,
                scopes TEXT NOT NULL,
                profile_image_url TEXT,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS twitch_bot_auth (
                user_id TEXT PRIMARY KEY,
                username TEXT NOT NULL,
                access_token TEXT NOT NULL,
                refresh_token TEXT NOT NULL,
                expires_at INTEGER NOT NULL,
                scopes TEXT NOT NULL,
                profile_image_url TEXT,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS twitch_channels (
                channel_id TEXT PRIMARY KEY,
                channel_name TEXT NOT NULL UNIQUE,
                display_name TEXT NOT NULL,
                is_live INTEGER NOT NULL DEFAULT 0,
                game_name TEXT,
                title TEXT,
                viewer_count INTEGER,
                started_at INTEGER,
                updated_at INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS twitch_chat_messages (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                channel TEXT NOT NULL,
                user_id TEXT NOT NULL,
                username TEXT NOT NULL,
                message TEXT NOT NULL,
                is_command INTEGER NOT NULL DEFAULT 0,
                timestamp INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS twitch_commands (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                command TEXT NOT NULL UNIQUE,
                handler_plugin TEXT NOT NULL,
                handler_method TEXT NOT NULL,
                permission_level TEXT NOT NULL DEFAULT 'everyone',
                cooldown_seconds INTEGER NOT NULL DEFAULT 0,
                enabled INTEGER NOT NULL DEFAULT 1,
                created_at INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS twitch_events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                event_type TEXT NOT NULL,
                event_data TEXT NOT NULL,
                user_id TEXT,
                username TEXT,
                channel TEXT,
                timestamp INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS twitch_eventsub_subscriptions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                subscription_id TEXT NOT NULL UNIQUE,
                subscription_type TEXT NOT NULL,
                condition TEXT NOT NULL,
                status TEXT NOT NULL,
                created_at INTEGER NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_chat_messages_channel ON twitch_chat_messages(channel, timestamp DESC);
            CREATE INDEX IF NOT EXISTS idx_chat_messages_user ON twitch_chat_messages(user_id);
            CREATE INDEX IF NOT EXISTS idx_events_type ON twitch_events(event_type, timestamp DESC);
            CREATE INDEX IF NOT EXISTS idx_events_user ON twitch_events(user_id);
            "#,
            // Migration: Add profile_image_url to existing tables
            r#"
            ALTER TABLE twitch_auth ADD COLUMN profile_image_url TEXT;
            "#,
            r#"
            ALTER TABLE twitch_bot_auth ADD COLUMN profile_image_url TEXT;
            "#,
        ])?;

        // Service: Send chat message
        ctx.provide_service("send_message", |input| async move {
            let channel: String = serde_json::from_value(input["channel"].clone())?;
            let message: String = serde_json::from_value(input["message"].clone())?;

            // NOTE: Actual IRC message sending would happen here
            log::info!("[Twitch] Sending message to {}: {}", channel, message);

            Ok(serde_json::json!({ "success": true }))
        }).await;

        // Service: Register command
        ctx.provide_service("register_command", |input| async move {
            let command: String = serde_json::from_value(input["command"].clone())?;
            let handler_plugin: String = serde_json::from_value(input["handler_plugin"].clone())?;
            let handler_method: String = serde_json::from_value(input["handler_method"].clone())?;
            let permission_level: String = serde_json::from_value(input["permission_level"].clone()).unwrap_or_else(|_| "everyone".to_string());
            let cooldown_seconds: i64 = serde_json::from_value(input["cooldown_seconds"].clone()).unwrap_or(0);

            let conn = crate::core::database::get_database_path();
            let conn = rusqlite::Connection::open(conn)?;

            let now = current_timestamp();
            conn.execute(
                "INSERT OR REPLACE INTO twitch_commands (command, handler_plugin, handler_method, permission_level, cooldown_seconds, enabled, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, 1, ?6)",
                rusqlite::params![command, handler_plugin, handler_method, permission_level, cooldown_seconds, now],
            )?;

            Ok(serde_json::json!({ "success": true }))
        }).await;

        // Service: Get channel info
        ctx.provide_service("get_channel_info", |input| async move {
            let channel_name: String = serde_json::from_value(input["channel_name"].clone())?;

            let conn = crate::core::database::get_database_path();
            let conn = rusqlite::Connection::open(conn)?;

            let channel: Option<serde_json::Value> = conn.query_row(
                "SELECT channel_id, channel_name, display_name, is_live, game_name, title, viewer_count, started_at
                 FROM twitch_channels WHERE channel_name = ?1",
                rusqlite::params![channel_name],
                |row| {
                    Ok(serde_json::json!({
                        "channel_id": row.get::<_, String>(0)?,
                        "channel_name": row.get::<_, String>(1)?,
                        "display_name": row.get::<_, String>(2)?,
                        "is_live": row.get::<_, i64>(3)? != 0,
                        "game_name": row.get::<_, Option<String>>(4)?,
                        "title": row.get::<_, Option<String>>(5)?,
                        "viewer_count": row.get::<_, Option<i64>>(6)?,
                        "started_at": row.get::<_, Option<i64>>(7)?,
                    }))
                }
            ).optional()?;

            Ok(serde_json::json!({ "channel": channel }))
        }).await;

        // Service: Update channel info
        ctx.provide_service("update_channel_info", |input| async move {
            let channel_id: String = serde_json::from_value(input["channel_id"].clone())?;
            let channel_name: String = serde_json::from_value(input["channel_name"].clone())?;
            let display_name: String = serde_json::from_value(input["display_name"].clone())?;
            let is_live: bool = serde_json::from_value(input["is_live"].clone()).unwrap_or(false);
            let game_name: Option<String> = serde_json::from_value(input["game_name"].clone()).ok();
            let title: Option<String> = serde_json::from_value(input["title"].clone()).ok();
            let viewer_count: Option<i64> = serde_json::from_value(input["viewer_count"].clone()).ok();
            let started_at: Option<i64> = serde_json::from_value(input["started_at"].clone()).ok();

            let conn = crate::core::database::get_database_path();
            let conn = rusqlite::Connection::open(conn)?;

            let now = current_timestamp();
            conn.execute(
                "INSERT OR REPLACE INTO twitch_channels (channel_id, channel_name, display_name, is_live, game_name, title, viewer_count, started_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                rusqlite::params![channel_id, channel_name, display_name, is_live as i64, game_name, title, viewer_count, started_at, now],
            )?;

            Ok(serde_json::json!({ "success": true }))
        }).await;

        // Service: Log chat message
        ctx.provide_service("log_message", |input| async move {
            let channel: String = serde_json::from_value(input["channel"].clone())?;
            let user_id: String = serde_json::from_value(input["user_id"].clone())?;
            let username: String = serde_json::from_value(input["username"].clone())?;
            let message: String = serde_json::from_value(input["message"].clone())?;
            let is_command: bool = serde_json::from_value(input["is_command"].clone()).unwrap_or(false);

            let conn = crate::core::database::get_database_path();
            let conn = rusqlite::Connection::open(conn)?;

            let now = current_timestamp();
            conn.execute(
                "INSERT INTO twitch_chat_messages (channel, user_id, username, message, is_command, timestamp)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                rusqlite::params![channel, user_id, username, message, is_command as i64, now],
            )?;

            Ok(serde_json::json!({ "success": true }))
        }).await;

        // Service: Get recent messages
        ctx.provide_service("get_recent_messages", |input| async move {
            let channel: String = serde_json::from_value(input["channel"].clone())?;
            let limit: i64 = serde_json::from_value(input["limit"].clone()).unwrap_or(100);

            let conn = crate::core::database::get_database_path();
            let conn = rusqlite::Connection::open(conn)?;

            let mut stmt = conn.prepare(
                "SELECT username, message, timestamp FROM twitch_chat_messages
                 WHERE channel = ?1 ORDER BY timestamp DESC LIMIT ?2"
            )?;

            let messages: Vec<serde_json::Value> = stmt.query_map(
                rusqlite::params![channel, limit],
                |row| {
                    Ok(serde_json::json!({
                        "username": row.get::<_, String>(0)?,
                        "message": row.get::<_, String>(1)?,
                        "timestamp": row.get::<_, i64>(2)?,
                    }))
                }
            )?.collect::<rusqlite::Result<Vec<_>>>()?;

            Ok(serde_json::json!({ "messages": messages }))
        }).await;

        // Service: Get config
        ctx.provide_service("get_config", |_input| async move {
            let conn = crate::core::database::get_database_path();
            let conn = rusqlite::Connection::open(conn)?;

            // Get all config values
            let client_id: Option<String> = conn.query_row(
                "SELECT value FROM twitch_config WHERE key = 'client_id'",
                [],
                |row| row.get(0)
            ).optional()?;

            let client_secret: Option<String> = conn.query_row(
                "SELECT value FROM twitch_config WHERE key = 'client_secret'",
                [],
                |row| row.get(0)
            ).optional()?;

            let broadcaster_username: Option<String> = conn.query_row(
                "SELECT value FROM twitch_config WHERE key = 'broadcaster_username'",
                [],
                |row| row.get(0)
            ).optional()?;

            let channels_json: Option<String> = conn.query_row(
                "SELECT value FROM twitch_config WHERE key = 'channels'",
                [],
                |row| row.get(0)
            ).optional()?;

            let channels: Vec<String> = if let Some(json) = channels_json {
                serde_json::from_str(&json).unwrap_or_else(|_| vec![])
            } else {
                vec![]
            };

            // Check if we have an auth token
            let has_token = conn.query_row(
                "SELECT COUNT(*) FROM twitch_auth",
                [],
                |row| row.get::<_, i64>(0)
            ).unwrap_or(0) > 0;

            Ok(serde_json::json!({
                "client_id": client_id.unwrap_or_default(),
                "client_secret": client_secret.map(|_| "***").unwrap_or_default(), // Don't send actual secret
                "broadcaster_username": broadcaster_username,
                "channels": channels,
                "has_token": has_token
            }))
        }).await;

        // Service: Save config
        ctx.provide_service("save_config", |input| async move {
            let conn = crate::core::database::get_database_path();
            let conn = rusqlite::Connection::open(conn)?;

            let now = current_timestamp();

            // Save client_id
            if let Some(client_id) = input["client_id"].as_str() {
                conn.execute(
                    "INSERT OR REPLACE INTO twitch_config (key, value, updated_at) VALUES ('client_id', ?1, ?2)",
                    rusqlite::params![client_id, now],
                )?;
            }

            // Save client_secret (only if provided and not the masked version)
            if let Some(client_secret) = input["client_secret"].as_str() {
                if !client_secret.is_empty() && client_secret != "***" {
                    conn.execute(
                        "INSERT OR REPLACE INTO twitch_config (key, value, updated_at) VALUES ('client_secret', ?1, ?2)",
                        rusqlite::params![client_secret, now],
                    )?;
                }
            }

            // Note: channels are now auto-configured from authenticated user
            // This can still be saved if explicitly provided for backward compatibility
            if let Some(channels) = input["channels"].as_array() {
                let channels_json = serde_json::to_string(&channels)?;
                conn.execute(
                    "INSERT OR REPLACE INTO twitch_config (key, value, updated_at) VALUES ('channels', ?1, ?2)",
                    rusqlite::params![channels_json, now],
                )?;
            }

            Ok(serde_json::json!({ "success": true }))
        }).await;

        // Register HTTP routes
        router::register_routes(ctx).await?;

        log::info!("[Twitch] Plugin initialized successfully");
        Ok(())
    }

    async fn start(&self, ctx: Arc<PluginContext>) -> Result<()> {
        log::info!("[Twitch] Starting plugin...");

        /*
         * ============================================================================
         * AVAILABLE TWITCH EVENTS (that any plugin can subscribe to)
         * ============================================================================
         *
         * IRC Events (emitted automatically when IRC is connected):
         *
         * Chat & Messages:
         * - twitch.chat_message        - User sent a message
         * - twitch.message_deleted     - Moderator deleted a message
         * - twitch.chat_cleared        - Chat was cleared
         * - twitch.whisper             - Private message received
         * - twitch.notice              - Twitch notice message
         *
         * Subscriptions:
         * - twitch.subscription        - User subscribed or resubscribed
         * - twitch.subscription_gift   - User gifted a subscription to someone
         * - twitch.mass_gift_subscription - User gifted multiple subs
         * - twitch.gift_paid_upgrade   - User upgraded from gifted to paid sub
         * - twitch.reward_gift         - Sub gifted via channel points
         * - twitch.anon_gift_paid_upgrade - Anonymous gift upgraded to paid
         *
         * Community:
         * - twitch.raid                - Channel was raided
         * - twitch.unraid              - Raid was cancelled
         * - twitch.bits                - User cheered bits
         * - twitch.bits_badge_tier     - User reached new bits badge tier
         * - twitch.ritual              - New viewer ritual
         *
         * Moderation:
         * - twitch.timeout             - User was timed out
         * - twitch.ban                 - User was banned
         *
         * Channel State:
         * - twitch.room_state          - Room settings changed (slow mode, emote-only, etc)
         * - twitch.user_state          - Bot's user state updated
         * - twitch.global_user_state   - Bot's global state after authentication
         * - twitch.user_join           - User joined channel
         * - twitch.user_part           - User left channel
         * - twitch.names_list          - List of users in channel (353)
         * - twitch.names_end           - End of names list (366)
         *
         * Connection:
         * - twitch.connected           - Successfully connected (001)
         * - twitch.reconnect           - Server requesting reconnect
         * - twitch.capability          - Capability negotiation (CAP)
         * - twitch.unknown_command     - Unknown command error (421)
         *
         * Special:
         * - twitch.shared_chat_notice  - Shared chat notification (experimental)
         *
         * Control Events (to send messages):
         * - twitch.send_message        - Emit this to send a chat message
         *                                Payload: {"channel": "name", "message": "text"}
         *
         * EventSub Events (require webhook setup - not yet implemented):
         * - twitch.follow              - New follower (requires EventSub)
         * - twitch.channel_points_redemption - Channel points redeemed
         * - twitch.stream_online       - Stream went online
         * - twitch.stream_offline      - Stream went offline
         * - twitch.channel_update      - Channel info updated
         * - twitch.poll_begin          - Poll started
         * - twitch.poll_progress       - Poll progress
         * - twitch.poll_end            - Poll ended
         * - twitch.prediction_begin    - Prediction started
         * - twitch.prediction_progress - Prediction progress
         * - twitch.prediction_lock     - Prediction locked
         * - twitch.prediction_end      - Prediction ended
         * - twitch.hype_train_begin    - Hype train started
         * - twitch.hype_train_progress - Hype train progress
         * - twitch.hype_train_end      - Hype train ended
         *
         * Example: Subscribe to an event
         * ```rust
         * let mut events = ctx.subscribe_to("twitch.subscription").await;
         * while let Ok(event) = events.recv().await {
         *     let username = event.payload["username"].as_str().unwrap();
         *     // React to subscription
         * }
         * ```
         * ============================================================================
         */

        // Get authentication info
        let conn = crate::core::database::get_database_path();
        let conn = rusqlite::Connection::open(&conn)?;

        let (access_token, username): (Option<String>, Option<String>) = conn.query_row(
            "SELECT access_token, username FROM twitch_auth LIMIT 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?))
        ).optional()?.unwrap_or((None, None));

        let channels_json: Option<String> = conn.query_row(
            "SELECT value FROM twitch_config WHERE key = 'channels'",
            [],
            |row| row.get(0)
        ).optional()?;

        let channels: Vec<String> = if let Some(json) = channels_json {
            serde_json::from_str(&json).unwrap_or_else(|_| vec![])
        } else {
            vec![]
        };

        drop(conn);

        // Start automatic token refresh task
        let ctx_refresh = ctx.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30 * 60)); // Check every 30 minutes

            loop {
                interval.tick().await;

                log::debug!("[Twitch] Checking if tokens need refresh...");

                let conn = crate::core::database::get_database_path();
                let conn = match rusqlite::Connection::open(&conn) {
                    Ok(c) => c,
                    Err(e) => {
                        log::error!("[Twitch] Failed to open database for token refresh: {}", e);
                        continue;
                    }
                };

                // Get client credentials (needed for both refreshes)
                let (client_id, client_secret): (Option<String>, Option<String>) = conn.query_row(
                    "SELECT
                        (SELECT value FROM twitch_config WHERE key = 'client_id'),
                        (SELECT value FROM twitch_config WHERE key = 'client_secret')",
                    [],
                    |row| Ok((row.get(0)?, row.get(1)?))
                ).unwrap_or((None, None));

                if let (Some(cid), Some(secret)) = (client_id.clone(), client_secret.clone()) {
                    let now = current_timestamp();

                    // === Refresh broadcaster token ===
                    let broadcaster_token_info: Option<(String, String, i64)> = conn.query_row(
                        "SELECT refresh_token, access_token, expires_at FROM twitch_auth LIMIT 1",
                        [],
                        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?))
                    ).optional().unwrap_or(None);

                    if let Some((refresh_token, access_token, expires_at)) = broadcaster_token_info {
                        let time_until_expiry = expires_at - now;

                        // Refresh if token expires within 1 hour (3600 seconds)
                        if time_until_expiry < 3600 {
                            log::info!("[Twitch] Broadcaster token expires in {} seconds, refreshing...", time_until_expiry);

                            let auth = auth::TwitchAuth::new(cid.clone(), secret.clone(), "".to_string());

                            match auth.refresh_access_token(&refresh_token).await {
                                Ok(new_token) => {
                                    let new_expires_at = now + new_token.expires_in;

                                    log::info!("[Twitch] Broadcaster token refreshed successfully, expires at {}", new_expires_at);

                                    // Update database
                                    if let Err(e) = conn.execute(
                                        "UPDATE twitch_auth SET access_token = ?1, refresh_token = ?2, expires_at = ?3, updated_at = ?4",
                                        rusqlite::params![new_token.access_token, new_token.refresh_token, new_expires_at, now],
                                    ) {
                                        log::error!("[Twitch] Failed to update broadcaster token in database: {}", e);
                                    } else {
                                        // Emit event that token was refreshed
                                        ctx_refresh.emit("twitch.broadcaster_token_refreshed", &serde_json::json!({
                                            "access_token": new_token.access_token,
                                            "expires_at": new_expires_at
                                        }));
                                    }
                                }
                                Err(e) => {
                                    log::error!("[Twitch] Failed to refresh broadcaster token: {}", e);

                                    // Emit event that token refresh failed
                                    ctx_refresh.emit("twitch.broadcaster_token_refresh_failed", &serde_json::json!({
                                        "error": e.to_string()
                                    }));
                                }
                            }
                        } else {
                            log::debug!("[Twitch] Broadcaster token is still valid for {} seconds", time_until_expiry);
                        }
                    }

                    // === Refresh bot token ===
                    let bot_token_info: Option<(String, String, i64)> = conn.query_row(
                        "SELECT refresh_token, access_token, expires_at FROM twitch_bot_auth LIMIT 1",
                        [],
                        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?))
                    ).optional().unwrap_or(None);

                    if let Some((refresh_token, access_token, expires_at)) = bot_token_info {
                        let time_until_expiry = expires_at - now;

                        // Refresh if token expires within 1 hour (3600 seconds)
                        if time_until_expiry < 3600 {
                            log::info!("[Twitch] Bot token expires in {} seconds, refreshing...", time_until_expiry);

                            let auth = auth::TwitchAuth::new(cid.clone(), secret.clone(), "".to_string());

                            match auth.refresh_access_token(&refresh_token).await {
                                Ok(new_token) => {
                                    let new_expires_at = now + new_token.expires_in;

                                    log::info!("[Twitch] Bot token refreshed successfully, expires at {}", new_expires_at);

                                    // Update database
                                    if let Err(e) = conn.execute(
                                        "UPDATE twitch_bot_auth SET access_token = ?1, refresh_token = ?2, expires_at = ?3, updated_at = ?4",
                                        rusqlite::params![new_token.access_token, new_token.refresh_token, new_expires_at, now],
                                    ) {
                                        log::error!("[Twitch] Failed to update bot token in database: {}", e);
                                    } else {
                                        // Emit event that token was refreshed
                                        ctx_refresh.emit("twitch.bot_token_refreshed", &serde_json::json!({
                                            "access_token": new_token.access_token,
                                            "expires_at": new_expires_at
                                        }));
                                    }
                                }
                                Err(e) => {
                                    log::error!("[Twitch] Failed to refresh bot token: {}", e);

                                    // Emit event that token refresh failed
                                    ctx_refresh.emit("twitch.bot_token_refresh_failed", &serde_json::json!({
                                        "error": e.to_string()
                                    }));
                                }
                            }
                        } else {
                            log::debug!("[Twitch] Bot token is still valid for {} seconds", time_until_expiry);
                        }
                    }
                } else {
                    log::warn!("[Twitch] Cannot refresh tokens: Client ID or Secret not configured");
                }
            }
        });

        // Start IRC client - prefer bot credentials if available, otherwise use broadcaster credentials
        let conn = crate::core::database::get_database_path();
        let conn = rusqlite::Connection::open(&conn)?;

        let (bot_token, bot_nick): (Option<String>, Option<String>) = conn.query_row(
            "SELECT access_token, username FROM twitch_bot_auth LIMIT 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?))
        ).optional()?.unwrap_or((None, None));

        drop(conn);

        // Decide which credentials to use for IRC
        let (irc_token, irc_nick, use_bot_auth) = if let (Some(bt), Some(bn)) = (bot_token, bot_nick) {
            log::info!("[Twitch] Using bot account '{}' for IRC", bn);
            (Some(bt), Some(bn), true)
        } else if let (Some(token), Some(nick)) = (access_token.clone(), username.clone()) {
            log::info!("[Twitch] Using broadcaster account '{}' for IRC (no bot configured)", nick);
            (Some(token), Some(nick), false)
        } else {
            (None, None, false)
        };

        if let (Some(token), Some(nick)) = (irc_token, irc_nick) {
            if !channels.is_empty() {
                log::info!("[Twitch] Starting IRC client for {} in channels: {:?}", nick, channels);

                let (irc_tx, mut irc_rx) = tokio::sync::mpsc::unbounded_channel();

                // Create shared token storage for IRC client
                let shared_token = Arc::new(tokio::sync::RwLock::new(String::new()));

                match irc::TwitchIrcClient::connect(token, nick, channels, irc_tx, shared_token.clone()).await {
                    Ok(irc_client) => {
                        log::info!("[Twitch] IRC client connected");

                        // Store IRC client in context for other plugins to use
                        let irc_client = Arc::new(tokio::sync::Mutex::new(irc_client));
                        let irc_client_for_service = irc_client.clone();

                        // Register send_message service
                        ctx.provide_service("send_message", move |input| {
                            let irc = irc_client_for_service.clone();
                            async move {
                                let channel: String = serde_json::from_value(input["channel"].clone())?;
                                let message: String = serde_json::from_value(input["message"].clone())?;

                                let client = irc.lock().await;
                                client.send_message(&channel, &message)?;

                                Ok(serde_json::json!({ "success": true }))
                            }
                        }).await;

                        // Process incoming IRC messages
                        let ctx_irc = ctx.clone();
                        tokio::spawn(async move {
                            while let Some(msg) = irc_rx.recv().await {
                                let channel = msg.params.get(0).map(|c| c.trim_start_matches('#').to_string());

                                match msg.command.as_str() {
                                    "PRIVMSG" => {
                                        if msg.params.len() >= 2 {
                                            let channel = msg.params[0].trim_start_matches('#');
                                            let message = &msg.params[1];

                                            // Extract username from prefix
                                            let username = msg.prefix.as_ref().and_then(|p| {
                                                p.split('!').next().map(|s| s.to_string())
                                            }).unwrap_or_else(|| "unknown".to_string());

                                            // Extract user ID, display name, badges from tags
                                            let user_id = msg.tags.get("user-id").cloned().unwrap_or_default();
                                            let display_name = msg.tags.get("display-name").cloned().unwrap_or_else(|| username.clone());
                                            let badges = msg.tags.get("badges").cloned().unwrap_or_default();
                                            let is_mod = msg.tags.get("mod").map(|v| v == "1").unwrap_or(false);
                                            let is_subscriber = msg.tags.get("subscriber").map(|v| v == "1").unwrap_or(false);
                                            let is_broadcaster = badges.contains("broadcaster");

                                            // Check for bits/cheers
                                            let bits = msg.tags.get("bits").and_then(|s| s.parse::<i64>().ok());

                                            log::info!("[Twitch IRC] Message from {} in {}: {}", username, channel, message);

                                            // If bits were sent, emit a separate bits event
                                            if let Some(bits_amount) = bits {
                                                log::info!("[Twitch] {} cheered {} bits in {}", display_name, bits_amount, channel);

                                                ctx_irc.emit("twitch.bits", &serde_json::json!({
                                                    "channel": channel,
                                                    "username": username,
                                                    "display_name": display_name,
                                                    "user_id": user_id,
                                                    "bits": bits_amount,
                                                    "message": message,
                                                    "tags": msg.tags.clone()
                                                }));
                                            }

                                            // Emit chat message event
                                            ctx_irc.emit("twitch.chat_message", &serde_json::json!({
                                                "channel": channel,
                                                "username": username,
                                                "display_name": display_name,
                                                "user_id": user_id,
                                                "message": message,
                                                "badges": badges,
                                                "is_mod": is_mod,
                                                "is_subscriber": is_subscriber,
                                                "is_broadcaster": is_broadcaster,
                                                "bits": bits,
                                                "tags": msg.tags
                                            }));
                                        }
                                    }
                                    "USERNOTICE" => {
                                        // Handles subs, resubs, gift subs, raids, rituals
                                        let msg_id = msg.tags.get("msg-id").map(|s| s.as_str()).unwrap_or("");
                                        let channel_name = channel.as_ref().map(|s| s.as_str()).unwrap_or("unknown");

                                        match msg_id {
                                            "sub" | "resub" => {
                                                let username = msg.tags.get("login").cloned().unwrap_or_default();
                                                let display_name = msg.tags.get("display-name").cloned().unwrap_or_else(|| username.clone());
                                                let months = msg.tags.get("msg-param-cumulative-months").and_then(|s| s.parse::<i64>().ok()).unwrap_or(1);
                                                let tier = msg.tags.get("msg-param-sub-plan").cloned().unwrap_or_else(|| "1000".to_string());
                                                let message = msg.params.get(1).cloned().unwrap_or_default();

                                                log::info!("[Twitch] {} subscribed (Tier {}, {} months)", display_name, tier, months);

                                                ctx_irc.emit("twitch.subscription", &serde_json::json!({
                                                    "channel": channel_name,
                                                    "username": username,
                                                    "display_name": display_name,
                                                    "months": months,
                                                    "tier": tier,
                                                    "message": message,
                                                    "is_resub": msg_id == "resub",
                                                    "tags": msg.tags
                                                }));
                                            }
                                            "subgift" => {
                                                let gifter = msg.tags.get("login").cloned().unwrap_or_default();
                                                let gifter_display = msg.tags.get("display-name").cloned().unwrap_or_else(|| gifter.clone());
                                                let recipient = msg.tags.get("msg-param-recipient-user-name").cloned().unwrap_or_default();
                                                let recipient_display = msg.tags.get("msg-param-recipient-display-name").cloned().unwrap_or_else(|| recipient.clone());
                                                let tier = msg.tags.get("msg-param-sub-plan").cloned().unwrap_or_else(|| "1000".to_string());
                                                let months = msg.tags.get("msg-param-months").and_then(|s| s.parse::<i64>().ok()).unwrap_or(1);

                                                log::info!("[Twitch] {} gifted a sub to {} (Tier {})", gifter_display, recipient_display, tier);

                                                ctx_irc.emit("twitch.subscription_gift", &serde_json::json!({
                                                    "channel": channel_name,
                                                    "gifter": gifter,
                                                    "gifter_display_name": gifter_display,
                                                    "recipient": recipient,
                                                    "recipient_display_name": recipient_display,
                                                    "tier": tier,
                                                    "months": months,
                                                    "tags": msg.tags
                                                }));
                                            }
                                            "submysterygift" => {
                                                let gifter = msg.tags.get("login").cloned().unwrap_or_default();
                                                let gifter_display = msg.tags.get("display-name").cloned().unwrap_or_else(|| gifter.clone());
                                                let count = msg.tags.get("msg-param-mass-gift-count").and_then(|s| s.parse::<i64>().ok()).unwrap_or(1);
                                                let tier = msg.tags.get("msg-param-sub-plan").cloned().unwrap_or_else(|| "1000".to_string());

                                                log::info!("[Twitch] {} gifted {} subs (Tier {})", gifter_display, count, tier);

                                                ctx_irc.emit("twitch.mass_gift_subscription", &serde_json::json!({
                                                    "channel": channel_name,
                                                    "gifter": gifter,
                                                    "gifter_display_name": gifter_display,
                                                    "count": count,
                                                    "tier": tier,
                                                    "tags": msg.tags
                                                }));
                                            }
                                            "raid" => {
                                                let raider = msg.tags.get("msg-param-displayName").cloned().unwrap_or_default();
                                                let viewer_count = msg.tags.get("msg-param-viewerCount").and_then(|s| s.parse::<i64>().ok()).unwrap_or(0);

                                                log::info!("[Twitch] Raided by {} with {} viewers", raider, viewer_count);

                                                ctx_irc.emit("twitch.raid", &serde_json::json!({
                                                    "channel": channel_name,
                                                    "raider": raider,
                                                    "viewer_count": viewer_count,
                                                    "tags": msg.tags
                                                }));
                                            }
                                            "ritual" => {
                                                let ritual_name = msg.tags.get("msg-param-ritual-name").cloned().unwrap_or_default();
                                                let username = msg.tags.get("login").cloned().unwrap_or_default();

                                                log::info!("[Twitch] Ritual: {} by {}", ritual_name, username);

                                                ctx_irc.emit("twitch.ritual", &serde_json::json!({
                                                    "channel": channel_name,
                                                    "ritual_name": ritual_name,
                                                    "username": username,
                                                    "tags": msg.tags
                                                }));
                                            }
                                            "giftpaidupgrade" => {
                                                let username = msg.tags.get("login").cloned().unwrap_or_default();
                                                let display_name = msg.tags.get("display-name").cloned().unwrap_or_else(|| username.clone());
                                                let gifter = msg.tags.get("msg-param-sender-login").cloned().unwrap_or_default();
                                                let gifter_name = msg.tags.get("msg-param-sender-name").cloned().unwrap_or_else(|| gifter.clone());

                                                log::info!("[Twitch] {} upgraded their gifted sub from {}", display_name, gifter_name);

                                                ctx_irc.emit("twitch.gift_paid_upgrade", &serde_json::json!({
                                                    "channel": channel_name,
                                                    "username": username,
                                                    "display_name": display_name,
                                                    "gifter": gifter,
                                                    "gifter_name": gifter_name,
                                                    "tags": msg.tags
                                                }));
                                            }
                                            "rewardgift" => {
                                                let username = msg.tags.get("login").cloned().unwrap_or_default();
                                                let display_name = msg.tags.get("display-name").cloned().unwrap_or_else(|| username.clone());
                                                let count = msg.tags.get("msg-param-gift-months").and_then(|s| s.parse::<i64>().ok()).unwrap_or(1);
                                                let tier = msg.tags.get("msg-param-sub-plan").cloned().unwrap_or_else(|| "1000".to_string());

                                                log::info!("[Twitch] {} gifted a sub using channel points (Tier {})", display_name, tier);

                                                ctx_irc.emit("twitch.reward_gift", &serde_json::json!({
                                                    "channel": channel_name,
                                                    "username": username,
                                                    "display_name": display_name,
                                                    "tier": tier,
                                                    "months": count,
                                                    "tags": msg.tags
                                                }));
                                            }
                                            "anongiftpaidupgrade" => {
                                                let username = msg.tags.get("login").cloned().unwrap_or_default();
                                                let display_name = msg.tags.get("display-name").cloned().unwrap_or_else(|| username.clone());

                                                log::info!("[Twitch] {} upgraded their anonymously gifted sub", display_name);

                                                ctx_irc.emit("twitch.anon_gift_paid_upgrade", &serde_json::json!({
                                                    "channel": channel_name,
                                                    "username": username,
                                                    "display_name": display_name,
                                                    "tags": msg.tags
                                                }));
                                            }
                                            "unraid" => {
                                                log::info!("[Twitch] Raid cancelled in {}", channel_name);

                                                ctx_irc.emit("twitch.unraid", &serde_json::json!({
                                                    "channel": channel_name,
                                                    "tags": msg.tags
                                                }));
                                            }
                                            "bitsbadgetier" => {
                                                let username = msg.tags.get("login").cloned().unwrap_or_default();
                                                let display_name = msg.tags.get("display-name").cloned().unwrap_or_else(|| username.clone());
                                                let threshold = msg.tags.get("msg-param-threshold").and_then(|s| s.parse::<i64>().ok()).unwrap_or(0);

                                                log::info!("[Twitch] {} reached a new bits badge tier: {}", display_name, threshold);

                                                ctx_irc.emit("twitch.bits_badge_tier", &serde_json::json!({
                                                    "channel": channel_name,
                                                    "username": username,
                                                    "display_name": display_name,
                                                    "threshold": threshold,
                                                    "tags": msg.tags
                                                }));
                                            }
                                            "sharedchatnotice" => {
                                                let message = msg.params.get(1).cloned().unwrap_or_default();

                                                log::info!("[Twitch] Shared chat notice in {}: {}", channel_name, message);

                                                ctx_irc.emit("twitch.shared_chat_notice", &serde_json::json!({
                                                    "channel": channel_name,
                                                    "message": message,
                                                    "tags": msg.tags
                                                }));
                                            }
                                            _ => {
                                                log::debug!("[Twitch] Unhandled USERNOTICE type: {}", msg_id);
                                            }
                                        }
                                    }
                                    "CLEARMSG" => {
                                        // Message deleted
                                        if let (Some(channel), Some(target_msg_id)) = (channel.as_ref(), msg.tags.get("target-msg-id")) {
                                            log::info!("[Twitch] Message deleted in {}: {}", channel, target_msg_id);

                                            ctx_irc.emit("twitch.message_deleted", &serde_json::json!({
                                                "channel": channel,
                                                "message_id": target_msg_id,
                                                "tags": msg.tags
                                            }));
                                        }
                                    }
                                    "CLEARCHAT" => {
                                        // Timeout or ban
                                        if let Some(channel) = channel.as_ref() {
                                            if let Some(target_user) = msg.params.get(1) {
                                                let duration = msg.tags.get("ban-duration").and_then(|s| s.parse::<i64>().ok());

                                                if let Some(secs) = duration {
                                                    log::info!("[Twitch] {} timed out in {} for {}s", target_user, channel, secs);

                                                    ctx_irc.emit("twitch.timeout", &serde_json::json!({
                                                        "channel": channel,
                                                        "username": target_user,
                                                        "duration": secs,
                                                        "tags": msg.tags
                                                    }));
                                                } else {
                                                    log::info!("[Twitch] {} banned in {}", target_user, channel);

                                                    ctx_irc.emit("twitch.ban", &serde_json::json!({
                                                        "channel": channel,
                                                        "username": target_user,
                                                        "tags": msg.tags
                                                    }));
                                                }
                                            } else {
                                                log::info!("[Twitch] Chat cleared in {}", channel);

                                                ctx_irc.emit("twitch.chat_cleared", &serde_json::json!({
                                                    "channel": channel,
                                                    "tags": msg.tags
                                                }));
                                            }
                                        }
                                    }
                                    "ROOMSTATE" => {
                                        // Room state changed
                                        if let Some(channel) = channel.as_ref() {
                                            ctx_irc.emit("twitch.room_state", &serde_json::json!({
                                                "channel": channel,
                                                "emote_only": msg.tags.get("emote-only").map(|v| v == "1").unwrap_or(false),
                                                "followers_only": msg.tags.get("followers-only").and_then(|s| s.parse::<i64>().ok()).unwrap_or(-1),
                                                "r9k": msg.tags.get("r9k").map(|v| v == "1").unwrap_or(false),
                                                "slow": msg.tags.get("slow").and_then(|s| s.parse::<i64>().ok()).unwrap_or(0),
                                                "subs_only": msg.tags.get("subs-only").map(|v| v == "1").unwrap_or(false),
                                                "tags": msg.tags
                                            }));
                                        }
                                    }
                                    "USERSTATE" => {
                                        // User state (when bot sends a message)
                                        if let Some(channel) = channel.as_ref() {
                                            ctx_irc.emit("twitch.user_state", &serde_json::json!({
                                                "channel": channel,
                                                "tags": msg.tags
                                            }));
                                        }
                                    }
                                    "JOIN" => {
                                        if let Some(channel) = channel.as_ref() {
                                            let username = msg.prefix.as_ref().and_then(|p| {
                                                p.split('!').next().map(|s| s.to_string())
                                            }).unwrap_or_default();

                                            log::info!("[Twitch IRC] {} joined {}", username, channel);

                                            ctx_irc.emit("twitch.user_join", &serde_json::json!({
                                                "channel": channel,
                                                "username": username
                                            }));
                                        }
                                    }
                                    "PART" => {
                                        if let Some(channel) = channel.as_ref() {
                                            let username = msg.prefix.as_ref().and_then(|p| {
                                                p.split('!').next().map(|s| s.to_string())
                                            }).unwrap_or_default();

                                            log::info!("[Twitch IRC] {} left {}", username, channel);

                                            ctx_irc.emit("twitch.user_part", &serde_json::json!({
                                                "channel": channel,
                                                "username": username
                                            }));
                                        }
                                    }
                                    "NOTICE" => {
                                        // Various notices from Twitch
                                        if let (Some(channel), Some(notice)) = (channel.as_ref(), msg.params.get(1)) {
                                            let msg_id = msg.tags.get("msg-id").cloned().unwrap_or_default();

                                            log::info!("[Twitch] Notice in {}: {}", channel, notice);

                                            ctx_irc.emit("twitch.notice", &serde_json::json!({
                                                "channel": channel,
                                                "message": notice,
                                                "msg_id": msg_id,
                                                "tags": msg.tags
                                            }));
                                        }
                                    }
                                    "WHISPER" => {
                                        // Private messages (if enabled)
                                        if msg.params.len() >= 2 {
                                            let username = msg.prefix.as_ref().and_then(|p| {
                                                p.split('!').next().map(|s| s.to_string())
                                            }).unwrap_or_default();
                                            let message = &msg.params[1];

                                            log::info!("[Twitch] Whisper from {}: {}", username, message);

                                            ctx_irc.emit("twitch.whisper", &serde_json::json!({
                                                "username": username,
                                                "message": message,
                                                "tags": msg.tags
                                            }));
                                        }
                                    }
                                    "RECONNECT" => {
                                        // Server requesting reconnection
                                        log::warn!("[Twitch IRC] Server requesting reconnect");

                                        ctx_irc.emit("twitch.reconnect", &serde_json::json!({
                                            "reason": "server_requested"
                                        }));

                                        // The connection will automatically reconnect due to the loop
                                    }
                                    "GLOBALUSERSTATE" => {
                                        // Global user state after authentication
                                        log::info!("[Twitch IRC] Global user state received");

                                        ctx_irc.emit("twitch.global_user_state", &serde_json::json!({
                                            "user_id": msg.tags.get("user-id").cloned(),
                                            "display_name": msg.tags.get("display-name").cloned(),
                                            "color": msg.tags.get("color").cloned(),
                                            "badges": msg.tags.get("badges").cloned(),
                                            "emote_sets": msg.tags.get("emote-sets").cloned(),
                                            "tags": msg.tags
                                        }));
                                    }
                                    "CAP" => {
                                        // Capability negotiation response
                                        if msg.params.len() >= 2 {
                                            let subcommand = &msg.params[0];
                                            let capabilities = msg.params.get(1).map(|s| s.as_str()).unwrap_or("");

                                            log::debug!("[Twitch IRC] CAP {}: {}", subcommand, capabilities);

                                            ctx_irc.emit("twitch.capability", &serde_json::json!({
                                                "subcommand": subcommand,
                                                "capabilities": capabilities
                                            }));
                                        }
                                    }
                                    "001" => {
                                        // RPL_WELCOME - Successful connection
                                        log::info!("[Twitch IRC] Successfully connected (RPL_WELCOME)");

                                        ctx_irc.emit("twitch.connected", &serde_json::json!({
                                            "message": msg.params.last().cloned().unwrap_or_default()
                                        }));
                                    }
                                    "002" | "003" | "004" => {
                                        // RPL_YOURHOST, RPL_CREATED, RPL_MYINFO
                                        log::debug!("[Twitch IRC] Server info ({}): {:?}", msg.command, msg.params);
                                    }
                                    "353" => {
                                        // RPL_NAMREPLY - Names list
                                        if let (Some(channel), Some(names)) = (msg.params.get(2), msg.params.get(3)) {
                                            log::debug!("[Twitch IRC] Names in {}: {}", channel, names);

                                            ctx_irc.emit("twitch.names_list", &serde_json::json!({
                                                "channel": channel.trim_start_matches('#'),
                                                "names": names.split_whitespace().collect::<Vec<_>>()
                                            }));
                                        }
                                    }
                                    "366" => {
                                        // RPL_ENDOFNAMES - End of names list
                                        if let Some(channel) = msg.params.get(1) {
                                            log::debug!("[Twitch IRC] End of names for {}", channel);

                                            ctx_irc.emit("twitch.names_end", &serde_json::json!({
                                                "channel": channel.trim_start_matches('#')
                                            }));
                                        }
                                    }
                                    "372" => {
                                        // RPL_MOTD - Message of the day
                                        log::debug!("[Twitch IRC] MOTD: {:?}", msg.params);
                                    }
                                    "375" => {
                                        // RPL_MOTDSTART - Start of MOTD
                                        log::debug!("[Twitch IRC] MOTD start");
                                    }
                                    "376" => {
                                        // RPL_ENDOFMOTD - End of MOTD
                                        log::debug!("[Twitch IRC] MOTD end");
                                    }
                                    "421" => {
                                        // ERR_UNKNOWNCOMMAND
                                        if let Some(unknown_cmd) = msg.params.get(1) {
                                            log::warn!("[Twitch IRC] Unknown command: {}", unknown_cmd);

                                            ctx_irc.emit("twitch.unknown_command", &serde_json::json!({
                                                "command": unknown_cmd
                                            }));
                                        }
                                    }
                                    _ => {
                                        log::debug!("[Twitch IRC] Unhandled command: {}", msg.command);
                                    }
                                }
                            }
                        });

                        // Listen for send_message events from other plugins
                        // Only create this listener once to prevent duplicates
                        if !LISTENERS_INITIALIZED.load(Ordering::Relaxed) {
                            let irc_for_events = irc_client.clone();
                            let ctx_send = ctx.clone();
                            tokio::spawn(async move {
                                let mut events = ctx_send.subscribe_to("twitch.send_message").await;

                                while let Ok(event) = events.recv().await {
                                    if let (Ok(channel), Ok(message)) = (
                                        serde_json::from_value::<String>(event.payload["channel"].clone()),
                                        serde_json::from_value::<String>(event.payload["message"].clone()),
                                    ) {
                                        let client = irc_for_events.lock().await;
                                        if let Err(e) = client.send_message(&channel, &message) {
                                            log::error!("[Twitch] Failed to send message: {}", e);
                                        }
                                    }
                                }
                            });
                        }

                        // Listen for token refresh events and update shared token + trigger reconnection
                        let irc_for_refresh = irc_client.clone();
                        let shared_token_for_refresh = shared_token.clone();
                        let ctx_refresh_listener = ctx.clone();
                        let using_bot = use_bot_auth;
                        tokio::spawn(async move {
                            let event_name = if using_bot {
                                "twitch.bot_token_refreshed"
                            } else {
                                "twitch.broadcaster_token_refreshed"
                            };

                            let mut events = ctx_refresh_listener.subscribe_to(event_name).await;

                            while let Ok(event) = events.recv().await {
                                log::info!("[Twitch] Token refreshed, updating shared token and triggering IRC reconnection...");

                                // Get the new token from the event payload (no database query needed!)
                                if let Ok(new_token) = serde_json::from_value::<String>(event.payload["access_token"].clone()) {
                                    let formatted_token = if new_token.starts_with("oauth:") {
                                        new_token
                                    } else {
                                        format!("oauth:{}", new_token)
                                    };

                                    // Update shared token in memory
                                    let mut token = shared_token_for_refresh.write().await;
                                    *token = formatted_token;
                                    drop(token);

                                    log::info!("[Twitch] Shared token updated in memory, disconnecting IRC to reconnect with new token");

                                    // Trigger reconnection by disconnecting
                                    let client = irc_for_refresh.lock().await;
                                    if let Err(e) = client.disconnect() {
                                        log::error!("[Twitch] Failed to trigger IRC disconnect: {}", e);
                                    }
                                } else {
                                    log::error!("[Twitch] Failed to get new token from refresh event payload");
                                }
                            }
                        });
                    }
                    Err(e) => {
                        log::error!("[Twitch] Failed to start IRC client: {}", e);
                    }
                }
            } else {
                log::warn!("[Twitch] No channels configured, IRC client not started");
            }
        } else {
            log::warn!("[Twitch] No authentication found, IRC client not started");
        }

        // Command handler
        // Only create this listener once to prevent duplicates
        if !LISTENERS_INITIALIZED.load(Ordering::Relaxed) {
            let ctx_commands = ctx.clone();
            tokio::spawn(async move {
                let mut events = ctx_commands.subscribe_to("twitch.chat_message").await;

                while let Ok(event) = events.recv().await {
                    if let (Ok(channel), Ok(username), Ok(message)) = (
                        serde_json::from_value::<String>(event.payload["channel"].clone()),
                        serde_json::from_value::<String>(event.payload["username"].clone()),
                        serde_json::from_value::<String>(event.payload["message"].clone()),
                    ) {
                        // Check if message is a command
                        if let Some(command_text) = message.strip_prefix('!') {
                            let parts: Vec<&str> = command_text.split_whitespace().collect();
                            if let Some(cmd) = parts.first() {
                                log::info!("[Twitch] Command received: !{} from {} in {}", cmd, username, channel);

                                ctx_commands.emit("twitch.command", &serde_json::json!({
                                    "command": cmd,
                                    "args": &parts[1..],
                                    "channel": channel,
                                    "username": username,
                                    "message": message
                                }));
                            }
                        }
                    }
                }
            });
        }

        // EventSub event processor - polls database queue and emits events
        let ctx_eventsub = ctx.clone();
        tokio::spawn(async move {
            log::info!("[Twitch] EventSub event processor started");

            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(2)); // Check every 2 seconds

            loop {
                interval.tick().await;

                // Poll database for queued events
                let conn = crate::core::database::get_database_path();
                let conn_result = rusqlite::Connection::open(&conn);

                if let Ok(conn) = conn_result {
                    // Get all unprocessed events
                    let events_result: Result<Vec<(i64, String, String)>, rusqlite::Error> = {
                        let mut stmt = match conn.prepare(
                            "SELECT id, event_type, event_data FROM twitch_events ORDER BY timestamp ASC LIMIT 100"
                        ) {
                            Ok(s) => s,
                            Err(e) => {
                                log::error!("[Twitch EventSub] Failed to prepare query: {}", e);
                                continue;
                            }
                        };

                        stmt.query_map([], |row| {
                            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
                        })
                        .and_then(|rows| rows.collect())
                    };

                    if let Ok(events) = events_result {
                        for (event_id, event_type, event_data_str) in events {
                            // Parse event data
                            if let Ok(event_data) = serde_json::from_str::<serde_json::Value>(&event_data_str) {
                                log::info!("[Twitch EventSub] Processing queued event: {}", event_type);

                                // Emit event to plugin system
                                ctx_eventsub.emit(&event_type, &event_data);

                                // Delete processed event
                                if let Err(e) = conn.execute(
                                    "DELETE FROM twitch_events WHERE id = ?1",
                                    rusqlite::params![event_id],
                                ) {
                                    log::error!("[Twitch EventSub] Failed to delete processed event: {}", e);
                                }
                            } else {
                                log::error!("[Twitch EventSub] Failed to parse event data for event {}", event_id);
                                // Delete malformed event
                                let _ = conn.execute("DELETE FROM twitch_events WHERE id = ?1", rusqlite::params![event_id]);
                            }
                        }
                    }
                }
            }
        });

        // API client periodic updates
        let ctx_api = ctx.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(300)); // Every 5 minutes

            loop {
                interval.tick().await;

                // NOTE: Actual Twitch API calls would happen here to update channel info
                log::debug!("[Twitch] API sync check");
            }
        });

        // Text commands response handler
        // Only create this listener once to prevent duplicates
        if !LISTENERS_INITIALIZED.load(Ordering::Relaxed) {
            let ctx_text_commands = ctx.clone();
            tokio::spawn(async move {
                let mut events = ctx_text_commands.subscribe_to("text_commands.executed").await;

                while let Ok(event) = events.recv().await {
                    if let (Ok(channel), Ok(command), Ok(response)) = (
                        serde_json::from_value::<String>(event.payload["channel"].clone()),
                        serde_json::from_value::<String>(event.payload["command"].clone()),
                        serde_json::from_value::<String>(event.payload["response"].clone()),
                    ) {
                        log::info!("[Twitch] Sending text command response for !{} in {}: {}", command, channel, response);

                        // Emit event to send the message (will be picked up by WebSocket/frontend)
                        ctx_text_commands.emit("twitch.send_message", &serde_json::json!({
                            "channel": channel,
                            "message": response,
                            "type": "text_command"
                        }));
                    }
                }
            });

            // Mark listeners as initialized after all listeners are created
            LISTENERS_INITIALIZED.store(true, Ordering::Relaxed);
            log::info!("[Twitch] Event listeners initialized");
        }

        log::info!("[Twitch] Plugin started successfully");
        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        log::info!("[Twitch] Stopping plugin...");
        // NOTE: Cleanup IRC connection, EventSub subscriptions, etc.
        Ok(())
    }
}

fn current_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}
