use crate::core::plugin::{Plugin, PluginMetadata};
use crate::core::plugin_context::PluginContext;
use async_trait::async_trait;
use std::sync::Arc;
use anyhow::Result;
use tokio::sync::RwLock;
use serenity::{
    async_trait as serenity_async_trait,
    model::{
        channel::Message,
        gateway::Ready,
        id::ChannelId,
    },
    prelude::*,
};

mod router;

pub struct DiscordPlugin;

// Discord bot state
pub struct BotState {
    pub client: Option<Client>,
    pub is_running: bool,
    pub config: BotConfig,
}

#[derive(Clone, Debug)]
pub struct BotConfig {
    pub token: Option<String>,
    pub command_prefix: String,
    pub enabled: bool,
}

impl Default for BotConfig {
    fn default() -> Self {
        Self {
            token: None,
            command_prefix: "!".to_string(),
            enabled: false,
        }
    }
}

// Global bot state
lazy_static::lazy_static! {
    pub static ref BOT_STATE: Arc<RwLock<BotState>> = Arc::new(RwLock::new(BotState {
        client: None,
        is_running: false,
        config: BotConfig::default(),
    }));
}

// Discord event handler
struct Handler;

#[serenity_async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _ctx: Context, ready: Ready) {
        log::info!("[Discord Bot] {} is connected!", ready.user.name);
    }

    async fn message(&self, ctx: Context, msg: Message) {
        // Ignore bot messages
        if msg.author.bot {
            return;
        }

        let state = BOT_STATE.read().await;
        let prefix = &state.config.command_prefix;

        // Check if message starts with prefix
        if msg.content.starts_with(prefix) {
            let command = msg.content[prefix.len()..].trim();
            handle_command(&ctx, &msg, command).await;
        }
    }
}

async fn handle_command(ctx: &Context, msg: &Message, command: &str) {
    let parts: Vec<&str> = command.split_whitespace().collect();
    if parts.is_empty() {
        return;
    }

    match parts[0].to_lowercase().as_str() {
        "ping" => {
            if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
                log::error!("[Discord Bot] Error sending message: {:?}", why);
            }
        }
        "help" => {
            let help_text = "**Available Commands:**\n\
                • `!ping` - Check if bot is responsive\n\
                • `!help` - Show this help message\n\
                • `!info` - Show bot information";

            if let Err(why) = msg.channel_id.say(&ctx.http, help_text).await {
                log::error!("[Discord Bot] Error sending message: {:?}", why);
            }
        }
        "info" => {
            let info_text = "**WebArcade Discord Bot**\n\
                A general-purpose Discord bot for your server.\n\
                Configure me through the WebArcade dashboard!";

            if let Err(why) = msg.channel_id.say(&ctx.http, info_text).await {
                log::error!("[Discord Bot] Error sending message: {:?}", why);
            }
        }
        _ => {
            // Unknown command - check custom commands from database
            check_custom_command(ctx, msg, parts[0]).await;
        }
    }
}

async fn check_custom_command(ctx: &Context, msg: &Message, command_name: &str) {
    let conn = crate::core::database::get_database_path();
    if let Ok(conn) = rusqlite::Connection::open(&conn) {
        let result = conn.query_row(
            "SELECT response FROM discord_commands WHERE name = ?1 AND enabled = 1",
            rusqlite::params![command_name],
            |row| row.get::<_, String>(0),
        );

        if let Ok(response) = result {
            if let Err(why) = msg.channel_id.say(&ctx.http, response).await {
                log::error!("[Discord Bot] Error sending message: {:?}", why);
            }
        }
    }
}

pub async fn start_bot(token: String) -> Result<()> {
    let mut state = BOT_STATE.write().await;

    if state.is_running {
        return Err(anyhow::anyhow!("Bot is already running"));
    }

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILDS;

    let client = Client::builder(&token, intents)
        .event_handler(Handler)
        .await?;

    state.is_running = true;
    state.config.token = Some(token);

    // Spawn client in background
    let mut client_clone = client;
    tokio::spawn(async move {
        if let Err(why) = client_clone.start().await {
            log::error!("[Discord Bot] Client error: {:?}", why);
            let mut state = BOT_STATE.write().await;
            state.is_running = false;
        }
    });

    log::info!("[Discord Bot] Started successfully");
    Ok(())
}

pub async fn stop_bot() -> Result<()> {
    let mut state = BOT_STATE.write().await;

    if !state.is_running {
        return Err(anyhow::anyhow!("Bot is not running"));
    }

    state.is_running = false;
    state.client = None;

    log::info!("[Discord Bot] Stopped successfully");
    Ok(())
}

pub async fn send_message(channel_id: u64, content: String) -> Result<()> {
    let state = BOT_STATE.read().await;

    if !state.is_running {
        return Err(anyhow::anyhow!("Bot is not running"));
    }

    if let Some(client) = &state.client {
        let channel = ChannelId::new(channel_id);
        channel.say(&client.http, content).await?;
        Ok(())
    } else {
        Err(anyhow::anyhow!("Bot client not available"))
    }
}

#[async_trait]
impl Plugin for DiscordPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            id: "discord".to_string(),
            name: "Discord Bot".to_string(),
            version: "1.0.0".to_string(),
            description: "General-purpose Discord bot with custom commands".to_string(),
            author: "WebArcade Team".to_string(),
            dependencies: vec![],
        }
    }

    async fn init(&self, ctx: &PluginContext) -> Result<()> {
        log::info!("[Discord] Initializing plugin...");

        // Initialize database tables
        let conn = crate::core::database::get_database_path();
        if let Ok(conn) = rusqlite::Connection::open(&conn) {
            // Create config table
            let _ = conn.execute(
                "CREATE TABLE IF NOT EXISTS discord_config (
                    id INTEGER PRIMARY KEY CHECK (id = 1),
                    bot_token TEXT,
                    command_prefix TEXT NOT NULL DEFAULT '!',
                    enabled INTEGER NOT NULL DEFAULT 0,
                    created_at INTEGER NOT NULL,
                    updated_at INTEGER NOT NULL
                )",
                [],
            );

            // Create commands table
            let _ = conn.execute(
                "CREATE TABLE IF NOT EXISTS discord_commands (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    name TEXT NOT NULL UNIQUE,
                    description TEXT,
                    response TEXT NOT NULL,
                    enabled INTEGER NOT NULL DEFAULT 1,
                    created_at INTEGER NOT NULL,
                    updated_at INTEGER NOT NULL
                )",
                [],
            );
        }

        // Register HTTP routes
        router::register_routes(ctx).await?;

        log::info!("[Discord] Plugin initialized successfully");
        Ok(())
    }

    async fn start(&self, _ctx: Arc<PluginContext>) -> Result<()> {
        log::info!("[Discord] Starting plugin...");

        // Load config from database and auto-start if enabled
        let conn = crate::core::database::get_database_path();
        if let Ok(conn) = rusqlite::Connection::open(&conn) {
            if let Ok((token, enabled)) = conn.query_row(
                "SELECT bot_token, enabled FROM discord_config WHERE id = 1",
                [],
                |row| Ok((row.get::<_, Option<String>>(0)?, row.get::<_, bool>(1)?)),
            ) {
                if enabled {
                    if let Some(token) = token {
                        if let Err(e) = start_bot(token).await {
                            log::error!("[Discord] Failed to auto-start bot: {}", e);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        log::info!("[Discord] Stopping plugin...");
        let _ = stop_bot().await;
        Ok(())
    }
}
