mod router;

use webarcade_api::prelude::*;
use std::sync::Arc;

pub struct SysmonitorPlugin;

#[async_trait]
impl Plugin for SysmonitorPlugin {
    plugin_metadata!("sysmonitor", "System Monitor", "1.0.0", "System monitoring plugin for CPU and RAM usage", author: "WebArcade");

    async fn init(&self, ctx: &Context) -> Result<()> {
        // Database migrations for storing historical data
        ctx.migrate(&[
            r"
            CREATE TABLE IF NOT EXISTS system_stats (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                cpu_usage REAL NOT NULL,
                ram_used INTEGER NOT NULL,
                ram_total INTEGER NOT NULL,
                timestamp INTEGER NOT NULL
            )
            ",
        ])?;

        // Register routes
        router::register_routes(ctx).await?;

        Ok(())
    }

    async fn start(&self, _ctx: Arc<Context>) -> Result<()> {
        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        Ok(())
    }
}
