use crate::core::plugin::Plugin;
use crate::core::plugin_context::PluginContext;
use crate::plugin_metadata;
use async_trait::async_trait;
use std::sync::Arc;
use anyhow::Result;

mod router;
mod twitch_irc;
mod twitch_eventsub;
mod twitch_api;

pub struct TwitchPlugin;

#[async_trait]
impl Plugin for TwitchPlugin {
    plugin_metadata!("twitch", "Twitch Integration", "1.0.0", "Twitch integration with broadcaster and bot account support, IRC, and EventSub");

    async fn init(&self, ctx: &PluginContext) -> Result<()> {
        log::info!("[Twitch] Initializing plugin...");

        // Create tables for storing Twitch configuration and state
        ctx.migrate(&[
            r#"
            CREATE TABLE IF NOT EXISTS twitch_accounts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                account_type TEXT NOT NULL CHECK(account_type IN ('broadcaster', 'bot')),
                username TEXT,
                user_id TEXT,
                access_token TEXT,
                refresh_token TEXT,
                token_expires_at INTEGER,
                scopes TEXT,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL,
                UNIQUE(account_type)
            )
            "#,
            r#"
            -- Migration to fix user_id UNIQUE constraint issue
            -- Drop and recreate table if it has the wrong schema
            DROP TABLE IF EXISTS twitch_accounts_old;
            CREATE TABLE twitch_accounts_old AS SELECT * FROM twitch_accounts;
            DROP TABLE twitch_accounts;
            CREATE TABLE twitch_accounts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                account_type TEXT NOT NULL CHECK(account_type IN ('broadcaster', 'bot')),
                username TEXT,
                user_id TEXT,
                access_token TEXT,
                refresh_token TEXT,
                token_expires_at INTEGER,
                scopes TEXT,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL,
                UNIQUE(account_type)
            );
            INSERT OR IGNORE INTO twitch_accounts
            SELECT * FROM twitch_accounts_old;
            DROP TABLE twitch_accounts_old;
            "#,
            r#"
            CREATE TABLE IF NOT EXISTS twitch_irc_messages (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                channel TEXT NOT NULL,
                username TEXT NOT NULL,
                user_id TEXT,
                message TEXT NOT NULL,
                timestamp INTEGER NOT NULL,
                is_action INTEGER DEFAULT 0,
                badges TEXT,
                color TEXT,
                display_name TEXT,
                emotes TEXT
            )
            "#,
            r#"
            CREATE TABLE IF NOT EXISTS twitch_eventsub_subscriptions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                subscription_id TEXT UNIQUE,
                subscription_type TEXT NOT NULL,
                version TEXT NOT NULL,
                status TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                cost INTEGER DEFAULT 0,
                condition TEXT
            )
            "#,
            r#"
            CREATE TABLE IF NOT EXISTS twitch_events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                event_type TEXT NOT NULL,
                event_data TEXT NOT NULL,
                subscription_id TEXT,
                timestamp INTEGER NOT NULL
            )
            "#,
            r#"
            CREATE TABLE IF NOT EXISTS twitch_settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                updated_at INTEGER NOT NULL
            )
            "#,
        ])?;

        log::info!("[Twitch] Tables created successfully");

        // Register HTTP routes
        router::register_routes(ctx).await?;

        // Provide services for other plugins to use
        let ctx_clone = ctx.clone();
        ctx.provide_service("send_chat_message", move |input| {
            let ctx = ctx_clone.clone();
            async move {
                let channel = input.get("channel")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing channel parameter"))?;
                let message = input.get("message")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing message parameter"))?;

                // Send message via IRC
                twitch_irc::send_message(&ctx, channel, message).await?;

                Ok(serde_json::json!({
                    "success": true,
                    "channel": channel,
                    "message": message
                }))
            }
        }).await;

        let ctx_clone = ctx.clone();
        ctx.provide_service("get_channel_info", move |input| {
            let ctx = ctx_clone.clone();
            async move {
                let channel = input.get("channel")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing channel parameter"))?;

                // Get channel info from Twitch API
                let info = twitch_api::get_channel_info(&ctx, channel).await?;

                Ok(info)
            }
        }).await;

        let ctx_clone = ctx.clone();
        ctx.provide_service("get_broadcaster_token", move |_input| {
            let ctx = ctx_clone.clone();
            async move {
                twitch_api::get_account_token(&ctx, "broadcaster").await
            }
        }).await;

        log::info!("[Twitch] Plugin initialized successfully");
        Ok(())
    }

    async fn start(&self, ctx: Arc<PluginContext>) -> Result<()> {
        log::info!("[Twitch] Starting plugin...");

        // Start IRC client in background
        let ctx_irc = ctx.clone();
        tokio::spawn(async move {
            if let Err(e) = twitch_irc::start_irc_client(ctx_irc).await {
                log::error!("[Twitch IRC] Error: {}", e);
            }
        });

        // Start EventSub listener in background
        let ctx_eventsub = ctx.clone();
        tokio::spawn(async move {
            if let Err(e) = twitch_eventsub::start_eventsub_listener(ctx_eventsub).await {
                log::error!("[Twitch EventSub] Error: {}", e);
            }
        });

        log::info!("[Twitch] Plugin started successfully");
        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        log::info!("[Twitch] Stopping plugin...");
        Ok(())
    }
}
