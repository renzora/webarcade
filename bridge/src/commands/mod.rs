// Command modules
pub mod database;
pub mod household;
pub mod todos;
pub mod tts_command;
pub mod dice;
pub mod hue;
pub mod hue_command;
pub mod watchtime;
pub mod followage;
pub mod uptime;
pub mod text_commands;
pub mod fun_commands;
pub mod levels;
pub mod user_profile;
pub mod debug_user;
pub mod wheel_command;
pub mod voice_commands;
pub mod effect_command;

use crate::modules::twitch::CommandSystem;
use database::Database;
use std::sync::Arc;

/// Register all commands from this module
pub async fn register_all_commands(command_system: &CommandSystem) {
    log::info!("📦 Registering custom commands...");

    // Initialize database
    let db = match Database::new() {
        Ok(db) => db,
        Err(e) => {
            log::error!("Failed to initialize database: {}", e);
            return;
        }
    };

    // Register household task commands
    household::register(command_system, db.clone()).await;

    // Register todo commands
    todos::register(command_system, db.clone()).await;

    // Register TTS command
    tts_command::register(command_system, db.clone()).await;

    // Register dice commands
    dice::register(command_system).await;

    // Register Hue commands
    hue_command::register(command_system, db.clone()).await;

    // Register watchtime commands
    watchtime::register(command_system, db.clone()).await;

    // Register followage command
    followage::register(command_system, db.clone()).await;

    // Register uptime commands
    uptime::register(command_system, db.clone()).await;

    // Register custom text commands handler
    text_commands::register(command_system, db.clone()).await;

    // Register fun commands (joke, dadjoke, 8ball, quote, roast, yomomma)
    fun_commands::register(command_system).await;

    // Register level commands
    levels::register(command_system, db.clone()).await;

    // Register user profile commands (birthday, location)
    user_profile::register(command_system, db.clone()).await;

    // Register debug commands
    debug_user::register(command_system, db.clone()).await;

    // Register wheel commands
    wheel_command::register(command_system, Arc::new(db.clone())).await;

    // Register voice commands
    voice_commands::register(command_system, db.clone()).await;

    // Register effect command
    effect_command::register(command_system).await;

    log::info!("✅ All custom commands registered");
}

/// Load text commands for a specific channel
pub async fn load_channel_text_commands(command_system: &CommandSystem, channel: &str) {
    // Initialize database
    let db = match Database::new() {
        Ok(db) => db,
        Err(e) => {
            log::error!("Failed to initialize database for text commands: {}", e);
            return;
        }
    };

    // Load text commands for this channel
    text_commands::load_text_commands(command_system, db, channel).await;
}
