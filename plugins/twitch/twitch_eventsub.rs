use anyhow::{Result, anyhow};
use std::sync::Arc;
use serde_json::Value;
use crate::core::plugin_context::PluginContext;
use futures_util::StreamExt;

use super::twitch_api;

// Twitch EventSub WebSocket endpoint
const EVENTSUB_WS_URL: &str = "wss://eventsub.wss.twitch.tv/ws";

pub async fn start_eventsub_listener(ctx: Arc<PluginContext>) -> Result<()> {
    log::info!("[Twitch EventSub] Starting EventSub listener...");

    // Get broadcaster token
    let token_result = twitch_api::get_account_token(&ctx, "broadcaster").await;
    if token_result.is_err() {
        log::warn!("[Twitch EventSub] No broadcaster account configured, EventSub disabled");
        return Ok(());
    }

    let token_data = token_result?;
    let access_token = token_data["access_token"]
        .as_str()
        .ok_or_else(|| anyhow!("Missing access token"))?;

    // Connect to EventSub WebSocket
    log::info!("[Twitch EventSub] Connecting to EventSub WebSocket...");

    match tokio_tungstenite::connect_async(EVENTSUB_WS_URL).await {
        Ok((ws_stream, _)) => {
            log::info!("[Twitch EventSub] Connected to EventSub WebSocket");

            let (_write, mut read) = ws_stream.split();

            // Handle incoming messages
            while let Some(message) = futures_util::StreamExt::next(&mut read).await {
                match message {
                    Ok(msg) => {
                        // Only process text messages, ignore ping/pong/binary
                        if msg.is_text() {
                            if let Ok(text) = msg.to_text() {
                                // Skip empty messages
                                if text.trim().is_empty() {
                                    log::debug!("[Twitch EventSub] Received empty message, skipping");
                                    continue;
                                }

                                log::debug!("[Twitch EventSub] Received message: {}", text);

                                if let Err(e) = handle_eventsub_message(&ctx, text, access_token).await {
                                    log::error!("[Twitch EventSub] Error handling message: {}", e);
                                }
                            }
                        } else if msg.is_ping() {
                            log::debug!("[Twitch EventSub] Received ping");
                        } else if msg.is_pong() {
                            log::debug!("[Twitch EventSub] Received pong");
                        } else if msg.is_close() {
                            log::info!("[Twitch EventSub] Received close frame");
                            break;
                        }
                    }
                    Err(e) => {
                        log::error!("[Twitch EventSub] WebSocket error: {}", e);
                        break;
                    }
                }
            }

            log::info!("[Twitch EventSub] WebSocket connection closed");
        }
        Err(e) => {
            log::error!("[Twitch EventSub] Failed to connect to WebSocket: {}", e);
        }
    }

    Ok(())
}

async fn handle_eventsub_message(ctx: &PluginContext, message: &str, _access_token: &str) -> Result<()> {
    let data: Value = serde_json::from_str(message)?;

    let message_type = data["metadata"]["message_type"]
        .as_str()
        .unwrap_or("unknown");

    match message_type {
        "session_welcome" => {
            log::info!("[Twitch EventSub] Received session welcome");
            let session_id = data["payload"]["session"]["id"]
                .as_str()
                .ok_or_else(|| anyhow!("Missing session ID"))?;

            log::info!("[Twitch EventSub] Session ID: {}", session_id);

            // Here you would subscribe to specific events
            // For now, we'll just log the session
        }
        "session_keepalive" => {
            log::debug!("[Twitch EventSub] Keepalive received");
        }
        "notification" => {
            log::info!("[Twitch EventSub] Received notification");

            let subscription_type = data["payload"]["subscription"]["type"]
                .as_str()
                .unwrap_or("unknown");
            let event_data = &data["payload"]["event"];

            // Save event to database
            save_eventsub_event(ctx, subscription_type, event_data).await?;

            // Emit event for other plugins
            ctx.emit("eventsub-event", &serde_json::json!({
                "type": subscription_type,
                "data": event_data
            }));

            log::info!("[Twitch EventSub] Event: {} - {:?}", subscription_type, event_data);
        }
        "session_reconnect" => {
            log::warn!("[Twitch EventSub] Session reconnect requested");
            // Handle reconnection
        }
        "revocation" => {
            log::warn!("[Twitch EventSub] Subscription revoked");
            let subscription_type = data["payload"]["subscription"]["type"]
                .as_str()
                .unwrap_or("unknown");
            log::warn!("[Twitch EventSub] Revoked subscription type: {}", subscription_type);
        }
        _ => {
            log::warn!("[Twitch EventSub] Unknown message type: {}", message_type);
        }
    }

    Ok(())
}

async fn save_eventsub_event(ctx: &PluginContext, event_type: &str, event_data: &Value) -> Result<()> {
    let conn = ctx.db()?;

    let timestamp = current_timestamp();
    let event_data_str = event_data.to_string();

    conn.execute(
        "INSERT INTO twitch_events (event_type, event_data, timestamp)
         VALUES (?1, ?2, ?3)",
        rusqlite::params![event_type, event_data_str, timestamp]
    )?;

    Ok(())
}

pub async fn create_eventsub_subscription(
    ctx: &PluginContext,
    subscription_type: &str,
    session_id: &str,
    condition: &Value,
) -> Result<Value> {
    let token_data = twitch_api::get_account_token(ctx, "broadcaster").await?;
    let access_token = token_data["access_token"]
        .as_str()
        .ok_or_else(|| anyhow!("Missing access token"))?;

    let client = reqwest::Client::new();

    let body = serde_json::json!({
        "type": subscription_type,
        "version": "1",
        "condition": condition,
        "transport": {
            "method": "websocket",
            "session_id": session_id
        }
    });

    let response = client
        .post("https://api.twitch.tv/helix/eventsub/subscriptions")
        .header("Authorization", format!("Bearer {}", access_token))
        .header("Client-Id", twitch_api::get_client_id_public()?)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await?;

    if !response.status().is_success() {
        let error_text = response.text().await?;
        return Err(anyhow!("Failed to create subscription: {}", error_text));
    }

    let result: Value = response.json().await?;

    // Save subscription to database
    if let Some(data) = result["data"].get(0) {
        let subscription_id = data["id"].as_str().unwrap_or("");
        let status = data["status"].as_str().unwrap_or("unknown");
        let cost = data["cost"].as_i64().unwrap_or(0);

        let conn = ctx.db()?;
        let timestamp = current_timestamp();
        let condition_str = condition.to_string();

        conn.execute(
            "INSERT INTO twitch_eventsub_subscriptions
             (subscription_id, subscription_type, version, status, created_at, cost, condition)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![
                subscription_id,
                subscription_type,
                "1",
                status,
                timestamp,
                cost,
                condition_str
            ]
        )?;
    }

    Ok(result)
}

pub async fn delete_eventsub_subscription(ctx: &PluginContext, subscription_id: &str) -> Result<()> {
    let token_data = twitch_api::get_account_token(ctx, "broadcaster").await?;
    let access_token = token_data["access_token"]
        .as_str()
        .ok_or_else(|| anyhow!("Missing access token"))?;

    let client = reqwest::Client::new();

    let response = client
        .delete(format!(
            "https://api.twitch.tv/helix/eventsub/subscriptions?id={}",
            subscription_id
        ))
        .header("Authorization", format!("Bearer {}", access_token))
        .header("Client-Id", twitch_api::get_client_id_public()?)
        .send()
        .await?;

    if !response.status().is_success() {
        let error_text = response.text().await?;
        return Err(anyhow!("Failed to delete subscription: {}", error_text));
    }

    // Remove from database
    let conn = ctx.db()?;
    conn.execute(
        "DELETE FROM twitch_eventsub_subscriptions WHERE subscription_id = ?1",
        [subscription_id]
    )?;

    Ok(())
}

fn current_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}
