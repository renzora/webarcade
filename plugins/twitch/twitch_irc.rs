use anyhow::{Result, anyhow};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use serde_json::Value;
use crate::core::plugin_context::PluginContext;

use super::twitch_api;

const IRC_SERVER: &str = "irc.chat.twitch.tv:6667";

lazy_static::lazy_static! {
    static ref IRC_CLIENT: Arc<RwLock<Option<IrcClient>>> = Arc::new(RwLock::new(None));
}

pub struct IrcClient {
    writer: Arc<RwLock<tokio::io::WriteHalf<TcpStream>>>,
    connected: bool,
    channel: String,
}

pub async fn start_irc_client(ctx: Arc<PluginContext>) -> Result<()> {
    log::info!("[Twitch IRC] Starting IRC client...");

    // Get bot account token (or broadcaster if no bot)
    let token_result = twitch_api::get_account_token(&ctx, "bot").await;
    let (access_token, username) = if let Ok(token_data) = token_result {
        let token = token_data["access_token"]
            .as_str()
            .ok_or_else(|| anyhow!("Missing access token"))?
            .to_string();

        let conn = ctx.db()?;
        let user: String = conn.query_row(
            "SELECT username FROM twitch_accounts WHERE account_type = ?1",
            ["bot"],
            |row| row.get(0)
        )?;

        (token, user)
    } else {
        // Try broadcaster account
        let token_data = twitch_api::get_account_token(&ctx, "broadcaster").await?;
        let token = token_data["access_token"]
            .as_str()
            .ok_or_else(|| anyhow!("Missing access token"))?
            .to_string();

        let conn = ctx.db()?;
        let user: String = conn.query_row(
            "SELECT username FROM twitch_accounts WHERE account_type = ?1",
            ["broadcaster"],
            |row| row.get(0)
        )?;

        (token, user)
    };

    // Get channel to join from broadcaster account
    let conn = ctx.db()?;
    let channel: String = conn.query_row(
        "SELECT username FROM twitch_accounts WHERE account_type = ?1",
        ["broadcaster"],
        |row| row.get(0)
    )?;

    log::info!("[Twitch IRC] Connecting as {} to channel {}", username, channel);

    // Connect to IRC
    let stream = TcpStream::connect(IRC_SERVER).await?;
    let (reader, writer) = tokio::io::split(stream);
    let writer = Arc::new(RwLock::new(writer));

    // Authenticate
    {
        let mut w = writer.write().await;
        w.write_all(format!("PASS oauth:{}\r\n", access_token).as_bytes()).await?;
        w.write_all(format!("NICK {}\r\n", username).as_bytes()).await?;
        w.write_all(b"CAP REQ :twitch.tv/membership twitch.tv/tags twitch.tv/commands\r\n").await?;
        w.flush().await?;
    }

    // Join channel
    {
        let mut w = writer.write().await;
        w.write_all(format!("JOIN #{}\r\n", channel).as_bytes()).await?;
        w.flush().await?;
    }

    // Store client globally
    {
        let mut client = IRC_CLIENT.write().await;
        *client = Some(IrcClient {
            writer: writer.clone(),
            connected: true,
            channel: channel.clone(),
        });
    }

    log::info!("[Twitch IRC] Connected successfully");

    // Start reading messages
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    loop {
        line.clear();
        match reader.read_line(&mut line).await {
            Ok(0) => {
                log::warn!("[Twitch IRC] Connection closed");
                break;
            }
            Ok(_) => {
                let line_trimmed = line.trim();
                if !line_trimmed.is_empty() {
                    handle_irc_message(&ctx, line_trimmed, &writer).await;
                }
            }
            Err(e) => {
                log::error!("[Twitch IRC] Read error: {}", e);
                break;
            }
        }
    }

    // Clear client on disconnect
    {
        let mut client = IRC_CLIENT.write().await;
        *client = None;
    }

    log::info!("[Twitch IRC] Disconnected");
    Ok(())
}

async fn handle_irc_message(
    ctx: &PluginContext,
    message: &str,
    writer: &Arc<RwLock<tokio::io::WriteHalf<TcpStream>>>,
) {
    // Handle PING
    if message.starts_with("PING") {
        if let Some(server) = message.strip_prefix("PING ") {
            let mut w = writer.write().await;
            let _ = w.write_all(format!("PONG {}\r\n", server).as_bytes()).await;
            let _ = w.flush().await;
            return;
        }
    }

    // Parse PRIVMSG
    if message.contains("PRIVMSG") {
        if let Some(parsed) = parse_privmsg(message) {
            // Save to database
            if let Err(e) = save_chat_message(ctx, &parsed).await {
                log::error!("[Twitch IRC] Failed to save message: {}", e);
            }

            // Emit event for other plugins
            ctx.emit("chat-message", &serde_json::json!({
                "channel": parsed.channel,
                "username": parsed.username,
                "message": parsed.message,
                "tags": parsed.tags
            }));

            log::debug!("[Twitch IRC] {} in #{}: {}", parsed.username, parsed.channel, parsed.message);
        }
    }
}

#[derive(Debug)]
struct ParsedMessage {
    channel: String,
    username: String,
    message: String,
    tags: std::collections::HashMap<String, String>,
}

fn parse_privmsg(line: &str) -> Option<ParsedMessage> {
    let mut tags = std::collections::HashMap::new();

    // Parse tags if present
    let line = if let Some(stripped) = line.strip_prefix('@') {
        if let Some(pos) = stripped.find(' ') {
            let tags_str = &stripped[..pos];
            for tag in tags_str.split(';') {
                if let Some(eq_pos) = tag.find('=') {
                    let key = &tag[..eq_pos];
                    let value = &tag[eq_pos + 1..];
                    tags.insert(key.to_string(), value.to_string());
                }
            }
            &stripped[pos + 1..]
        } else {
            stripped
        }
    } else {
        line
    };

    // Parse the rest: :user!user@user.tmi.twitch.tv PRIVMSG #channel :message
    let parts: Vec<&str> = line.splitn(4, ' ').collect();
    if parts.len() < 4 {
        return None;
    }

    let user_part = parts[0].strip_prefix(':')?;
    let username = user_part.split('!').next()?.to_string();
    let channel = parts[2].strip_prefix('#')?.to_string();
    let message = parts[3].strip_prefix(':')?.to_string();

    Some(ParsedMessage {
        channel,
        username,
        message,
        tags,
    })
}

async fn save_chat_message(ctx: &PluginContext, msg: &ParsedMessage) -> Result<()> {
    let conn = ctx.db()?;

    let timestamp = current_timestamp();
    let is_action = if msg.message.starts_with("\u{0001}ACTION") { 1 } else { 0 };
    let color = msg.tags.get("color").map(|s| s.as_str()).unwrap_or("");
    let display_name = msg.tags.get("display-name").map(|s| s.as_str()).unwrap_or(&msg.username);
    let user_id = msg.tags.get("user-id").map(|s| s.as_str()).unwrap_or("");
    let badges = msg.tags.get("badges").map(|s| s.as_str()).unwrap_or("");
    let emotes = msg.tags.get("emotes").map(|s| s.as_str()).unwrap_or("");

    conn.execute(
        "INSERT INTO twitch_irc_messages
         (channel, username, user_id, message, timestamp, is_action, badges, color, display_name, emotes)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        rusqlite::params![
            &msg.channel,
            &msg.username,
            user_id,
            &msg.message,
            timestamp,
            is_action,
            badges,
            color,
            display_name,
            emotes
        ]
    )?;

    Ok(())
}

pub async fn send_message(ctx: &PluginContext, channel: &str, message: &str) -> Result<()> {
    let client = IRC_CLIENT.read().await;

    if let Some(irc_client) = client.as_ref() {
        let mut writer = irc_client.writer.write().await;
        writer.write_all(format!("PRIVMSG #{} :{}\r\n", channel, message).as_bytes()).await?;
        writer.flush().await?;

        log::info!("[Twitch IRC] Sent message to #{}: {}", channel, message);
        Ok(())
    } else {
        Err(anyhow!("IRC client not connected"))
    }
}

pub async fn get_irc_status() -> Result<Value> {
    let client = IRC_CLIENT.read().await;

    if let Some(irc_client) = client.as_ref() {
        Ok(serde_json::json!({
            "connected": irc_client.connected,
            "channel": irc_client.channel
        }))
    } else {
        Ok(serde_json::json!({
            "connected": false,
            "channel": null
        }))
    }
}

fn current_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}
