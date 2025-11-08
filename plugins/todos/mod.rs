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

pub struct TodosPlugin;

#[async_trait]
impl Plugin for TodosPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            id: "todos".to_string(),
            name: "Todos System".to_string(),
            version: "1.0.0".to_string(),
            description: "Todo list and task management".to_string(),
            author: "WebArcade Team".to_string(),
            dependencies: vec![],
        }
    }

    async fn init(&self, ctx: &PluginContext) -> Result<()> {
        log::info!("[Todos] Initializing plugin...");

        // Note: Table already exists with rewarded column
        ctx.migrate(&[
            r#"
            CREATE TABLE IF NOT EXISTS todos (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                channel TEXT NOT NULL,
                username TEXT NOT NULL,
                task TEXT NOT NULL,
                completed INTEGER NOT NULL DEFAULT 0,
                created_at INTEGER NOT NULL,
                completed_at INTEGER,
                rewarded INTEGER NOT NULL DEFAULT 0
            );

            CREATE INDEX IF NOT EXISTS idx_todos_channel ON todos(channel);
            CREATE INDEX IF NOT EXISTS idx_todos_completed ON todos(completed);
            CREATE INDEX IF NOT EXISTS idx_todos_user ON todos(channel, username, completed);
            "#,
        ])?;

        // Register services
        ctx.provide_service("create_todo", |input| async move {
            let channel: String = serde_json::from_value(input["channel"].clone())?;
            let username: String = serde_json::from_value(input["username"].clone())?;
            let task: String = serde_json::from_value(input["task"].clone())?;

            let conn = crate::core::database::get_database_path();
            let conn = rusqlite::Connection::open(conn)?;

            let todo_id = database::create_todo(&conn, &channel, &username, &task)?;
            Ok(serde_json::json!({ "todo_id": todo_id }))
        }).await;

        ctx.provide_service("get_todos", |input| async move {
            let channel: String = serde_json::from_value(input["channel"].clone())?;
            let completed: Option<bool> = serde_json::from_value(input["completed"].clone()).ok();

            let conn = crate::core::database::get_database_path();
            let conn = rusqlite::Connection::open(conn)?;

            let todos = database::get_todos(&conn, &channel, completed)?;
            Ok(serde_json::to_value(todos)?)
        }).await;

        ctx.provide_service("toggle_todo", |input| async move {
            let todo_id: i64 = serde_json::from_value(input["todo_id"].clone())?;

            let conn = crate::core::database::get_database_path();
            let conn = rusqlite::Connection::open(conn)?;

            database::toggle_todo(&conn, todo_id)?;
            Ok(serde_json::json!({ "success": true }))
        }).await;

        ctx.provide_service("delete_todo", |input| async move {
            let todo_id: i64 = serde_json::from_value(input["todo_id"].clone())?;

            let conn = crate::core::database::get_database_path();
            let conn = rusqlite::Connection::open(conn)?;

            database::delete_todo(&conn, todo_id)?;
            Ok(serde_json::json!({ "success": true }))
        }).await;

        // Register HTTP routes
        router::register_routes(ctx).await?;

        log::info!("[Todos] Plugin initialized successfully");
        Ok(())
    }

    async fn start(&self, _ctx: Arc<PluginContext>) -> Result<()> {
        log::info!("[Todos] Starting plugin...");
        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        log::info!("[Todos] Stopping plugin...");
        Ok(())
    }
}
