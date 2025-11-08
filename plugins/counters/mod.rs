use crate::core::plugin::{Plugin, PluginMetadata};
use crate::core::plugin_context::PluginContext;
use async_trait::async_trait;
use std::sync::Arc;
use anyhow::Result;
use serde_json::json;

mod router;

pub struct CountersPlugin;

#[async_trait]
impl Plugin for CountersPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            id: "counters".to_string(),
            name: "Counters System".to_string(),
            version: "1.0.0".to_string(),
            description: "Simple counter tracking for streams".to_string(),
            author: "WebArcade Team".to_string(),
            dependencies: vec![],
        }
    }

    async fn init(&self, ctx: &PluginContext) -> Result<()> {
        log::info!("[Counters] Initializing plugin...");

        // Note: Table already exists in database with last_updated column
        // Migration is kept for reference but won't modify existing table
        ctx.migrate(&[
            r#"
            CREATE TABLE IF NOT EXISTS counters (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                channel TEXT NOT NULL,
                task TEXT NOT NULL,
                count INTEGER NOT NULL DEFAULT 0,
                last_updated INTEGER NOT NULL,
                UNIQUE(channel, task)
            );

            CREATE INDEX IF NOT EXISTS idx_counters_channel ON counters(channel);
            CREATE INDEX IF NOT EXISTS idx_channel_task ON counters(channel, task);
            "#,
        ])?;

        // Register services

        // Get all counters for a channel
        ctx.provide_service("get_counters", |input| async move {
            log::info!("[Counters] get_counters called with input: {:?}", input);
            let channel: String = serde_json::from_value(input["channel"].clone())?;
            log::info!("[Counters] Querying for channel: {}", channel);

            let db_path = crate::core::database::get_database_path();
            log::info!("[Counters] Using database at: {:?}", db_path);
            let conn = rusqlite::Connection::open(&db_path)?;

            let mut stmt = conn.prepare(
                "SELECT task, count FROM counters WHERE channel = ?1 ORDER BY task ASC"
            )?;

            let counters: Vec<serde_json::Value> = stmt.query_map([&channel], |row| {
                Ok(json!({
                    "task": row.get::<_, String>(0)?,
                    "count": row.get::<_, i64>(1)?
                }))
            })?.collect::<rusqlite::Result<Vec<_>>>()?;

            log::info!("[Counters] Found {} counters", counters.len());
            Ok(json!(counters))
        }).await;

        // Increment counter
        ctx.provide_service("increment_counter", |input| async move {
            let channel: String = serde_json::from_value(input["channel"].clone())?;
            let task: String = serde_json::from_value(input["task"].clone())?;

            let conn = crate::core::database::get_database_path();
            let conn = rusqlite::Connection::open(conn)?;

            let now = current_timestamp();

            // Insert or update
            conn.execute(
                "INSERT INTO counters (channel, task, count, last_updated)
                 VALUES (?1, ?2, 1, ?3)
                 ON CONFLICT(channel, task) DO UPDATE SET
                   count = count + 1,
                   last_updated = ?3",
                rusqlite::params![channel, task, now],
            )?;

            Ok(json!({ "success": true }))
        }).await;

        // Decrement counter
        ctx.provide_service("decrement_counter", |input| async move {
            let channel: String = serde_json::from_value(input["channel"].clone())?;
            let task: String = serde_json::from_value(input["task"].clone())?;

            let conn = crate::core::database::get_database_path();
            let conn = rusqlite::Connection::open(conn)?;

            let now = current_timestamp();

            // Update only if exists and count > 0
            conn.execute(
                "UPDATE counters
                 SET count = MAX(0, count - 1), last_updated = ?3
                 WHERE channel = ?1 AND task = ?2",
                rusqlite::params![channel, task, now],
            )?;

            Ok(json!({ "success": true }))
        }).await;

        // Reset counter
        ctx.provide_service("reset_counter", |input| async move {
            let channel: String = serde_json::from_value(input["channel"].clone())?;
            let task: String = serde_json::from_value(input["task"].clone())?;

            let conn = crate::core::database::get_database_path();
            let conn = rusqlite::Connection::open(conn)?;

            let now = current_timestamp();

            conn.execute(
                "UPDATE counters
                 SET count = 0, last_updated = ?3
                 WHERE channel = ?1 AND task = ?2",
                rusqlite::params![channel, task, now],
            )?;

            Ok(json!({ "success": true }))
        }).await;

        // Register HTTP routes
        router::register_routes(ctx).await?;

        log::info!("[Counters] Plugin initialized successfully");
        Ok(())
    }

    async fn start(&self, _ctx: Arc<PluginContext>) -> Result<()> {
        log::info!("[Counters] Starting plugin...");
        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        log::info!("[Counters] Stopping plugin...");
        Ok(())
    }
}

fn current_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}
