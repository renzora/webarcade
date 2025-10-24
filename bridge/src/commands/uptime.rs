use crate::modules::twitch::{CommandSystem, Command, CommandContext, PermissionLevel, TwitchIRCManager, TwitchAPI};
use std::sync::Arc;
use super::database::Database;

/// Helper function to format uptime duration with years, months, weeks, days, hours, minutes
fn format_uptime(seconds: i64) -> String {
    let years = seconds / (365 * 24 * 3600);
    let remaining = seconds % (365 * 24 * 3600);

    let months = remaining / (30 * 24 * 3600);
    let remaining = remaining % (30 * 24 * 3600);

    let weeks = remaining / (7 * 24 * 3600);
    let remaining = remaining % (7 * 24 * 3600);

    let days = remaining / (24 * 3600);
    let remaining = remaining % (24 * 3600);

    let hours = remaining / 3600;
    let remaining = remaining % 3600;

    let minutes = remaining / 60;
    let secs = remaining % 60;

    let mut parts = Vec::new();

    if years > 0 {
        parts.push(format!("{}y", years));
    }
    if months > 0 {
        parts.push(format!("{}mo", months));
    }
    if weeks > 0 {
        parts.push(format!("{}w", weeks));
    }
    if days > 0 {
        parts.push(format!("{}d", days));
    }
    if hours > 0 {
        parts.push(format!("{}h", hours));
    }
    if minutes > 0 {
        parts.push(format!("{}m", minutes));
    }
    if secs > 0 || parts.is_empty() {
        parts.push(format!("{}s", secs));
    }

    // Show the top 3 most significant units
    parts.into_iter().take(3).collect::<Vec<_>>().join(" ")
}

pub async fn register(command_system: &CommandSystem, db: Database) {
    // !uptime command - check stream uptime
    let db_clone = db.clone();
    let command = Command {
        name: "uptime".to_string(),
        aliases: vec![],
        description: "Check how long the stream has been live".to_string(),
        usage: "!uptime".to_string(),
        permission: PermissionLevel::Everyone,
        cooldown_seconds: 5,
        enabled: true,
        handler: Arc::new(move |ctx, irc, _api| {
            let irc = irc.clone();
            let channel = ctx.channel.clone();
            let db = db_clone.clone();

            tokio::spawn(async move {
                match db.get_stream_uptime(&channel) {
                    Ok(Some(seconds)) => {
                        let formatted = format_uptime(seconds);
                        let _ = irc.send_message(&channel, &format!("🔴 Stream has been live for: {}", formatted)).await;
                    }
                    Ok(None) => {
                        let _ = irc.send_message(&channel, "Stream is currently offline.").await;
                    }
                    Err(e) => {
                        log::error!("Database error: {}", e);
                        let _ = irc.send_message(&channel, "Database error!").await;
                    }
                }
            });

            Ok(None)
        }),
    };

    command_system.register_command(command).await;

    // !startstream command - broadcaster/mod only, marks stream as live
    let db_clone = db.clone();
    let command = Command {
        name: "startstream".to_string(),
        aliases: vec!["golive".to_string()],
        description: "Mark stream as live (Broadcaster/Mod only)".to_string(),
        usage: "!startstream".to_string(),
        permission: PermissionLevel::Moderator,
        cooldown_seconds: 3,
        enabled: true,
        handler: Arc::new(move |ctx, irc, _api| {
            let irc = irc.clone();
            let channel = ctx.channel.clone();
            let db = db_clone.clone();
            let username = ctx.message.username.clone();

            tokio::spawn(async move {
                match db.start_stream(&channel) {
                    Ok(_) => {
                        let _ = irc.send_message(&channel, &format!("Stream marked as LIVE by @{}! Uptime tracking started. 🔴", username)).await;
                    }
                    Err(e) => {
                        log::error!("Database error: {}", e);
                        let _ = irc.send_message(&channel, "Database error!").await;
                    }
                }
            });

            Ok(None)
        }),
    };

    command_system.register_command(command).await;

    // !endstream command - broadcaster/mod only, marks stream as offline
    let db_clone = db.clone();
    let command = Command {
        name: "endstream".to_string(),
        aliases: vec!["gooffline".to_string()],
        description: "Mark stream as offline (Broadcaster/Mod only)".to_string(),
        usage: "!endstream".to_string(),
        permission: PermissionLevel::Moderator,
        cooldown_seconds: 3,
        enabled: true,
        handler: Arc::new(move |ctx, irc, _api| {
            let irc = irc.clone();
            let channel = ctx.channel.clone();
            let db = db_clone.clone();
            let username = ctx.message.username.clone();

            tokio::spawn(async move {
                match db.end_stream(&channel) {
                    Ok(_) => {
                        let _ = irc.send_message(&channel, &format!("Stream marked as OFFLINE by @{}. Thanks for watching! 👋", username)).await;
                    }
                    Err(e) => {
                        log::error!("Database error: {}", e);
                        let _ = irc.send_message(&channel, "Database error!").await;
                    }
                }
            });

            Ok(None)
        }),
    };

    command_system.register_command(command).await;

    log::info!("✅ Registered uptime commands");
}
