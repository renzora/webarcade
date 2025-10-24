use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

use super::twitch_api::TwitchAPI;
use super::twitch_auth::TwitchAuth;
use super::twitch_commands::{Command, CommandSystem, SimpleCommand};
use super::twitch_config::TwitchConfigManager;
use super::twitch_irc_client::{TwitchEvent, TwitchIRCManager};
use super::stream_tracker::StreamTracker;
use super::text_command_timer::TextCommandTimer;
use crate::commands::database::Database;
use crate::commands::levels;

/// Status of the Twitch bot
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BotStatus {
    Disconnected,
    Connecting,
    Connected,
    Error,
}

/// Bot statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotStats {
    pub status: BotStatus,
    pub connected_channels: Vec<String>,
    pub total_messages: u64,
    pub total_commands: u64,
    pub uptime_seconds: u64,
    pub authenticated_user: Option<String>,
}

/// Twitch manager that orchestrates all components
pub struct TwitchManager {
    config_manager: Arc<TwitchConfigManager>,
    auth: Arc<TwitchAuth>,
    api: Arc<TwitchAPI>,
    irc_manager: Arc<TwitchIRCManager>,
    command_system: Arc<CommandSystem>,
    event_sender: broadcast::Sender<TwitchEvent>,
    status: Arc<RwLock<BotStatus>>,
    stats: Arc<RwLock<BotStats>>,
    start_time: Arc<RwLock<Option<i64>>>,
    profile_cache: Arc<RwLock<HashMap<String, String>>>, // user_id -> profile_image_url
    stream_tracker: Option<Arc<StreamTracker>>, // Optional: only if database is available
    text_command_timer: Option<Arc<TextCommandTimer>>, // Optional: only if database is available
}

impl TwitchManager {
    /// Create a new Twitch manager
    pub fn new() -> (Self, broadcast::Receiver<TwitchEvent>) {
        let config_manager = Arc::new(TwitchConfigManager::new());
        let auth = Arc::new(TwitchAuth::new(config_manager.clone()));
        let api = Arc::new(TwitchAPI::new(config_manager.clone(), auth.clone()));

        let (irc_manager, mut irc_event_receiver) =
            TwitchIRCManager::new(config_manager.clone(), auth.clone());
        let irc_manager = Arc::new(irc_manager);

        let command_system = Arc::new(CommandSystem::new(
            "!".to_string(),
            irc_manager.clone(),
            api.clone(),
        ));

        let (event_sender, event_receiver) = broadcast::channel(1000);

        let status = Arc::new(RwLock::new(BotStatus::Disconnected));
        let stats = Arc::new(RwLock::new(BotStats {
            status: BotStatus::Disconnected,
            connected_channels: vec![],
            total_messages: 0,
            total_commands: 0,
            uptime_seconds: 0,
            authenticated_user: None,
        }));

        let profile_cache = Arc::new(RwLock::new(HashMap::new()));

        // Initialize stream tracker and text command timer if database is available
        let (stream_tracker, text_command_timer) = match Database::new() {
            Ok(db) => {
                let tracker = Some(Arc::new(StreamTracker::new(
                    api.clone(),
                    db.clone(),
                    config_manager.clone(),
                )));
                let timer = Some(Arc::new(TextCommandTimer::new(
                    db,
                    irc_manager.clone(),
                    config_manager.clone(),
                )));
                (tracker, timer)
            }
            Err(_) => (None, None),
        };

        let manager = Self {
            config_manager,
            auth,
            api: api.clone(),
            irc_manager,
            command_system,
            event_sender: event_sender.clone(),
            status: status.clone(),
            stats: stats.clone(),
            start_time: Arc::new(RwLock::new(None)),
            profile_cache: profile_cache.clone(),
            stream_tracker: stream_tracker.clone(),
            text_command_timer: text_command_timer.clone(),
        };

        // Spawn event forwarder
        let event_sender_clone = event_sender.clone();
        let command_system_clone = manager.command_system.clone();
        let irc_manager_clone = manager.irc_manager.clone();
        let stats_clone = stats.clone();
        let profile_cache_clone = profile_cache.clone();
        let api_clone = api.clone();
        let stream_tracker_clone = stream_tracker.clone();

        // Initialize database for XP tracking
        let db = Database::new().ok();

        tokio::spawn(async move {
            while let Ok(mut event) = irc_event_receiver.recv().await {
                // Enrich chat messages with profile images and user data
                if let TwitchEvent::ChatMessage(ref mut msg) = event {
                    // Check cache first for profile image
                    let mut cache = profile_cache_clone.write().await;
                    if !cache.contains_key(&msg.user_id) {
                        // Fetch from API
                        if let Ok(Some(user)) = api_clone.get_user_by_id(&msg.user_id).await {
                            cache.insert(msg.user_id.clone(), user.profile_image_url.clone());
                            msg.profile_image_url = Some(user.profile_image_url);
                        }
                    } else {
                        msg.profile_image_url = cache.get(&msg.user_id).cloned();
                    }
                    drop(cache);

                    // Enrich with database user data (location, birthday, level)
                    if let Some(ref database) = db {
                        // Get location
                        if let Ok(Some(location)) = database.get_user_location(&msg.channel, &msg.username) {
                            msg.location_flag = TwitchIRCManager::get_location_flag(&location);
                        }

                        // Check birthday
                        if let Ok(Some(birthday)) = database.get_user_birthday(&msg.channel, &msg.username) {
                            msg.is_birthday = TwitchIRCManager::is_birthday_today(&birthday);
                        }

                        // Get level
                        if let Ok(Some((level, _, _, _))) = database.get_user_level(&msg.channel, &msg.username) {
                            msg.level = Some(level);
                        }
                    }
                }

                // Update stats based on event
                match &event {
                    TwitchEvent::ChatMessage(msg) => {
                        let mut stats = stats_clone.write().await;
                        stats.total_messages += 1;
                        drop(stats);

                        // Mark user as active for watchtime tracking
                        if let Some(ref tracker) = stream_tracker_clone {
                            tracker.mark_user_active(&msg.username).await;
                        }

                        // Award XP for non-command messages
                        if !msg.message.starts_with("!") {
                            if let Some(ref database) = db {
                                if let Some((old_level, new_level, total_xp, xp_needed)) = levels::award_message_xp(database, &msg.channel, &msg.username, &irc_manager_clone).await {
                                    // Broadcast level up event
                                    let level_up_event = TwitchEvent::LevelUp(super::twitch_irc_client::LevelUpEvent {
                                        username: msg.username.clone(),
                                        channel: msg.channel.clone(),
                                        old_level,
                                        new_level,
                                        total_xp,
                                        xp_needed,
                                    });
                                    let _ = event_sender_clone.send(level_up_event);
                                }
                            }
                        }

                        // Handle TTS for eligible messages
                        if !msg.message.starts_with("!") {
                            if let Some(ref database) = db {
                                // Check if TTS is enabled
                                if let Ok(true) = database.is_tts_enabled(&msg.channel) {
                                    // Check if user has TTS privileges
                                    let is_broadcaster = msg.badges.iter().any(|b| b.starts_with("broadcaster"));
                                    if let Ok(true) = database.has_tts_privilege(&msg.channel, &msg.username, is_broadcaster) {
                                        // Get user's voice preference
                                        let voice = database.get_tts_voice(&msg.channel, &msg.username)
                                            .unwrap_or_else(|_| "Brian".to_string());

                                        // Speak the message
                                        use crate::commands::tts_command;
                                        tts_command::speak_text(&msg.message, &voice);
                                    }
                                }
                            }
                        }

                        // Check if it's a command
                        if msg.message.starts_with("!") {
                            let mut stats = stats_clone.write().await;
                            stats.total_commands += 1;
                            drop(stats);

                            // Handle command
                            if let Err(e) = command_system_clone.handle_message(msg.clone()).await {
                                log::error!("Error handling command: {}", e);
                            }
                        }
                    }
                    TwitchEvent::Connected { channels } => {
                        let mut stats = stats_clone.write().await;
                        stats.status = BotStatus::Connected;
                        stats.connected_channels = channels.clone();
                    }
                    TwitchEvent::Disconnected { .. } => {
                        let mut stats = stats_clone.write().await;
                        stats.status = BotStatus::Disconnected;
                        stats.connected_channels.clear();
                    }
                    _ => {}
                }

                // Forward event to subscribers
                let _ = event_sender_clone.send(event);
            }
        });

        (manager, event_receiver)
    }

    /// Start the Twitch bot
    pub async fn start(&self) -> Result<()> {
        log::info!("Starting Twitch bot...");

        {
            let mut status = self.status.write().await;
            *status = BotStatus::Connecting;
        }

        // Check if we have tokens at all
        let config = self.config_manager.load()?;
        if config.access_token.is_none() || config.refresh_token.is_none() {
            anyhow::bail!("Not authenticated. Please complete OAuth flow first.");
        }

        // Get valid token (this will auto-refresh if expired)
        let _token = self.auth.get_valid_token().await
            .context("Failed to get valid access token")?;

        // Get authenticated user info
        let user = self
            .api
            .get_authenticated_user()
            .await
            .context("Failed to get authenticated user")?;

        log::info!("Authenticated as: {} ({})", user.display_name, user.login);

        {
            let mut stats = self.stats.write().await;
            stats.authenticated_user = Some(user.display_name);
        }

        // Start IRC client
        self.irc_manager
            .start()
            .await
            .context("Failed to start IRC client")?;

        // Update status
        {
            let mut status = self.status.write().await;
            *status = BotStatus::Connected;
        }

        // Set start time
        {
            let mut start_time = self.start_time.write().await;
            *start_time = Some(chrono::Utc::now().timestamp());
        }

        // Start automatic stream tracker
        if let Some(ref tracker) = self.stream_tracker {
            tracker.start().await?;
            log::info!("🎬 Automatic stream & watchtime tracking started");
        }

        // Start text command timer
        if let Some(ref timer) = self.text_command_timer {
            timer.start().await?;
            log::info!("⏰ Text command auto-post timer started");
        }

        log::info!("Twitch bot started successfully");

        Ok(())
    }

    /// Stop the Twitch bot
    pub async fn stop(&self) -> Result<()> {
        log::info!("Stopping Twitch bot...");

        // Stop stream tracker
        if let Some(ref tracker) = self.stream_tracker {
            tracker.stop().await;
            log::info!("Stream tracker stopped");
        }

        // Stop text command timer
        if let Some(ref timer) = self.text_command_timer {
            timer.stop().await;
            log::info!("Text command timer stopped");
        }

        self.irc_manager.stop().await?;

        {
            let mut status = self.status.write().await;
            *status = BotStatus::Disconnected;
        }

        {
            let mut start_time = self.start_time.write().await;
            *start_time = None;
        }

        log::info!("Twitch bot stopped");

        Ok(())
    }

    /// Check if bot is running
    pub async fn is_running(&self) -> bool {
        self.irc_manager.is_running().await
    }

    /// Get bot statistics
    pub async fn get_stats(&self) -> BotStats {
        let mut stats = self.stats.read().await.clone();

        // Update uptime
        if let Some(start_time) = *self.start_time.read().await {
            let now = chrono::Utc::now().timestamp();
            stats.uptime_seconds = (now - start_time) as u64;
        }

        stats
    }

    /// Get config manager
    pub fn get_config_manager(&self) -> Arc<TwitchConfigManager> {
        self.config_manager.clone()
    }

    /// Get auth manager
    pub fn get_auth(&self) -> Arc<TwitchAuth> {
        self.auth.clone()
    }

    /// Get API client
    pub fn get_api(&self) -> Arc<TwitchAPI> {
        self.api.clone()
    }

    /// Get IRC manager
    pub fn get_irc_manager(&self) -> Arc<TwitchIRCManager> {
        self.irc_manager.clone()
    }

    /// Get command system
    pub fn get_command_system(&self) -> Arc<CommandSystem> {
        self.command_system.clone()
    }

    /// Get event sender for subscribing
    pub fn get_event_sender(&self) -> broadcast::Sender<TwitchEvent> {
        self.event_sender.clone()
    }

    /// Send a message to a channel
    pub async fn send_message(&self, channel: &str, message: &str) -> Result<()> {
        self.irc_manager.send_message(channel, message).await
    }

    /// Register a command
    pub async fn register_command(&self, command: Command) {
        self.command_system.register_command(command).await;
    }

    /// Register a simple text command
    pub async fn register_simple_command(&self, simple: SimpleCommand) {
        self.command_system.register_simple_command(simple).await;
    }

    /// Unregister a command
    pub async fn unregister_command(&self, name: &str) {
        self.command_system.unregister_command(name).await;
    }

    /// Get all registered commands
    pub async fn get_commands(&self) -> Vec<Command> {
        self.command_system.get_commands().await
    }

    /// Join a new channel
    pub async fn join_channel(&self, channel: &str) -> Result<()> {
        self.irc_manager.join_channel(channel).await?;

        // Update config
        let mut config = self.config_manager.load()?;
        if !config.channels.contains(&channel.to_string()) {
            config.channels.push(channel.to_string());
            self.config_manager.save(&config)?;
        }

        Ok(())
    }

    /// Leave a channel
    pub async fn part_channel(&self, channel: &str) -> Result<()> {
        self.irc_manager.part_channel(channel).await?;

        // Update config
        let mut config = self.config_manager.load()?;
        config.channels.retain(|c| c != channel);
        self.config_manager.save(&config)?;

        Ok(())
    }
}

impl Clone for TwitchManager {
    fn clone(&self) -> Self {
        Self {
            config_manager: self.config_manager.clone(),
            auth: self.auth.clone(),
            api: self.api.clone(),
            irc_manager: self.irc_manager.clone(),
            command_system: self.command_system.clone(),
            event_sender: self.event_sender.clone(),
            status: self.status.clone(),
            stats: self.stats.clone(),
            start_time: self.start_time.clone(),
            profile_cache: self.profile_cache.clone(),
            stream_tracker: self.stream_tracker.clone(),
            text_command_timer: self.text_command_timer.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_manager_creation() {
        let (manager, _receiver) = TwitchManager::new();
        assert!(!manager.is_running().await);
    }

    #[tokio::test]
    async fn test_stats() {
        let (manager, _receiver) = TwitchManager::new();
        let stats = manager.get_stats().await;

        assert_eq!(stats.total_messages, 0);
        assert_eq!(stats.total_commands, 0);
    }
}
