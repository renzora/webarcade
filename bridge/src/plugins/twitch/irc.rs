// Twitch IRC client implementation
// This module handles the connection to Twitch IRC chat

use anyhow::Result;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct TwitchIrcClient {
    sender: mpsc::UnboundedSender<IrcCommand>,
}

pub enum IrcCommand {
    SendMessage { channel: String, message: String },
    JoinChannel { channel: String },
    PartChannel { channel: String },
    Disconnect, // Force disconnect to trigger reconnection with fresh token
}

pub struct IrcMessage {
    pub raw: String,
    pub prefix: Option<String>,
    pub command: String,
    pub params: Vec<String>,
    pub tags: std::collections::HashMap<String, String>,
}

impl IrcMessage {
    pub fn parse(line: &str) -> Option<Self> {
        let raw = line.to_string();
        let mut tags = std::collections::HashMap::new();
        let mut prefix = None;
        let mut pos = 0;

        // Parse tags (if present)
        if line.starts_with('@') {
            if let Some(space_pos) = line.find(' ') {
                let tags_str = &line[1..space_pos];
                for tag in tags_str.split(';') {
                    if let Some(eq_pos) = tag.find('=') {
                        let key = &tag[..eq_pos];
                        let value = &tag[eq_pos + 1..];
                        tags.insert(key.to_string(), value.to_string());
                    }
                }
                pos = space_pos + 1;
            }
        }

        let rest = &line[pos..];

        // Parse prefix (if present)
        let rest = if rest.starts_with(':') {
            if let Some(space_pos) = rest.find(' ') {
                prefix = Some(rest[1..space_pos].to_string());
                &rest[space_pos + 1..]
            } else {
                return None;
            }
        } else {
            rest
        };

        // Parse command and params
        let mut parts: Vec<&str> = rest.split_whitespace().collect();
        if parts.is_empty() {
            return None;
        }

        let command = parts[0].to_string();
        parts.remove(0);

        let mut params = Vec::new();
        let mut trailing_start = None;

        for (i, part) in parts.iter().enumerate() {
            if part.starts_with(':') {
                trailing_start = Some(i);
                break;
            }
            params.push(part.to_string());
        }

        // Handle trailing parameter
        if let Some(start) = trailing_start {
            let trailing = parts[start..].join(" ");
            params.push(trailing[1..].to_string()); // Remove leading ':'
        }

        Some(IrcMessage {
            raw,
            prefix,
            command,
            params,
            tags,
        })
    }
}

impl TwitchIrcClient {
    pub async fn connect(
        oauth_token: String,
        nickname: String,
        channels: Vec<String>,
        event_sender: mpsc::UnboundedSender<IrcMessage>,
        shared_token: Arc<RwLock<String>>, // Shared token that can be updated by refresh task
    ) -> Result<Self> {
        let (cmd_tx, mut cmd_rx) = mpsc::unbounded_channel::<IrcCommand>();

        let initial_token = if oauth_token.starts_with("oauth:") {
            oauth_token
        } else {
            format!("oauth:{}", oauth_token)
        };

        // Initialize the shared token with the initial value
        {
            let mut token = shared_token.write().await;
            *token = initial_token;
        }

        tokio::spawn(async move {
            loop {
                log::info!("[Twitch IRC] Connecting to irc.chat.twitch.tv:6667");

                // Read the current token from shared memory
                let current_token = {
                    let token = shared_token.read().await;
                    token.clone()
                };

                match TcpStream::connect("irc.chat.twitch.tv:6667").await {
                    Ok(stream) => {
                        log::info!("[Twitch IRC] Connected successfully");

                        if let Err(e) = Self::handle_connection(
                            stream,
                            current_token,
                            nickname.clone(),
                            channels.clone(),
                            &mut cmd_rx,
                            event_sender.clone(),
                        ).await {
                            log::error!("[Twitch IRC] Connection error: {}", e);
                        }
                    }
                    Err(e) => {
                        log::error!("[Twitch IRC] Failed to connect: {}", e);
                    }
                }

                log::info!("[Twitch IRC] Reconnecting in 5 seconds...");
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            }
        });

        Ok(TwitchIrcClient { sender: cmd_tx })
    }

    async fn handle_connection(
        stream: TcpStream,
        oauth_token: String,
        nickname: String,
        channels: Vec<String>,
        cmd_rx: &mut mpsc::UnboundedReceiver<IrcCommand>,
        event_sender: mpsc::UnboundedSender<IrcMessage>,
    ) -> Result<()> {
        let (read_half, mut write_half) = stream.into_split();
        let mut reader = BufReader::new(read_half);

        // Authenticate
        write_half.write_all(format!("PASS {}\r\n", oauth_token).as_bytes()).await?;
        write_half.write_all(format!("NICK {}\r\n", nickname).as_bytes()).await?;

        // Request capabilities
        write_half.write_all(b"CAP REQ :twitch.tv/tags twitch.tv/commands twitch.tv/membership\r\n").await?;
        write_half.flush().await?;

        log::info!("[Twitch IRC] Sent authentication");

        // Join channels after authentication
        let channels_to_join = channels.clone();
        let write_half = Arc::new(tokio::sync::Mutex::new(write_half));
        let write_half_for_reader = write_half.clone();

        // Spawn task to read messages
        let event_sender_clone = event_sender.clone();
        tokio::spawn(async move {
            let mut line = String::new();
            loop {
                line.clear();
                match reader.read_line(&mut line).await {
                    Ok(0) => {
                        log::warn!("[Twitch IRC] Connection closed");
                        break;
                    }
                    Ok(_) => {
                        let line = line.trim();
                        if line.is_empty() {
                            continue;
                        }

                        log::debug!("[Twitch IRC] <<< {}", line);

                        // Handle PING
                        if line.starts_with("PING") {
                            if let Some(msg) = line.strip_prefix("PING ") {
                                let mut writer = write_half_for_reader.lock().await;
                                if let Err(e) = writer.write_all(format!("PONG {}\r\n", msg).as_bytes()).await {
                                    log::error!("[Twitch IRC] Failed to send PONG: {}", e);
                                    break;
                                }
                                if let Err(e) = writer.flush().await {
                                    log::error!("[Twitch IRC] Failed to flush PONG: {}", e);
                                    break;
                                }
                                log::debug!("[Twitch IRC] >>> PONG {}", msg);
                            }
                            continue;
                        }

                        // Parse message
                        if let Some(msg) = IrcMessage::parse(line) {
                            // Check for successful authentication (001 = RPL_WELCOME)
                            if msg.command == "001" {
                                log::info!("[Twitch IRC] Authentication successful");

                                // Join channels
                                let mut writer = write_half_for_reader.lock().await;
                                for channel in &channels_to_join {
                                    let channel_name = if channel.starts_with('#') {
                                        channel.clone()
                                    } else {
                                        format!("#{}", channel)
                                    };

                                    if let Err(e) = writer.write_all(format!("JOIN {}\r\n", channel_name).as_bytes()).await {
                                        log::error!("[Twitch IRC] Failed to join {}: {}", channel_name, e);
                                    } else {
                                        log::info!("[Twitch IRC] Joining {}", channel_name);
                                    }
                                }
                                if let Err(e) = writer.flush().await {
                                    log::error!("[Twitch IRC] Failed to flush JOIN commands: {}", e);
                                }
                            }

                            // Send to event handler
                            if let Err(e) = event_sender_clone.send(msg) {
                                log::error!("[Twitch IRC] Failed to send event: {}", e);
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        log::error!("[Twitch IRC] Read error: {}", e);
                        break;
                    }
                }
            }
        });

        // Handle outgoing commands
        while let Some(cmd) = cmd_rx.recv().await {
            let mut writer = write_half.lock().await;
            match cmd {
                IrcCommand::SendMessage { channel, message } => {
                    let channel_name = if channel.starts_with('#') {
                        channel
                    } else {
                        format!("#{}", channel)
                    };

                    let msg = format!("PRIVMSG {} :{}\r\n", channel_name, message);
                    log::debug!("[Twitch IRC] >>> {}", msg.trim());

                    if let Err(e) = writer.write_all(msg.as_bytes()).await {
                        log::error!("[Twitch IRC] Failed to send message: {}", e);
                        return Err(e.into());
                    }
                    if let Err(e) = writer.flush().await {
                        log::error!("[Twitch IRC] Failed to flush: {}", e);
                        return Err(e.into());
                    }
                }
                IrcCommand::JoinChannel { channel } => {
                    let channel_name = if channel.starts_with('#') {
                        channel
                    } else {
                        format!("#{}", channel)
                    };

                    let msg = format!("JOIN {}\r\n", channel_name);
                    log::info!("[Twitch IRC] >>> {}", msg.trim());

                    if let Err(e) = writer.write_all(msg.as_bytes()).await {
                        log::error!("[Twitch IRC] Failed to join channel: {}", e);
                        return Err(e.into());
                    }
                    if let Err(e) = writer.flush().await {
                        log::error!("[Twitch IRC] Failed to flush: {}", e);
                        return Err(e.into());
                    }
                }
                IrcCommand::PartChannel { channel } => {
                    let channel_name = if channel.starts_with('#') {
                        channel
                    } else {
                        format!("#{}", channel)
                    };

                    let msg = format!("PART {}\r\n", channel_name);
                    log::info!("[Twitch IRC] >>> {}", msg.trim());

                    if let Err(e) = writer.write_all(msg.as_bytes()).await {
                        log::error!("[Twitch IRC] Failed to part channel: {}", e);
                        return Err(e.into());
                    }
                    if let Err(e) = writer.flush().await {
                        log::error!("[Twitch IRC] Failed to flush: {}", e);
                        return Err(e.into());
                    }
                }
                IrcCommand::Disconnect => {
                    log::info!("[Twitch IRC] Disconnect requested, forcing reconnection");
                    return Err(anyhow::anyhow!("Disconnect requested"));
                }
            }
        }

        Ok(())
    }

    pub fn send_message(&self, channel: &str, message: &str) -> Result<()> {
        self.sender.send(IrcCommand::SendMessage {
            channel: channel.to_string(),
            message: message.to_string(),
        })?;
        Ok(())
    }

    pub fn join_channel(&self, channel: &str) -> Result<()> {
        self.sender.send(IrcCommand::JoinChannel {
            channel: channel.to_string(),
        })?;
        Ok(())
    }

    pub fn part_channel(&self, channel: &str) -> Result<()> {
        self.sender.send(IrcCommand::PartChannel {
            channel: channel.to_string(),
        })?;
        Ok(())
    }

    pub fn disconnect(&self) -> Result<()> {
        self.sender.send(IrcCommand::Disconnect)?;
        Ok(())
    }
}
