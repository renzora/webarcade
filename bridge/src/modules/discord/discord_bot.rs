use anyhow::Result;
use serenity::{
    async_trait,
    client::{Client, Context, EventHandler},
    model::{channel::Message, gateway::Ready},
    prelude::*,
};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::commands::database::Database;

#[derive(Clone)]
pub struct DiscordBotConfig {
    pub bot_token: String,
    pub channel_id: String,
    pub command_prefix: String,
    pub max_queue_size: i64,
}

pub struct DiscordBot {
    config: Arc<RwLock<Option<DiscordBotConfig>>>,
    database: Arc<Database>,
}

struct Handler {
    config: Arc<RwLock<Option<DiscordBotConfig>>>,
    database: Arc<Database>,
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        // Ignore bot messages
        if msg.author.bot {
            return;
        }

        let config_guard = self.config.read().await;
        let config = match config_guard.as_ref() {
            Some(c) => c,
            None => return,
        };

        // Check if message is in the configured channel
        if msg.channel_id.to_string() != config.channel_id {
            return;
        }

        // Check if message starts with the command prefix
        if !msg.content.starts_with(&config.command_prefix) {
            return;
        }

        // Extract song query
        let song_query = msg.content[config.command_prefix.len()..].trim();
        if song_query.is_empty() {
            if let Err(e) = msg.reply(&ctx.http, "Please provide a song name!").await {
                log::error!("Failed to send reply: {}", e);
            }
            return;
        }

        // Check queue size
        match self.database.get_pending_song_requests_count() {
            Ok(count) => {
                if count >= config.max_queue_size {
                    if let Err(e) = msg.reply(&ctx.http, "Song queue is full! Please wait for some songs to play.").await {
                        log::error!("Failed to send reply: {}", e);
                    }
                    return;
                }
            }
            Err(e) => {
                log::error!("Failed to check queue size: {}", e);
                return;
            }
        }

        // Add song request to database
        match self.database.add_song_request(
            song_query,
            &msg.author.id.to_string(),
            &msg.author.name,
        ) {
            Ok(_) => {
                let reply = format!("✅ Added **{}** to the queue!", song_query);
                if let Err(e) = msg.reply(&ctx.http, &reply).await {
                    log::error!("Failed to send reply: {}", e);
                }
                log::info!("Song requested by {}: {}", msg.author.name, song_query);
            }
            Err(e) => {
                log::error!("Failed to add song request: {}", e);
                if let Err(e) = msg.reply(&ctx.http, "Failed to add song to queue. Please try again.").await {
                    log::error!("Failed to send error reply: {}", e);
                }
            }
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        log::info!("🎵 Discord bot is ready! Logged in as {}", ready.user.name);
    }
}

impl DiscordBot {
    pub fn new(database: Arc<Database>) -> Self {
        Self {
            config: Arc::new(RwLock::new(None)),
            database,
        }
    }

    pub async fn set_config(&self, config: DiscordBotConfig) {
        let mut config_guard = self.config.write().await;
        *config_guard = Some(config);
    }

    pub async fn start(&self) -> Result<Client> {
        let config_guard = self.config.read().await;
        let config = config_guard
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Discord bot config not set"))?;

        let token = config.bot_token.clone();
        drop(config_guard);

        let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

        let handler = Handler {
            config: self.config.clone(),
            database: self.database.clone(),
        };

        let client = Client::builder(&token, intents)
            .event_handler(handler)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to create Discord client: {}", e))?;

        Ok(client)
    }
}
