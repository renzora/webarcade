use crate::modules::twitch::{CommandSystem, Command, PermissionLevel};
use std::sync::Arc;
use super::database::Database;

/// Available StreamElements TTS voices
const VOICES: &[(&str, &str)] = &[
    // English voices
    ("Brian", "English (UK) - Male"),
    ("Amy", "English (UK) - Female"),
    ("Emma", "English (UK) - Female"),
    ("Geraint", "English (Welsh) - Male"),
    ("Russell", "English (Australian) - Male"),
    ("Nicole", "English (Australian) - Female"),
    ("Joey", "English (US) - Male"),
    ("Justin", "English (US) - Male (Child)"),
    ("Matthew", "English (US) - Male"),
    ("Ivy", "English (US) - Female (Child)"),
    ("Joanna", "English (US) - Female"),
    ("Kendra", "English (US) - Female"),
    ("Kimberly", "English (US) - Female"),
    ("Salli", "English (US) - Female"),
    ("Raveena", "English (Indian) - Female"),

    // Other languages
    ("Cristiano", "Portuguese (European) - Male"),
    ("Ines", "Portuguese (European) - Female"),
    ("Vitoria", "Portuguese (Brazilian) - Female"),
    ("Ricardo", "Portuguese (Brazilian) - Male"),
    ("Mizuki", "Japanese - Female"),
    ("Takumi", "Japanese - Male"),
    ("Seoyeon", "Korean - Female"),
    ("Liv", "Norwegian - Female"),
    ("Lotte", "Dutch - Female"),
    ("Ruben", "Dutch - Male"),
    ("Jacek", "Polish - Male"),
    ("Jan", "Polish - Male"),
    ("Ewa", "Polish - Female"),
    ("Maja", "Polish - Female"),
    ("Filiz", "Turkish - Female"),
    ("Astrid", "Swedish - Female"),
    ("Maxim", "Russian - Male"),
    ("Tatyana", "Russian - Female"),
    ("Carmen", "Romanian - Female"),
    ("Gwyneth", "Welsh - Female"),
    ("Mads", "Danish - Male"),
    ("Naja", "Danish - Female"),
    ("Hans", "German - Male"),
    ("Marlene", "German - Female"),
    ("Vicki", "German - Female"),
    ("Karl", "Icelandic - Male"),
    ("Dora", "Icelandic - Female"),
    ("Giorgio", "Italian - Male"),
    ("Carla", "Italian - Female"),
    ("Bianca", "Italian - Female"),
    ("Celine", "French - Female"),
    ("Lea", "French - Female"),
    ("Mathieu", "French - Male"),
    ("Chantal", "French (Canadian) - Female"),
    ("Penelope", "Spanish (US) - Female"),
    ("Miguel", "Spanish (US) - Male"),
    ("Enrique", "Spanish (European) - Male"),
    ("Conchita", "Spanish (European) - Female"),
    ("Lucia", "Spanish (European) - Female"),
];

/// Get voice name by partial match (case insensitive)
fn find_voice(query: &str) -> Option<&'static str> {
    let query_lower = query.to_lowercase();

    // Exact match first
    if let Some((name, _)) = VOICES.iter().find(|(name, _)| name.to_lowercase() == query_lower) {
        return Some(name);
    }

    // Partial match
    VOICES.iter()
        .find(|(name, desc)| {
            name.to_lowercase().contains(&query_lower) ||
            desc.to_lowercase().contains(&query_lower)
        })
        .map(|(name, _)| *name)
}

pub async fn register(command_system: &CommandSystem, db: Database) {
    // !voice command - set TTS voice
    let voice_command = Command {
        name: "voice".to_string(),
        aliases: vec!["ttsvoice".to_string()],
        description: "Set your TTS voice preference".to_string(),
        usage: "!voice <voice_name> or !voice to see current".to_string(),
        permission: PermissionLevel::Everyone,
        cooldown_seconds: 3,
        enabled: true,
        handler: Arc::new(move |ctx, irc, _api| {
            let irc = irc.clone();
            let channel = ctx.channel.clone();
            let db = db.clone();
            let username = ctx.message.username.clone();
            let args = ctx.args.clone();

            tokio::spawn(async move {
                // Check if user has TTS privileges
                let is_broadcaster = ctx.message.badges.iter().any(|b| b.starts_with("broadcaster"));
                let has_privilege = match db.has_tts_privilege(&channel, &username, is_broadcaster) {
                    Ok(privilege) => privilege,
                    Err(e) => {
                        log::error!("Database error: {}", e);
                        let _ = irc.send_message(&channel, "Database error!").await;
                        return;
                    }
                };

                if !has_privilege {
                    let _ = irc.send_message(&channel,
                        &format!("@{} You need TTS privileges to set a voice. Ask a mod to add you!", username)
                    ).await;
                    return;
                }

                if args.is_empty() {
                    // Show current voice
                    match db.get_tts_voice(&channel, &username) {
                        Ok(voice) => {
                            let _ = irc.send_message(&channel,
                                &format!("@{} Your current TTS voice is: {}. Use !voice <name> to change it or !voices to see all options.", username, voice)
                            ).await;
                        }
                        Err(e) => {
                            log::error!("Database error: {}", e);
                            let _ = irc.send_message(&channel, "Database error!").await;
                        }
                    }
                    return;
                }

                let voice_query = args.join(" ");

                // Find matching voice
                match find_voice(&voice_query) {
                    Some(voice_name) => {
                        match db.set_tts_voice(&channel, &username, voice_name) {
                            Ok(_) => {
                                let voice_desc = VOICES.iter()
                                    .find(|(name, _)| name == &voice_name)
                                    .map(|(_, desc)| *desc)
                                    .unwrap_or("Unknown");

                                let _ = irc.send_message(&channel,
                                    &format!("✅ @{} Your TTS voice is now set to: {} ({})", username, voice_name, voice_desc)
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
                            &format!("@{} Voice '{}' not found. Use !voices to see all available voices.", username, voice_query)
                        ).await;
                    }
                }
            });

            Ok(None)
        }),
    };

    // !voices command - list available voices
    let voices_command = Command {
        name: "voices".to_string(),
        aliases: vec!["ttsvoices".to_string()],
        description: "List available TTS voices".to_string(),
        usage: "!voices [language]".to_string(),
        permission: PermissionLevel::Everyone,
        cooldown_seconds: 5,
        enabled: true,
        handler: Arc::new(|ctx, irc, _api| {
            let irc = irc.clone();
            let channel = ctx.channel.clone();
            let username = ctx.message.username.clone();
            let args = ctx.args.clone();

            tokio::spawn(async move {
                let filter = args.first().map(|s| s.to_lowercase());

                let voice_list: Vec<String> = VOICES.iter()
                    .filter(|(name, desc)| {
                        if let Some(ref f) = filter {
                            desc.to_lowercase().contains(f) || name.to_lowercase().contains(f)
                        } else {
                            true
                        }
                    })
                    .take(15) // Limit to avoid spam
                    .map(|(name, desc)| format!("{} ({})", name, desc))
                    .collect();

                if voice_list.is_empty() {
                    let _ = irc.send_message(&channel,
                        &format!("@{} No voices found matching '{}'", username, filter.unwrap_or_default())
                    ).await;
                } else {
                    let count_text = if VOICES.len() > 15 && filter.is_none() {
                        format!(" (showing 15/{} voices)", VOICES.len())
                    } else {
                        String::new()
                    };

                    let _ = irc.send_message(&channel,
                        &format!("🔊 Available TTS voices{}: {}", count_text, voice_list.join(", "))
                    ).await;
                }
            });

            Ok(None)
        }),
    };

    command_system.register_command(voice_command).await;
    command_system.register_command(voices_command).await;
    log::info!("✅ Registered voice commands");
}
