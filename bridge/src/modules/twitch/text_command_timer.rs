use anyhow::Result;
use std::sync::Arc;
use tokio::time::{interval, Duration};
use tokio::sync::RwLock;

use super::twitch_config::TwitchConfigManager;
use super::twitch_irc_client::TwitchIRCManager;
use crate::commands::database::Database;

/// Timer system for auto-posting text commands
pub struct TextCommandTimer {
    db: Database,
    irc_manager: Arc<TwitchIRCManager>,
    config_manager: Arc<TwitchConfigManager>,
    is_running: Arc<RwLock<bool>>,
}

impl TextCommandTimer {
    pub fn new(
        db: Database,
        irc_manager: Arc<TwitchIRCManager>,
        config_manager: Arc<TwitchConfigManager>,
    ) -> Self {
        Self {
            db,
            irc_manager,
            config_manager,
            is_running: Arc::new(RwLock::new(false)),
        }
    }

    /// Start the timer system
    pub async fn start(&self) -> Result<()> {
        {
            let is_running = self.is_running.read().await;
            if *is_running {
                log::warn!("Text command timer already running");
                return Ok(());
            }
        }

        {
            let mut is_running = self.is_running.write().await;
            *is_running = true;
        }

        let db = self.db.clone();
        let irc_manager = self.irc_manager.clone();
        let config_manager = self.config_manager.clone();
        let is_running = self.is_running.clone();

        tokio::spawn(async move {
            // Check every 30 seconds for commands that need to be posted
            let mut check_interval = interval(Duration::from_secs(30));

            log::info!("⏰ Text command timer started");

            loop {
                check_interval.tick().await;

                // Check if we should stop
                if !*is_running.read().await {
                    log::info!("Text command timer stopped");
                    break;
                }

                // Get configured channels
                if let Ok(config) = config_manager.load() {
                    for channel in &config.channels {
                        // Get due auto-post commands for this channel
                        match db.get_due_auto_post_commands(channel) {
                            Ok(commands) => {
                                for (command_name, response, _interval) in commands {
                                    // Check if stream is live (optional: only post when streaming)
                                    let should_post = match db.is_stream_live(channel) {
                                        Ok(is_live) => is_live,
                                        Err(_) => true, // Post anyway if we can't check
                                    };

                                    if should_post {
                                        // Post the command response
                                        match irc_manager.send_message(channel, &response).await {
                                            Ok(_) => {
                                                log::info!("⏰ Auto-posted text command '{}' in {}", command_name, channel);
                                                // Update last_posted_at
                                                if let Err(e) = db.update_text_command_posted(channel, &command_name) {
                                                    log::error!("Failed to update last_posted_at for {}: {}", command_name, e);
                                                }
                                            }
                                            Err(e) => {
                                                log::error!("Failed to auto-post command '{}': {}", command_name, e);
                                            }
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                log::error!("Failed to get auto-post commands for {}: {}", channel, e);
                            }
                        }
                    }
                }
            }
        });

        Ok(())
    }

    /// Stop the timer system
    pub async fn stop(&self) {
        let mut is_running = self.is_running.write().await;
        *is_running = false;
        log::info!("Stopping text command timer");
    }
}
