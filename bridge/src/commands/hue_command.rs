use crate::modules::twitch::{CommandSystem, Command, PermissionLevel};
use crate::commands::database::Database;
use crate::commands::hue::HueClient;
use std::sync::Arc;

pub async fn register(command_system: &CommandSystem, db: Database) {
    let db_clone = db.clone();
    command_system.register_command(Command {
        name: "lights".to_string(),
        aliases: vec!["hue".to_string()],
        description: "Control Philips Hue lights".to_string(),
        usage: "!lights <on|off|red|blue|green|purple|orange|yellow|cyan|pink|white>".to_string(),
        permission: PermissionLevel::Broadcaster,
        cooldown_seconds: 2,
        enabled: true,
        handler: Arc::new(move |ctx, irc, _api| {
            let db = db_clone.clone();
            let irc = irc.clone();
            let channel = ctx.channel.clone();
            let args = ctx.args.clone();

            tokio::spawn(async move {
                let response = handle_lights_command(&args, &db).await;
                let _ = irc.send_message(&channel, &response).await;
            });

            Ok(None)
        }),
    }).await;
}

async fn handle_lights_command(args: &[String], db: &Database) -> String {
    // Get Hue config from database
    let config = match db.get_hue_config() {
        Ok(Some((bridge_ip, username))) => (bridge_ip, username),
        Ok(None) => return "⚠️ Hue bridge not configured. Use the Hue Settings viewport to set up your bridge.".to_string(),
        Err(e) => return format!("❌ Database error: {}", e),
    };

    let client = HueClient::new(config.0, config.1);

    if args.is_empty() {
        return "Usage: !lights <on|off|red|blue|green|purple|orange|yellow|cyan|pink|white|custom_scene_name>".to_string();
    }

    let command = args[0].to_lowercase();

    match command.as_str() {
        "on" => {
            match client.set_all_lights(true).await {
                Ok(_) => "💡 Lights turned on!".to_string(),
                Err(e) => format!("❌ Failed to turn on lights: {}", e),
            }
        }
        "off" => {
            match client.set_all_lights(false).await {
                Ok(_) => "🌙 Lights turned off!".to_string(),
                Err(e) => format!("❌ Failed to turn off lights: {}", e),
            }
        }
        // Handle built-in color scenes
        "red" | "blue" | "green" | "purple" | "orange" | "yellow" | "cyan" | "pink" | "white" => {
            match client.set_scene(&command).await {
                Ok(_) => format!("🎨 Lights set to {}!", command),
                Err(e) => format!("❌ Failed to set scene: {}", e),
            }
        }
        // Check for custom scenes
        _ => {
            // First try animated scenes (by tag)
            match db.get_animated_scene_by_tag(&command) {
                Ok(Some((_id, name, steps))) => {
                    // Spawn the animation in the background
                    let client_clone = client.clone();
                    tokio::spawn(async move {
                        let _ = client_clone.play_animated_scene(steps).await;
                    });
                    format!("🎬 Playing scene '{}'!", name)
                }
                Ok(None) => {
                    // Try simple color preset
                    match db.get_hue_scene(&command) {
                        Ok(Some((r, g, b))) => {
                            match client.set_all_lights_rgb(r, g, b).await {
                                Ok(_) => format!("🎨 Lights set to {}!", command),
                                Err(e) => format!("❌ Failed to set scene: {}", e),
                            }
                        }
                        Ok(None) => format!("❌ Unknown scene '{}'. Use: on, off, or a color name.", command),
                        Err(e) => format!("❌ Database error: {}", e),
                    }
                }
                Err(e) => format!("❌ Database error: {}", e),
            }
        }
    }
}
