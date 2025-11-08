use crate::core::plugin::{Plugin, PluginMetadata};
use crate::core::plugin_context::PluginContext;
use async_trait::async_trait;
use std::sync::Arc;
use anyhow::Result;

mod router;

pub struct SongRequestsPlugin;

#[async_trait]
impl Plugin for SongRequestsPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            id: "song_requests".to_string(),
            name: "Song Requests".to_string(),
            version: "1.0.0".to_string(),
            description: "Song request queue management system".to_string(),
            author: "WebArcade Team".to_string(),
            dependencies: vec![],
        }
    }

    async fn init(&self, ctx: &PluginContext) -> Result<()> {
        log::info!("[SongRequests] Initializing plugin...");

        ctx.migrate(&[
            r#"
            CREATE TABLE IF NOT EXISTS song_requests (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                song_query TEXT NOT NULL,
                requester_name TEXT NOT NULL,
                requester_id TEXT,
                status TEXT NOT NULL DEFAULT 'pending',
                requested_at INTEGER NOT NULL,
                played_at INTEGER
            );
            "#,
            r#"
            CREATE INDEX IF NOT EXISTS idx_song_requests_status ON song_requests(status);
            "#,
            r#"
            CREATE INDEX IF NOT EXISTS idx_song_requests_requested_at ON song_requests(requested_at);
            "#,
        ])?;

        // Add columns if they don't exist (migration for existing tables)
        let db_path = crate::core::database::get_database_path();
        if let Ok(conn) = rusqlite::Connection::open(&db_path) {
            let now = current_timestamp();

            // Try to add created_at column (ignore error if already exists)
            let _ = conn.execute(
                &format!("ALTER TABLE song_requests ADD COLUMN created_at INTEGER DEFAULT {}", now),
                [],
            );

            // Try to add updated_at column (ignore error if already exists)
            let _ = conn.execute(
                &format!("ALTER TABLE song_requests ADD COLUMN updated_at INTEGER DEFAULT {}", now),
                [],
            );
        }

        // Register HTTP routes
        router::register_routes(ctx).await?;

        log::info!("[SongRequests] Plugin initialized");
        Ok(())
    }

    async fn start(&self, _ctx: Arc<PluginContext>) -> Result<()> {
        log::info!("[SongRequests] Plugin started");
        Ok(())
    }
}

fn current_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}
