use crate::modules::twitch::{CommandSystem, Command, CommandContext, PermissionLevel, TwitchIRCManager, TwitchAPI};
use std::sync::Arc;
use super::database::Database;

pub async fn register(command_system: &CommandSystem, db: Database) {
    let command = Command {
        name: "task".to_string(),
        aliases: vec!["tasks".to_string(), "todo".to_string(), "todos".to_string()],
        description: "Manage your personal task list".to_string(),
        usage: "!task [<task> | list | done [#id] | remove <#id> | remove all | clear (mod) | @user]".to_string(),
        permission: PermissionLevel::Everyone,
        cooldown_seconds: 2,
        enabled: true,
        handler: Arc::new(move |ctx, irc, _api| {
            let irc = irc.clone();
            let channel = ctx.channel.clone();
            let db = db.clone();
            let username = ctx.message.username.clone();
            let args = ctx.args.clone();

            tokio::spawn(async move {
                // Helper function to parse ID (handles #9 or 9 format)
                let parse_id = |id_str: &str| -> Option<i64> {
                    let cleaned = id_str.trim_start_matches('#');
                    cleaned.parse::<i64>().ok()
                };

                // Parse subcommand
                let subcommand = args.first().map(|s| s.as_str());

                match subcommand {
                    // !task done [#id] or !task complete [#id]
                    Some("done") | Some("complete") => {
                        if args.len() > 1 {
                            // !task done #9 or !task done 9
                            let id_str = &args[1];
                            match parse_id(id_str) {
                                Some(id) => {
                                    match db.complete_todo(&channel, &username, id) {
                                        Ok((true, should_award_xp)) => {
                                            // Award XP if applicable
                                            if should_award_xp {
                                                let _ = db.add_user_xp(&channel, &username, 50, 0);
                                            }
                                            let _ = irc.send_message(&channel,
                                                &format!("🎉 @{} Completed task #{}!", username, id)
                                            ).await;
                                        }
                                        Ok((false, _)) => {
                                            let _ = irc.send_message(&channel,
                                                &format!("@{} Task #{} not found or already completed.", username, id)
                                            ).await;
                                        }
                                        Err(e) => {
                                            log::error!("Database error: {}", e);
                                            let _ = irc.send_message(&channel, "Database error!").await;
                                        }
                                    }
                                }
                                None => {
                                    let _ = irc.send_message(&channel,
                                        &format!("@{} Invalid ID. Usage: !task done [#id]", username)
                                    ).await;
                                }
                            }
                        } else {
                            // !task done (no ID - complete latest task)
                            match db.get_latest_todo(&channel, &username) {
                                Ok(Some((id, task_text))) => {
                                    match db.complete_todo(&channel, &username, id) {
                                        Ok((true, should_award_xp)) => {
                                            // Award XP if applicable
                                            if should_award_xp {
                                                let _ = db.add_user_xp(&channel, &username, 50, 0);
                                            }
                                            let _ = irc.send_message(&channel,
                                                &format!("🎉 @{} Completed task #{}: {}", username, id, task_text)
                                            ).await;
                                        }
                                        Ok((false, _)) => {
                                            let _ = irc.send_message(&channel,
                                                &format!("@{} Failed to complete task #{}.", username, id)
                                            ).await;
                                        }
                                        Err(e) => {
                                            log::error!("Database error: {}", e);
                                            let _ = irc.send_message(&channel, "Database error!").await;
                                        }
                                    }
                                }
                                Ok(None) => {
                                    let _ = irc.send_message(&channel,
                                        &format!("@{} You have no active tasks to complete!", username)
                                    ).await;
                                }
                                Err(e) => {
                                    log::error!("Database error: {}", e);
                                    let _ = irc.send_message(&channel, "Database error!").await;
                                }
                            }
                        }
                    }

                    // !task remove all - remove all tasks for the user
                    Some("remove") | Some("delete") | Some("rm") if args.get(1).map(|s| s.as_str()) == Some("all") => {
                        match db.remove_all_user_todos(&channel, &username) {
                            Ok(count) => {
                                if count > 0 {
                                    let _ = irc.send_message(&channel,
                                        &format!("🗑️ @{} Removed all {} task(s).", username, count)
                                    ).await;
                                } else {
                                    let _ = irc.send_message(&channel,
                                        &format!("@{} You have no tasks to remove.", username)
                                    ).await;
                                }
                            }
                            Err(e) => {
                                log::error!("Database error: {}", e);
                                let _ = irc.send_message(&channel, "Database error!").await;
                            }
                        }
                    }

                    // !task remove #id or !task delete #id
                    Some("remove") | Some("delete") | Some("rm") if args.len() > 1 => {
                        let id_str = &args[1];
                        match parse_id(id_str) {
                            Some(id) => {
                                match db.remove_todo(&channel, &username, id) {
                                    Ok(true) => {
                                        let _ = irc.send_message(&channel,
                                            &format!("🗑️ @{} Removed task #{}.", username, id)
                                        ).await;
                                    }
                                    Ok(false) => {
                                        let _ = irc.send_message(&channel,
                                            &format!("@{} Task #{} not found.", username, id)
                                        ).await;
                                    }
                                    Err(e) => {
                                        log::error!("Database error: {}", e);
                                        let _ = irc.send_message(&channel, "Database error!").await;
                                    }
                                }
                            }
                            None => {
                                let _ = irc.send_message(&channel,
                                    &format!("@{} Invalid ID. Usage: !task remove <#id>", username)
                                ).await;
                            }
                        }
                    }

                    // !task clear - moderator only, clear all tasks in channel
                    Some("clear") => {
                        let is_moderator = ctx.message.is_moderator;
                        let is_broadcaster = ctx.message.badges.iter().any(|b| b.starts_with("broadcaster"));

                        if !is_moderator && !is_broadcaster {
                            let _ = irc.send_message(&channel,
                                &format!("@{} Only moderators can clear all tasks!", username)
                            ).await;
                            return;
                        }

                        match db.clear_all_channel_todos(&channel) {
                            Ok(count) => {
                                if count > 0 {
                                    let _ = irc.send_message(&channel,
                                        &format!("🗑️ @{} Cleared all {} task(s) from the channel.", username, count)
                                    ).await;
                                } else {
                                    let _ = irc.send_message(&channel,
                                        &format!("@{} No tasks to clear.", username)
                                    ).await;
                                }
                            }
                            Err(e) => {
                                log::error!("Database error: {}", e);
                                let _ = irc.send_message(&channel, "Database error!").await;
                            }
                        }
                    }

                    // !task @username - view someone else's tasks
                    Some(target) if target.starts_with('@') => {
                        let target_user = target.trim_start_matches('@');

                        match db.get_user_todos(&channel, target_user) {
                            Ok(todos) => {
                                if todos.is_empty() {
                                    let _ = irc.send_message(&channel,
                                        &format!("📝 @{} has no active tasks!", target_user)
                                    ).await;
                                } else {
                                    let todo_list: Vec<String> = todos.iter()
                                        .take(3) // Show max 3 tasks
                                        .map(|(id, task, _)| format!("#{}: {}", id, task))
                                        .collect();

                                    let count = todos.len();
                                    let suffix = if count > 3 { format!(" (+{} more)", count - 3) } else { String::new() };

                                    let _ = irc.send_message(&channel,
                                        &format!("📝 @{}'s tasks: {}{}", target_user, todo_list.join(" | "), suffix)
                                    ).await;
                                }
                            }
                            Err(e) => {
                                log::error!("Database error: {}", e);
                                let _ = irc.send_message(&channel, "Database error!").await;
                            }
                        }
                    }

                    // !task or !task list - view own tasks
                    None | Some("list") | Some("show") => {
                        match db.get_user_todos(&channel, &username) {
                            Ok(todos) => {
                                if todos.is_empty() {
                                    let _ = irc.send_message(&channel,
                                        &format!("📝 @{} You have no active tasks! Use: !task <task>", username)
                                    ).await;
                                } else {
                                    let todo_list: Vec<String> = todos.iter()
                                        .take(3) // Show max 3 tasks
                                        .map(|(id, task, _)| format!("#{}: {}", id, task))
                                        .collect();

                                    let count = todos.len();
                                    let suffix = if count > 3 { format!(" (+{} more)", count - 3) } else { String::new() };

                                    let _ = irc.send_message(&channel,
                                        &format!("📝 @{}'s tasks: {}{}", username, todo_list.join(" | "), suffix)
                                    ).await;
                                }
                            }
                            Err(e) => {
                                log::error!("Database error: {}", e);
                                let _ = irc.send_message(&channel, "Database error!").await;
                            }
                        }
                    }

                    // Default - !task <task description> - add a task
                    Some(_) => {
                        let task = args.join(" ");

                        if task.len() > 200 {
                            let _ = irc.send_message(&channel,
                                &format!("@{} Task too long! Keep it under 200 characters.", username)
                            ).await;
                            return;
                        }

                        match db.add_todo(&channel, &username, &task) {
                            Ok(id) => {
                                let _ = irc.send_message(&channel,
                                    &format!("✅ @{} Added task #{}: {}", username, id, task)
                                ).await;
                            }
                            Err(_) => {
                                // Silent fail - don't reveal spam prevention to users
                            }
                        }
                    }
                }
            });

            Ok(None)
        }),
    };

    command_system.register_command(command).await;
    log::info!("✅ Registered task command");
}
