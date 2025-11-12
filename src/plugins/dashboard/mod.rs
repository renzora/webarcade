use crate::core::plugin::Plugin;
use crate::core::plugin_context::PluginContext;
use crate::plugin_metadata;
use async_trait::async_trait;
use std::sync::Arc;
use anyhow::Result;

mod router;

pub struct DashboardPlugin;

#[async_trait]
impl Plugin for DashboardPlugin {
    plugin_metadata!("dashboard", "Dashboard", "1.0.0", "Dashboard widget management");

    async fn init(&self, ctx: &PluginContext) -> Result<()> {
        log::info!("[Dashboard] Initializing plugin...");

        // Create database schema
        ctx.migrate(&[
            // Migration 1: Create dashboards and widget_instances tables
            r#"
            CREATE TABLE IF NOT EXISTS dashboards (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS widget_instances (
                id TEXT PRIMARY KEY,
                dashboard_id TEXT NOT NULL,
                widget_id TEXT NOT NULL,
                order_index INTEGER NOT NULL,
                columns INTEGER NOT NULL DEFAULT 1,
                config TEXT,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL,
                FOREIGN KEY (dashboard_id) REFERENCES dashboards(id) ON DELETE CASCADE
            );

            CREATE INDEX IF NOT EXISTS idx_widget_instances_dashboard
                ON widget_instances(dashboard_id);

            -- Insert default dashboard if none exists
            INSERT OR IGNORE INTO dashboards (id, name, created_at, updated_at)
            VALUES ('default', 'Main Dashboard', strftime('%s', 'now'), strftime('%s', 'now'));
            "#,
        ])?;

        // Register HTTP routes
        router::register_routes(ctx).await?;

        log::info!("[Dashboard] Plugin initialized successfully");
        Ok(())
    }

    async fn start(&self, _ctx: Arc<PluginContext>) -> Result<()> {
        log::info!("[Dashboard] Starting plugin...");
        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        log::info!("[Dashboard] Stopping plugin...");
        Ok(())
    }
}
