use crate::modules::twitch::{CommandSystem, Command, CommandContext, PermissionLevel, TwitchIRCManager, TwitchAPI};
use std::sync::Arc;
use super::database::Database;

/// Replace variables in the response text
fn replace_variables(text: &str, ctx: &CommandContext) -> String {
    let mut result = text.to_string();

    // Replace {username} with the user who triggered the command
    result = result.replace("{username}", &ctx.message.username);

    // Replace {channel} with the channel name
    result = result.replace("{channel}", &ctx.channel);

    // Replace {displayname} with display name (fallback to username)
    result = result.replace("{displayname}", &ctx.message.username);

    // Replace {args} with all arguments joined by space
    let args = ctx.args.join(" ");
    result = result.replace("{args}", &args);

    // Replace {count} with number of arguments
    result = result.replace("{count}", &ctx.args.len().to_string());

    result
}

pub async fn register(command_system: &CommandSystem, db: Database) {
    log::info!("✅ Custom text commands will be loaded dynamically");
}

/// Register a dynamic text command
pub async fn register_text_command(
    command_system: &CommandSystem,
    db: Database,
    channel: &str,
    command_name: &str,
    response: &str
) {
    let command_lower = command_name.to_lowercase();
    let response_clone = response.to_string();
    let db_clone = db.clone();

    let command = Command {
        name: command_lower.clone(),
        aliases: vec![],
        description: format!("Custom text command: {}", command_name),
        usage: format!("!{}", command_lower),
        permission: PermissionLevel::Everyone,
        cooldown_seconds: 5,
        enabled: true,
        handler: Arc::new(move |ctx, irc, _api| {
            let irc = irc.clone();
            let channel = ctx.channel.clone();
            let response = response_clone.clone();

            tokio::spawn(async move {
                // Replace variables in the response
                let message = replace_variables(&response, &ctx);

                let _ = irc.send_message(&channel, &message).await;
            });

            Ok(None)
        }),
    };

    command_system.register_command(command).await;
}

/// Load all custom text commands from the database
pub async fn load_text_commands(command_system: &CommandSystem, db: Database, channel: &str) {
    match db.get_all_text_commands(channel) {
        Ok(commands) => {
            let count = commands.len();
            for (command_name, response, _auto_post, _interval) in commands {
                register_text_command(command_system, db.clone(), channel, &command_name, &response).await;
                log::info!("Registered custom text command: !{}", command_name);
            }
            log::info!("✅ Loaded {} custom text commands for channel: {}", count, channel);
        }
        Err(e) => {
            log::error!("Failed to load custom text commands: {}", e);
        }
    }
}
