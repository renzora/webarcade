use crate::core::plugin::{Plugin, PluginMetadata};
use crate::core::plugin_context::PluginContext;
use async_trait::async_trait;
use std::sync::Arc;
use anyhow::Result;
use serde_json::json;

mod router;

pub struct WheelPlugin;

#[async_trait]
impl Plugin for WheelPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            id: "wheel".to_string(),
            name: "Wheel System".to_string(),
            version: "1.0.0".to_string(),
            description: "Spin wheel game with customizable options".to_string(),
            author: "WebArcade Team".to_string(),
            dependencies: vec!["currency".to_string()],
        }
    }

    async fn init(&self, ctx: &PluginContext) -> Result<()> {
        log::info!("[Wheel] Initializing plugin...");

        ctx.migrate(&[
            r#"
            CREATE TABLE IF NOT EXISTS wheel_options (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                channel TEXT NOT NULL DEFAULT 'global',
                option_text TEXT NOT NULL,
                color TEXT NOT NULL,
                weight INTEGER NOT NULL DEFAULT 1,
                chance_percentage REAL,
                enabled INTEGER NOT NULL DEFAULT 1,
                prize_type TEXT,
                prize_data TEXT
            );

            CREATE TABLE IF NOT EXISTS wheel_spins (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                channel TEXT NOT NULL DEFAULT 'global',
                user_id TEXT NOT NULL,
                username TEXT NOT NULL,
                result TEXT NOT NULL,
                prize_type TEXT,
                prize_data TEXT,
                created_at INTEGER NOT NULL
            );
            "#,
        ])?;

        // Register services

        // Get all options for a channel
        ctx.provide_service("get_options", |input| async move {
            let channel: String = serde_json::from_value(input["channel"].clone())
                .unwrap_or_else(|_| "global".to_string());

            let conn = crate::core::database::get_database_path();
            let conn = rusqlite::Connection::open(conn)?;

            let mut stmt = conn.prepare(
                "SELECT id, channel, option_text, color, weight, chance_percentage, enabled, prize_type, prize_data
                 FROM wheel_options WHERE channel = ?1 ORDER BY id ASC"
            )?;

            let options: Vec<serde_json::Value> = stmt.query_map([&channel], |row| {
                Ok(json!({
                    "id": row.get::<_, i64>(0)?,
                    "channel": row.get::<_, String>(1)?,
                    "option_text": row.get::<_, String>(2)?,
                    "color": row.get::<_, String>(3)?,
                    "weight": row.get::<_, i64>(4)?,
                    "chance_percentage": row.get::<_, Option<f64>>(5)?,
                    "enabled": row.get::<_, i64>(6)?,
                    "prize_type": row.get::<_, Option<String>>(7)?,
                    "prize_data": row.get::<_, Option<String>>(8)?
                }))
            })?.collect::<rusqlite::Result<Vec<_>>>()?;

            Ok(json!(options))
        }).await;

        // Add new option
        ctx.provide_service("add_option", |input| async move {
            let channel: String = serde_json::from_value(input["channel"].clone())?;
            let option_text: String = serde_json::from_value(input["option_text"].clone())?;
            let color: String = serde_json::from_value(input["color"].clone())?;
            let weight: i64 = serde_json::from_value(input["weight"].clone()).unwrap_or(1);
            let chance_percentage: Option<f64> = serde_json::from_value(input["chance_percentage"].clone()).ok();
            let prize_type: Option<String> = serde_json::from_value(input["prize_type"].clone()).ok();
            let prize_data: Option<String> = serde_json::from_value(input["prize_data"].clone()).ok();

            let conn = crate::core::database::get_database_path();
            let conn = rusqlite::Connection::open(conn)?;

            conn.execute(
                "INSERT INTO wheel_options (channel, option_text, color, weight, chance_percentage, enabled, prize_type, prize_data)
                 VALUES (?1, ?2, ?3, ?4, ?5, 1, ?6, ?7)",
                rusqlite::params![channel, option_text, color, weight, chance_percentage, prize_type, prize_data],
            )?;

            Ok(json!({ "id": conn.last_insert_rowid() }))
        }).await;

        // Update option
        ctx.provide_service("update_option", |input| async move {
            let id: i64 = serde_json::from_value(input["id"].clone())?;
            let option_text: String = serde_json::from_value(input["option_text"].clone())?;
            let color: String = serde_json::from_value(input["color"].clone())?;
            let weight: i64 = serde_json::from_value(input["weight"].clone()).unwrap_or(1);
            let chance_percentage: Option<f64> = serde_json::from_value(input["chance_percentage"].clone()).ok();
            let prize_type: Option<String> = serde_json::from_value(input["prize_type"].clone()).ok();
            let prize_data: Option<String> = serde_json::from_value(input["prize_data"].clone()).ok();

            let conn = crate::core::database::get_database_path();
            let conn = rusqlite::Connection::open(conn)?;

            conn.execute(
                "UPDATE wheel_options
                 SET option_text = ?1, color = ?2, weight = ?3, chance_percentage = ?4, prize_type = ?5, prize_data = ?6
                 WHERE id = ?7",
                rusqlite::params![option_text, color, weight, chance_percentage, prize_type, prize_data, id],
            )?;

            Ok(json!({ "success": true }))
        }).await;

        // Delete option
        ctx.provide_service("delete_option", |input| async move {
            let id: i64 = serde_json::from_value(input["id"].clone())?;

            let conn = crate::core::database::get_database_path();
            let conn = rusqlite::Connection::open(conn)?;

            conn.execute("DELETE FROM wheel_options WHERE id = ?1", [id])?;

            Ok(json!({ "success": true }))
        }).await;

        // Toggle option enabled/disabled
        ctx.provide_service("toggle_option", |input| async move {
            let id: i64 = serde_json::from_value(input["id"].clone())?;

            let conn = crate::core::database::get_database_path();
            let conn = rusqlite::Connection::open(conn)?;

            conn.execute(
                "UPDATE wheel_options SET enabled = 1 - enabled WHERE id = ?1",
                [id]
            )?;

            Ok(json!({ "success": true }))
        }).await;

        // Spin wheel - prize callbacks will be handled by the HTTP handler
        ctx.provide_service("spin_wheel", |input| async move {
                let channel: String = serde_json::from_value(input["channel"].clone())?;

                let conn = crate::core::database::get_database_path();
                let conn = rusqlite::Connection::open(conn)?;

                // Get all enabled options with their data
                let mut stmt = conn.prepare(
                    "SELECT id, option_text, color, weight, chance_percentage, prize_type, prize_data
                     FROM wheel_options WHERE channel = ?1 AND enabled = 1"
                )?;

                struct WheelOption {
                    id: i64,
                    text: String,
                    color: String,
                    weight: i64,
                    chance_percentage: Option<f64>,
                    prize_type: Option<String>,
                    prize_data: Option<String>,
                }

                let options: Vec<WheelOption> = stmt.query_map([&channel], |row| {
                    Ok(WheelOption {
                        id: row.get(0)?,
                        text: row.get(1)?,
                        color: row.get(2)?,
                        weight: row.get(3)?,
                        chance_percentage: row.get(4)?,
                        prize_type: row.get(5)?,
                        prize_data: row.get(6)?,
                    })
                })?.collect::<rusqlite::Result<Vec<_>>>()?;

                if options.is_empty() {
                    return Err(anyhow::anyhow!("No wheel options available"));
                }

                // Weighted random selection
                use rand::Rng;
                let total_weight: i64 = options.iter().map(|o| o.weight).sum();
                let mut rng = rand::thread_rng();
                let mut roll = rng.gen_range(0..total_weight);

                let mut winner_option = &options[0];
                for option in &options {
                    if roll < option.weight {
                        winner_option = option;
                        break;
                    }
                    roll -= option.weight;
                }

                // Prepare all options for overlay display
                let display_options: Vec<serde_json::Value> = options.iter().map(|opt| {
                    json!({
                        "text": opt.text,
                        "color": opt.color
                    })
                }).collect();

                // Prize callbacks will be handled by the HTTP router after this service returns
                // The HTTP handler has access to the service registry and can call other plugins

                Ok(json!({
                    "winner": winner_option.text,
                    "prize_type": winner_option.prize_type,
                    "prize_data": winner_option.prize_data,
                    "options": display_options
                }))
        }).await;

        // Register HTTP routes
        router::register_routes(ctx).await?;

        log::info!("[Wheel] Plugin initialized successfully");
        Ok(())
    }

    async fn start(&self, _ctx: Arc<PluginContext>) -> Result<()> {
        log::info!("[Wheel] Starting plugin...");
        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        log::info!("[Wheel] Stopping plugin...");
        Ok(())
    }
}
