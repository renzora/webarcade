use crate::commands::database::Database;
use crate::modules::twitch::{CommandSystem, Command, PermissionLevel, TwitchIRCManager};
use crate::modules::twitch::twitch_irc_client::{TwitchEvent, WheelOption, WheelSpinEvent};
use log::error;
use std::sync::Arc;

/// Register wheel commands
pub async fn register(command_system: &CommandSystem, database: Arc<Database>) {
    let spin_command = Command {
        name: "spin".to_string(),
        aliases: vec!["wheel".to_string()],
        description: "Spin the wheel to pick a random option".to_string(),
        usage: "!spin".to_string(),
        permission: PermissionLevel::Everyone,
        cooldown_seconds: 10,
        enabled: true,
        handler: Arc::new(move |ctx, irc, _api| {
            let db = database.clone();
            let irc = irc.clone();
            let channel = ctx.channel.clone();
            let username = ctx.message.username.clone();

            Ok(Some(tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async move {
                    handle_spin_command(&channel, &username, db, irc).await
                })
            })))
        }),
    };

    command_system.register_command(spin_command).await;
}

/// Handle the !spin command
async fn handle_spin_command(
    channel: &str,
    username: &str,
    database: Arc<Database>,
    irc: Arc<TwitchIRCManager>,
) -> String {
    log::info!("🎡 Spin command triggered by {} in channel {}", username, channel);

    // Get wheel options from database
    let options_raw = match database.get_wheel_options(channel) {
        Ok(opts) => {
            log::info!("Found {} wheel options for channel {}", opts.len(), channel);
            opts
        },
        Err(e) => {
            error!("Failed to get wheel options: {}", e);
            return "❌ Failed to load wheel options".to_string();
        }
    };

    // Filter to only enabled options
    let enabled_options: Vec<_> = options_raw
        .into_iter()
        .filter(|opt| opt.enabled == 1)
        .collect();

    log::info!("Found {} enabled wheel options", enabled_options.len());

    if enabled_options.is_empty() {
        log::warn!("No enabled wheel options for channel {}", channel);
        return "❌ No wheel options available. Add some in the Wheel viewport first!".to_string();
    }

    // Build weighted list based on weight values
    let mut weighted_options = Vec::new();
    for option in &enabled_options {
        for _ in 0..option.weight {
            weighted_options.push(option);
        }
    }

    // Pick a random winner
    let winner_index = fastrand::usize(0..weighted_options.len());
    let winner = weighted_options[winner_index];

    log::info!("🎯 Wheel winner: {}", winner.option_text);

    // Convert to WheelOption format for event
    let wheel_options: Vec<WheelOption> = enabled_options
        .iter()
        .map(|opt| WheelOption {
            text: opt.option_text.clone(),
            color: opt.color.clone(),
        })
        .collect();

    // Record the spin in database
    if let Err(e) = database.record_wheel_spin(channel, &winner.option_text, Some(username)) {
        error!("Failed to record wheel spin: {}", e);
    } else {
        log::info!("✅ Recorded wheel spin in database");
    }

    // Broadcast the wheel spin event via WebSocket
    let wheel_event = TwitchEvent::WheelSpin(WheelSpinEvent {
        channel: channel.to_string(),
        winner: winner.option_text.clone(),
        options: wheel_options,
        triggered_by: Some(username.to_string()),
    });

    let event_sender = irc.get_event_sender();
    if let Err(e) = event_sender.send(wheel_event) {
        error!("❌ Failed to broadcast wheel spin event: {}", e);
    } else {
        log::info!("✅ Broadcasted wheel spin event via WebSocket");
    }

    // Return a chat message (the overlay will show the animation)
    format!("🎡 Spinning the wheel for @{}...", username)
}
