use crate::core::plugin::Plugin;
use crate::core::plugin_context::PluginContext;
use crate::plugin_metadata;
use async_trait::async_trait;
use std::sync::Arc;
use anyhow::Result;

mod router;

pub struct PhilipsHuePlugin;

#[async_trait]
impl Plugin for PhilipsHuePlugin {
    plugin_metadata!("philips-hue", "Philips Hue", "1.0.0", "Control your Philips Hue smart lights");

    async fn init(&self, ctx: &PluginContext) -> Result<()> {
        log::info!("[Philips Hue] Initializing plugin...");

        // Create tables for storing Hue Bridge configuration and light states
        ctx.migrate(&[
            r#"
            CREATE TABLE IF NOT EXISTS hue_bridges (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                ip_address TEXT NOT NULL UNIQUE,
                username TEXT,
                created_at INTEGER NOT NULL,
                last_connected INTEGER
            )
            "#,
            r#"
            CREATE TABLE IF NOT EXISTS hue_lights (
                id TEXT PRIMARY KEY,
                bridge_id INTEGER NOT NULL,
                name TEXT NOT NULL,
                type TEXT NOT NULL,
                state_on INTEGER NOT NULL,
                brightness INTEGER,
                hue INTEGER,
                saturation INTEGER,
                color_temp INTEGER,
                reachable INTEGER NOT NULL,
                last_updated INTEGER NOT NULL,
                FOREIGN KEY (bridge_id) REFERENCES hue_bridges(id) ON DELETE CASCADE
            )
            "#,
            r#"
            CREATE TABLE IF NOT EXISTS hue_groups (
                id TEXT PRIMARY KEY,
                bridge_id INTEGER NOT NULL,
                name TEXT NOT NULL,
                type TEXT NOT NULL,
                lights TEXT NOT NULL,
                state_on INTEGER NOT NULL,
                brightness INTEGER,
                last_updated INTEGER NOT NULL,
                FOREIGN KEY (bridge_id) REFERENCES hue_bridges(id) ON DELETE CASCADE
            )
            "#,
        ])?;

        log::info!("[Philips Hue] Tables created successfully");

        // Register HTTP routes
        router::register_routes(ctx).await?;

        log::info!("[Philips Hue] Plugin initialized successfully");
        Ok(())
    }

    async fn start(&self, _ctx: Arc<PluginContext>) -> Result<()> {
        log::info!("[Philips Hue] Starting plugin...");
        log::info!("[Philips Hue] Plugin started successfully");
        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        log::info!("[Philips Hue] Stopping plugin...");
        Ok(())
    }
}
