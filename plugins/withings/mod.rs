use crate::core::plugin::Plugin;
use crate::core::plugin_context::PluginContext;
use crate::plugin_metadata;
use async_trait::async_trait;
use std::sync::Arc;
use anyhow::Result;

mod router;

pub struct WithingsPlugin;

#[async_trait]
impl Plugin for WithingsPlugin {
    plugin_metadata!("withings", "Withings", "1.0.0", "Withings health data integration for body composition tracking");

    async fn init(&self, ctx: &PluginContext) -> Result<()> {
        // Create database tables for Withings data
        ctx.migrate(&[
            r#"
            CREATE TABLE IF NOT EXISTS withings_measurements (
                id TEXT PRIMARY KEY,
                timestamp INTEGER NOT NULL,
                weight REAL,
                fat_mass REAL,
                muscle_mass REAL,
                hydration REAL,
                bone_mass REAL,
                fat_ratio REAL,
                fat_free_mass REAL,
                created_at INTEGER NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_measurements_time
            ON withings_measurements(timestamp DESC);
            "#,
        ])?;

        // Register HTTP routes
        router::register_routes(ctx).await?;

        Ok(())
    }

    async fn start(&self, _ctx: Arc<PluginContext>) -> Result<()> {
        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        Ok(())
    }
}
