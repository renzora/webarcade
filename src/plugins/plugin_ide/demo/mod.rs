mod router;

use webarcade_api::prelude::*;
use std::sync::Arc;

pub struct DemoPlugin;

#[async_trait]
impl Plugin for DemoPlugin {
    plugin_metadata!("demo", "demo", "1.0.0", "demo", author: "demo");

    async fn init(&self, ctx: &Context) -> Result<()> {
        log::info!("[demo] Initializing");

        // Database migrations
        ctx.migrate(&[
            r"
            CREATE TABLE IF NOT EXISTS demo_data (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                created_at INTEGER NOT NULL
            )
            ",
        ])?;

        // Register routes
        router::register_routes(ctx).await?;

        log::info!("[demo] Initialized successfully");
        Ok(())
    }

    async fn start(&self, _ctx: Arc<Context>) -> Result<()> {
        log::info!("[demo] Started");
        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        log::info!("[demo] Stopped");
        Ok(())
    }
}
