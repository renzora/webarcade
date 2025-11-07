use crate::core::plugin::{Plugin, PluginMetadata};
use crate::core::plugin_context::PluginContext;
use async_trait::async_trait;
use std::sync::Arc;
use anyhow::Result;

mod events;
mod database;
mod router;

pub use events::*;
pub use database::*;

pub struct AuctionPlugin;

#[async_trait]
impl Plugin for AuctionPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            id: "auction".to_string(),
            name: "Auction System".to_string(),
            version: "1.0.0".to_string(),
            description: "Live auction system for collectible items".to_string(),
            author: "WebArcade Team".to_string(),
            dependencies: vec![], // Could depend on "currency" plugin if needed
        }
    }

    async fn init(&self, ctx: &PluginContext) -> Result<()> {
        log::info!("[Auction] Initializing plugin...");

        // Register HTTP routes
        router::register_routes(ctx).await?;

        // Run database migrations
        ctx.migrate(&[
            // Migration 1: Create auctions and bids tables
            r#"
            CREATE TABLE IF NOT EXISTS auctions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                item_id TEXT NOT NULL,
                item_name TEXT NOT NULL,
                item_rarity TEXT NOT NULL,
                starting_bid INTEGER NOT NULL,
                current_bid INTEGER,
                current_bidder TEXT,
                created_by TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'active',
                created_at INTEGER NOT NULL,
                ends_at INTEGER NOT NULL,
                ended_at INTEGER
            );

            CREATE TABLE IF NOT EXISTS auction_bids (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                auction_id INTEGER NOT NULL,
                user_id TEXT NOT NULL,
                username TEXT NOT NULL,
                amount INTEGER NOT NULL,
                created_at INTEGER NOT NULL,
                FOREIGN KEY (auction_id) REFERENCES auctions(id)
            );

            CREATE INDEX IF NOT EXISTS idx_auctions_status ON auctions(status);
            CREATE INDEX IF NOT EXISTS idx_auction_bids_auction_id ON auction_bids(auction_id);
            "#,
        ])?;

        // Register services that other plugins can call
        let ctx_clone = ctx.plugin_id().to_string();
        ctx.provide_service("create_auction", move |input| {
            let plugin_id = ctx_clone.clone();
            async move {
                let req: CreateAuctionRequest = serde_json::from_value(input)?;

                // Get database connection
                let ctx = PluginContext::new(
                    plugin_id,
                    Arc::new(crate::core::EventBus::new()),
                    Arc::new(crate::core::ServiceRegistry::new()),
                    crate::core::RouterRegistry::new(),
                    serde_json::json!({}),
                    crate::core::database::get_database_path().to_string_lossy().to_string(),
                );
                let conn = ctx.db()?;

                let auction_id = database::create_auction(
                    &conn,
                    &req.item_id,
                    &req.item_name,
                    &req.item_rarity,
                    req.starting_bid,
                    &req.created_by,
                    req.duration_seconds,
                )?;

                Ok(serde_json::json!({ "auction_id": auction_id }))
            }
        }).await;

        ctx.provide_service("place_bid", move |input| async move {
            let req: PlaceBidRequest = serde_json::from_value(input)?;
            // Implementation here
            Ok(serde_json::json!({ "success": true }))
        }).await;

        ctx.provide_service("get_active_auctions", move |_input| async move {
            // Implementation here
            Ok(serde_json::json!({ "auctions": [] }))
        }).await;

        log::info!("[Auction] Plugin initialized successfully");
        Ok(())
    }

    async fn start(&self, ctx: Arc<PluginContext>) -> Result<()> {
        log::info!("[Auction] Starting plugin...");

        // Subscribe to events from other plugins if needed
        // For example, listen to Twitch chat for !auction commands
        let ctx_clone = ctx.clone();
        tokio::spawn(async move {
            let mut events = ctx_clone.subscribe_to("twitch.chat_message").await;

            while let Ok(event) = events.recv().await {
                // Handle Twitch messages
                log::debug!("[Auction] Received event: {:?}", event);
                // Parse and handle auction commands
            }
        });

        // Background task to check for ended auctions
        let ctx_clone = ctx.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(5));
            loop {
                interval.tick().await;

                // Check for ended auctions
                if let Ok(conn) = ctx_clone.db() {
                    if let Err(e) = database::check_ended_auctions(&conn, &ctx_clone) {
                        log::error!("[Auction] Error checking ended auctions: {}", e);
                    }
                }
            }
        });

        log::info!("[Auction] Plugin started successfully");
        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        log::info!("[Auction] Stopping plugin...");
        Ok(())
    }
}
