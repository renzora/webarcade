use crate::modules::twitch::{CommandSystem, Command, CommandContext, PermissionLevel, TwitchIRCManager, TwitchAPI};
use std::sync::Arc;
use super::database::Database;

/// Helper function to format watchtime duration
fn format_watchtime(minutes: i64) -> String {
    let hours = minutes / 60;
    let mins = minutes % 60;

    if hours > 0 {
        format!("{}h {}m", hours, mins)
    } else {
        format!("{}m", mins)
    }
}

pub async fn register(command_system: &CommandSystem, db: Database) {
    // !watchtime command - check your own or another user's watchtime
    let db_clone = db.clone();
    let command = Command {
        name: "watchtime".to_string(),
        aliases: vec!["wt".to_string(), "watch".to_string()],
        description: "Check watchtime for yourself or another user".to_string(),
        usage: "!watchtime [username]".to_string(),
        permission: PermissionLevel::Everyone,
        cooldown_seconds: 5,
        enabled: true,
        handler: Arc::new(move |ctx, irc, _api| {
            let irc = irc.clone();
            let channel = ctx.channel.clone();
            let db = db_clone.clone();
            let requester = ctx.message.username.clone();

            // Check if a username was provided
            let target_user = if let Some(user) = ctx.args.first() {
                user.trim_start_matches('@').to_string()
            } else {
                requester.clone()
            };

            tokio::spawn(async move {
                match db.get_watchtime(&channel, &target_user) {
                    Ok(minutes) => {
                        if minutes == 0 {
                            let msg = if target_user.to_lowercase() == requester.to_lowercase() {
                                format!("@{} You haven't been tracked watching yet! Keep chatting to build watchtime.", requester)
                            } else {
                                format!("@{} has no recorded watchtime yet.", target_user)
                            };
                            let _ = irc.send_message(&channel, &msg).await;
                        } else {
                            let formatted = format_watchtime(minutes);
                            let msg = if target_user.to_lowercase() == requester.to_lowercase() {
                                format!("@{} You've watched for {}! ⏱️", requester, formatted)
                            } else {
                                format!("@{} has watched for {}! ⏱️", target_user, formatted)
                            };
                            let _ = irc.send_message(&channel, &msg).await;
                        }
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

    // !topwatchers command - show leaderboard
    let db_clone = db.clone();
    let command = Command {
        name: "topwatchers".to_string(),
        aliases: vec!["leaderboard".to_string(), "watchers".to_string()],
        description: "Show top watchers leaderboard".to_string(),
        usage: "!topwatchers".to_string(),
        permission: PermissionLevel::Everyone,
        cooldown_seconds: 10,
        enabled: true,
        handler: Arc::new(move |ctx, irc, _api| {
            let irc = irc.clone();
            let channel = ctx.channel.clone();
            let db = db_clone.clone();

            tokio::spawn(async move {
                match db.get_top_watchers(&channel, 5) {
                    Ok(watchers) => {
                        if watchers.is_empty() {
                            let _ = irc.send_message(&channel, "No watchtime data yet! Keep chatting to track your time.").await;
                        } else {
                            let leaderboard: Vec<String> = watchers.iter()
                                .enumerate()
                                .map(|(i, (user, mins))| {
                                    let medal = match i {
                                        0 => "🥇",
                                        1 => "🥈",
                                        2 => "🥉",
                                        _ => "📊",
                                    };
                                    format!("{} {}: {}", medal, user, format_watchtime(*mins))
                                })
                                .collect();

                            let _ = irc.send_message(&channel, &format!("Top Watchers: {}", leaderboard.join(" | "))).await;
                        }
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

    log::info!("✅ Registered watchtime commands");
}
