use crate::core::plugin::{Plugin, PluginMetadata};
use crate::core::plugin_context::PluginContext;
use async_trait::async_trait;
use std::sync::Arc;
use anyhow::Result;

mod database;
mod events;
mod router;

pub use events::*;

pub struct TickerPlugin;

#[async_trait]
impl Plugin for TickerPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            id: "ticker".to_string(),
            name: "Ticker System".to_string(),
            version: "1.0.0".to_string(),
            description: "Scrolling ticker with messages, events, and status info".to_string(),
            author: "WebArcade Team".to_string(),
            dependencies: vec![],
        }
    }

    async fn init(&self, ctx: &PluginContext) -> Result<()> {
        log::info!("[Ticker] Initializing plugin...");

        ctx.migrate(&[
            r#"
            CREATE TABLE IF NOT EXISTS ticker_messages (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                message TEXT NOT NULL,
                enabled INTEGER NOT NULL DEFAULT 1,
                is_sticky INTEGER NOT NULL DEFAULT 0,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS ticker_segments (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                type TEXT NOT NULL,
                enabled INTEGER NOT NULL DEFAULT 1,
                content TEXT NOT NULL,
                position INTEGER NOT NULL DEFAULT 0,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS ticker_events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                event_type TEXT NOT NULL,
                event_data TEXT NOT NULL,
                display_text TEXT NOT NULL,
                is_sticky INTEGER NOT NULL DEFAULT 0,
                created_at INTEGER NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_ticker_events_created_at ON ticker_events(created_at DESC);
            "#,
        ])?;

        // Register HTTP routes
        router::register_routes(ctx).await?;

        // Register services
        ctx.provide_service("add_message", |input| async move {
            let message: String = serde_json::from_value(input["message"].clone())?;
            let is_sticky: bool = serde_json::from_value(input["is_sticky"].clone()).unwrap_or(false);

            let conn = crate::core::database::get_database_path();
            let conn = rusqlite::Connection::open(conn)?;

            let now = current_timestamp();
            conn.execute(
                "INSERT INTO ticker_messages (message, enabled, is_sticky, created_at, updated_at)
                 VALUES (?1, 1, ?2, ?3, ?3)",
                rusqlite::params![message, is_sticky as i64, now],
            )?;

            Ok(serde_json::json!({ "id": conn.last_insert_rowid() }))
        }).await;

        ctx.provide_service("get_messages", |_input| async move {
            let conn = crate::core::database::get_database_path();
            let conn = rusqlite::Connection::open(conn)?;

            let mut stmt = conn.prepare(
                "SELECT id, message, enabled, is_sticky, created_at, updated_at
                 FROM ticker_messages WHERE enabled = 1 ORDER BY created_at DESC"
            )?;

            let messages: Vec<serde_json::Value> = stmt.query_map([], |row| {
                Ok(serde_json::json!({
                    "id": row.get::<_, i64>(0)?,
                    "message": row.get::<_, String>(1)?,
                    "enabled": row.get::<_, i64>(2)? != 0,
                    "is_sticky": row.get::<_, i64>(3)? != 0,
                    "created_at": row.get::<_, i64>(4)?,
                    "updated_at": row.get::<_, i64>(5)?,
                }))
            })?.collect::<rusqlite::Result<Vec<_>>>()?;

            Ok(serde_json::json!({ "messages": messages }))
        }).await;

        ctx.provide_service("delete_message", |input| async move {
            let id: i64 = serde_json::from_value(input["id"].clone())?;

            let conn = crate::core::database::get_database_path();
            let conn = rusqlite::Connection::open(conn)?;

            conn.execute("DELETE FROM ticker_messages WHERE id = ?1", rusqlite::params![id])?;
            Ok(serde_json::json!({ "success": true }))
        }).await;

        ctx.provide_service("add_event", |input| async move {
            let event_type: String = serde_json::from_value(input["event_type"].clone())?;
            let event_data: String = serde_json::from_value(input["event_data"].clone())?;
            let display_text: String = serde_json::from_value(input["display_text"].clone())?;
            let is_sticky: bool = serde_json::from_value(input["is_sticky"].clone()).unwrap_or(false);

            let conn = crate::core::database::get_database_path();
            let conn = rusqlite::Connection::open(conn)?;

            let now = current_timestamp();
            conn.execute(
                "INSERT INTO ticker_events (event_type, event_data, display_text, is_sticky, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                rusqlite::params![event_type, event_data, display_text, is_sticky as i64, now],
            )?;

            Ok(serde_json::json!({ "id": conn.last_insert_rowid() }))
        }).await;

        log::info!("[Ticker] Plugin initialized successfully");
        Ok(())
    }

    async fn start(&self, ctx: Arc<PluginContext>) -> Result<()> {
        log::info!("[Ticker] Starting plugin...");

        // Subscribe to Twitch events to add to ticker
        let ctx_clone = ctx.clone();
        tokio::spawn(async move {
            let mut events = ctx_clone.subscribe_to("twitch.follow").await;

            while let Ok(event) = events.recv().await {
                if let Ok(username) = serde_json::from_value::<String>(event.payload["username"].clone()) {
                    let display_text = format!("{} just followed!", username);
                    let _ = ctx_clone.call_service("ticker", "add_event", serde_json::json!({
                        "event_type": "follow",
                        "event_data": serde_json::to_string(&event.payload).unwrap_or_default(),
                        "display_text": display_text,
                        "is_sticky": false
                    })).await;
                }
            }
        });

        log::info!("[Ticker] Plugin started successfully");
        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        log::info!("[Ticker] Stopping plugin...");
        Ok(())
    }
}

fn current_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}
