use crate::modules::twitch::{CommandSystem, Command, PermissionLevel};
use std::sync::Arc;
use super::database::Database;
use reqwest;
use std::path::PathBuf;
use std::fs;
use std::io::BufReader;
use rodio::{Decoder, OutputStream, Sink};
use tokio::sync::mpsc;
use std::sync::OnceLock;

/// TTS message for queueing
#[derive(Debug, Clone)]
struct TTSMessage {
    text: String,
    voice: String,
}

/// Global TTS queue sender
static TTS_QUEUE: OnceLock<mpsc::UnboundedSender<TTSMessage>> = OnceLock::new();

/// Initialize TTS queue
fn get_tts_queue() -> &'static mpsc::UnboundedSender<TTSMessage> {
    TTS_QUEUE.get_or_init(|| {
        let (tx, mut rx) = mpsc::unbounded_channel::<TTSMessage>();

        // Spawn the queue processor
        tokio::spawn(async move {
            log::info!("🔊 TTS queue processor started");

            while let Some(msg) = rx.recv().await {
                log::debug!("Processing TTS message: {} (voice: {})", msg.text, msg.voice);

                // URL encode the text
                let encoded_text = percent_encoding::utf8_percent_encode(
                    &msg.text,
                    percent_encoding::NON_ALPHANUMERIC
                ).to_string();

                // Use StreamElements TTS API
                let tts_url = format!(
                    "https://api.streamelements.com/kappa/v2/speech?voice={}&text={}",
                    msg.voice, encoded_text
                );

                // Download and play the audio file (blocking until complete)
                if let Err(e) = download_and_play_tts(&tts_url).await {
                    log::error!("TTS playback failed: {}", e);
                }
            }

            log::info!("🔇 TTS queue processor stopped");
        });

        tx
    })
}

/// Speak text using external TTS API and rodio audio playback (queued)
pub fn speak_text(text: &str, voice: &str) {
    let msg = TTSMessage {
        text: text.to_string(),
        voice: voice.to_string(),
    };

    // Send to queue (non-blocking)
    let queue = get_tts_queue();
    if let Err(e) = queue.send(msg) {
        log::error!("Failed to queue TTS message: {}", e);
    }
}

async fn download_and_play_tts(url: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Create temp directory if it doesn't exist
    let temp_dir = PathBuf::from("data/temp");
    fs::create_dir_all(&temp_dir)?;

    // Download the audio file
    let response = reqwest::get(url).await?;
    let bytes = response.bytes().await?;

    // Save to temp file
    let temp_file = temp_dir.join("tts_current.mp3");
    fs::write(&temp_file, bytes)?;

    // Play audio using rodio in a blocking task
    let temp_file_clone = temp_file.clone();
    tokio::task::spawn_blocking(move || {
        if let Err(e) = play_audio_file(&temp_file_clone) {
            log::error!("Audio playback error: {}", e);
        }
    }).await?;

    Ok(())
}

fn play_audio_file(file_path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    // Get an output stream handle to the default physical sound device
    let (_stream, stream_handle) = OutputStream::try_default()?;

    // Create a sink for audio playback
    let sink = Sink::try_new(&stream_handle)?;

    // Load the audio file
    let file = fs::File::open(file_path)?;
    let buf_reader = BufReader::new(file);

    // Decode the audio file
    let source = Decoder::new(buf_reader)?;

    // Play the audio
    sink.append(source);

    // Wait for playback to finish
    sink.sleep_until_end();

    Ok(())
}

pub async fn register(command_system: &CommandSystem, db: Database) {
    let command = Command {
        name: "tts".to_string(),
        aliases: vec![],
        description: "Manage text-to-speech settings".to_string(),
        usage: "!tts [on|off|mode|add|remove|list]".to_string(),
        permission: PermissionLevel::Broadcaster,
        cooldown_seconds: 1,
        enabled: true,
        handler: Arc::new(move |ctx, irc, _api| {
            let irc = irc.clone();
            let channel = ctx.channel.clone();
            let db = db.clone();
            let username = ctx.message.username.clone();
            let args = ctx.args.clone();

            tokio::spawn(async move {
                let subcommand = args.first().map(|s| s.as_str());

                match subcommand {
                    Some("on") | Some("enable") => {
                        match db.set_tts_enabled(&channel, true) {
                            Ok(_) => {
                                let mode = db.get_tts_mode(&channel).unwrap_or_else(|_| "broadcaster".to_string());
                                let _ = irc.send_message(&channel,
                                    &format!("🔊 TTS is now ON! Mode: {}", mode)
                                ).await;
                            }
                            Err(e) => {
                                log::error!("Database error: {}", e);
                                let _ = irc.send_message(&channel, "Database error!").await;
                            }
                        }
                    }

                    Some("off") | Some("disable") => {
                        match db.set_tts_enabled(&channel, false) {
                            Ok(_) => {
                                let _ = irc.send_message(&channel, "🔇 TTS is now OFF.").await;
                            }
                            Err(e) => {
                                log::error!("Database error: {}", e);
                                let _ = irc.send_message(&channel, "Database error!").await;
                            }
                        }
                    }

                    Some("mode") => {
                        if let Some(mode) = args.get(1).map(|s| s.as_str()) {
                            match mode {
                                "broadcaster" | "whitelist" | "everyone" => {
                                    match db.set_tts_mode(&channel, mode) {
                                        Ok(_) => {
                                            let description = match mode {
                                                "broadcaster" => "Only broadcaster messages will be spoken",
                                                "whitelist" => "Only whitelisted users' messages will be spoken",
                                                "everyone" => "All messages will be spoken",
                                                _ => "Mode updated"
                                            };
                                            let _ = irc.send_message(&channel,
                                                &format!("✅ TTS mode set to: {}. {}", mode, description)
                                            ).await;
                                        }
                                        Err(e) => {
                                            log::error!("Database error: {}", e);
                                            let _ = irc.send_message(&channel, "Database error!").await;
                                        }
                                    }
                                }
                                _ => {
                                    let _ = irc.send_message(&channel,
                                        "@{} Invalid mode. Use: broadcaster, whitelist, or everyone"
                                    ).await;
                                }
                            }
                        } else {
                            let mode = db.get_tts_mode(&channel).unwrap_or_else(|_| "broadcaster".to_string());
                            let _ = irc.send_message(&channel,
                                &format!("Current TTS mode: {}", mode)
                            ).await;
                        }
                    }

                    Some("add") => {
                        if let Some(target_user) = args.get(1) {
                            match db.add_tts_user(&channel, target_user) {
                                Ok(_) => {
                                    let _ = irc.send_message(&channel,
                                        &format!("✅ Added {} to TTS whitelist", target_user)
                                    ).await;
                                }
                                Err(e) => {
                                    log::error!("Database error: {}", e);
                                    let _ = irc.send_message(&channel, "Database error!").await;
                                }
                            }
                        } else {
                            let _ = irc.send_message(&channel,
                                "@{} Usage: !tts add <username>"
                            ).await;
                        }
                    }

                    Some("remove") | Some("rm") => {
                        if let Some(target_user) = args.get(1) {
                            match db.remove_tts_user(&channel, target_user) {
                                Ok(true) => {
                                    let _ = irc.send_message(&channel,
                                        &format!("✅ Removed {} from TTS whitelist", target_user)
                                    ).await;
                                }
                                Ok(false) => {
                                    let _ = irc.send_message(&channel,
                                        &format!("❌ {} was not on the whitelist", target_user)
                                    ).await;
                                }
                                Err(e) => {
                                    log::error!("Database error: {}", e);
                                    let _ = irc.send_message(&channel, "Database error!").await;
                                }
                            }
                        } else {
                            let _ = irc.send_message(&channel,
                                "@{} Usage: !tts remove <username>"
                            ).await;
                        }
                    }

                    Some("list") => {
                        match db.get_tts_users(&channel) {
                            Ok(users) => {
                                if users.is_empty() {
                                    let _ = irc.send_message(&channel,
                                        "TTS whitelist is empty. Add users with: !tts add <username>"
                                    ).await;
                                } else {
                                    let _ = irc.send_message(&channel,
                                        &format!("TTS whitelist: {}", users.join(", "))
                                    ).await;
                                }
                            }
                            Err(e) => {
                                log::error!("Database error: {}", e);
                                let _ = irc.send_message(&channel, "Database error!").await;
                            }
                        }
                    }

                    // Check status
                    None => {
                        match (db.is_tts_enabled(&channel), db.get_tts_mode(&channel)) {
                            (Ok(enabled), Ok(mode)) => {
                                let status = if enabled { "ON 🔊" } else { "OFF 🔇" };
                                let _ = irc.send_message(&channel,
                                    &format!("TTS: {} | Mode: {} | Commands: on/off/mode/add/remove/list", status, mode)
                                ).await;
                            }
                            _ => {
                                let _ = irc.send_message(&channel, "Database error!").await;
                            }
                        }
                    }

                    _ => {
                        let _ = irc.send_message(&channel,
                            &format!("@{} Usage: !tts [on|off|mode|add|remove|list]", username)
                        ).await;
                    }
                }
            });

            Ok(None)
        }),
    };

    command_system.register_command(command).await;
    log::info!("✅ Registered TTS command");
}
