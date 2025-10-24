use crate::modules::twitch::{CommandSystem, Command, PermissionLevel, TwitchIRCManager};
use std::sync::Arc;
use super::database::Database;

pub async fn register(command_system: &CommandSystem, db: Database) {
    // !debuguser command - show user's database info
    let db_clone = db.clone();
    let command = Command {
        name: "debuguser".to_string(),
        aliases: vec![],
        description: "Debug: Show your user database info".to_string(),
        usage: "!debuguser".to_string(),
        permission: PermissionLevel::Everyone,
        cooldown_seconds: 5,
        enabled: true,
        handler: Arc::new(move |ctx, irc, _api| {
            let irc = irc.clone();
            let channel = ctx.channel.clone();
            let db = db_clone.clone();
            let username = ctx.message.username.clone();

            tokio::spawn(async move {
                // Get location
                let location = db.get_user_location(&channel, &username)
                    .ok()
                    .flatten();

                // Get flag
                let flag = location.as_ref()
                    .and_then(|loc| TwitchIRCManager::get_location_flag(loc));

                // Get level
                let level = db.get_user_level(&channel, &username)
                    .ok()
                    .flatten()
                    .map(|(lvl, _, _, _)| lvl);

                // Get birthday
                let birthday = db.get_user_birthday(&channel, &username)
                    .ok()
                    .flatten();

                let msg = format!(
                    "@{} Debug Info | Location: {:?} | Flag: {:?} | Level: {:?} | Birthday: {:?}",
                    username,
                    location,
                    flag,
                    level,
                    birthday
                );

                let _ = irc.send_message(&channel, &msg).await;
            });

            Ok(None)
        }),
    };

    command_system.register_command(command).await;

    log::info!("✅ Registered debug user command");
}
