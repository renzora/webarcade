use crate::core::plugin::Plugin;
use crate::core::plugin_context::PluginContext;
use crate::plugin_metadata;
use async_trait::async_trait;
use std::sync::Arc;
use anyhow::Result;
use sysinfo::System;

mod router;

pub struct SystemPlugin;

#[async_trait]
impl Plugin for SystemPlugin {
    plugin_metadata!("system", "System Monitor", "1.0.0", "System resource monitoring (CPU, RAM, GPU)");

    async fn init(&self, ctx: &PluginContext) -> Result<()> {
        log::info!("[System] Initializing plugin...");

        // Create system_settings table
        ctx.migrate(&[
            r#"
            CREATE TABLE IF NOT EXISTS system_settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            )
            "#,
        ])?;

        // Register services
        ctx.provide_service("get_stats", |_input| async move {
            let mut sys = System::new_all();
            sys.refresh_all();

            // Note: sysinfo 0.30 requires a delay for accurate CPU readings
            // For initial reading, we return 0.0 or use cached values
            let cpu_count = sys.cpus().len();
            let total_memory = sys.total_memory();
            let used_memory = sys.used_memory();
            let memory_usage = (used_memory as f64 / total_memory as f64) * 100.0;

            Ok(serde_json::json!({
                "cpu": {
                    "cores": cpu_count,
                },
                "memory": {
                    "total": total_memory,
                    "used": used_memory,
                    "usage_percent": memory_usage,
                },
            }))
        }).await;

        ctx.provide_service("get_cpu", |_input| async move {
            let sys = System::new_all();

            Ok(serde_json::json!({
                "cores": sys.cpus().len(),
            }))
        }).await;

        ctx.provide_service("get_memory", |_input| async move {
            let mut sys = System::new_all();
            sys.refresh_memory();

            let total_memory = sys.total_memory();
            let used_memory = sys.used_memory();
            let memory_usage = (used_memory as f64 / total_memory as f64) * 100.0;

            Ok(serde_json::json!({
                "total": total_memory,
                "used": used_memory,
                "usage_percent": memory_usage,
            }))
        }).await;

        // Register HTTP routes
        router::register_routes(ctx).await?;

        log::info!("[System] Plugin initialized successfully");
        Ok(())
    }

    async fn start(&self, ctx: Arc<PluginContext>) -> Result<()> {
        log::info!("[System] Starting plugin...");

        // Background monitoring task
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));

            loop {
                interval.tick().await;

                // Get system stats
                if let Ok(result) = ctx.call_service("system", "get_stats", serde_json::json!({})).await {
                    // Emit system stats event
                    ctx.emit("system.stats", &result);
                }
            }
        });

        log::info!("[System] Plugin started successfully");
        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        log::info!("[System] Stopping plugin...");
        Ok(())
    }
}
