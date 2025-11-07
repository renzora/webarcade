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

pub struct WithingsPlugin;

#[async_trait]
impl Plugin for WithingsPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            id: "withings".to_string(),
            name: "Withings Health Integration".to_string(),
            version: "1.0.0".to_string(),
            description: "Integration with Withings health devices and API for weight, activity, sleep tracking".to_string(),
            author: "WebArcade Team".to_string(),
            dependencies: vec![],
        }
    }

    async fn init(&self, ctx: &PluginContext) -> Result<()> {
        log::info!("[Withings] Initializing plugin...");

        ctx.migrate(&[
            r#"
            CREATE TABLE IF NOT EXISTS withings_auth (
                user_id TEXT PRIMARY KEY,
                access_token TEXT NOT NULL,
                refresh_token TEXT NOT NULL,
                expires_at INTEGER NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS withings_weight_measurements (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id TEXT NOT NULL,
                weight_kg REAL NOT NULL,
                fat_mass_kg REAL,
                muscle_mass_kg REAL,
                bone_mass_kg REAL,
                water_percentage REAL,
                measured_at INTEGER NOT NULL,
                synced_at INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS withings_activity (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id TEXT NOT NULL,
                date TEXT NOT NULL,
                steps INTEGER NOT NULL,
                distance_meters INTEGER,
                calories INTEGER,
                active_minutes INTEGER,
                synced_at INTEGER NOT NULL,
                UNIQUE(user_id, date)
            );

            CREATE TABLE IF NOT EXISTS withings_sleep (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id TEXT NOT NULL,
                date TEXT NOT NULL,
                total_sleep_minutes INTEGER NOT NULL,
                deep_sleep_minutes INTEGER,
                light_sleep_minutes INTEGER,
                rem_sleep_minutes INTEGER,
                awake_minutes INTEGER,
                sleep_score INTEGER,
                synced_at INTEGER NOT NULL,
                UNIQUE(user_id, date)
            );

            CREATE TABLE IF NOT EXISTS withings_goals (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id TEXT NOT NULL,
                goal_type TEXT NOT NULL,
                goal_value REAL NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL,
                UNIQUE(user_id, goal_type)
            );

            CREATE INDEX IF NOT EXISTS idx_weight_user_measured ON withings_weight_measurements(user_id, measured_at DESC);
            CREATE INDEX IF NOT EXISTS idx_activity_user_date ON withings_activity(user_id, date DESC);
            CREATE INDEX IF NOT EXISTS idx_sleep_user_date ON withings_sleep(user_id, date DESC);
            "#,
        ])?;

        // Service: Store weight measurement
        ctx.provide_service("record_weight", |input| async move {
            let user_id: String = serde_json::from_value(input["user_id"].clone())?;
            let weight_kg: f64 = serde_json::from_value(input["weight_kg"].clone())?;
            let fat_mass_kg: Option<f64> = serde_json::from_value(input["fat_mass_kg"].clone()).ok();
            let muscle_mass_kg: Option<f64> = serde_json::from_value(input["muscle_mass_kg"].clone()).ok();
            let bone_mass_kg: Option<f64> = serde_json::from_value(input["bone_mass_kg"].clone()).ok();
            let water_percentage: Option<f64> = serde_json::from_value(input["water_percentage"].clone()).ok();
            let measured_at: i64 = serde_json::from_value(input["measured_at"].clone())?;

            let conn = crate::core::database::get_database_path();
            let conn = rusqlite::Connection::open(conn)?;

            let now = current_timestamp();
            conn.execute(
                "INSERT INTO withings_weight_measurements (user_id, weight_kg, fat_mass_kg, muscle_mass_kg, bone_mass_kg, water_percentage, measured_at, synced_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                rusqlite::params![user_id, weight_kg, fat_mass_kg, muscle_mass_kg, bone_mass_kg, water_percentage, measured_at, now],
            )?;

            Ok(serde_json::json!({ "id": conn.last_insert_rowid(), "success": true }))
        }).await;

        // Service: Get weight history
        ctx.provide_service("get_weight_history", |input| async move {
            let user_id: String = serde_json::from_value(input["user_id"].clone())?;
            let limit: i64 = serde_json::from_value(input["limit"].clone()).unwrap_or(30);

            let conn = crate::core::database::get_database_path();
            let conn = rusqlite::Connection::open(conn)?;

            let mut stmt = conn.prepare(
                "SELECT id, weight_kg, fat_mass_kg, muscle_mass_kg, bone_mass_kg, water_percentage, measured_at
                 FROM withings_weight_measurements
                 WHERE user_id = ?1
                 ORDER BY measured_at DESC
                 LIMIT ?2"
            )?;

            let measurements: Vec<serde_json::Value> = stmt.query_map(
                rusqlite::params![user_id, limit],
                |row| {
                    Ok(serde_json::json!({
                        "id": row.get::<_, i64>(0)?,
                        "weight_kg": row.get::<_, f64>(1)?,
                        "fat_mass_kg": row.get::<_, Option<f64>>(2)?,
                        "muscle_mass_kg": row.get::<_, Option<f64>>(3)?,
                        "bone_mass_kg": row.get::<_, Option<f64>>(4)?,
                        "water_percentage": row.get::<_, Option<f64>>(5)?,
                        "measured_at": row.get::<_, i64>(6)?,
                    }))
                }
            )?.collect::<rusqlite::Result<Vec<_>>>()?;

            Ok(serde_json::json!({ "measurements": measurements }))
        }).await;

        // Service: Record activity
        ctx.provide_service("record_activity", |input| async move {
            let user_id: String = serde_json::from_value(input["user_id"].clone())?;
            let date: String = serde_json::from_value(input["date"].clone())?;
            let steps: i64 = serde_json::from_value(input["steps"].clone())?;
            let distance_meters: Option<i64> = serde_json::from_value(input["distance_meters"].clone()).ok();
            let calories: Option<i64> = serde_json::from_value(input["calories"].clone()).ok();
            let active_minutes: Option<i64> = serde_json::from_value(input["active_minutes"].clone()).ok();

            let conn = crate::core::database::get_database_path();
            let conn = rusqlite::Connection::open(conn)?;

            let now = current_timestamp();
            conn.execute(
                "INSERT OR REPLACE INTO withings_activity (user_id, date, steps, distance_meters, calories, active_minutes, synced_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                rusqlite::params![user_id, date, steps, distance_meters, calories, active_minutes, now],
            )?;

            Ok(serde_json::json!({ "success": true }))
        }).await;

        // Service: Get activity history
        ctx.provide_service("get_activity_history", |input| async move {
            let user_id: String = serde_json::from_value(input["user_id"].clone())?;
            let limit: i64 = serde_json::from_value(input["limit"].clone()).unwrap_or(30);

            let conn = crate::core::database::get_database_path();
            let conn = rusqlite::Connection::open(conn)?;

            let mut stmt = conn.prepare(
                "SELECT id, date, steps, distance_meters, calories, active_minutes
                 FROM withings_activity
                 WHERE user_id = ?1
                 ORDER BY date DESC
                 LIMIT ?2"
            )?;

            let activities: Vec<serde_json::Value> = stmt.query_map(
                rusqlite::params![user_id, limit],
                |row| {
                    Ok(serde_json::json!({
                        "id": row.get::<_, i64>(0)?,
                        "date": row.get::<_, String>(1)?,
                        "steps": row.get::<_, i64>(2)?,
                        "distance_meters": row.get::<_, Option<i64>>(3)?,
                        "calories": row.get::<_, Option<i64>>(4)?,
                        "active_minutes": row.get::<_, Option<i64>>(5)?,
                    }))
                }
            )?.collect::<rusqlite::Result<Vec<_>>>()?;

            Ok(serde_json::json!({ "activities": activities }))
        }).await;

        // Service: Record sleep
        ctx.provide_service("record_sleep", |input| async move {
            let user_id: String = serde_json::from_value(input["user_id"].clone())?;
            let date: String = serde_json::from_value(input["date"].clone())?;
            let total_sleep_minutes: i64 = serde_json::from_value(input["total_sleep_minutes"].clone())?;
            let deep_sleep_minutes: Option<i64> = serde_json::from_value(input["deep_sleep_minutes"].clone()).ok();
            let light_sleep_minutes: Option<i64> = serde_json::from_value(input["light_sleep_minutes"].clone()).ok();
            let rem_sleep_minutes: Option<i64> = serde_json::from_value(input["rem_sleep_minutes"].clone()).ok();
            let awake_minutes: Option<i64> = serde_json::from_value(input["awake_minutes"].clone()).ok();
            let sleep_score: Option<i64> = serde_json::from_value(input["sleep_score"].clone()).ok();

            let conn = crate::core::database::get_database_path();
            let conn = rusqlite::Connection::open(conn)?;

            let now = current_timestamp();
            conn.execute(
                "INSERT OR REPLACE INTO withings_sleep (user_id, date, total_sleep_minutes, deep_sleep_minutes, light_sleep_minutes, rem_sleep_minutes, awake_minutes, sleep_score, synced_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                rusqlite::params![user_id, date, total_sleep_minutes, deep_sleep_minutes, light_sleep_minutes, rem_sleep_minutes, awake_minutes, sleep_score, now],
            )?;

            Ok(serde_json::json!({ "success": true }))
        }).await;

        // Service: Get sleep history
        ctx.provide_service("get_sleep_history", |input| async move {
            let user_id: String = serde_json::from_value(input["user_id"].clone())?;
            let limit: i64 = serde_json::from_value(input["limit"].clone()).unwrap_or(30);

            let conn = crate::core::database::get_database_path();
            let conn = rusqlite::Connection::open(conn)?;

            let mut stmt = conn.prepare(
                "SELECT id, date, total_sleep_minutes, deep_sleep_minutes, light_sleep_minutes, rem_sleep_minutes, awake_minutes, sleep_score
                 FROM withings_sleep
                 WHERE user_id = ?1
                 ORDER BY date DESC
                 LIMIT ?2"
            )?;

            let sleep_data: Vec<serde_json::Value> = stmt.query_map(
                rusqlite::params![user_id, limit],
                |row| {
                    Ok(serde_json::json!({
                        "id": row.get::<_, i64>(0)?,
                        "date": row.get::<_, String>(1)?,
                        "total_sleep_minutes": row.get::<_, i64>(2)?,
                        "deep_sleep_minutes": row.get::<_, Option<i64>>(3)?,
                        "light_sleep_minutes": row.get::<_, Option<i64>>(4)?,
                        "rem_sleep_minutes": row.get::<_, Option<i64>>(5)?,
                        "awake_minutes": row.get::<_, Option<i64>>(6)?,
                        "sleep_score": row.get::<_, Option<i64>>(7)?,
                    }))
                }
            )?.collect::<rusqlite::Result<Vec<_>>>()?;

            Ok(serde_json::json!({ "sleep_data": sleep_data }))
        }).await;

        // Service: Set goal
        ctx.provide_service("set_goal", |input| async move {
            let user_id: String = serde_json::from_value(input["user_id"].clone())?;
            let goal_type: String = serde_json::from_value(input["goal_type"].clone())?;
            let goal_value: f64 = serde_json::from_value(input["goal_value"].clone())?;

            let conn = crate::core::database::get_database_path();
            let conn = rusqlite::Connection::open(conn)?;

            let now = current_timestamp();
            conn.execute(
                "INSERT OR REPLACE INTO withings_goals (user_id, goal_type, goal_value, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?4)",
                rusqlite::params![user_id, goal_type, goal_value, now],
            )?;

            Ok(serde_json::json!({ "success": true }))
        }).await;

        // Service: Get goals
        ctx.provide_service("get_goals", |input| async move {
            let user_id: String = serde_json::from_value(input["user_id"].clone())?;

            let conn = crate::core::database::get_database_path();
            let conn = rusqlite::Connection::open(conn)?;

            let mut stmt = conn.prepare(
                "SELECT id, goal_type, goal_value, created_at FROM withings_goals WHERE user_id = ?1"
            )?;

            let goals: Vec<serde_json::Value> = stmt.query_map(rusqlite::params![user_id], |row| {
                Ok(serde_json::json!({
                    "id": row.get::<_, i64>(0)?,
                    "goal_type": row.get::<_, String>(1)?,
                    "goal_value": row.get::<_, f64>(2)?,
                    "created_at": row.get::<_, i64>(3)?,
                }))
            })?.collect::<rusqlite::Result<Vec<_>>>()?;

            Ok(serde_json::json!({ "goals": goals }))
        }).await;

        // Service: Get latest weight
        ctx.provide_service("get_latest_weight", |input| async move {
            let user_id: String = serde_json::from_value(input["user_id"].clone())?;

            let conn = crate::core::database::get_database_path();
            let conn = rusqlite::Connection::open(conn)?;

            let weight: Option<serde_json::Value> = conn.query_row(
                "SELECT weight_kg, measured_at FROM withings_weight_measurements
                 WHERE user_id = ?1
                 ORDER BY measured_at DESC
                 LIMIT 1",
                rusqlite::params![user_id],
                |row| {
                    Ok(serde_json::json!({
                        "weight_kg": row.get::<_, f64>(0)?,
                        "measured_at": row.get::<_, i64>(1)?,
                    }))
                }
            ).optional()?;

            Ok(serde_json::json!({ "latest_weight": weight }))
        }).await;

        // Service: Get stats
        ctx.provide_service("get_stats", |input| async move {
            let user_id: String = serde_json::from_value(input["user_id"].clone())?;

            let conn = crate::core::database::get_database_path();
            let conn = rusqlite::Connection::open(conn)?;

            let weight_count: i64 = conn.query_row(
                "SELECT COUNT(*) FROM withings_weight_measurements WHERE user_id = ?1",
                rusqlite::params![user_id],
                |row| row.get(0),
            )?;

            let activity_count: i64 = conn.query_row(
                "SELECT COUNT(*) FROM withings_activity WHERE user_id = ?1",
                rusqlite::params![user_id],
                |row| row.get(0),
            )?;

            let sleep_count: i64 = conn.query_row(
                "SELECT COUNT(*) FROM withings_sleep WHERE user_id = ?1",
                rusqlite::params![user_id],
                |row| row.get(0),
            )?;

            Ok(serde_json::json!({
                "weight_measurements": weight_count,
                "activity_days": activity_count,
                "sleep_days": sleep_count
            }))
        }).await;

        // Register HTTP routes
        router::register_routes(ctx).await?;

        log::info!("[Withings] Plugin initialized successfully");
        Ok(())
    }

    async fn start(&self, ctx: Arc<PluginContext>) -> Result<()> {
        log::info!("[Withings] Starting plugin...");

        // Background task: Sync data periodically (stub)
        let ctx_clone = ctx.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(3600)); // Every hour

            loop {
                interval.tick().await;

                // NOTE: Actual Withings API sync would happen here
                // For now, just emit a sync event
                ctx_clone.emit("withings.sync_started", &serde_json::json!({
                    "timestamp": current_timestamp()
                }));

                log::debug!("[Withings] Periodic sync check");
            }
        });

        log::info!("[Withings] Plugin started successfully");
        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        log::info!("[Withings] Stopping plugin...");
        Ok(())
    }
}

fn current_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}
