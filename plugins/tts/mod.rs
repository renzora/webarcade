use crate::core::plugin::{Plugin, PluginMetadata};
use crate::core::plugin_context::PluginContext;
use async_trait::async_trait;
use std::sync::Arc;
use anyhow::Result;
use rusqlite::OptionalExtension;

mod database;
mod events;
mod router;

pub use events::*;

pub struct TtsPlugin;

/// Available StreamElements TTS voices
const VOICES: &[(&str, &str)] = &[
    // English voices
    ("Brian", "English (UK) - Male"),
    ("Amy", "English (UK) - Female"),
    ("Emma", "English (UK) - Female"),
    ("Geraint", "English (Welsh) - Male"),
    ("Russell", "English (Australian) - Male"),
    ("Nicole", "English (Australian) - Female"),
    ("Joey", "English (US) - Male"),
    ("Justin", "English (US) - Male (Child)"),
    ("Matthew", "English (US) - Male"),
    ("Ivy", "English (US) - Female (Child)"),
    ("Joanna", "English (US) - Female"),
    ("Kendra", "English (US) - Female"),
    ("Kimberly", "English (US) - Female"),
    ("Salli", "English (US) - Female"),
    ("Raveena", "English (Indian) - Female"),

    // Other languages
    ("Cristiano", "Portuguese (European) - Male"),
    ("Ines", "Portuguese (European) - Female"),
    ("Vitoria", "Portuguese (Brazilian) - Female"),
    ("Ricardo", "Portuguese (Brazilian) - Male"),
    ("Mizuki", "Japanese - Female"),
    ("Takumi", "Japanese - Male"),
    ("Seoyeon", "Korean - Female"),
    ("Liv", "Norwegian - Female"),
    ("Lotte", "Dutch - Female"),
    ("Ruben", "Dutch - Male"),
    ("Jacek", "Polish - Male"),
    ("Jan", "Polish - Male"),
    ("Ewa", "Polish - Female"),
    ("Maja", "Polish - Female"),
    ("Filiz", "Turkish - Female"),
    ("Astrid", "Swedish - Female"),
    ("Maxim", "Russian - Male"),
    ("Tatyana", "Russian - Female"),
    ("Carmen", "Romanian - Female"),
    ("Gwyneth", "Welsh - Female"),
    ("Mads", "Danish - Male"),
    ("Naja", "Danish - Female"),
    ("Hans", "German - Male"),
    ("Marlene", "German - Female"),
    ("Vicki", "German - Female"),
    ("Karl", "Icelandic - Male"),
    ("Dora", "Icelandic - Female"),
    ("Giorgio", "Italian - Male"),
    ("Carla", "Italian - Female"),
    ("Bianca", "Italian - Female"),
    ("Celine", "French - Female"),
    ("Lea", "French - Female"),
    ("Mathieu", "French - Male"),
    ("Chantal", "French (Canadian) - Female"),
    ("Penelope", "Spanish (US) - Female"),
    ("Miguel", "Spanish (US) - Male"),
    ("Enrique", "Spanish (European) - Male"),
    ("Conchita", "Spanish (European) - Female"),
    ("Lucia", "Spanish (European) - Female"),
];

/// Get voice name by partial match (case insensitive)
fn find_voice(query: &str) -> Option<&'static str> {
    let query_lower = query.to_lowercase();

    // Exact match first
    if let Some((name, _)) = VOICES.iter().find(|(name, _)| name.to_lowercase() == query_lower) {
        return Some(name);
    }

    // Partial match
    VOICES.iter()
        .find(|(name, desc)| {
            name.to_lowercase().contains(&query_lower) ||
            desc.to_lowercase().contains(&query_lower)
        })
        .map(|(name, _)| *name)
}

#[async_trait]
impl Plugin for TtsPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            id: "tts".to_string(),
            name: "Text-to-Speech".to_string(),
            version: "1.0.0".to_string(),
            description: "Text-to-speech engine with multiple voices and queuing".to_string(),
            author: "WebArcade Team".to_string(),
            dependencies: vec![],
        }
    }

    async fn init(&self, ctx: &PluginContext) -> Result<()> {
        log::info!("[TTS] Initializing plugin...");

        ctx.migrate(&[
            r#"
            CREATE TABLE IF NOT EXISTS tts_queue (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                text TEXT NOT NULL,
                voice TEXT NOT NULL DEFAULT 'default',
                priority INTEGER NOT NULL DEFAULT 0,
                status TEXT NOT NULL DEFAULT 'pending',
                requested_by TEXT,
                created_at INTEGER NOT NULL,
                started_at INTEGER,
                completed_at INTEGER
            );

            CREATE TABLE IF NOT EXISTS tts_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                text TEXT NOT NULL,
                voice TEXT NOT NULL,
                requested_by TEXT,
                duration_ms INTEGER,
                created_at INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS tts_voices (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                voice_id TEXT NOT NULL UNIQUE,
                voice_name TEXT NOT NULL,
                language TEXT NOT NULL,
                engine TEXT NOT NULL,
                enabled INTEGER NOT NULL DEFAULT 1,
                created_at INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS tts_settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                updated_at INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS tts_whitelist (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                channel TEXT NOT NULL,
                username TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                UNIQUE(channel, username)
            );

            CREATE TABLE IF NOT EXISTS tts_channel_settings (
                channel TEXT PRIMARY KEY,
                enabled INTEGER NOT NULL DEFAULT 0,
                mode TEXT NOT NULL DEFAULT 'broadcaster'
            );

            CREATE TABLE IF NOT EXISTS tts_users (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                channel TEXT NOT NULL,
                username TEXT NOT NULL,
                voice TEXT NOT NULL DEFAULT 'Brian',
                UNIQUE(channel, username)
            );

            CREATE INDEX IF NOT EXISTS idx_tts_queue_status ON tts_queue(status, priority DESC, created_at);
            CREATE INDEX IF NOT EXISTS idx_tts_history_created_at ON tts_history(created_at DESC);
            CREATE INDEX IF NOT EXISTS idx_tts_whitelist_channel ON tts_whitelist(channel);
            CREATE INDEX IF NOT EXISTS idx_tts_users ON tts_users(channel, username);
            "#,
        ])?;

        // Initialize default settings
        let conn = crate::core::database::get_database_path();
        let conn = rusqlite::Connection::open(&conn)?;
        let now = current_timestamp();

        let _ = conn.execute(
            "INSERT OR IGNORE INTO tts_settings (key, value, updated_at) VALUES ('enabled', 'true', ?1)",
            rusqlite::params![now],
        );

        let _ = conn.execute(
            "INSERT OR IGNORE INTO tts_settings (key, value, updated_at) VALUES ('default_voice', 'default', ?1)",
            rusqlite::params![now],
        );

        let _ = conn.execute(
            "INSERT OR IGNORE INTO tts_settings (key, value, updated_at) VALUES ('volume', '0.7', ?1)",
            rusqlite::params![now],
        );

        let _ = conn.execute(
            "INSERT OR IGNORE INTO tts_settings (key, value, updated_at) VALUES ('rate', '1.0', ?1)",
            rusqlite::params![now],
        );

        // Register default voices
        let _ = conn.execute(
            "INSERT OR IGNORE INTO tts_voices (voice_id, voice_name, language, engine, enabled, created_at)
             VALUES ('default', 'Default Voice', 'en-US', 'system', 1, ?1)",
            rusqlite::params![now],
        );

        drop(conn);

        // Service: Add to TTS queue
        ctx.provide_service("speak", |input| async move {
            let text: String = serde_json::from_value(input["text"].clone())?;
            let voice: String = serde_json::from_value(input["voice"].clone()).unwrap_or_else(|_| "default".to_string());
            let priority: i64 = serde_json::from_value(input["priority"].clone()).unwrap_or(0);
            let requested_by: Option<String> = serde_json::from_value(input["requested_by"].clone()).ok();

            let conn = crate::core::database::get_database_path();
            let conn = rusqlite::Connection::open(conn)?;

            let now = current_timestamp();
            conn.execute(
                "INSERT INTO tts_queue (text, voice, priority, status, requested_by, created_at)
                 VALUES (?1, ?2, ?3, 'pending', ?4, ?5)",
                rusqlite::params![text, voice, priority, requested_by, now],
            )?;

            let id = conn.last_insert_rowid();
            Ok(serde_json::json!({ "id": id, "success": true }))
        }).await;

        // Service: Get next from queue
        ctx.provide_service("get_next_tts", |_input| async move {
            let conn = crate::core::database::get_database_path();
            let conn = rusqlite::Connection::open(conn)?;

            // Get highest priority pending item
            let result: Option<serde_json::Value> = conn.query_row(
                "SELECT id, text, voice, requested_by FROM tts_queue
                 WHERE status = 'pending'
                 ORDER BY priority DESC, created_at ASC
                 LIMIT 1",
                [],
                |row| {
                    Ok(serde_json::json!({
                        "id": row.get::<_, i64>(0)?,
                        "text": row.get::<_, String>(1)?,
                        "voice": row.get::<_, String>(2)?,
                        "requested_by": row.get::<_, Option<String>>(3)?,
                    }))
                }
            ).optional()?;

            // Mark as processing
            if let Some(ref item) = result {
                if let Some(id) = item["id"].as_i64() {
                    let now = current_timestamp();
                    conn.execute(
                        "UPDATE tts_queue SET status = 'processing', started_at = ?1 WHERE id = ?2",
                        rusqlite::params![now, id],
                    )?;
                }
            }

            Ok(serde_json::json!({ "item": result }))
        }).await;

        // Service: Mark TTS complete
        ctx.provide_service("complete_tts", |input| async move {
            let id: i64 = serde_json::from_value(input["id"].clone())?;
            let duration_ms: Option<i64> = serde_json::from_value(input["duration_ms"].clone()).ok();

            let conn = crate::core::database::get_database_path();
            let conn = rusqlite::Connection::open(conn)?;

            let now = current_timestamp();

            // Get item details before updating
            let (text, voice, requested_by): (String, String, Option<String>) = conn.query_row(
                "SELECT text, voice, requested_by FROM tts_queue WHERE id = ?1",
                rusqlite::params![id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )?;

            // Mark as complete
            conn.execute(
                "UPDATE tts_queue SET status = 'completed', completed_at = ?1 WHERE id = ?2",
                rusqlite::params![now, id],
            )?;

            // Add to history
            conn.execute(
                "INSERT INTO tts_history (text, voice, requested_by, duration_ms, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                rusqlite::params![text, voice, requested_by, duration_ms, now],
            )?;

            // Clean up old queue items (completed > 1 hour ago)
            let one_hour_ago = now - 3600;
            conn.execute(
                "DELETE FROM tts_queue WHERE status = 'completed' AND completed_at < ?1",
                rusqlite::params![one_hour_ago],
            )?;

            Ok(serde_json::json!({ "success": true }))
        }).await;

        // Service: Get queue status
        ctx.provide_service("get_queue_status", |_input| async move {
            let conn = crate::core::database::get_database_path();
            let conn = rusqlite::Connection::open(conn)?;

            let pending: i64 = conn.query_row(
                "SELECT COUNT(*) FROM tts_queue WHERE status = 'pending'",
                [],
                |row| row.get(0),
            )?;

            let processing: i64 = conn.query_row(
                "SELECT COUNT(*) FROM tts_queue WHERE status = 'processing'",
                [],
                |row| row.get(0),
            )?;

            Ok(serde_json::json!({
                "pending": pending,
                "processing": processing,
                "total": pending + processing
            }))
        }).await;

        // Service: Get available voices
        ctx.provide_service("get_voices", |_input| async move {
            let conn = crate::core::database::get_database_path();
            let conn = rusqlite::Connection::open(conn)?;

            let mut stmt = conn.prepare(
                "SELECT voice_id, voice_name, language, engine FROM tts_voices WHERE enabled = 1"
            )?;

            let voices: Vec<serde_json::Value> = stmt.query_map([], |row| {
                Ok(serde_json::json!({
                    "voice_id": row.get::<_, String>(0)?,
                    "voice_name": row.get::<_, String>(1)?,
                    "language": row.get::<_, String>(2)?,
                    "engine": row.get::<_, String>(3)?,
                }))
            })?.collect::<rusqlite::Result<Vec<_>>>()?;

            Ok(serde_json::json!({ "voices": voices }))
        }).await;

        // Service: Update TTS settings
        ctx.provide_service("update_setting", |input| async move {
            let key: String = serde_json::from_value(input["key"].clone())?;
            let value: String = serde_json::from_value(input["value"].clone())?;

            let conn = crate::core::database::get_database_path();
            let conn = rusqlite::Connection::open(conn)?;

            let now = current_timestamp();
            conn.execute(
                "INSERT OR REPLACE INTO tts_settings (key, value, updated_at) VALUES (?1, ?2, ?3)",
                rusqlite::params![key, value, now],
            )?;

            Ok(serde_json::json!({ "success": true }))
        }).await;

        // Service: Get TTS settings
        ctx.provide_service("get_settings", |_input| async move {
            let conn = crate::core::database::get_database_path();
            let conn = rusqlite::Connection::open(conn)?;

            let mut stmt = conn.prepare("SELECT key, value FROM tts_settings")?;
            let settings: Vec<(String, String)> = stmt.query_map([], |row| {
                Ok((row.get(0)?, row.get(1)?))
            })?.collect::<rusqlite::Result<Vec<_>>>()?;

            let mut settings_map = serde_json::Map::new();
            for (key, value) in settings {
                settings_map.insert(key, serde_json::Value::String(value));
            }

            Ok(serde_json::json!({ "settings": settings_map }))
        }).await;

        // Whitelist management services
        ctx.provide_service("get_whitelist_users", |input| async move {
            let channel: String = serde_json::from_value(input["channel"].clone())?;

            let conn = crate::core::database::get_database_path();
            let conn = rusqlite::Connection::open(conn)?;

            let mut stmt = conn.prepare(
                "SELECT username FROM tts_whitelist WHERE channel = ?1 ORDER BY username ASC"
            )?;

            let users: Vec<String> = stmt.query_map([&channel], |row| {
                row.get(0)
            })?.collect::<rusqlite::Result<Vec<_>>>()?;

            Ok(serde_json::json!(users))
        }).await;

        ctx.provide_service("add_whitelist_user", |input| async move {
            let channel: String = serde_json::from_value(input["channel"].clone())?;
            let username: String = serde_json::from_value(input["username"].clone())?;

            let conn = crate::core::database::get_database_path();
            let conn = rusqlite::Connection::open(conn)?;

            let now = current_timestamp();

            conn.execute(
                "INSERT OR IGNORE INTO tts_whitelist (channel, username, created_at) VALUES (?1, ?2, ?3)",
                rusqlite::params![channel, username, now],
            )?;

            Ok(serde_json::json!({ "success": true }))
        }).await;

        ctx.provide_service("remove_whitelist_user", |input| async move {
            let channel: String = serde_json::from_value(input["channel"].clone())?;
            let username: String = serde_json::from_value(input["username"].clone())?;

            let conn = crate::core::database::get_database_path();
            let conn = rusqlite::Connection::open(conn)?;

            conn.execute(
                "DELETE FROM tts_whitelist WHERE channel = ?1 AND username = ?2",
                rusqlite::params![channel, username],
            )?;

            Ok(serde_json::json!({ "success": true }))
        }).await;

        // Channel settings services
        ctx.provide_service("get_channel_settings", |input| async move {
            let channel: String = serde_json::from_value(input["channel"].clone())?;

            let conn = crate::core::database::get_database_path();
            let conn = rusqlite::Connection::open(conn)?;

            let result: Option<(i64, String)> = conn.query_row(
                "SELECT enabled, mode FROM tts_channel_settings WHERE channel = ?1",
                [&channel],
                |row| Ok((row.get(0)?, row.get(1)?))
            ).optional()?;

            let (enabled, mode) = result.unwrap_or((0, "broadcaster".to_string()));

            Ok(serde_json::json!({
                "enabled": enabled != 0,
                "mode": mode
            }))
        }).await;

        ctx.provide_service("update_channel_settings", |input| async move {
            let channel: String = serde_json::from_value(input["channel"].clone())?;
            let enabled: bool = serde_json::from_value(input["enabled"].clone())?;
            let mode: String = serde_json::from_value(input["mode"].clone())?;

            let conn = crate::core::database::get_database_path();
            let conn = rusqlite::Connection::open(conn)?;

            conn.execute(
                "INSERT OR REPLACE INTO tts_channel_settings (channel, enabled, mode)
                 VALUES (?1, ?2, ?3)",
                rusqlite::params![channel, enabled as i64, mode],
            )?;

            Ok(serde_json::json!({ "success": true }))
        }).await;

        // User voice preference services
        ctx.provide_service("set_user_voice", |input| async move {
            let channel: String = serde_json::from_value(input["channel"].clone())?;
            let username: String = serde_json::from_value(input["username"].clone())?;
            let voice: String = serde_json::from_value(input["voice"].clone())?;

            let conn = crate::core::database::get_database_path();
            let conn = rusqlite::Connection::open(conn)?;

            conn.execute(
                "INSERT OR REPLACE INTO tts_users (channel, username, voice)
                 VALUES (?1, ?2, ?3)",
                rusqlite::params![channel, username, voice],
            )?;

            Ok(serde_json::json!({ "success": true }))
        }).await;

        ctx.provide_service("get_user_voice", |input| async move {
            let channel: String = serde_json::from_value(input["channel"].clone())?;
            let username: String = serde_json::from_value(input["username"].clone())?;

            let conn = crate::core::database::get_database_path();
            let conn = rusqlite::Connection::open(conn)?;

            let voice: Option<String> = conn.query_row(
                "SELECT voice FROM tts_users WHERE channel = ?1 AND username = ?2",
                rusqlite::params![channel, username],
                |row| row.get(0),
            ).optional()?;

            Ok(serde_json::json!({ "voice": voice.unwrap_or_else(|| "Brian".to_string()) }))
        }).await;

        // Register HTTP routes
        router::register_routes(ctx).await?;

        log::info!("[TTS] Plugin initialized successfully");
        Ok(())
    }

    async fn start(&self, ctx: Arc<PluginContext>) -> Result<()> {
        log::info!("[TTS] Starting plugin...");

        // Subscribe to TTS request events
        let ctx_clone = ctx.clone();
        tokio::spawn(async move {
            let mut events = ctx_clone.subscribe_to("tts.request").await;

            while let Ok(event) = events.recv().await {
                if let Ok(text) = serde_json::from_value::<String>(event.payload["text"].clone()) {
                    let voice = serde_json::from_value::<String>(event.payload["voice"].clone()).unwrap_or_else(|_| "default".to_string());
                    let priority = serde_json::from_value::<i64>(event.payload["priority"].clone()).unwrap_or(0);
                    let requested_by = serde_json::from_value::<String>(event.payload["requested_by"].clone()).ok();

                    // Add to queue
                    if let Ok(result) = ctx_clone.call_service("tts", "speak", serde_json::json!({
                        "text": text,
                        "voice": voice,
                        "priority": priority,
                        "requested_by": requested_by
                    })).await {
                        if let Some(id) = result["id"].as_i64() {
                            // Emit queued event
                            ctx_clone.emit("tts.queued", &serde_json::json!({
                                "id": id,
                                "text": text,
                                "voice": voice
                            }));
                        }
                    }
                }
            }
        });

        // Subscribe to chat messages for TTS commands and auto-TTS
        let ctx_chat = ctx.clone();
        tokio::spawn(async move {
            let mut events = ctx_chat.subscribe_to("twitch.chat_message").await;

            while let Ok(event) = events.recv().await {
                let channel = match serde_json::from_value::<String>(event.payload["channel"].clone()) {
                    Ok(c) => c,
                    Err(_) => continue,
                };

                let message = match serde_json::from_value::<String>(event.payload["message"].clone()) {
                    Ok(m) => m,
                    Err(_) => continue,
                };

                let username = match serde_json::from_value::<String>(event.payload["username"].clone()) {
                    Ok(u) => u,
                    Err(_) => continue,
                };

                // Use the boolean flags provided in the event
                let is_broadcaster = event.payload["is_broadcaster"].as_bool().unwrap_or(false);
                let is_mod = event.payload["is_mod"].as_bool().unwrap_or(false);

                // Handle TTS commands
                if message.starts_with("!tts ") {
                    let parts: Vec<&str> = message.split_whitespace().collect();
                    if parts.len() < 2 {
                        continue;
                    }

                    let subcommand = parts[1];

                    // Only broadcaster and mods can use TTS commands
                    if !is_broadcaster && !is_mod {
                        log::info!("[TTS] User {} is not authorized to use TTS commands", username);
                        continue;
                    }

                    match subcommand {
                        "on" => {
                            log::info!("[TTS] Enabling TTS for channel {}", channel);
                            let _ = ctx_chat.call_service("tts", "update_channel_settings", serde_json::json!({
                                "channel": channel,
                                "enabled": true,
                                "mode": "whitelist"
                            })).await;

                            // Send confirmation to chat
                            ctx_chat.emit("twitch.send_message", &serde_json::json!({
                                "channel": channel,
                                "message": "TTS has been enabled!"
                            }));
                        }
                        "off" => {
                            log::info!("[TTS] Disabling TTS for channel {}", channel);
                            let _ = ctx_chat.call_service("tts", "update_channel_settings", serde_json::json!({
                                "channel": channel,
                                "enabled": false,
                                "mode": "whitelist"
                            })).await;

                            // Send confirmation to chat
                            ctx_chat.emit("twitch.send_message", &serde_json::json!({
                                "channel": channel,
                                "message": "TTS has been disabled!"
                            }));
                        }
                        "add" => {
                            if parts.len() < 3 {
                                log::warn!("[TTS] !tts add command requires username parameter");
                                continue;
                            }

                            let target_username = parts[2].trim_start_matches('@').to_lowercase();
                            log::info!("[TTS] Adding {} to TTS whitelist for channel {}", target_username, channel);

                            let _ = ctx_chat.call_service("tts", "add_whitelist_user", serde_json::json!({
                                "channel": channel,
                                "username": target_username
                            })).await;

                            // Send confirmation to chat
                            ctx_chat.emit("twitch.send_message", &serde_json::json!({
                                "channel": channel,
                                "message": format!("Added {} to TTS whitelist!", target_username)
                            }));
                        }
                        "remove" => {
                            if parts.len() < 3 {
                                log::warn!("[TTS] !tts remove command requires username parameter");
                                continue;
                            }

                            let target_username = parts[2].trim_start_matches('@').to_lowercase();
                            log::info!("[TTS] Removing {} from TTS whitelist for channel {}", target_username, channel);

                            let _ = ctx_chat.call_service("tts", "remove_whitelist_user", serde_json::json!({
                                "channel": channel,
                                "username": target_username
                            })).await;

                            // Send confirmation to chat
                            ctx_chat.emit("twitch.send_message", &serde_json::json!({
                                "channel": channel,
                                "message": format!("Removed {} from TTS whitelist!", target_username)
                            }));
                        }
                        _ => {
                            log::debug!("[TTS] Unknown TTS subcommand: {}", subcommand);
                        }
                    }
                } else if message == "!voices" || message.starts_with("!voices ") {
                    // List available TTS voices (available to everyone)
                    let parts: Vec<&str> = message.split_whitespace().collect();
                    let filter = parts.get(1).map(|s| s.to_lowercase());

                    log::info!("[TTS] User {} requested voice list (filter: {:?})", username, filter);

                    let filtered_voices: Vec<(&str, &str)> = VOICES.iter()
                        .filter(|(name, desc)| {
                            if let Some(ref f) = filter {
                                desc.to_lowercase().contains(f) || name.to_lowercase().contains(f)
                            } else {
                                true
                            }
                        })
                        .take(20)
                        .copied()
                        .collect();

                    if filtered_voices.is_empty() {
                        ctx_chat.emit("twitch.send_message", &serde_json::json!({
                            "channel": channel,
                            "message": format!("@{} No voices found matching '{}'", username, filter.unwrap_or_default())
                        }));
                    } else {
                        // Build message with just voice names to fit in Twitch's 500 char limit
                        let voice_names: Vec<&str> = filtered_voices.iter().map(|(name, _)| *name).collect();

                        // Split into chunks that fit Twitch's message limit (500 chars)
                        let mut current_msg = String::from("🔊 TTS Voices: ");
                        let mut messages = Vec::new();

                        for (i, name) in voice_names.iter().enumerate() {
                            let separator = if i > 0 { ", " } else { "" };
                            let addition = format!("{}{}", separator, name);

                            // Check if adding this voice would exceed ~450 chars (safe limit)
                            if current_msg.len() + addition.len() > 450 {
                                messages.push(current_msg.clone());
                                current_msg = format!("🔊 TTS Voices (cont.): {}", name);
                            } else {
                                current_msg.push_str(&addition);
                            }
                        }

                        if !current_msg.is_empty() {
                            messages.push(current_msg);
                        }

                        // Send all message chunks with small delay
                        for msg in messages {
                            ctx_chat.emit("twitch.send_message", &serde_json::json!({
                                "channel": channel,
                                "message": msg
                            }));
                            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                        }

                        // Send usage tip
                        ctx_chat.emit("twitch.send_message", &serde_json::json!({
                            "channel": channel,
                            "message": format!("@{} Use !voice <name> to set your voice. Filter: !voices english/french/male/female", username)
                        }));
                    }
                } else if message == "!voice" || message.starts_with("!voice ") {
                    // Set or show user's TTS voice (available to everyone)
                    let parts: Vec<&str> = message.split_whitespace().collect();

                    if parts.len() == 1 {
                        // Show current voice
                        match ctx_chat.call_service("tts", "get_user_voice", serde_json::json!({
                            "channel": channel,
                            "username": username
                        })).await {
                            Ok(result) => {
                                if let Some(voice) = result["voice"].as_str() {
                                    // Check if user has custom voice set
                                    let conn = crate::core::database::get_database_path();
                                    let has_custom = if let Ok(conn) = rusqlite::Connection::open(&conn) {
                                        conn.query_row(
                                            "SELECT COUNT(*) FROM tts_users WHERE channel = ?1 AND username = ?2",
                                            rusqlite::params![&channel, &username],
                                            |row| row.get::<_, i64>(0),
                                        ).unwrap_or(0) > 0
                                    } else {
                                        false
                                    };

                                    let status = if has_custom {
                                        format!("Your TTS voice is set to: {}", voice)
                                    } else {
                                        format!("You haven't set a TTS voice yet (using default: {}). Set one to enable TTS!", voice)
                                    };

                                    ctx_chat.emit("twitch.send_message", &serde_json::json!({
                                        "channel": channel,
                                        "message": format!("@{} {}. Use !voice <name> to change it or !voices to see all options.", username, status)
                                    }));
                                }
                            }
                            Err(e) => {
                                log::error!("[TTS] Error getting user voice: {}", e);
                            }
                        }
                    } else {
                        // Set new voice
                        let voice_query = parts[1..].join(" ");

                        match find_voice(&voice_query) {
                            Some(voice_name) => {
                                log::info!("[TTS] User {} setting voice to {}", username, voice_name);

                                match ctx_chat.call_service("tts", "set_user_voice", serde_json::json!({
                                    "channel": channel,
                                    "username": username,
                                    "voice": voice_name
                                })).await {
                                    Ok(_) => {
                                        let voice_desc = VOICES.iter()
                                            .find(|(name, _)| name == &voice_name)
                                            .map(|(_, desc)| *desc)
                                            .unwrap_or("Unknown");

                                        ctx_chat.emit("twitch.send_message", &serde_json::json!({
                                            "channel": channel,
                                            "message": format!("✅ @{} Your TTS voice is now set to: {} ({})", username, voice_name, voice_desc)
                                        }));
                                    }
                                    Err(e) => {
                                        log::error!("[TTS] Error setting voice: {}", e);
                                    }
                                }
                            }
                            None => {
                                ctx_chat.emit("twitch.send_message", &serde_json::json!({
                                    "channel": channel,
                                    "message": format!("@{} Voice '{}' not found. Use !voices to see all available voices.", username, voice_query)
                                }));
                            }
                        }
                    }
                } else {
                    // Auto-TTS for regular messages
                    // Check if TTS is enabled for this channel
                    match ctx_chat.call_service("tts", "get_channel_settings", serde_json::json!({
                        "channel": channel
                    })).await {
                        Ok(settings) => {
                            let enabled = settings["enabled"].as_bool().unwrap_or(false);
                            let mode = settings["mode"].as_str().unwrap_or("broadcaster");

                            if !enabled {
                                continue;
                            }

                            // Check if user is allowed to use TTS based on mode
                            let is_allowed = match mode {
                                "broadcaster" => is_broadcaster,
                                "whitelist" => {
                                    if is_broadcaster || is_mod {
                                        true
                                    } else {
                                        // Check whitelist
                                        match ctx_chat.call_service("tts", "get_whitelist_users", serde_json::json!({
                                            "channel": channel
                                        })).await {
                                            Ok(users) => {
                                                users.as_array()
                                                    .map(|arr| arr.iter().any(|u| u.as_str() == Some(&username)))
                                                    .unwrap_or(false)
                                            }
                                            Err(_) => false,
                                        }
                                    }
                                }
                                "everyone" => true,
                                _ => false,
                            };

                            if is_allowed {
                                log::info!("[TTS] Auto-TTS: {} said: {}", username, message);

                                // Get user's voice preference
                                let conn = crate::core::database::get_database_path();
                                let voice = if let Ok(conn) = rusqlite::Connection::open(&conn) {
                                    conn.query_row(
                                        "SELECT voice FROM tts_users WHERE channel = ?1 AND username = ?2",
                                        rusqlite::params![&channel, &username],
                                        |row| row.get::<_, String>(0),
                                    ).unwrap_or_else(|_| "Brian".to_string())
                                } else {
                                    "Brian".to_string()
                                };

                                // Add to TTS queue
                                let _ = ctx_chat.call_service("tts", "speak", serde_json::json!({
                                    "text": message,
                                    "voice": voice,
                                    "priority": 0,
                                    "requested_by": username
                                })).await;
                            }
                        }
                        Err(e) => {
                            log::error!("[TTS] Error getting channel settings: {}", e);
                        }
                    }
                }
            }
        });

        // Background worker to process TTS queue
        let ctx_worker = ctx.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(500));

            loop {
                interval.tick().await;

                // Get next item from queue
                if let Ok(result) = ctx_worker.call_service("tts", "get_next_tts", serde_json::json!({})).await {
                    if let Some(item) = result["item"].as_object() {
                        if let (Some(id), Some(text), Some(voice)) = (
                            item["id"].as_i64(),
                            item["text"].as_str(),
                            item["voice"].as_str(),
                        ) {
                            // Emit processing event
                            ctx_worker.emit("tts.processing", &serde_json::json!({
                                "id": id,
                                "text": text,
                                "voice": voice
                            }));

                            // Download and play TTS audio
                            let duration_ms = match download_and_play_tts(text, voice).await {
                                Ok(duration) => duration,
                                Err(e) => {
                                    log::error!("[TTS] Failed to play TTS: {}", e);
                                    100 // Default duration on error
                                }
                            };

                            // Mark as complete
                            let _ = ctx_worker.call_service("tts", "complete_tts", serde_json::json!({
                                "id": id,
                                "duration_ms": duration_ms
                            })).await;

                            // Emit completed event
                            ctx_worker.emit("tts.completed", &serde_json::json!({
                                "id": id,
                                "text": text,
                                "voice": voice
                            }));
                        }
                    }
                }
            }
        });

        log::info!("[TTS] Plugin started successfully");
        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        log::info!("[TTS] Stopping plugin...");
        Ok(())
    }
}

pub(crate) fn current_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

/// Download and play TTS audio from StreamElements API
async fn download_and_play_tts(text: &str, voice: &str) -> Result<i64> {
    use std::time::Instant;

    // URL encode the text
    let encoded_text = percent_encoding::utf8_percent_encode(
        text,
        percent_encoding::NON_ALPHANUMERIC
    ).to_string();

    // Use StreamElements TTS API
    let tts_url = format!(
        "https://api.streamelements.com/kappa/v2/speech?voice={}&text={}",
        voice, encoded_text
    );

    log::info!("[TTS] Downloading audio from StreamElements API...");
    let start = Instant::now();

    // Download the audio file
    let response = reqwest::get(&tts_url).await
        .map_err(|e| anyhow::anyhow!("Failed to download TTS audio: {}", e))?;

    let bytes = response.bytes().await
        .map_err(|e| anyhow::anyhow!("Failed to read TTS audio bytes: {}", e))?;

    log::info!("[TTS] Downloaded {} bytes, playing audio...", bytes.len());

    // Create temp directory if it doesn't exist
    let temp_dir = std::path::PathBuf::from("data/temp");
    std::fs::create_dir_all(&temp_dir)
        .map_err(|e| anyhow::anyhow!("Failed to create temp directory: {}", e))?;

    // Save to temp file
    let temp_file = temp_dir.join("tts_current.mp3");
    std::fs::write(&temp_file, bytes)
        .map_err(|e| anyhow::anyhow!("Failed to write temp file: {}", e))?;

    // Play audio using rodio in a blocking task
    let temp_file_clone = temp_file.clone();
    tokio::task::spawn_blocking(move || {
        play_audio_file(&temp_file_clone)
    }).await
        .map_err(|e| anyhow::anyhow!("Failed to spawn audio task: {}", e))??;

    let duration_ms = start.elapsed().as_millis() as i64;
    log::info!("[TTS] Playback completed in {}ms", duration_ms);

    Ok(duration_ms)
}

/// Play audio file using rodio (blocking)
fn play_audio_file(file_path: &std::path::PathBuf) -> Result<()> {
    use std::io::BufReader;
    use rodio::{Decoder, OutputStream, Sink};

    // Get an output stream handle to the default physical sound device
    let (_stream, stream_handle) = OutputStream::try_default()
        .map_err(|e| anyhow::anyhow!("Failed to get audio output: {}", e))?;

    // Create a sink for audio playback
    let sink = Sink::try_new(&stream_handle)
        .map_err(|e| anyhow::anyhow!("Failed to create audio sink: {}", e))?;

    // Load the audio file
    let file = std::fs::File::open(file_path)
        .map_err(|e| anyhow::anyhow!("Failed to open audio file: {}", e))?;
    let buf_reader = BufReader::new(file);

    // Decode the audio file
    let source = Decoder::new(buf_reader)
        .map_err(|e| anyhow::anyhow!("Failed to decode audio: {}", e))?;

    // Play the audio
    sink.append(source);

    // Wait for playback to finish
    sink.sleep_until_end();

    Ok(())
}
