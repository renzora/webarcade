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

pub struct ConfessionsPlugin;

#[async_trait]
impl Plugin for ConfessionsPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            id: "confessions".to_string(),
            name: "Confessions System".to_string(),
            version: "1.0.0".to_string(),
            description: "Anonymous confession submission and moderation".to_string(),
            author: "WebArcade Team".to_string(),
            dependencies: vec![],
        }
    }

    async fn init(&self, ctx: &PluginContext) -> Result<()> {
        log::info!("[Confessions] Initializing plugin...");

        // Migration 1: Create tables if not exists
        ctx.migrate(&[
            r#"
            CREATE TABLE IF NOT EXISTS confessions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                confession_text TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'pending',
                submitted_by TEXT,
                submitted_at INTEGER NOT NULL,
                approved_at INTEGER,
                approved_by TEXT,
                rejected_at INTEGER,
                rejected_by TEXT,
                rejection_reason TEXT,
                displayed_at INTEGER,
                display_count INTEGER NOT NULL DEFAULT 0
            );

            CREATE TABLE IF NOT EXISTS confession_reactions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                confession_id INTEGER NOT NULL,
                user_id TEXT NOT NULL,
                reaction_type TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                UNIQUE(confession_id, user_id, reaction_type),
                FOREIGN KEY (confession_id) REFERENCES confessions(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS confession_settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                updated_at INTEGER NOT NULL
            );
            "#,
        ])?;

        // Migration 2: Add indexes (only if schema matches)
        let conn = crate::core::database::get_database_path();
        if let Ok(conn) = rusqlite::Connection::open(conn) {
            // Check if status column exists
            let has_status = conn.query_row(
                "SELECT COUNT(*) FROM pragma_table_info('confessions') WHERE name = 'status'",
                [],
                |row| row.get::<_, i64>(0)
            ).unwrap_or(0) > 0;

            if has_status {
                let _ = conn.execute("CREATE INDEX IF NOT EXISTS idx_confessions_status ON confessions(status, submitted_at DESC)", []);
                let _ = conn.execute("CREATE INDEX IF NOT EXISTS idx_confession_reactions_confession_id ON confession_reactions(confession_id)", []);
            } else {
                log::warn!("[Confessions] Skipping index creation - status column not found in existing schema");
            }
        }

        // Initialize default settings
        let conn = crate::core::database::get_database_path();
        let conn = rusqlite::Connection::open(&conn)?;
        let now = current_timestamp();

        let _ = conn.execute(
            "INSERT OR IGNORE INTO confession_settings (key, value, updated_at) VALUES ('enabled', 'true', ?1)",
            rusqlite::params![now],
        );

        let _ = conn.execute(
            "INSERT OR IGNORE INTO confession_settings (key, value, updated_at) VALUES ('require_moderation', 'true', ?1)",
            rusqlite::params![now],
        );

        let _ = conn.execute(
            "INSERT OR IGNORE INTO confession_settings (key, value, updated_at) VALUES ('min_length', '10', ?1)",
            rusqlite::params![now],
        );

        let _ = conn.execute(
            "INSERT OR IGNORE INTO confession_settings (key, value, updated_at) VALUES ('max_length', '500', ?1)",
            rusqlite::params![now],
        );

        drop(conn);

        // Register HTTP routes
        router::register_routes(ctx).await?;

        // Service: Submit confession
        ctx.provide_service("submit_confession", |input| async move {
            let confession_text: String = serde_json::from_value(input["confession_text"].clone())?;
            let submitted_by: Option<String> = serde_json::from_value(input["submitted_by"].clone()).ok();

            let conn = crate::core::database::get_database_path();
            let conn = rusqlite::Connection::open(conn)?;

            // Get settings
            let require_moderation: bool = conn.query_row(
                "SELECT value FROM confession_settings WHERE key = 'require_moderation'",
                [],
                |row| row.get::<_, String>(0),
            ).map(|v| v == "true").unwrap_or(true);

            let min_length: i64 = conn.query_row(
                "SELECT value FROM confession_settings WHERE key = 'min_length'",
                [],
                |row| row.get::<_, String>(0),
            ).ok().and_then(|v| v.parse().ok()).unwrap_or(10);

            let max_length: i64 = conn.query_row(
                "SELECT value FROM confession_settings WHERE key = 'max_length'",
                [],
                |row| row.get::<_, String>(0),
            ).ok().and_then(|v| v.parse().ok()).unwrap_or(500);

            // Validate length
            let text_len = confession_text.len() as i64;
            if text_len < min_length {
                return Err(anyhow::anyhow!("Confession too short (minimum {} characters)", min_length));
            }
            if text_len > max_length {
                return Err(anyhow::anyhow!("Confession too long (maximum {} characters)", max_length));
            }

            let now = current_timestamp();
            let status = if require_moderation { "pending" } else { "approved" };

            conn.execute(
                "INSERT INTO confessions (confession_text, status, submitted_by, submitted_at, approved_at)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                rusqlite::params![
                    confession_text,
                    status,
                    submitted_by,
                    now,
                    if require_moderation { None } else { Some(now) }
                ],
            )?;

            let id = conn.last_insert_rowid();
            Ok(serde_json::json!({
                "id": id,
                "success": true,
                "status": status
            }))
        }).await;

        // Service: Get pending confessions (for moderation)
        ctx.provide_service("get_pending_confessions", |_input| async move {
            let conn = crate::core::database::get_database_path();
            let conn = rusqlite::Connection::open(conn)?;

            let mut stmt = conn.prepare(
                "SELECT id, confession_text, submitted_at FROM confessions
                 WHERE status = 'pending'
                 ORDER BY submitted_at ASC"
            )?;

            let confessions: Vec<serde_json::Value> = stmt.query_map([], |row| {
                Ok(serde_json::json!({
                    "id": row.get::<_, i64>(0)?,
                    "confession_text": row.get::<_, String>(1)?,
                    "submitted_at": row.get::<_, i64>(2)?,
                }))
            })?.collect::<rusqlite::Result<Vec<_>>>()?;

            Ok(serde_json::json!({ "confessions": confessions }))
        }).await;

        // Service: Approve confession
        ctx.provide_service("approve_confession", |input| async move {
            let id: i64 = serde_json::from_value(input["id"].clone())?;
            let approved_by: String = serde_json::from_value(input["approved_by"].clone())?;

            let conn = crate::core::database::get_database_path();
            let conn = rusqlite::Connection::open(conn)?;

            let now = current_timestamp();
            conn.execute(
                "UPDATE confessions SET status = 'approved', approved_at = ?1, approved_by = ?2
                 WHERE id = ?3 AND status = 'pending'",
                rusqlite::params![now, approved_by, id],
            )?;

            Ok(serde_json::json!({ "success": true }))
        }).await;

        // Service: Reject confession
        ctx.provide_service("reject_confession", |input| async move {
            let id: i64 = serde_json::from_value(input["id"].clone())?;
            let rejected_by: String = serde_json::from_value(input["rejected_by"].clone())?;
            let rejection_reason: Option<String> = serde_json::from_value(input["rejection_reason"].clone()).ok();

            let conn = crate::core::database::get_database_path();
            let conn = rusqlite::Connection::open(conn)?;

            let now = current_timestamp();
            conn.execute(
                "UPDATE confessions SET status = 'rejected', rejected_at = ?1, rejected_by = ?2, rejection_reason = ?3
                 WHERE id = ?4 AND status = 'pending'",
                rusqlite::params![now, rejected_by, rejection_reason, id],
            )?;

            Ok(serde_json::json!({ "success": true }))
        }).await;

        // Service: Get random approved confession
        ctx.provide_service("get_random_confession", |_input| async move {
            let conn = crate::core::database::get_database_path();
            let conn = rusqlite::Connection::open(conn)?;

            let confession: Option<serde_json::Value> = conn.query_row(
                "SELECT id, confession_text, display_count FROM confessions
                 WHERE status = 'approved'
                 ORDER BY RANDOM()
                 LIMIT 1",
                [],
                |row| {
                    Ok(serde_json::json!({
                        "id": row.get::<_, i64>(0)?,
                        "confession_text": row.get::<_, String>(1)?,
                        "display_count": row.get::<_, i64>(2)?,
                    }))
                }
            ).optional()?;

            // Increment display count
            if let Some(ref conf) = confession {
                if let Some(id) = conf["id"].as_i64() {
                    let now = current_timestamp();
                    conn.execute(
                        "UPDATE confessions SET display_count = display_count + 1, displayed_at = ?1
                         WHERE id = ?2",
                        rusqlite::params![now, id],
                    )?;
                }
            }

            Ok(serde_json::json!({ "confession": confession }))
        }).await;

        // Service: Add reaction to confession
        ctx.provide_service("add_reaction", |input| async move {
            let confession_id: i64 = serde_json::from_value(input["confession_id"].clone())?;
            let user_id: String = serde_json::from_value(input["user_id"].clone())?;
            let reaction_type: String = serde_json::from_value(input["reaction_type"].clone())?;

            let conn = crate::core::database::get_database_path();
            let conn = rusqlite::Connection::open(conn)?;

            let now = current_timestamp();
            conn.execute(
                "INSERT OR IGNORE INTO confession_reactions (confession_id, user_id, reaction_type, created_at)
                 VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params![confession_id, user_id, reaction_type, now],
            )?;

            Ok(serde_json::json!({ "success": true }))
        }).await;

        // Service: Get confession stats
        ctx.provide_service("get_stats", |_input| async move {
            let conn = crate::core::database::get_database_path();
            let conn = rusqlite::Connection::open(conn)?;

            let total: i64 = conn.query_row("SELECT COUNT(*) FROM confessions", [], |row| row.get(0))?;
            let pending: i64 = conn.query_row("SELECT COUNT(*) FROM confessions WHERE status = 'pending'", [], |row| row.get(0))?;
            let approved: i64 = conn.query_row("SELECT COUNT(*) FROM confessions WHERE status = 'approved'", [], |row| row.get(0))?;
            let rejected: i64 = conn.query_row("SELECT COUNT(*) FROM confessions WHERE status = 'rejected'", [], |row| row.get(0))?;

            Ok(serde_json::json!({
                "total": total,
                "pending": pending,
                "approved": approved,
                "rejected": rejected
            }))
        }).await;

        // Service: Update settings
        ctx.provide_service("update_setting", |input| async move {
            let key: String = serde_json::from_value(input["key"].clone())?;
            let value: String = serde_json::from_value(input["value"].clone())?;

            let conn = crate::core::database::get_database_path();
            let conn = rusqlite::Connection::open(conn)?;

            let now = current_timestamp();
            conn.execute(
                "INSERT OR REPLACE INTO confession_settings (key, value, updated_at) VALUES (?1, ?2, ?3)",
                rusqlite::params![key, value, now],
            )?;

            Ok(serde_json::json!({ "success": true }))
        }).await;

        log::info!("[Confessions] Plugin initialized successfully");
        Ok(())
    }

    async fn start(&self, ctx: Arc<PluginContext>) -> Result<()> {
        log::info!("[Confessions] Starting plugin...");

        // Subscribe to confession submission events
        let ctx_clone = ctx.clone();
        tokio::spawn(async move {
            let mut events = ctx_clone.subscribe_to("confessions.submit_request").await;

            while let Ok(event) = events.recv().await {
                if let Ok(confession_text) = serde_json::from_value::<String>(event.payload["confession_text"].clone()) {
                    let submitted_by = serde_json::from_value::<String>(event.payload["submitted_by"].clone()).ok();

                    if let Ok(result) = ctx_clone.call_service("confessions", "submit_confession", serde_json::json!({
                        "confession_text": confession_text,
                        "submitted_by": submitted_by
                    })).await {
                        if result["success"].as_bool() == Some(true) {
                            let status = result["status"].as_str().unwrap_or("pending");
                            let id = result["id"].as_i64().unwrap_or(0);

                            ctx_clone.emit("confessions.submitted", &serde_json::json!({
                                "id": id,
                                "status": status,
                                "submitted_by": submitted_by
                            }));

                            // If auto-approved, emit approved event
                            if status == "approved" {
                                ctx_clone.emit("confessions.approved", &serde_json::json!({
                                    "id": id,
                                    "auto_approved": true
                                }));
                            }
                        }
                    }
                }
            }
        });

        log::info!("[Confessions] Plugin started successfully");
        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        log::info!("[Confessions] Stopping plugin...");
        Ok(())
    }
}

fn current_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}
