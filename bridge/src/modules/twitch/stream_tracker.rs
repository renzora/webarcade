use anyhow::Result;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};
use crate::commands::database::Database;
use super::twitch_api::TwitchAPI;
use super::twitch_config::TwitchConfigManager;

/// Tracks stream status and viewer watchtime automatically
pub struct StreamTracker {
    api: Arc<TwitchAPI>,
    db: Database,
    config_manager: Arc<TwitchConfigManager>,
    is_running: Arc<RwLock<bool>>,
    active_chatters: Arc<RwLock<HashSet<String>>>, // Track who has chatted recently
    bot_user_id: Arc<RwLock<Option<String>>>, // Cached bot user ID
}

impl StreamTracker {
    pub fn new(
        api: Arc<TwitchAPI>,
        db: Database,
        config_manager: Arc<TwitchConfigManager>,
    ) -> Self {
        Self {
            api,
            db,
            config_manager,
            is_running: Arc::new(RwLock::new(false)),
            active_chatters: Arc::new(RwLock::new(HashSet::new())),
            bot_user_id: Arc::new(RwLock::new(None)),
        }
    }

    /// Mark a user as active (they sent a message)
    pub async fn mark_user_active(&self, username: &str) {
        let mut chatters = self.active_chatters.write().await;
        chatters.insert(username.to_lowercase());
    }

    /// Start the automatic stream and watchtime tracking
    pub async fn start(&self) -> Result<()> {
        {
            let is_running = self.is_running.read().await;
            if *is_running {
                log::warn!("Stream tracker already running");
                return Ok(());
            }
        }

        {
            let mut is_running = self.is_running.write().await;
            *is_running = true;
        }

        let api = self.api.clone();
        let db = self.db.clone();
        let config_manager = self.config_manager.clone();
        let is_running = self.is_running.clone();
        let _active_chatters = self.active_chatters.clone();
        let bot_user_id = self.bot_user_id.clone();

        tokio::spawn(async move {
            let mut check_interval = interval(Duration::from_secs(60)); // Check stream status every 60 seconds
            let mut watchtime_interval = interval(Duration::from_secs(60)); // Update watchtime every 60 seconds (1 minute)

            log::info!("🎬 Stream tracker started");

            // Fetch and cache the bot's user ID
            let moderator_id = match api.get_authenticated_user().await {
                Ok(user) => {
                    log::info!("✅ Bot authenticated as: {} (ID: {})", user.login, user.id);
                    let mut bot_id = bot_user_id.write().await;
                    *bot_id = Some(user.id.clone());
                    user.id
                }
                Err(e) => {
                    log::error!("❌ Failed to get authenticated user: {}", e);
                    log::error!("Stream tracker cannot start without bot user ID");
                    return;
                }
            };

            loop {
                tokio::select! {
                    _ = check_interval.tick() => {
                        // Check if we should stop
                        if !*is_running.read().await {
                            log::info!("Stream tracker stopped");
                            break;
                        }

                        // Check stream status for each configured channel
                        if let Ok(config) = config_manager.load() {
                            for channel in &config.channels {
                                match Self::check_and_update_stream_status(&api, &db, channel).await {
                                    Ok(is_live) => {
                                        if is_live {
                                            log::debug!("Channel {} is live", channel);
                                        }
                                    }
                                    Err(e) => {
                                        log::error!("Failed to check stream status for {}: {}", channel, e);
                                    }
                                }
                            }
                        }
                    }

                    _ = watchtime_interval.tick() => {
                        // Check if we should stop
                        if !*is_running.read().await {
                            break;
                        }

                        // Update watchtime for ALL users in chat (including lurkers)
                        if let Ok(config) = config_manager.load() {
                            for channel in &config.channels {
                                // Only update if stream is live
                                match db.is_stream_live(channel) {
                                    Ok(true) => {
                                        // Get broadcaster info to fetch chatters
                                        match api.get_user_by_login(channel).await {
                                            Ok(Some(broadcaster)) => {
                                                // Fetch all users currently in chat (including lurkers)
                                                // Use bot's user ID as moderator_id, not broadcaster's ID
                                                match api.get_chatters(&broadcaster.id, &moderator_id).await {
                                                    Ok(chatters) => {
                                                        if !chatters.is_empty() {
                                                            log::info!("Updating watchtime for {} users in {} (including lurkers)", chatters.len(), channel);

                                                            // Update watchtime for everyone in chat (1 minute)
                                                            for chatter in chatters.iter() {
                                                                match db.update_watchtime(channel, &chatter.user_login, 1) {
                                                                    Ok(total) => {
                                                                        log::debug!("Updated watchtime for {}: {} minutes total", chatter.user_login, total);
                                                                    }
                                                                    Err(e) => {
                                                                        log::error!("Failed to update watchtime for {}: {}", chatter.user_login, e);
                                                                    }
                                                                }
                                                            }
                                                        } else {
                                                            log::debug!("No users in chat for {}", channel);
                                                        }
                                                    }
                                                    Err(e) => {
                                                        log::error!("Failed to fetch chatters for {}: {}", channel, e);
                                                    }
                                                }
                                            }
                                            Ok(None) => {
                                                log::warn!("Broadcaster user not found for channel: {}", channel);
                                            }
                                            Err(e) => {
                                                log::error!("Failed to get broadcaster info for {}: {}", channel, e);
                                            }
                                        }
                                    }
                                    Ok(false) => {
                                        // Stream is offline, don't update watchtime
                                    }
                                    Err(e) => {
                                        log::error!("Failed to check if stream is live: {}", e);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        });

        Ok(())
    }

    /// Stop the tracker
    pub async fn stop(&self) {
        let mut is_running = self.is_running.write().await;
        *is_running = false;
    }

    /// Check stream status and update database automatically
    async fn check_and_update_stream_status(
        api: &Arc<TwitchAPI>,
        db: &Database,
        channel: &str,
    ) -> Result<bool> {
        // Get channel's stream status from Twitch API
        match api.get_stream(channel).await {
            Ok(Some(stream)) => {
                // Stream is live
                let was_live = db.is_stream_live(channel).unwrap_or(false);

                if !was_live {
                    // Stream just went live
                    db.start_stream(channel)?;
                    log::info!("🔴 Stream for {} is now LIVE! Auto-tracking started.", channel);
                }

                Ok(true)
            }
            Ok(None) => {
                // Stream is offline
                let was_live = db.is_stream_live(channel).unwrap_or(false);

                if was_live {
                    // Stream just went offline
                    db.end_stream(channel)?;
                    log::info!("⚫ Stream for {} is now OFFLINE. Auto-tracking stopped.", channel);
                }

                Ok(false)
            }
            Err(e) => {
                log::error!("Failed to check stream status: {}", e);
                Err(e)
            }
        }
    }
}
