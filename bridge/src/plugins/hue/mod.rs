use crate::core::plugin::{Plugin, PluginMetadata};
use crate::core::plugin_context::PluginContext;
use async_trait::async_trait;
use std::sync::Arc;
use anyhow::Result;

mod database;
mod events;
mod router;

pub use events::*;

pub struct HuePlugin;

#[async_trait]
impl Plugin for HuePlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            id: "hue".to_string(),
            name: "Philips Hue Integration".to_string(),
            version: "1.0.0".to_string(),
            description: "Control Philips Hue smart lights and create light effects".to_string(),
            author: "WebArcade Team".to_string(),
            dependencies: vec![],
        }
    }

    async fn init(&self, ctx: &PluginContext) -> Result<()> {
        log::info!("[Hue] Initializing plugin...");

        // Register HTTP routes
        router::register_routes(ctx).await?;

        ctx.migrate(&[
            r#"
            CREATE TABLE IF NOT EXISTS hue_bridges (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                bridge_id TEXT NOT NULL UNIQUE,
                ip_address TEXT NOT NULL,
                username TEXT NOT NULL,
                name TEXT NOT NULL,
                enabled INTEGER NOT NULL DEFAULT 1,
                last_connected INTEGER,
                created_at INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS hue_lights (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                bridge_id TEXT NOT NULL,
                light_id TEXT NOT NULL,
                light_name TEXT NOT NULL,
                light_type TEXT NOT NULL,
                capabilities TEXT,
                enabled INTEGER NOT NULL DEFAULT 1,
                last_seen INTEGER,
                created_at INTEGER NOT NULL,
                UNIQUE(bridge_id, light_id)
            );

            CREATE TABLE IF NOT EXISTS hue_scenes (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                scene_name TEXT NOT NULL UNIQUE,
                scene_data TEXT NOT NULL,
                description TEXT,
                created_by TEXT,
                created_at INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS hue_automations (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                automation_name TEXT NOT NULL UNIQUE,
                trigger_event TEXT NOT NULL,
                scene_id INTEGER,
                light_ids TEXT,
                action_data TEXT NOT NULL,
                enabled INTEGER NOT NULL DEFAULT 1,
                created_at INTEGER NOT NULL,
                FOREIGN KEY (scene_id) REFERENCES hue_scenes(id) ON DELETE SET NULL
            );

            CREATE TABLE IF NOT EXISTS hue_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                action_type TEXT NOT NULL,
                light_id TEXT,
                scene_id INTEGER,
                action_data TEXT NOT NULL,
                triggered_by TEXT,
                timestamp INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS hue_animated_scenes (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                scene_name TEXT NOT NULL UNIQUE,
                description TEXT,
                frames_data TEXT NOT NULL,
                duration_ms INTEGER NOT NULL,
                created_at INTEGER NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_hue_lights_bridge_id ON hue_lights(bridge_id);
            CREATE INDEX IF NOT EXISTS idx_hue_history_timestamp ON hue_history(timestamp DESC);
            CREATE INDEX IF NOT EXISTS idx_hue_automations_enabled ON hue_automations(enabled, trigger_event);
            "#,
        ])?;

        // Service: Register bridge
        ctx.provide_service("register_bridge", |input| async move {
            let bridge_id: String = serde_json::from_value(input["bridge_id"].clone())?;
            let ip_address: String = serde_json::from_value(input["ip_address"].clone())?;
            let username: String = serde_json::from_value(input["username"].clone())?;
            let name: String = serde_json::from_value(input["name"].clone())?;

            let conn = crate::core::database::get_database_path();
            let conn = rusqlite::Connection::open(conn)?;

            let now = current_timestamp();
            conn.execute(
                "INSERT OR REPLACE INTO hue_bridges (bridge_id, ip_address, username, name, enabled, created_at)
                 VALUES (?1, ?2, ?3, ?4, 1, ?5)",
                rusqlite::params![bridge_id, ip_address, username, name, now],
            )?;

            Ok(serde_json::json!({ "success": true }))
        }).await;

        // Service: Register light
        ctx.provide_service("register_light", |input| async move {
            let bridge_id: String = serde_json::from_value(input["bridge_id"].clone())?;
            let light_id: String = serde_json::from_value(input["light_id"].clone())?;
            let light_name: String = serde_json::from_value(input["light_name"].clone())?;
            let light_type: String = serde_json::from_value(input["light_type"].clone())?;
            let capabilities: Option<String> = serde_json::from_value(input["capabilities"].clone()).ok();

            let conn = crate::core::database::get_database_path();
            let conn = rusqlite::Connection::open(conn)?;

            let now = current_timestamp();
            conn.execute(
                "INSERT OR REPLACE INTO hue_lights (bridge_id, light_id, light_name, light_type, capabilities, enabled, last_seen, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, 1, ?6, ?6)",
                rusqlite::params![bridge_id, light_id, light_name, light_type, capabilities, now],
            )?;

            Ok(serde_json::json!({ "success": true }))
        }).await;

        // Service: Get lights
        ctx.provide_service("get_lights", |input| async move {
            let bridge_id: Option<String> = serde_json::from_value(input["bridge_id"].clone()).ok();

            let conn = crate::core::database::get_database_path();
            let conn = rusqlite::Connection::open(conn)?;

            let mut query = "SELECT id, bridge_id, light_id, light_name, light_type, enabled FROM hue_lights WHERE 1=1".to_string();
            let mut params: Vec<String> = Vec::new();

            if let Some(ref bid) = bridge_id {
                query.push_str(" AND bridge_id = ?");
                params.push(bid.clone());
            }

            query.push_str(" AND enabled = 1 ORDER BY light_name");

            let mut stmt = conn.prepare(&query)?;
            let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p as &dyn rusqlite::ToSql).collect();

            let lights: Vec<serde_json::Value> = stmt.query_map(param_refs.as_slice(), |row| {
                Ok(serde_json::json!({
                    "id": row.get::<_, i64>(0)?,
                    "bridge_id": row.get::<_, String>(1)?,
                    "light_id": row.get::<_, String>(2)?,
                    "light_name": row.get::<_, String>(3)?,
                    "light_type": row.get::<_, String>(4)?,
                    "enabled": row.get::<_, i64>(5)? != 0,
                }))
            })?.collect::<rusqlite::Result<Vec<_>>>()?;

            Ok(serde_json::json!({ "lights": lights }))
        }).await;

        // Service: Control light (stub - actual API calls would go here)
        ctx.provide_service("set_light_state", |input| async move {
            let light_id: String = serde_json::from_value(input["light_id"].clone())?;
            let on: Option<bool> = serde_json::from_value(input["on"].clone()).ok();
            let brightness: Option<i64> = serde_json::from_value(input["brightness"].clone()).ok();
            let color: Option<String> = serde_json::from_value(input["color"].clone()).ok();
            let triggered_by: Option<String> = serde_json::from_value(input["triggered_by"].clone()).ok();

            // NOTE: Actual Hue API call would happen here
            // For now, just log the action to history

            let conn = crate::core::database::get_database_path();
            let conn = rusqlite::Connection::open(conn)?;

            let action_data = serde_json::json!({
                "on": on,
                "brightness": brightness,
                "color": color
            });

            let now = current_timestamp();
            conn.execute(
                "INSERT INTO hue_history (action_type, light_id, action_data, triggered_by, timestamp)
                 VALUES ('set_state', ?1, ?2, ?3, ?4)",
                rusqlite::params![light_id, action_data.to_string(), triggered_by, now],
            )?;

            Ok(serde_json::json!({
                "success": true,
                "light_id": light_id,
                "state": action_data
            }))
        }).await;

        // Service: Create scene
        ctx.provide_service("create_scene", |input| async move {
            let scene_name: String = serde_json::from_value(input["scene_name"].clone())?;
            let scene_data: String = serde_json::from_value(input["scene_data"].clone())?;
            let description: Option<String> = serde_json::from_value(input["description"].clone()).ok();
            let created_by: Option<String> = serde_json::from_value(input["created_by"].clone()).ok();

            let conn = crate::core::database::get_database_path();
            let conn = rusqlite::Connection::open(conn)?;

            let now = current_timestamp();
            conn.execute(
                "INSERT INTO hue_scenes (scene_name, scene_data, description, created_by, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                rusqlite::params![scene_name, scene_data, description, created_by, now],
            )?;

            Ok(serde_json::json!({ "id": conn.last_insert_rowid(), "success": true }))
        }).await;

        // Service: Activate scene
        ctx.provide_service("activate_scene", |input| async move {
            let scene_id: i64 = serde_json::from_value(input["scene_id"].clone())?;
            let triggered_by: Option<String> = serde_json::from_value(input["triggered_by"].clone()).ok();

            let conn = crate::core::database::get_database_path();
            let conn = rusqlite::Connection::open(conn)?;

            // Get scene data
            let (scene_name, scene_data): (String, String) = conn.query_row(
                "SELECT scene_name, scene_data FROM hue_scenes WHERE id = ?1",
                rusqlite::params![scene_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )?;

            // NOTE: Actual scene activation would happen here

            // Log to history
            let now = current_timestamp();
            conn.execute(
                "INSERT INTO hue_history (action_type, scene_id, action_data, triggered_by, timestamp)
                 VALUES ('activate_scene', ?1, ?2, ?3, ?4)",
                rusqlite::params![scene_id, scene_data, triggered_by, now],
            )?;

            Ok(serde_json::json!({
                "success": true,
                "scene_name": scene_name
            }))
        }).await;

        // Service: Create automation
        ctx.provide_service("create_automation", |input| async move {
            let automation_name: String = serde_json::from_value(input["automation_name"].clone())?;
            let trigger_event: String = serde_json::from_value(input["trigger_event"].clone())?;
            let scene_id: Option<i64> = serde_json::from_value(input["scene_id"].clone()).ok();
            let light_ids: Option<String> = serde_json::from_value(input["light_ids"].clone()).ok();
            let action_data: String = serde_json::from_value(input["action_data"].clone())?;

            let conn = crate::core::database::get_database_path();
            let conn = rusqlite::Connection::open(conn)?;

            let now = current_timestamp();
            conn.execute(
                "INSERT INTO hue_automations (automation_name, trigger_event, scene_id, light_ids, action_data, enabled, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, 1, ?6)",
                rusqlite::params![automation_name, trigger_event, scene_id, light_ids, action_data, now],
            )?;

            Ok(serde_json::json!({ "id": conn.last_insert_rowid(), "success": true }))
        }).await;

        // Service: Get scenes
        ctx.provide_service("get_scenes", |_input| async move {
            let conn = crate::core::database::get_database_path();
            let conn = rusqlite::Connection::open(conn)?;

            let mut stmt = conn.prepare(
                "SELECT id, scene_name, description, created_by FROM hue_scenes ORDER BY created_at DESC"
            )?;

            let scenes: Vec<serde_json::Value> = stmt.query_map([], |row| {
                Ok(serde_json::json!({
                    "id": row.get::<_, i64>(0)?,
                    "scene_name": row.get::<_, String>(1)?,
                    "description": row.get::<_, Option<String>>(2)?,
                    "created_by": row.get::<_, Option<String>>(3)?,
                }))
            })?.collect::<rusqlite::Result<Vec<_>>>()?;

            Ok(serde_json::json!({ "scenes": scenes }))
        }).await;

        log::info!("[Hue] Plugin initialized successfully");
        Ok(())
    }

    async fn start(&self, ctx: Arc<PluginContext>) -> Result<()> {
        log::info!("[Hue] Starting plugin...");

        // Subscribe to automation triggers
        let ctx_clone = ctx.clone();
        tokio::spawn(async move {
            // Get all enabled automations
            let conn = crate::core::database::get_database_path();
            if let Ok(conn) = rusqlite::Connection::open(conn) {
                if let Ok(mut stmt) = conn.prepare("SELECT id, trigger_event, scene_id, action_data FROM hue_automations WHERE enabled = 1") {
                    if let Ok(automations) = stmt.query_map([], |row| {
                        Ok((
                            row.get::<_, i64>(0)?,
                            row.get::<_, String>(1)?,
                            row.get::<_, Option<i64>>(2)?,
                            row.get::<_, String>(3)?
                        ))
                    }) {
                        for automation_result in automations {
                            if let Ok((_id, trigger_event, scene_id, _action_data)) = automation_result {
                                let ctx_inner = ctx_clone.clone();

                                // Subscribe to trigger event
                                tokio::spawn(async move {
                                    let mut events = ctx_inner.subscribe_to(&trigger_event).await;

                                    while let Ok(_event) = events.recv().await {
                                        // Activate scene if specified
                                        if let Some(sid) = scene_id {
                                            let _ = ctx_inner.call_service("hue", "activate_scene", serde_json::json!({
                                                "scene_id": sid,
                                                "triggered_by": "automation"
                                            })).await;

                                            ctx_inner.emit("hue.automation_triggered", &serde_json::json!({
                                                "trigger_event": trigger_event,
                                                "scene_id": sid
                                            }));
                                        }
                                    }
                                });
                            }
                        }
                    }
                }
            }
        });

        log::info!("[Hue] Plugin started successfully");
        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        log::info!("[Hue] Stopping plugin...");
        Ok(())
    }
}

fn current_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}
