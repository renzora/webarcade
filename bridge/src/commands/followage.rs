use crate::modules::twitch::{CommandSystem, Command, CommandContext, PermissionLevel, TwitchIRCManager, TwitchAPI};
use std::sync::Arc;
use super::database::Database;

/// Helper function to format followage duration
fn format_followage(followed_at: &str) -> Result<String, chrono::ParseError> {
    let followed_date = chrono::DateTime::parse_from_rfc3339(followed_at)?;
    let now = chrono::Utc::now();
    let duration = now.signed_duration_since(followed_date);

    let days = duration.num_days();
    let years = days / 365;
    let remaining_days = days % 365;
    let months = remaining_days / 30;
    let remaining_days = remaining_days % 30;

    let mut parts = Vec::new();

    if years > 0 {
        parts.push(format!("{}y", years));
    }
    if months > 0 {
        parts.push(format!("{}mo", months));
    }
    if remaining_days > 0 || parts.is_empty() {
        parts.push(format!("{}d", remaining_days));
    }

    Ok(parts.join(" "))
}

pub async fn register(command_system: &CommandSystem, db: Database) {
    // !followage command - check your own or another user's followage
    let db_clone = db.clone();
    let command = Command {
        name: "followage".to_string(),
        aliases: vec!["followtime".to_string(), "fa".to_string()],
        description: "Check how long you or another user has been following".to_string(),
        usage: "!followage [username]".to_string(),
        permission: PermissionLevel::Everyone,
        cooldown_seconds: 5,
        enabled: true,
        handler: Arc::new(move |ctx, irc, api| {
            let irc = irc.clone();
            let channel = ctx.channel.clone();
            let db = db_clone.clone();
            let api = api.clone();
            let requester = ctx.message.username.clone();

            // Check if a username was provided
            let target_user = if let Some(user) = ctx.args.first() {
                user.trim_start_matches('@').to_string()
            } else {
                requester.clone()
            };

            tokio::spawn(async move {
                // First, check database for cached followage
                match db.get_user_followed_at(&channel, &target_user) {
                    Ok(Some(followed_at)) => {
                        // We have cached data, use it
                        match format_followage(&followed_at) {
                            Ok(formatted) => {
                                let msg = if target_user.to_lowercase() == requester.to_lowercase() {
                                    format!("@{} You've been following for {}! 💜", requester, formatted)
                                } else {
                                    format!("@{} has been following for {}! 💜", target_user, formatted)
                                };
                                let _ = irc.send_message(&channel, &msg).await;
                            }
                            Err(e) => {
                                log::error!("Failed to parse followed_at date: {}", e);
                                let _ = irc.send_message(&channel, "Error parsing followage data!").await;
                            }
                        }
                    }
                    Ok(None) | Err(_) => {
                        // No cached data, fetch from Twitch API
                        // First get the broadcaster and user IDs
                        let broadcaster_result = api.get_user_by_login(&channel).await;
                        let user_result = api.get_user_by_login(&target_user).await;

                        match (broadcaster_result, user_result) {
                            (Ok(Some(broadcaster)), Ok(Some(user))) => {
                                // Check follow status
                                match api.get_user_follow_status(&broadcaster.id, &user.id).await {
                                    Ok(Some(followed_at)) => {
                                        // Cache the result
                                        let _ = db.set_user_followed_at(&channel, &target_user, &followed_at);

                                        match format_followage(&followed_at) {
                                            Ok(formatted) => {
                                                let msg = if target_user.to_lowercase() == requester.to_lowercase() {
                                                    format!("@{} You've been following for {}! 💜", requester, formatted)
                                                } else {
                                                    format!("@{} has been following for {}! 💜", target_user, formatted)
                                                };
                                                let _ = irc.send_message(&channel, &msg).await;
                                            }
                                            Err(e) => {
                                                log::error!("Failed to parse followed_at date: {}", e);
                                                let _ = irc.send_message(&channel, "Error parsing followage data!").await;
                                            }
                                        }
                                    }
                                    Ok(None) => {
                                        let msg = if target_user.to_lowercase() == requester.to_lowercase() {
                                            format!("@{} You're not following this channel yet!", requester)
                                        } else {
                                            format!("@{} is not following this channel.", target_user)
                                        };
                                        let _ = irc.send_message(&channel, &msg).await;
                                    }
                                    Err(e) => {
                                        log::error!("Failed to check follow status: {}", e);
                                        let _ = irc.send_message(&channel, "Failed to check follow status!").await;
                                    }
                                }
                            }
                            _ => {
                                let _ = irc.send_message(&channel, "Failed to fetch user information!").await;
                            }
                        }
                    }
                }
            });

            Ok(None)
        }),
    };

    command_system.register_command(command).await;

    log::info!("✅ Registered followage command");
}
