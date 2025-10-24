use crate::modules::twitch::{CommandSystem, Command, PermissionLevel, TwitchIRCManager};
use std::sync::Arc;
use super::database::Database;

const XP_COOLDOWN_SECONDS: i64 = 60; // 1 minute cooldown between XP gains
const MIN_XP_PER_MESSAGE: i64 = 10;
const MAX_XP_PER_MESSAGE: i64 = 15;

/// Calculate XP needed to reach next level from current XP
fn xp_to_next_level(current_xp: i64) -> (i64, i64) {
    let current_level = Database::calculate_level_from_xp(current_xp);
    let next_level = current_level + 1;

    // Calculate total XP needed for next level
    let mut total_needed = 0;
    for lvl in 1..next_level {
        total_needed += Database::xp_for_level(lvl);
    }

    let xp_remaining = total_needed - current_xp;
    (next_level, xp_remaining)
}

/// Award XP to a user for sending a message
/// Returns (old_level, new_level, total_xp, xp_needed) if user leveled up
pub async fn award_message_xp(db: &Database, channel: &str, username: &str, irc: &Arc<TwitchIRCManager>) -> Option<(i64, i64, i64, i64)> {
    // Use fastrand which is Send-safe for async contexts
    let xp_gain = fastrand::i64(MIN_XP_PER_MESSAGE..=MAX_XP_PER_MESSAGE);

    match db.add_user_xp(channel, username, xp_gain, XP_COOLDOWN_SECONDS) {
        Ok(Some((old_level, new_level, _, total_xp))) => {
            // Check if user leveled up
            if new_level > old_level {
                // Calculate XP needed for next level
                let (next_level, xp_needed) = xp_to_next_level(total_xp);

                // Send exciting level up message
                let level_emoji = match new_level {
                    1..=5 => "⭐",
                    6..=10 => "🌟",
                    11..=20 => "💫",
                    21..=50 => "✨",
                    _ => "🏆",
                };

                let msg = format!(
                    "🎊 LEVEL UP! 🎊 @{} reached Level {} {}! [XP: {} | Next Level: {} XP needed]",
                    username, new_level, level_emoji, total_xp, xp_needed
                );
                let _ = irc.send_message(channel, &msg).await;
                log::info!("User {} leveled up: {} -> {} (XP: {})", username, old_level, new_level, total_xp);

                // Return level up data
                return Some((old_level, new_level, total_xp, xp_needed));
            }
        }
        Ok(None) => {
            // User is on cooldown, no XP awarded
        }
        Err(e) => {
            log::error!("Failed to award XP to {}: {}", username, e);
        }
    }

    None
}

pub async fn register(command_system: &CommandSystem, db: Database) {
    // !level command - check your own or another user's level
    let db_clone = db.clone();
    let command = Command {
        name: "level".to_string(),
        aliases: vec!["lvl".to_string(), "rank".to_string()],
        description: "Check your level and XP".to_string(),
        usage: "!level [username]".to_string(),
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
                match db.get_user_level(&channel, &target_user) {
                    Ok(Some((level, xp, total_messages, _))) => {
                        // Calculate XP to next level
                        let (next_level, xp_needed) = xp_to_next_level(xp);

                        // Get rank
                        let rank = db.get_user_rank(&channel, &target_user)
                            .unwrap_or(None);

                        let rank_str = if let Some(r) = rank {
                            format!(" | Rank #{}", r)
                        } else {
                            String::new()
                        };

                        let msg = if target_user.to_lowercase() == requester.to_lowercase() {
                            format!(
                                "@{} Level {} | XP: {} ({} to Level {}) | Messages: {}{}",
                                requester, level, xp, xp_needed, next_level, total_messages, rank_str
                            )
                        } else {
                            format!(
                                "@{} is Level {} | XP: {} ({} to Level {}) | Messages: {}{}",
                                target_user, level, xp, xp_needed, next_level, total_messages, rank_str
                            )
                        };
                        let _ = irc.send_message(&channel, &msg).await;
                    }
                    Ok(None) => {
                        let msg = if target_user.to_lowercase() == requester.to_lowercase() {
                            format!("@{} You haven't earned any XP yet! Send some messages to start leveling up.", requester)
                        } else {
                            format!("@{} hasn't earned any XP yet.", target_user)
                        };
                        let _ = irc.send_message(&channel, &msg).await;
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

    // !leaderboard command - show top users by level
    let db_clone = db.clone();
    let command = Command {
        name: "leaderboard".to_string(),
        aliases: vec!["levels".to_string(), "top".to_string()],
        description: "Show the level leaderboard".to_string(),
        usage: "!leaderboard".to_string(),
        permission: PermissionLevel::Everyone,
        cooldown_seconds: 10,
        enabled: true,
        handler: Arc::new(move |ctx, irc, _api| {
            let irc = irc.clone();
            let channel = ctx.channel.clone();
            let db = db_clone.clone();

            tokio::spawn(async move {
                match db.get_level_leaderboard(&channel, 5) {
                    Ok(users) => {
                        if users.is_empty() {
                            let _ = irc.send_message(&channel, "No level data yet! Start chatting to earn XP and level up!").await;
                        } else {
                            let leaderboard: Vec<String> = users.iter()
                                .enumerate()
                                .map(|(i, (user, level, xp))| {
                                    let medal = match i {
                                        0 => "🥇",
                                        1 => "🥈",
                                        2 => "🥉",
                                        _ => "📊",
                                    };
                                    format!("{} {} (Lvl {} | {} XP)", medal, user, level, xp)
                                })
                                .collect();

                            let _ = irc.send_message(&channel, &format!("Top Levels: {}", leaderboard.join(" | "))).await;
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

    log::info!("✅ Registered level commands");
}
