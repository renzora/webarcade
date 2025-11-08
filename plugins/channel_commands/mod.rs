use crate::core::plugin::{Plugin, PluginMetadata};
use crate::core::plugin_context::PluginContext;
use crate::plugins::twitch::TwitchApiClient;
use async_trait::async_trait;
use std::sync::Arc;
use anyhow::Result;

pub struct ChannelCommandsPlugin;

#[async_trait]
impl Plugin for ChannelCommandsPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            id: "channel_commands".to_string(),
            name: "Channel Commands".to_string(),
            version: "1.0.0".to_string(),
            description: "Built-in commands for managing channel information (title, category)".to_string(),
            author: "WebArcade Team".to_string(),
            dependencies: vec!["twitch".to_string()],
        }
    }

    async fn init(&self, _ctx: &PluginContext) -> Result<()> {
        log::info!("[ChannelCommands] Initializing plugin...");
        log::info!("[ChannelCommands] Plugin initialized successfully");
        Ok(())
    }

    async fn start(&self, ctx: Arc<PluginContext>) -> Result<()> {
        log::info!("[ChannelCommands] Starting plugin...");

        // Subscribe to chat messages to handle channel commands
        tokio::spawn(async move {
            let mut events = ctx.subscribe_to("twitch.chat_message").await;

            while let Ok(event) = events.recv().await {
                let channel = match serde_json::from_value::<String>(event.payload["channel"].clone()) {
                    Ok(c) => c,
                    Err(_) => continue,
                };

                let message = match serde_json::from_value::<String>(event.payload["message"].clone()) {
                    Ok(m) => m,
                    Err(_) => continue,
                };

                let username = match serde_json::from_value::<String>(event.payload["username"].clone()) {
                    Ok(u) => u,
                    Err(_) => continue,
                };

                let user_id = match serde_json::from_value::<String>(event.payload["user_id"].clone()) {
                    Ok(u) => u,
                    Err(_) => continue,
                };

                // Extract permission level from badges
                let is_broadcaster = event.payload["is_broadcaster"].as_bool().unwrap_or(false);
                let is_mod = event.payload["is_mod"].as_bool().unwrap_or(false);

                // Only handle commands from broadcaster or moderators
                if !is_broadcaster && !is_mod {
                    continue;
                }

                // Handle !settitle command
                if message.starts_with("!settitle ") {
                    let new_title = message.strip_prefix("!settitle ").unwrap_or("").trim();

                    if new_title.is_empty() {
                        ctx.emit("twitch.send_message", &serde_json::json!({
                            "channel": channel,
                            "message": format!("@{} Usage: !settitle <new title>", username)
                        }));
                        continue;
                    }

                    log::info!("[ChannelCommands] {} is setting title to: {}", username, new_title);

                    // Get broadcaster ID from database
                    let broadcaster_id = match get_broadcaster_id(&channel).await {
                        Ok(id) => id,
                        Err(e) => {
                            log::error!("[ChannelCommands] Failed to get broadcaster ID: {}", e);
                            ctx.emit("twitch.send_message", &serde_json::json!({
                                "channel": channel,
                                "message": format!("@{} Failed to get broadcaster info", username)
                            }));
                            continue;
                        }
                    };

                    // Get Twitch API credentials
                    let (access_token, client_id) = match get_twitch_credentials().await {
                        Ok(creds) => creds,
                        Err(e) => {
                            log::error!("[ChannelCommands] Failed to get Twitch credentials: {}", e);
                            ctx.emit("twitch.send_message", &serde_json::json!({
                                "channel": channel,
                                "message": format!("@{} Failed to get API credentials", username)
                            }));
                            continue;
                        }
                    };

                    // Create API client and update title
                    let api_client = TwitchApiClient::new(client_id, access_token);
                    match api_client.modify_channel_info(&broadcaster_id, Some(new_title), None).await {
                        Ok(_) => {
                            log::info!("[ChannelCommands] Successfully updated title");
                            ctx.emit("twitch.send_message", &serde_json::json!({
                                "channel": channel,
                                "message": format!("@{} Title updated successfully!", username)
                            }));
                        }
                        Err(e) => {
                            log::error!("[ChannelCommands] Failed to update title: {}", e);
                            ctx.emit("twitch.send_message", &serde_json::json!({
                                "channel": channel,
                                "message": format!("@{} Failed to update title: {}", username, e)
                            }));
                        }
                    }
                }
                // Handle !setcategory command
                else if message.starts_with("!setcategory ") || message.starts_with("!setgame ") {
                    let prefix = if message.starts_with("!setcategory ") {
                        "!setcategory "
                    } else {
                        "!setgame "
                    };

                    let category_name = message.strip_prefix(prefix).unwrap_or("").trim();

                    if category_name.is_empty() {
                        ctx.emit("twitch.send_message", &serde_json::json!({
                            "channel": channel,
                            "message": format!("@{} Usage: !setcategory <game/category name>", username)
                        }));
                        continue;
                    }

                    log::info!("[ChannelCommands] {} is setting category to: {}", username, category_name);

                    // Get broadcaster ID from database
                    let broadcaster_id = match get_broadcaster_id(&channel).await {
                        Ok(id) => id,
                        Err(e) => {
                            log::error!("[ChannelCommands] Failed to get broadcaster ID: {}", e);
                            ctx.emit("twitch.send_message", &serde_json::json!({
                                "channel": channel,
                                "message": format!("@{} Failed to get broadcaster info", username)
                            }));
                            continue;
                        }
                    };

                    // Get Twitch API credentials
                    let (access_token, client_id) = match get_twitch_credentials().await {
                        Ok(creds) => creds,
                        Err(e) => {
                            log::error!("[ChannelCommands] Failed to get Twitch credentials: {}", e);
                            ctx.emit("twitch.send_message", &serde_json::json!({
                                "channel": channel,
                                "message": format!("@{} Failed to get API credentials", username)
                            }));
                            continue;
                        }
                    };

                    // Create API client and search for category
                    let api_client = TwitchApiClient::new(client_id, access_token);

                    // Search for the category
                    let category_id = match api_client.search_categories(category_name).await {
                        Ok(result) => {
                            if let Some(data) = result.get("data").and_then(|d| d.as_array()) {
                                if let Some(first_match) = data.first() {
                                    if let Some(id) = first_match.get("id").and_then(|i| i.as_str()) {
                                        id.to_string()
                                    } else {
                                        log::error!("[ChannelCommands] No ID found in category result");
                                        ctx.emit("twitch.send_message", &serde_json::json!({
                                            "channel": channel,
                                            "message": format!("@{} Category '{}' not found", username, category_name)
                                        }));
                                        continue;
                                    }
                                } else {
                                    log::error!("[ChannelCommands] No categories found matching '{}'", category_name);
                                    ctx.emit("twitch.send_message", &serde_json::json!({
                                        "channel": channel,
                                        "message": format!("@{} Category '{}' not found", username, category_name)
                                    }));
                                    continue;
                                }
                            } else {
                                log::error!("[ChannelCommands] Invalid response from search_categories");
                                ctx.emit("twitch.send_message", &serde_json::json!({
                                    "channel": channel,
                                    "message": format!("@{} Failed to search for category", username)
                                }));
                                continue;
                            }
                        }
                        Err(e) => {
                            log::error!("[ChannelCommands] Failed to search categories: {}", e);
                            ctx.emit("twitch.send_message", &serde_json::json!({
                                "channel": channel,
                                "message": format!("@{} Failed to search for category: {}", username, e)
                            }));
                            continue;
                        }
                    };

                    // Update the category
                    match api_client.modify_channel_info(&broadcaster_id, None, Some(&category_id)).await {
                        Ok(_) => {
                            log::info!("[ChannelCommands] Successfully updated category");
                            ctx.emit("twitch.send_message", &serde_json::json!({
                                "channel": channel,
                                "message": format!("@{} Category updated to '{}'!", username, category_name)
                            }));
                        }
                        Err(e) => {
                            log::error!("[ChannelCommands] Failed to update category: {}", e);
                            ctx.emit("twitch.send_message", &serde_json::json!({
                                "channel": channel,
                                "message": format!("@{} Failed to update category: {}", username, e)
                            }));
                        }
                    }
                }
            }
        });

        log::info!("[ChannelCommands] Plugin started successfully");
        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        log::info!("[ChannelCommands] Stopping plugin...");
        Ok(())
    }
}

// Helper function to get broadcaster ID from database
async fn get_broadcaster_id(channel_name: &str) -> Result<String> {
    let conn = crate::core::database::get_database_path();
    let conn = rusqlite::Connection::open(&conn)?;

    let broadcaster_id: String = conn.query_row(
        "SELECT channel_id FROM twitch_channels WHERE channel_name = ?1",
        rusqlite::params![channel_name],
        |row| row.get(0),
    ).or_else(|_| {
        // If not found in channels table, try to get from auth table
        conn.query_row(
            "SELECT user_id FROM twitch_auth LIMIT 1",
            [],
            |row| row.get(0),
        )
    })?;

    Ok(broadcaster_id)
}

// Helper function to get Twitch API credentials
async fn get_twitch_credentials() -> Result<(String, String)> {
    let conn = crate::core::database::get_database_path();
    let conn = rusqlite::Connection::open(&conn)?;

    let access_token: String = conn.query_row(
        "SELECT access_token FROM twitch_auth LIMIT 1",
        [],
        |row| row.get(0),
    )?;

    let client_id: String = conn.query_row(
        "SELECT value FROM twitch_config WHERE key = 'client_id'",
        [],
        |row| row.get(0),
    )?;

    Ok((access_token, client_id))
}
