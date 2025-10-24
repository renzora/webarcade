use crate::modules::twitch::{CommandSystem, Command, PermissionLevel};
use std::sync::Arc;
use super::database::Database;

/// Validate date format (YYYY-MM-DD)
fn validate_date(date: &str) -> bool {
    let parts: Vec<&str> = date.split('-').collect();
    if parts.len() != 3 {
        return false;
    }

    // Parse year, month, day
    let year = parts[0].parse::<u16>().ok();
    let month = parts[1].parse::<u8>().ok();
    let day = parts[2].parse::<u8>().ok();

    match (year, month, day) {
        (Some(y), Some(m), Some(d)) => {
            // Basic validation: year 1900-2100, month 1-12, day 1-31
            y >= 1900 && y <= 2100 && m >= 1 && m <= 12 && d >= 1 && d <= 31
        }
        _ => false,
    }
}

/// Format birthday for display
fn format_birthday(birthday: &str) -> String {
    let parts: Vec<&str> = birthday.split('-').collect();
    if parts.len() != 3 {
        return birthday.to_string();
    }

    let month_names = [
        "January", "February", "March", "April", "May", "June",
        "July", "August", "September", "October", "November", "December"
    ];

    if let (Ok(year), Ok(month), Ok(day)) = (
        parts[0].parse::<u16>(),
        parts[1].parse::<usize>(),
        parts[2].parse::<u8>()
    ) {
        if month >= 1 && month <= 12 {
            return format!("{} {}, {}", month_names[month - 1], day, year);
        }
    }

    birthday.to_string()
}

pub async fn register(command_system: &CommandSystem, db: Database) {
    // !setbirthday command - set your birthday
    let db_clone = db.clone();
    let command = Command {
        name: "setbirthday".to_string(),
        aliases: vec!["birthday".to_string(), "bday".to_string()],
        description: "Set your birthday (format: YYYY-MM-DD)".to_string(),
        usage: "!setbirthday YYYY-MM-DD".to_string(),
        permission: PermissionLevel::Everyone,
        cooldown_seconds: 5,
        enabled: true,
        handler: Arc::new(move |ctx, irc, _api| {
            let irc = irc.clone();
            let channel = ctx.channel.clone();
            let db = db_clone.clone();
            let username = ctx.message.username.clone();

            // Check if date was provided
            if ctx.args.is_empty() {
                // No date provided - show current birthday
                tokio::spawn(async move {
                    match db.get_user_birthday(&channel, &username) {
                        Ok(Some(birthday)) => {
                            let formatted = format_birthday(&birthday);
                            let _ = irc.send_message(&channel, &format!("@{} Your birthday is set to: {}", username, formatted)).await;
                        }
                        Ok(None) => {
                            let _ = irc.send_message(&channel, &format!("@{} You haven't set your birthday yet! Use !setbirthday YYYY-MM-DD", username)).await;
                        }
                        Err(e) => {
                            log::error!("Database error: {}", e);
                            let _ = irc.send_message(&channel, "Database error!").await;
                        }
                    }
                });
                return Ok(None);
            }

            let birthday = ctx.args[0].clone();

            // Validate date format
            if !validate_date(&birthday) {
                tokio::spawn(async move {
                    let _ = irc.send_message(&channel, &format!("@{} Invalid date format! Please use YYYY-MM-DD (e.g., 1990-05-15)", username)).await;
                });
                return Ok(None);
            }

            tokio::spawn(async move {
                match db.set_user_birthday(&channel, &username, &birthday) {
                    Ok(_) => {
                        let formatted = format_birthday(&birthday);
                        let _ = irc.send_message(&channel, &format!("@{} Birthday set to: {}! 🎂", username, formatted)).await;
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

    // !setlocation command - set your location
    let db_clone = db.clone();
    let command = Command {
        name: "setlocation".to_string(),
        aliases: vec!["location".to_string(), "loc".to_string()],
        description: "Set your location".to_string(),
        usage: "!setlocation <location>".to_string(),
        permission: PermissionLevel::Everyone,
        cooldown_seconds: 5,
        enabled: true,
        handler: Arc::new(move |ctx, irc, _api| {
            let irc = irc.clone();
            let channel = ctx.channel.clone();
            let db = db_clone.clone();
            let username = ctx.message.username.clone();

            // Check if location was provided
            if ctx.args.is_empty() {
                // No location provided - show current location
                tokio::spawn(async move {
                    match db.get_user_location(&channel, &username) {
                        Ok(Some(location)) => {
                            let _ = irc.send_message(&channel, &format!("@{} Your location is set to: {}", username, location)).await;
                        }
                        Ok(None) => {
                            let _ = irc.send_message(&channel, &format!("@{} You haven't set your location yet! Use !setlocation <location>", username)).await;
                        }
                        Err(e) => {
                            log::error!("Database error: {}", e);
                            let _ = irc.send_message(&channel, "Database error!").await;
                        }
                    }
                });
                return Ok(None);
            }

            let location = ctx.args.join(" ");

            // Limit location length
            if location.len() > 100 {
                tokio::spawn(async move {
                    let _ = irc.send_message(&channel, &format!("@{} Location is too long! Please keep it under 100 characters.", username)).await;
                });
                return Ok(None);
            }

            tokio::spawn(async move {
                match db.set_user_location(&channel, &username, &location) {
                    Ok(_) => {
                        let _ = irc.send_message(&channel, &format!("@{} Location set to: {}! 📍", username, location)).await;
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

    // !checkbirthday command - check another user's birthday
    let db_clone = db.clone();
    let command = Command {
        name: "checkbirthday".to_string(),
        aliases: vec![],
        description: "Check a user's birthday".to_string(),
        usage: "!checkbirthday <username>".to_string(),
        permission: PermissionLevel::Everyone,
        cooldown_seconds: 5,
        enabled: true,
        handler: Arc::new(move |ctx, irc, _api| {
            let irc = irc.clone();
            let channel = ctx.channel.clone();
            let db = db_clone.clone();
            let requester = ctx.message.username.clone();

            // Check if username was provided
            if ctx.args.is_empty() {
                tokio::spawn(async move {
                    let _ = irc.send_message(&channel, &format!("@{} Usage: !checkbirthday <username>", requester)).await;
                });
                return Ok(None);
            }

            let target_user = ctx.args[0].trim_start_matches('@').to_string();

            tokio::spawn(async move {
                match db.get_user_birthday(&channel, &target_user) {
                    Ok(Some(birthday)) => {
                        let formatted = format_birthday(&birthday);
                        let _ = irc.send_message(&channel, &format!("@{}'s birthday is: {} 🎂", target_user, formatted)).await;
                    }
                    Ok(None) => {
                        let _ = irc.send_message(&channel, &format!("@{} hasn't set their birthday yet.", target_user)).await;
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

    // !checklocation command - check another user's location
    let db_clone = db.clone();
    let command = Command {
        name: "checklocation".to_string(),
        aliases: vec![],
        description: "Check a user's location".to_string(),
        usage: "!checklocation <username>".to_string(),
        permission: PermissionLevel::Everyone,
        cooldown_seconds: 5,
        enabled: true,
        handler: Arc::new(move |ctx, irc, _api| {
            let irc = irc.clone();
            let channel = ctx.channel.clone();
            let db = db_clone.clone();
            let requester = ctx.message.username.clone();

            // Check if username was provided
            if ctx.args.is_empty() {
                tokio::spawn(async move {
                    let _ = irc.send_message(&channel, &format!("@{} Usage: !checklocation <username>", requester)).await;
                });
                return Ok(None);
            }

            let target_user = ctx.args[0].trim_start_matches('@').to_string();

            tokio::spawn(async move {
                match db.get_user_location(&channel, &target_user) {
                    Ok(Some(location)) => {
                        let _ = irc.send_message(&channel, &format!("@{} is from: {} 📍", target_user, location)).await;
                    }
                    Ok(None) => {
                        let _ = irc.send_message(&channel, &format!("@{} hasn't set their location yet.", target_user)).await;
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

    log::info!("✅ Registered user profile commands");
}
