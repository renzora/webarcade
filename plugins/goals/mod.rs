use crate::core::plugin::{Plugin, PluginMetadata};
use crate::core::plugin_context::PluginContext;
use async_trait::async_trait;
use std::sync::Arc;
use anyhow::Result;
use rusqlite::OptionalExtension;

mod database;
mod events;
mod router;

pub use database::*;
pub use events::*;

pub struct GoalsPlugin;

#[async_trait]
impl Plugin for GoalsPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            id: "goals".to_string(),
            name: "Goals System".to_string(),
            version: "1.0.0".to_string(),
            description: "Goal tracking and progress management".to_string(),
            author: "WebArcade Team".to_string(),
            dependencies: vec![],
        }
    }

    async fn init(&self, ctx: &PluginContext) -> Result<()> {
        log::info!("[Goals] Initializing plugin...");

        ctx.migrate(&[
            r#"
            CREATE TABLE IF NOT EXISTS goals (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                channel TEXT NOT NULL,
                title TEXT NOT NULL,
                description TEXT,
                type TEXT NOT NULL,
                target INTEGER NOT NULL,
                current INTEGER NOT NULL DEFAULT 0,
                is_sub_goal INTEGER NOT NULL DEFAULT 0,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_goals_channel ON goals(channel);
            "#,
        ])?;

        // Register HTTP routes
        router::register_routes(ctx).await?;

        // Register services
        ctx.provide_service("create_goal", |input| async move {
            let channel: String = serde_json::from_value(input["channel"].clone())?;
            let title: String = serde_json::from_value(input["title"].clone())?;
            let description: Option<String> = serde_json::from_value(input["description"].clone()).ok();
            let goal_type: String = serde_json::from_value(input["type"].clone())?;
            let target: i64 = serde_json::from_value(input["target"].clone())?;
            let is_sub_goal: bool = serde_json::from_value(input["is_sub_goal"].clone()).unwrap_or(false);

            let conn = crate::core::database::get_database_path();
            let conn = rusqlite::Connection::open(conn)?;

            let goal_id = database::create_goal(&conn, &channel, &title, description.as_deref(), &goal_type, target, is_sub_goal)?;
            Ok(serde_json::json!({ "goal_id": goal_id }))
        }).await;

        ctx.provide_service("update_progress", |input| async move {
            let goal_id: i64 = serde_json::from_value(input["goal_id"].clone())?;
            let amount: i64 = serde_json::from_value(input["amount"].clone())?;

            let conn = crate::core::database::get_database_path();
            let conn = rusqlite::Connection::open(conn)?;

            database::update_progress(&conn, goal_id, amount)?;
            Ok(serde_json::json!({ "success": true }))
        }).await;

        ctx.provide_service("get_goals", |input| async move {
            let channel: String = serde_json::from_value(input["channel"].clone())?;

            let conn = crate::core::database::get_database_path();
            let conn = rusqlite::Connection::open(conn)?;

            let goals = database::get_goals(&conn, &channel)?;
            Ok(serde_json::to_value(goals)?)
        }).await;

        ctx.provide_service("delete_goal", |input| async move {
            let goal_id: i64 = serde_json::from_value(input["goal_id"].clone())?;

            let conn = crate::core::database::get_database_path();
            let conn = rusqlite::Connection::open(conn)?;

            database::delete_goal(&conn, goal_id)?;
            Ok(serde_json::json!({ "success": true }))
        }).await;

        ctx.provide_service("set_progress", |input| async move {
            let goal_id: i64 = serde_json::from_value(input["goal_id"].clone())?;
            let current: i64 = serde_json::from_value(input["current"].clone())?;

            let conn = crate::core::database::get_database_path();
            let conn = rusqlite::Connection::open(conn)?;

            database::set_progress(&conn, goal_id, current)?;
            Ok(serde_json::json!({ "success": true }))
        }).await;

        ctx.provide_service("update_goal", |input| async move {
            let goal_id: i64 = serde_json::from_value(input["goal_id"].clone())?;
            let title: Option<String> = serde_json::from_value(input["title"].clone()).ok();
            let description: Option<String> = serde_json::from_value(input["description"].clone()).ok();
            let goal_type: Option<String> = serde_json::from_value(input["type"].clone()).ok();
            let target: Option<i64> = serde_json::from_value(input["target"].clone()).ok();
            let current: Option<i64> = serde_json::from_value(input["current"].clone()).ok();
            let is_sub_goal: Option<bool> = serde_json::from_value(input["is_sub_goal"].clone()).ok();

            let conn = crate::core::database::get_database_path();
            let conn = rusqlite::Connection::open(conn)?;

            database::update_goal(
                &conn,
                goal_id,
                title.as_deref(),
                description.as_deref(),
                goal_type.as_deref(),
                target,
                current,
                is_sub_goal,
            )?;
            Ok(serde_json::json!({ "success": true }))
        }).await;

        ctx.provide_service("sync_twitch", |input| async move {
            let goal_id: i64 = serde_json::from_value(input["goal_id"].clone())?;

            let conn = crate::core::database::get_database_path();
            let conn = rusqlite::Connection::open(conn)?;

            // Get the goal to check its type
            let goal = database::get_goal(&conn, goal_id)?
                .ok_or_else(|| anyhow::anyhow!("Goal not found"))?;

            // Fetch current count from Twitch based on goal type
            match goal.goal_type.as_str() {
                "follower" => {
                    // Call Twitch API to get follower count
                    // This would require access to Twitch API - for now return error
                    return Err(anyhow::anyhow!("Twitch sync not implemented yet"));
                }
                "subscriber" => {
                    // Call Twitch API to get subscriber count
                    return Err(anyhow::anyhow!("Twitch sync not implemented yet"));
                }
                _ => {
                    return Err(anyhow::anyhow!("Goal type does not support Twitch sync"));
                }
            }
        }).await;

        log::info!("[Goals] Plugin initialized successfully");
        Ok(())
    }

    async fn start(&self, ctx: Arc<PluginContext>) -> Result<()> {
        log::info!("[Goals] Starting plugin...");

        // Subscribe to events that might contribute to goals
        let ctx_clone = ctx.clone();
        tokio::spawn(async move {
            let mut events = ctx_clone.subscribe_to("twitch.follow").await;

            while let Ok(event) = events.recv().await {
                // Check if there's a follower goal and increment it
                if let Ok(channel) = serde_json::from_value::<String>(event.payload["channel"].clone()) {
                    let _ = update_goal_by_type(&ctx_clone, &channel, "followers", 1).await;
                }
            }
        });

        let ctx_clone = ctx.clone();
        tokio::spawn(async move {
            let mut events = ctx_clone.subscribe_to("twitch.subscription").await;

            while let Ok(event) = events.recv().await {
                if let Ok(channel) = serde_json::from_value::<String>(event.payload["channel"].clone()) {
                    let _ = update_goal_by_type(&ctx_clone, &channel, "subscribers", 1).await;
                }
            }
        });

        log::info!("[Goals] Plugin started successfully");
        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        log::info!("[Goals] Stopping plugin...");
        Ok(())
    }
}

async fn update_goal_by_type(ctx: &PluginContext, channel: &str, goal_type: &str, amount: i64) -> Result<()> {
    let conn = ctx.db()?;

    // Find goal of this type
    let goal_id: Option<i64> = conn.query_row(
        "SELECT id FROM goals WHERE channel = ?1 AND type = ?2 AND current < target",
        rusqlite::params![channel, goal_type],
        |row| row.get(0),
    ).optional()?;

    if let Some(id) = goal_id {
        database::update_progress(&conn, id, amount)?;

        // Check if goal completed
        let current: i64 = conn.query_row(
            "SELECT current FROM goals WHERE id = ?1",
            rusqlite::params![id],
            |row| row.get(0),
        )?;

        let target: i64 = conn.query_row(
            "SELECT target FROM goals WHERE id = ?1",
            rusqlite::params![id],
            |row| row.get(0),
        )?;

        if current >= target {
            ctx.emit("goals.completed", &GoalCompletedEvent {
                goal_id: id,
                channel: channel.to_string(),
                goal_type: goal_type.to_string(),
            });
        }
    }

    Ok(())
}
