use crate::core::plugin::{Plugin, PluginMetadata};
use crate::core::plugin_context::PluginContext;
use async_trait::async_trait;
use std::sync::Arc;
use anyhow::Result;

mod events;
mod router;

pub use events::*;

pub struct PacksPlugin;

#[async_trait]
impl Plugin for PacksPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            id: "packs".to_string(),
            name: "Pack Opening System".to_string(),
            version: "1.0.0".to_string(),
            description: "Collectible pack opening and inventory management".to_string(),
            author: "WebArcade Team".to_string(),
            dependencies: vec!["currency".to_string()],
        }
    }

    async fn init(&self, ctx: &PluginContext) -> Result<()> {
        log::info!("[Packs] Initializing plugin...");

        // Register HTTP routes
        router::register_routes(ctx).await?;

        // Migration 1: Create tables if not exists
        ctx.migrate(&[
            r#"
            CREATE TABLE IF NOT EXISTS pack_definitions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                description TEXT NOT NULL,
                cost INTEGER NOT NULL,
                enabled INTEGER NOT NULL DEFAULT 1,
                image_url TEXT,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS pack_items (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                pack_id INTEGER NOT NULL,
                item_name TEXT NOT NULL,
                item_description TEXT,
                rarity TEXT NOT NULL,
                weight INTEGER NOT NULL DEFAULT 1,
                created_at INTEGER NOT NULL,
                FOREIGN KEY (pack_id) REFERENCES pack_definitions(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS user_inventory (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id TEXT NOT NULL,
                username TEXT NOT NULL,
                item_name TEXT NOT NULL,
                item_description TEXT,
                rarity TEXT NOT NULL,
                pack_name TEXT NOT NULL,
                obtained_at INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS pack_openings (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id TEXT NOT NULL,
                username TEXT NOT NULL,
                pack_id INTEGER NOT NULL,
                pack_name TEXT NOT NULL,
                items_obtained TEXT NOT NULL,
                cost INTEGER NOT NULL,
                opened_at INTEGER NOT NULL,
                FOREIGN KEY (pack_id) REFERENCES pack_definitions(id)
            );
            "#,
        ])?;

        // Migration 2: Add indexes (only if schema matches)
        let conn = crate::core::database::get_database_path();
        if let Ok(conn) = rusqlite::Connection::open(conn) {
            // Check if pack_id column exists in pack_items table
            let has_pack_id = conn.query_row(
                "SELECT COUNT(*) FROM pragma_table_info('pack_items') WHERE name = 'pack_id'",
                [],
                |row| row.get::<_, i64>(0)
            ).unwrap_or(0) > 0;

            if has_pack_id {
                let _ = conn.execute("CREATE INDEX IF NOT EXISTS idx_user_inventory_user_id ON user_inventory(user_id)", []);
                let _ = conn.execute("CREATE INDEX IF NOT EXISTS idx_pack_openings_user_id ON pack_openings(user_id)", []);
                let _ = conn.execute("CREATE INDEX IF NOT EXISTS idx_pack_items_pack_id ON pack_items(pack_id)", []);
            } else {
                log::warn!("[Packs] Skipping index creation - pack_id column not found in pack_items table");
            }
        }

        // Service: Create pack definition
        ctx.provide_service("create_pack", |input| async move {
            let name: String = serde_json::from_value(input["name"].clone())?;
            let description: String = serde_json::from_value(input["description"].clone())?;
            let cost: i64 = serde_json::from_value(input["cost"].clone())?;
            let image_url: Option<String> = serde_json::from_value(input["image_url"].clone()).ok();

            let conn = crate::core::database::get_database_path();
            let conn = rusqlite::Connection::open(conn)?;

            let now = current_timestamp();
            conn.execute(
                "INSERT INTO pack_definitions (name, description, cost, enabled, image_url, created_at, updated_at)
                 VALUES (?1, ?2, ?3, 1, ?4, ?5, ?5)",
                rusqlite::params![name, description, cost, image_url, now],
            )?;

            let pack_id = conn.last_insert_rowid();
            Ok(serde_json::json!({ "id": pack_id, "success": true }))
        }).await;

        // Service: Add item to pack
        ctx.provide_service("add_pack_item", |input| async move {
            let pack_id: i64 = serde_json::from_value(input["pack_id"].clone())?;
            let item_name: String = serde_json::from_value(input["item_name"].clone())?;
            let item_description: Option<String> = serde_json::from_value(input["item_description"].clone()).ok();
            let rarity: String = serde_json::from_value(input["rarity"].clone())?;
            let weight: i64 = serde_json::from_value(input["weight"].clone()).unwrap_or(1);

            let conn = crate::core::database::get_database_path();
            let conn = rusqlite::Connection::open(conn)?;

            let now = current_timestamp();
            conn.execute(
                "INSERT INTO pack_items (pack_id, item_name, item_description, rarity, weight, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                rusqlite::params![pack_id, item_name, item_description, rarity, weight, now],
            )?;

            Ok(serde_json::json!({ "id": conn.last_insert_rowid(), "success": true }))
        }).await;

        // Service: Get available packs
        ctx.provide_service("get_packs", |_input| async move {
            let conn = crate::core::database::get_database_path();
            let conn = rusqlite::Connection::open(conn)?;

            let mut stmt = conn.prepare(
                "SELECT id, name, description, cost, image_url FROM pack_definitions WHERE enabled = 1"
            )?;

            let packs: Vec<serde_json::Value> = stmt.query_map([], |row| {
                Ok(serde_json::json!({
                    "id": row.get::<_, i64>(0)?,
                    "name": row.get::<_, String>(1)?,
                    "description": row.get::<_, String>(2)?,
                    "cost": row.get::<_, i64>(3)?,
                    "image_url": row.get::<_, Option<String>>(4)?,
                }))
            })?.collect::<rusqlite::Result<Vec<_>>>()?;

            Ok(serde_json::json!({ "packs": packs }))
        }).await;

        // Service: Open pack
        ctx.provide_service("open_pack", |input| async move {
            let user_id: String = serde_json::from_value(input["user_id"].clone())?;
            let username: String = serde_json::from_value(input["username"].clone())?;
            let pack_id: i64 = serde_json::from_value(input["pack_id"].clone())?;

            let conn = crate::core::database::get_database_path();
            let conn = rusqlite::Connection::open(conn)?;

            // Get pack info
            let pack_info: (String, String, i64) = conn.query_row(
                "SELECT name, description, cost FROM pack_definitions WHERE id = ?1 AND enabled = 1",
                rusqlite::params![pack_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )?;

            let (pack_name, _pack_desc, cost) = pack_info;

            // Get all items in pack with weights
            let mut stmt = conn.prepare(
                "SELECT item_name, item_description, rarity, weight FROM pack_items WHERE pack_id = ?1"
            )?;

            let items: Vec<(String, Option<String>, String, i64)> = stmt.query_map(
                rusqlite::params![pack_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
            )?.collect::<rusqlite::Result<Vec<_>>>()?;

            if items.is_empty() {
                return Err(anyhow::anyhow!("Pack has no items"));
            }

            // Select random items based on weights (3 items per pack)
            let mut obtained_items = Vec::new();
            let items_to_draw = 3.min(items.len());

            for _ in 0..items_to_draw {
                let selected = select_weighted_item(&items);
                obtained_items.push(selected.clone());
            }

            // Add items to user inventory
            let now = current_timestamp();
            for (item_name, item_desc, rarity, _) in &obtained_items {
                conn.execute(
                    "INSERT INTO user_inventory (user_id, username, item_name, item_description, rarity, pack_name, obtained_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                    rusqlite::params![user_id, username, item_name, item_desc, rarity, pack_name, now],
                )?;
            }

            // Record pack opening
            let items_json = serde_json::to_string(&obtained_items)?;
            conn.execute(
                "INSERT INTO pack_openings (user_id, username, pack_id, pack_name, items_obtained, cost, opened_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                rusqlite::params![user_id, username, pack_id, pack_name, items_json, cost, now],
            )?;

            Ok(serde_json::json!({
                "success": true,
                "items": obtained_items.iter().map(|(name, desc, rarity, _)| {
                    serde_json::json!({
                        "name": name,
                        "description": desc,
                        "rarity": rarity
                    })
                }).collect::<Vec<_>>(),
                "cost": cost
            }))
        }).await;

        // Service: Get user inventory
        ctx.provide_service("get_inventory", |input| async move {
            let user_id: String = serde_json::from_value(input["user_id"].clone())?;

            let conn = crate::core::database::get_database_path();
            let conn = rusqlite::Connection::open(conn)?;

            let mut stmt = conn.prepare(
                "SELECT id, item_name, item_description, rarity, pack_name, obtained_at
                 FROM user_inventory WHERE user_id = ?1 ORDER BY obtained_at DESC"
            )?;

            let items: Vec<serde_json::Value> = stmt.query_map(rusqlite::params![user_id], |row| {
                Ok(serde_json::json!({
                    "id": row.get::<_, i64>(0)?,
                    "item_name": row.get::<_, String>(1)?,
                    "item_description": row.get::<_, Option<String>>(2)?,
                    "rarity": row.get::<_, String>(3)?,
                    "pack_name": row.get::<_, String>(4)?,
                    "obtained_at": row.get::<_, i64>(5)?,
                }))
            })?.collect::<rusqlite::Result<Vec<_>>>()?;

            Ok(serde_json::json!({ "items": items, "total": items.len() }))
        }).await;

        // Service: Get pack opening history
        ctx.provide_service("get_pack_history", |input| async move {
            let user_id: String = serde_json::from_value(input["user_id"].clone())?;

            let conn = crate::core::database::get_database_path();
            let conn = rusqlite::Connection::open(conn)?;

            let mut stmt = conn.prepare(
                "SELECT id, pack_name, items_obtained, cost, opened_at
                 FROM pack_openings WHERE user_id = ?1 ORDER BY opened_at DESC LIMIT 50"
            )?;

            let openings: Vec<serde_json::Value> = stmt.query_map(rusqlite::params![user_id], |row| {
                let items_json: String = row.get(2)?;
                let items: Vec<serde_json::Value> = serde_json::from_str(&items_json).unwrap_or_default();

                Ok(serde_json::json!({
                    "id": row.get::<_, i64>(0)?,
                    "pack_name": row.get::<_, String>(1)?,
                    "items": items,
                    "cost": row.get::<_, i64>(3)?,
                    "opened_at": row.get::<_, i64>(4)?,
                }))
            })?.collect::<rusqlite::Result<Vec<_>>>()?;

            Ok(serde_json::json!({ "openings": openings }))
        }).await;

        log::info!("[Packs] Plugin initialized successfully");
        Ok(())
    }

    async fn start(&self, ctx: Arc<PluginContext>) -> Result<()> {
        log::info!("[Packs] Starting plugin...");

        // Subscribe to pack purchase events
        tokio::spawn(async move {
            let mut events = ctx.subscribe_to("packs.purchase_request").await;

            while let Ok(event) = events.recv().await {
                if let (Ok(user_id), Ok(username), Ok(pack_id)) = (
                    serde_json::from_value::<String>(event.payload["user_id"].clone()),
                    serde_json::from_value::<String>(event.payload["username"].clone()),
                    serde_json::from_value::<i64>(event.payload["pack_id"].clone()),
                ) {
                    // Get pack cost
                    if let Ok(packs_result) = ctx.call_service("packs", "get_packs", serde_json::json!({})).await {
                        if let Some(packs) = packs_result["packs"].as_array() {
                            if let Some(pack) = packs.iter().find(|p| p["id"].as_i64() == Some(pack_id)) {
                                let cost = pack["cost"].as_i64().unwrap_or(0);

                                // Check balance
                                if let Ok(balance_result) = ctx.call_service("currency", "get_balance", serde_json::json!({
                                    "user_id": user_id
                                })).await {
                                    let balance = balance_result["balance"].as_i64().unwrap_or(0);

                                    if balance >= cost {
                                        // Deduct currency
                                        let _ = ctx.call_service("currency", "deduct_currency", serde_json::json!({
                                            "user_id": user_id,
                                            "username": username,
                                            "amount": cost,
                                            "reason": format!("Purchased pack: {}", pack["name"].as_str().unwrap_or("Unknown"))
                                        })).await;

                                        // Open pack
                                        if let Ok(open_result) = ctx.call_service("packs", "open_pack", serde_json::json!({
                                            "user_id": user_id,
                                            "username": username,
                                            "pack_id": pack_id
                                        })).await {
                                            // Emit pack opened event
                                            ctx.emit("packs.opened", &serde_json::json!({
                                                "user_id": user_id,
                                                "username": username,
                                                "pack_id": pack_id,
                                                "pack_name": pack["name"],
                                                "items": open_result["items"],
                                                "cost": cost
                                            }));
                                        }
                                    } else {
                                        // Emit insufficient funds event
                                        ctx.emit("packs.insufficient_funds", &serde_json::json!({
                                            "user_id": user_id,
                                            "username": username,
                                            "required": cost,
                                            "balance": balance
                                        }));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        });

        log::info!("[Packs] Plugin started successfully");
        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        log::info!("[Packs] Stopping plugin...");
        Ok(())
    }
}

fn current_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

fn select_weighted_item(items: &[(String, Option<String>, String, i64)]) -> (String, Option<String>, String, i64) {
    use rand::Rng;

    let total_weight: i64 = items.iter().map(|(_, _, _, w)| w).sum();
    let mut rng = rand::thread_rng();
    let mut roll = rng.gen_range(0..total_weight);

    for item in items {
        if roll < item.3 {
            return item.clone();
        }
        roll -= item.3;
    }

    items[0].clone()
}
