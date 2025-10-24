use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

use super::discord_bot::{DiscordBot, DiscordBotConfig};
use crate::commands::database::Database;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DiscordStatus {
    Disconnected,
    Connecting,
    Connected,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordStats {
    pub status: DiscordStatus,
    pub queue_size: i64,
}

pub struct DiscordManager {
    database: Arc<Database>,
    bot: Arc<DiscordBot>,
    status: Arc<RwLock<DiscordStatus>>,
    bot_handle: Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>,
}

impl DiscordManager {
    pub fn new(database: Arc<Database>) -> Self {
        let bot = Arc::new(DiscordBot::new(database.clone()));

        Self {
            database,
            bot,
            status: Arc::new(RwLock::new(DiscordStatus::Disconnected)),
            bot_handle: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn get_stats(&self) -> Result<DiscordStats> {
        let status = self.status.read().await.clone();
        let queue_size = self.database.get_pending_song_requests_count()?;

        Ok(DiscordStats { status, queue_size })
    }

    pub async fn start(&self) -> Result<()> {
        // Check if already running
        {
            let status = self.status.read().await;
            if matches!(*status, DiscordStatus::Connected | DiscordStatus::Connecting) {
                return Ok(());
            }
        }

        // Load config from database
        let config = self.database.get_discord_config()?;
        if config.bot_token.is_none() || config.channel_id.is_none() {
            anyhow::bail!("Discord bot not configured");
        }

        let bot_token = config.bot_token.unwrap();
        let channel_id = config.channel_id.unwrap();

        // Update status
        {
            let mut status = self.status.write().await;
            *status = DiscordStatus::Connecting;
        }

        // Set bot config
        self.bot
            .set_config(DiscordBotConfig {
                bot_token: bot_token.clone(),
                channel_id,
                command_prefix: config.command_prefix,
                max_queue_size: config.max_queue_size,
            })
            .await;

        // Start bot
        let bot = self.bot.clone();
        let status = self.status.clone();

        let handle = tokio::spawn(async move {
            match bot.start().await {
                Ok(mut client) => {
                    {
                        let mut status_guard = status.write().await;
                        *status_guard = DiscordStatus::Connected;
                    }

                    log::info!("🎵 Discord bot started successfully");

                    if let Err(e) = client.start().await {
                        log::error!("Discord client error: {}", e);
                        let mut status_guard = status.write().await;
                        *status_guard = DiscordStatus::Error;
                    }
                }
                Err(e) => {
                    log::error!("Failed to start Discord bot: {}", e);
                    let mut status_guard = status.write().await;
                    *status_guard = DiscordStatus::Error;
                }
            }
        });

        // Store handle
        {
            let mut bot_handle = self.bot_handle.write().await;
            *bot_handle = Some(handle);
        }

        Ok(())
    }

    pub async fn stop(&self) -> Result<()> {
        // Update status
        {
            let mut status = self.status.write().await;
            *status = DiscordStatus::Disconnected;
        }

        // Stop bot
        let mut bot_handle = self.bot_handle.write().await;
        if let Some(handle) = bot_handle.take() {
            handle.abort();
            log::info!("🎵 Discord bot stopped");
        }

        Ok(())
    }

    pub async fn restart(&self) -> Result<()> {
        self.stop().await?;
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        self.start().await?;
        Ok(())
    }
}
