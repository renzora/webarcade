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

pub struct UserProfilesPlugin;

#[async_trait]
impl Plugin for UserProfilesPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            id: "user_profiles".to_string(),
            name: "User Profiles".to_string(),
            version: "1.0.0".to_string(),
            description: "User profile management with birthdays, locations, and custom fields".to_string(),
            author: "WebArcade Team".to_string(),
            dependencies: vec![],
        }
    }

    async fn init(&self, ctx: &PluginContext) -> Result<()> {
        log::info!("[UserProfiles] Initializing plugin...");

        ctx.migrate(&[
            r#"
            CREATE TABLE IF NOT EXISTS user_profiles (
                user_id TEXT PRIMARY KEY,
                username TEXT NOT NULL,
                display_name TEXT,
                birthday TEXT,
                location TEXT,
                timezone TEXT,
                bio TEXT,
                pronouns TEXT,
                social_links TEXT,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS user_profile_fields (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id TEXT NOT NULL,
                field_name TEXT NOT NULL,
                field_value TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL,
                UNIQUE(user_id, field_name),
                FOREIGN KEY (user_id) REFERENCES user_profiles(user_id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS birthday_reminders (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id TEXT NOT NULL,
                username TEXT NOT NULL,
                birthday TEXT NOT NULL,
                last_reminded INTEGER,
                created_at INTEGER NOT NULL,
                FOREIGN KEY (user_id) REFERENCES user_profiles(user_id) ON DELETE CASCADE
            );

            CREATE INDEX IF NOT EXISTS idx_user_profile_fields_user_id ON user_profile_fields(user_id);
            CREATE INDEX IF NOT EXISTS idx_birthday_reminders_birthday ON birthday_reminders(birthday);
            "#,
        ])?;

        // Service: Create or update profile
        ctx.provide_service("update_profile", |input| async move {
            let user_id: String = serde_json::from_value(input["user_id"].clone())?;
            let username: String = serde_json::from_value(input["username"].clone())?;
            let display_name: Option<String> = serde_json::from_value(input["display_name"].clone()).ok();
            let birthday: Option<String> = serde_json::from_value(input["birthday"].clone()).ok();
            let location: Option<String> = serde_json::from_value(input["location"].clone()).ok();
            let timezone: Option<String> = serde_json::from_value(input["timezone"].clone()).ok();
            let bio: Option<String> = serde_json::from_value(input["bio"].clone()).ok();
            let pronouns: Option<String> = serde_json::from_value(input["pronouns"].clone()).ok();
            let social_links: Option<String> = serde_json::from_value(input["social_links"].clone()).ok();

            let conn = crate::core::database::get_database_path();
            let conn = rusqlite::Connection::open(conn)?;

            let now = current_timestamp();

            // Check if profile exists
            let exists: bool = conn.query_row(
                "SELECT COUNT(*) FROM user_profiles WHERE user_id = ?1",
                rusqlite::params![user_id],
                |row| row.get::<_, i64>(0).map(|count| count > 0),
            )?;

            if exists {
                // Update existing profile
                conn.execute(
                    "UPDATE user_profiles SET username = ?1, display_name = ?2, birthday = ?3,
                     location = ?4, timezone = ?5, bio = ?6, pronouns = ?7, social_links = ?8, updated_at = ?9
                     WHERE user_id = ?10",
                    rusqlite::params![username, display_name, birthday, location, timezone, bio, pronouns, social_links, now, user_id],
                )?;
            } else {
                // Create new profile
                conn.execute(
                    "INSERT INTO user_profiles (user_id, username, display_name, birthday, location, timezone, bio, pronouns, social_links, created_at, updated_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?10)",
                    rusqlite::params![user_id, username, display_name, birthday, location, timezone, bio, pronouns, social_links, now],
                )?;
            }

            // If birthday was set, update birthday reminder
            if let Some(bday) = birthday {
                conn.execute(
                    "INSERT OR REPLACE INTO birthday_reminders (user_id, username, birthday, created_at)
                     VALUES (?1, ?2, ?3, ?4)",
                    rusqlite::params![user_id, username, bday, now],
                )?;
            }

            Ok(serde_json::json!({ "success": true }))
        }).await;

        // Service: Get profile
        ctx.provide_service("get_profile", |input| async move {
            let user_id: String = serde_json::from_value(input["user_id"].clone())?;

            let conn = crate::core::database::get_database_path();
            let conn = rusqlite::Connection::open(conn)?;

            let profile: Option<serde_json::Value> = conn.query_row(
                "SELECT user_id, username, display_name, birthday, location, timezone, bio, pronouns, social_links, created_at, updated_at
                 FROM user_profiles WHERE user_id = ?1",
                rusqlite::params![user_id],
                |row| {
                    Ok(serde_json::json!({
                        "user_id": row.get::<_, String>(0)?,
                        "username": row.get::<_, String>(1)?,
                        "display_name": row.get::<_, Option<String>>(2)?,
                        "birthday": row.get::<_, Option<String>>(3)?,
                        "location": row.get::<_, Option<String>>(4)?,
                        "timezone": row.get::<_, Option<String>>(5)?,
                        "bio": row.get::<_, Option<String>>(6)?,
                        "pronouns": row.get::<_, Option<String>>(7)?,
                        "social_links": row.get::<_, Option<String>>(8)?,
                        "created_at": row.get::<_, i64>(9)?,
                        "updated_at": row.get::<_, i64>(10)?,
                    }))
                }
            ).optional()?;

            // Get custom fields
            let mut custom_fields = serde_json::Map::new();
            if profile.is_some() {
                let mut stmt = conn.prepare(
                    "SELECT field_name, field_value FROM user_profile_fields WHERE user_id = ?1"
                )?;

                let fields: Vec<(String, String)> = stmt.query_map(rusqlite::params![user_id], |row| {
                    Ok((row.get(0)?, row.get(1)?))
                })?.collect::<rusqlite::Result<Vec<_>>>()?;

                for (name, value) in fields {
                    custom_fields.insert(name, serde_json::Value::String(value));
                }
            }

            Ok(serde_json::json!({
                "profile": profile,
                "custom_fields": custom_fields
            }))
        }).await;

        // Service: Set custom field
        ctx.provide_service("set_custom_field", |input| async move {
            let user_id: String = serde_json::from_value(input["user_id"].clone())?;
            let field_name: String = serde_json::from_value(input["field_name"].clone())?;
            let field_value: String = serde_json::from_value(input["field_value"].clone())?;

            let conn = crate::core::database::get_database_path();
            let conn = rusqlite::Connection::open(conn)?;

            let now = current_timestamp();

            conn.execute(
                "INSERT OR REPLACE INTO user_profile_fields (user_id, field_name, field_value, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?4)",
                rusqlite::params![user_id, field_name, field_value, now],
            )?;

            Ok(serde_json::json!({ "success": true }))
        }).await;

        // Service: Get today's birthdays
        ctx.provide_service("get_todays_birthdays", |_input| async move {
            let conn = crate::core::database::get_database_path();
            let conn = rusqlite::Connection::open(conn)?;

            // Get current date in MM-DD format
            let now = chrono::Utc::now();
            let today = now.format("%m-%d").to_string();

            let mut stmt = conn.prepare(
                "SELECT user_id, username, birthday FROM birthday_reminders WHERE substr(birthday, 6, 5) = ?1"
            )?;

            let birthdays: Vec<serde_json::Value> = stmt.query_map(rusqlite::params![today], |row| {
                Ok(serde_json::json!({
                    "user_id": row.get::<_, String>(0)?,
                    "username": row.get::<_, String>(1)?,
                    "birthday": row.get::<_, String>(2)?,
                }))
            })?.collect::<rusqlite::Result<Vec<_>>>()?;

            Ok(serde_json::json!({ "birthdays": birthdays }))
        }).await;

        // Service: Search profiles
        ctx.provide_service("search_profiles", |input| async move {
            let query: String = serde_json::from_value(input["query"].clone())?;
            let limit: i64 = serde_json::from_value(input["limit"].clone()).unwrap_or(50);

            let conn = crate::core::database::get_database_path();
            let conn = rusqlite::Connection::open(conn)?;

            let search_pattern = format!("%{}%", query);
            let mut stmt = conn.prepare(
                "SELECT user_id, username, display_name, location, bio
                 FROM user_profiles
                 WHERE username LIKE ?1 OR display_name LIKE ?1 OR location LIKE ?1
                 LIMIT ?2"
            )?;

            let profiles: Vec<serde_json::Value> = stmt.query_map(
                rusqlite::params![search_pattern, limit],
                |row| {
                    Ok(serde_json::json!({
                        "user_id": row.get::<_, String>(0)?,
                        "username": row.get::<_, String>(1)?,
                        "display_name": row.get::<_, Option<String>>(2)?,
                        "location": row.get::<_, Option<String>>(3)?,
                        "bio": row.get::<_, Option<String>>(4)?,
                    }))
                }
            )?.collect::<rusqlite::Result<Vec<_>>>()?;

            Ok(serde_json::json!({ "profiles": profiles }))
        }).await;

        // Register HTTP routes
        router::register_routes(ctx).await?;

        log::info!("[UserProfiles] Plugin initialized successfully");
        Ok(())
    }

    async fn start(&self, ctx: Arc<PluginContext>) -> Result<()> {
        log::info!("[UserProfiles] Starting plugin...");

        // Background task: Check for birthdays daily
        let ctx_clone = ctx.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(3600)); // Check every hour

            loop {
                interval.tick().await;

                if let Ok(result) = ctx_clone.call_service("user_profiles", "get_todays_birthdays", serde_json::json!({})).await {
                    if let Some(birthdays) = result["birthdays"].as_array() {
                        if !birthdays.is_empty() {
                            // Emit birthday event for each user
                            for birthday in birthdays {
                                ctx_clone.emit("user_profiles.birthday", birthday);
                            }
                        }
                    }
                }
            }
        });

        log::info!("[UserProfiles] Plugin started successfully");
        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        log::info!("[UserProfiles] Stopping plugin...");
        Ok(())
    }
}

fn current_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}
