use crate::core::plugin::{Plugin, PluginMetadata};
use crate::core::plugin_context::PluginContext;
use async_trait::async_trait;
use std::sync::Arc;
use anyhow::Result;

mod database;
mod events;
mod router;

pub use events::*;

pub struct HouseholdPlugin;

#[async_trait]
impl Plugin for HouseholdPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            id: "household".to_string(),
            name: "Household Tasks".to_string(),
            version: "1.0.0".to_string(),
            description: "Household task and chore management system".to_string(),
            author: "WebArcade Team".to_string(),
            dependencies: vec![],
        }
    }

    async fn init(&self, ctx: &PluginContext) -> Result<()> {
        log::info!("[Household] Initializing plugin...");

        router::register_routes(ctx).await?;

        ctx.migrate(&[
            r#"
            CREATE TABLE IF NOT EXISTS household_tasks (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                task_name TEXT NOT NULL,
                description TEXT,
                category TEXT,
                priority TEXT NOT NULL DEFAULT 'medium',
                recurrence TEXT,
                recurrence_interval INTEGER,
                assigned_to TEXT,
                created_by TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS household_completions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                task_id INTEGER NOT NULL,
                task_name TEXT NOT NULL,
                completed_by TEXT NOT NULL,
                completion_notes TEXT,
                completed_at INTEGER NOT NULL,
                FOREIGN KEY (task_id) REFERENCES household_tasks(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS household_reminders (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                task_id INTEGER NOT NULL,
                remind_at INTEGER NOT NULL,
                reminded INTEGER NOT NULL DEFAULT 0,
                created_at INTEGER NOT NULL,
                FOREIGN KEY (task_id) REFERENCES household_tasks(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS household_members (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id TEXT NOT NULL UNIQUE,
                username TEXT NOT NULL,
                role TEXT NOT NULL DEFAULT 'member',
                joined_at INTEGER NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_household_tasks_assigned_to ON household_tasks(assigned_to);
            CREATE INDEX IF NOT EXISTS idx_household_completions_task_id ON household_completions(task_id);
            CREATE INDEX IF NOT EXISTS idx_household_completions_completed_at ON household_completions(completed_at DESC);
            CREATE INDEX IF NOT EXISTS idx_household_reminders_remind_at ON household_reminders(remind_at, reminded);
            "#,
        ])?;

        // Service: Create task
        ctx.provide_service("create_task", |input| async move {
            let task_name: String = serde_json::from_value(input["task_name"].clone())?;
            let description: Option<String> = serde_json::from_value(input["description"].clone()).ok();
            let category: Option<String> = serde_json::from_value(input["category"].clone()).ok();
            let priority: String = serde_json::from_value(input["priority"].clone()).unwrap_or_else(|_| "medium".to_string());
            let recurrence: Option<String> = serde_json::from_value(input["recurrence"].clone()).ok();
            let recurrence_interval: Option<i64> = serde_json::from_value(input["recurrence_interval"].clone()).ok();
            let assigned_to: Option<String> = serde_json::from_value(input["assigned_to"].clone()).ok();
            let created_by: String = serde_json::from_value(input["created_by"].clone())?;

            let conn = crate::core::database::get_database_path();
            let conn = rusqlite::Connection::open(conn)?;

            let now = current_timestamp();
            conn.execute(
                "INSERT INTO household_tasks (task_name, description, category, priority, recurrence, recurrence_interval, assigned_to, created_by, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?9)",
                rusqlite::params![task_name, description, category, priority, recurrence, recurrence_interval, assigned_to, created_by, now],
            )?;

            let task_id = conn.last_insert_rowid();
            Ok(serde_json::json!({ "id": task_id, "success": true }))
        }).await;

        // Service: Get tasks
        ctx.provide_service("get_tasks", |input| async move {
            let assigned_to: Option<String> = serde_json::from_value(input["assigned_to"].clone()).ok();
            let category: Option<String> = serde_json::from_value(input["category"].clone()).ok();

            let conn = crate::core::database::get_database_path();
            let conn = rusqlite::Connection::open(conn)?;

            let mut query = "SELECT id, task_name, description, category, priority, recurrence, recurrence_interval, assigned_to, created_at
                             FROM household_tasks WHERE 1=1".to_string();
            let mut params: Vec<String> = Vec::new();

            if let Some(ref assigned) = assigned_to {
                query.push_str(" AND assigned_to = ?");
                params.push(assigned.clone());
            }

            if let Some(ref cat) = category {
                query.push_str(" AND category = ?");
                params.push(cat.clone());
            }

            query.push_str(" ORDER BY priority DESC, created_at DESC");

            let mut stmt = conn.prepare(&query)?;
            let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p as &dyn rusqlite::ToSql).collect();

            let tasks: Vec<serde_json::Value> = stmt.query_map(param_refs.as_slice(), |row| {
                Ok(serde_json::json!({
                    "id": row.get::<_, i64>(0)?,
                    "task_name": row.get::<_, String>(1)?,
                    "description": row.get::<_, Option<String>>(2)?,
                    "category": row.get::<_, Option<String>>(3)?,
                    "priority": row.get::<_, String>(4)?,
                    "recurrence": row.get::<_, Option<String>>(5)?,
                    "recurrence_interval": row.get::<_, Option<i64>>(6)?,
                    "assigned_to": row.get::<_, Option<String>>(7)?,
                    "created_at": row.get::<_, i64>(8)?,
                }))
            })?.collect::<rusqlite::Result<Vec<_>>>()?;

            Ok(serde_json::json!({ "tasks": tasks }))
        }).await;

        // Service: Complete task
        ctx.provide_service("complete_task", |input| async move {
            let task_id: i64 = serde_json::from_value(input["task_id"].clone())?;
            let completed_by: String = serde_json::from_value(input["completed_by"].clone())?;
            let completion_notes: Option<String> = serde_json::from_value(input["completion_notes"].clone()).ok();

            let conn = crate::core::database::get_database_path();
            let conn = rusqlite::Connection::open(conn)?;

            // Get task name
            let task_name: String = conn.query_row(
                "SELECT task_name FROM household_tasks WHERE id = ?1",
                rusqlite::params![task_id],
                |row| row.get(0),
            )?;

            let now = current_timestamp();
            conn.execute(
                "INSERT INTO household_completions (task_id, task_name, completed_by, completion_notes, completed_at)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                rusqlite::params![task_id, task_name, completed_by, completion_notes, now],
            )?;

            Ok(serde_json::json!({ "success": true, "completion_id": conn.last_insert_rowid() }))
        }).await;

        // Service: Get completion history
        ctx.provide_service("get_completions", |input| async move {
            let task_id: Option<i64> = serde_json::from_value(input["task_id"].clone()).ok();
            let completed_by: Option<String> = serde_json::from_value(input["completed_by"].clone()).ok();
            let limit: i64 = serde_json::from_value(input["limit"].clone()).unwrap_or(50);

            let conn = crate::core::database::get_database_path();
            let conn = rusqlite::Connection::open(conn)?;

            let mut query = "SELECT id, task_id, task_name, completed_by, completion_notes, completed_at
                             FROM household_completions WHERE 1=1".to_string();
            let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

            if let Some(tid) = task_id {
                query.push_str(" AND task_id = ?");
                params.push(Box::new(tid));
            }

            if let Some(ref by) = completed_by {
                query.push_str(" AND completed_by = ?");
                params.push(Box::new(by.clone()));
            }

            query.push_str(" ORDER BY completed_at DESC LIMIT ?");
            params.push(Box::new(limit));

            let mut stmt = conn.prepare(&query)?;
            let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

            let completions: Vec<serde_json::Value> = stmt.query_map(param_refs.as_slice(), |row| {
                Ok(serde_json::json!({
                    "id": row.get::<_, i64>(0)?,
                    "task_id": row.get::<_, i64>(1)?,
                    "task_name": row.get::<_, String>(2)?,
                    "completed_by": row.get::<_, String>(3)?,
                    "completion_notes": row.get::<_, Option<String>>(4)?,
                    "completed_at": row.get::<_, i64>(5)?,
                }))
            })?.collect::<rusqlite::Result<Vec<_>>>()?;

            Ok(serde_json::json!({ "completions": completions }))
        }).await;

        // Service: Add household member
        ctx.provide_service("add_member", |input| async move {
            let user_id: String = serde_json::from_value(input["user_id"].clone())?;
            let username: String = serde_json::from_value(input["username"].clone())?;
            let role: String = serde_json::from_value(input["role"].clone()).unwrap_or_else(|_| "member".to_string());

            let conn = crate::core::database::get_database_path();
            let conn = rusqlite::Connection::open(conn)?;

            let now = current_timestamp();
            conn.execute(
                "INSERT OR IGNORE INTO household_members (user_id, username, role, joined_at)
                 VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params![user_id, username, role, now],
            )?;

            Ok(serde_json::json!({ "success": true }))
        }).await;

        // Service: Get members
        ctx.provide_service("get_members", |_input| async move {
            let conn = crate::core::database::get_database_path();
            let conn = rusqlite::Connection::open(conn)?;

            let mut stmt = conn.prepare(
                "SELECT user_id, username, role, joined_at FROM household_members ORDER BY joined_at"
            )?;

            let members: Vec<serde_json::Value> = stmt.query_map([], |row| {
                Ok(serde_json::json!({
                    "user_id": row.get::<_, String>(0)?,
                    "username": row.get::<_, String>(1)?,
                    "role": row.get::<_, String>(2)?,
                    "joined_at": row.get::<_, i64>(3)?,
                }))
            })?.collect::<rusqlite::Result<Vec<_>>>()?;

            Ok(serde_json::json!({ "members": members }))
        }).await;

        // Service: Get stats
        ctx.provide_service("get_stats", |input| async move {
            let user_id: Option<String> = serde_json::from_value(input["user_id"].clone()).ok();

            let conn = crate::core::database::get_database_path();
            let conn = rusqlite::Connection::open(conn)?;

            let total_tasks: i64 = conn.query_row("SELECT COUNT(*) FROM household_tasks", [], |row| row.get(0))?;

            let completions_count: i64 = if let Some(ref uid) = user_id {
                conn.query_row(
                    "SELECT COUNT(*) FROM household_completions WHERE completed_by = ?1",
                    rusqlite::params![uid],
                    |row| row.get(0),
                )?
            } else {
                conn.query_row("SELECT COUNT(*) FROM household_completions", [], |row| row.get(0))?
            };

            Ok(serde_json::json!({
                "total_tasks": total_tasks,
                "total_completions": completions_count
            }))
        }).await;

        // Service: Delete task
        ctx.provide_service("delete_task", |input| async move {
            let task_id: i64 = serde_json::from_value(input["task_id"].clone())?;

            let conn = crate::core::database::get_database_path();
            let conn = rusqlite::Connection::open(conn)?;

            conn.execute("DELETE FROM household_tasks WHERE id = ?1", rusqlite::params![task_id])?;
            Ok(serde_json::json!({ "success": true }))
        }).await;

        log::info!("[Household] Plugin initialized successfully");
        Ok(())
    }

    async fn start(&self, ctx: Arc<PluginContext>) -> Result<()> {
        log::info!("[Household] Starting plugin...");

        // Subscribe to task completion events
        let ctx_clone = ctx.clone();
        tokio::spawn(async move {
            let mut events = ctx_clone.subscribe_to("household.complete_request").await;

            while let Ok(event) = events.recv().await {
                if let (Ok(task_id), Ok(completed_by)) = (
                    serde_json::from_value::<i64>(event.payload["task_id"].clone()),
                    serde_json::from_value::<String>(event.payload["completed_by"].clone()),
                ) {
                    let completion_notes = serde_json::from_value::<String>(event.payload["completion_notes"].clone()).ok();

                    if let Ok(result) = ctx_clone.call_service("household", "complete_task", serde_json::json!({
                        "task_id": task_id,
                        "completed_by": completed_by,
                        "completion_notes": completion_notes
                    })).await {
                        if result["success"].as_bool() == Some(true) {
                            ctx_clone.emit("household.task_completed", &serde_json::json!({
                                "task_id": task_id,
                                "completed_by": completed_by,
                                "completion_id": result["completion_id"]
                            }));
                        }
                    }
                }
            }
        });

        log::info!("[Household] Plugin started successfully");
        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        log::info!("[Household] Stopping plugin...");
        Ok(())
    }
}

fn current_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}
