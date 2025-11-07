// EventSub Webhook Server
// Handles incoming webhook callbacks from Twitch EventSub

use super::eventsub::{EventSubManager, EventSubWebhookPayload};
use hyper::{Request, Response, StatusCode, Method};
use hyper::body::Bytes;
use http_body_util::{BodyExt, Full};
use std::sync::Arc;

pub async fn handle_eventsub_webhook_simple(
    req: Request<hyper::body::Incoming>,
    eventsub_manager: Arc<EventSubManager>,
) -> Result<Response<Full<Bytes>>, hyper::Error> {
    log::info!("[Twitch EventSub] 📨 Webhook request received");

    // Only accept POST requests
    if req.method() != Method::POST {
        log::warn!("[Twitch EventSub] Method not allowed: {}", req.method());
        return Ok(Response::builder()
            .status(StatusCode::METHOD_NOT_ALLOWED)
            .body(Full::new(Bytes::from("Method not allowed")))
            .unwrap());
    }

    // Extract headers (clone the values so we can consume req later)
    let message_id = req.headers()
        .get("twitch-eventsub-message-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_default();

    let message_type = req.headers()
        .get("twitch-eventsub-message-type")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_default();

    let timestamp = req.headers()
        .get("twitch-eventsub-message-timestamp")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_default();

    let signature = req.headers()
        .get("twitch-eventsub-message-signature")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_default();

    log::info!("[Twitch EventSub] Message type: {}, ID: {}", message_type, message_id);

    // Read body
    let body_bytes = match req.collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(e) => {
            log::error!("[Twitch EventSub] Failed to read request body: {}", e);
            return Ok(Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(Full::new(Bytes::from("Failed to read body")))
                .unwrap());
        }
    };

    let body_str = match std::str::from_utf8(&body_bytes) {
        Ok(s) => s,
        Err(e) => {
            log::error!("[Twitch EventSub] ❌ Invalid UTF-8 in body: {}", e);
            return Ok(Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(Full::new(Bytes::from("Invalid UTF-8")))
                .unwrap());
        }
    };

    log::debug!("[Twitch EventSub] Body received (first 200 chars): {}", &body_str.chars().take(200).collect::<String>());

    // Verify signature
    if !eventsub_manager.verify_signature(body_str, &signature, &message_id, &timestamp) {
        log::warn!("[Twitch EventSub] ❌ Invalid signature for message {}", message_id);
        log::warn!("[Twitch EventSub] Signature: {}", signature);
        log::warn!("[Twitch EventSub] Timestamp: {}", timestamp);
        return Ok(Response::builder()
            .status(StatusCode::FORBIDDEN)
            .body(Full::new(Bytes::from("Invalid signature")))
            .unwrap());
    }

    log::info!("[Twitch EventSub] ✅ Signature verified");

    // Parse payload
    let payload: EventSubWebhookPayload = match serde_json::from_str(body_str) {
        Ok(p) => p,
        Err(e) => {
            log::error!("[Twitch EventSub] ❌ Failed to parse webhook payload: {}", e);
            log::error!("[Twitch EventSub] Body was: {}", body_str);
            return Ok(Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(Full::new(Bytes::from("Invalid JSON")))
                .unwrap());
        }
    };

    log::info!("[Twitch EventSub] ✅ Payload parsed successfully");

    match message_type.as_str() {
        "webhook_callback_verification" => {
            // Respond with challenge to verify webhook
            if let Some(challenge) = payload.challenge {
                log::info!("[Twitch EventSub] 🔐 Webhook verification challenge received");
                log::info!("[Twitch EventSub] 📤 Responding with challenge: {}", challenge);
                return Ok(Response::builder()
                    .status(StatusCode::OK)
                    .header("Content-Type", "text/plain")
                    .body(Full::new(Bytes::from(challenge)))
                    .unwrap());
            } else {
                log::error!("[Twitch EventSub] Challenge missing from verification request");
                return Ok(Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .body(Full::new(Bytes::from("Challenge missing")))
                    .unwrap());
            }
        }
        "notification" => {
            // Handle actual event
            let event_type = &payload.subscription.subscription_type;
            log::info!("[Twitch EventSub] Received {} event", event_type);

            // Log the event (without emitting for now - that requires PluginContext)
            if let Some(event_data) = &payload.event {
                handle_event_notification_simple(event_type, event_data).await;
            } else {
                log::error!("[Twitch EventSub] Notification missing event data");
            }

            return Ok(Response::builder()
                .status(StatusCode::NO_CONTENT)
                .body(Full::new(Bytes::new()))
                .unwrap());
        }
        "revocation" => {
            // Subscription was revoked
            log::warn!("[Twitch EventSub] Subscription {} was revoked", payload.subscription.id);
            return Ok(Response::builder()
                .status(StatusCode::NO_CONTENT)
                .body(Full::new(Bytes::new()))
                .unwrap());
        }
        _ => {
            log::warn!("[Twitch EventSub] Unknown message type: {}", message_type);
            return Ok(Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(Full::new(Bytes::from("Unknown message type")))
                .unwrap());
        }
    }
}

async fn handle_event_notification_simple(event_type: &str, event_data: &serde_json::Value) {
    match event_type {
        "channel.follow" => {
            // Extract follow event data
            if let (Some(user_id), Some(user_name)) = (
                event_data.get("user_id").and_then(|v| v.as_str()),
                event_data.get("user_name").and_then(|v| v.as_str()),
            ) {
                let followed_at = event_data.get("followed_at")
                    .and_then(|v| v.as_str())
                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                    .map(|dt| dt.timestamp())
                    .unwrap_or_else(|| chrono::Utc::now().timestamp());

                log::info!("[Twitch EventSub] New follower: {} ({})", user_name, user_id);

                // Queue event for plugin system to process
                let conn = crate::core::database::get_database_path();
                if let Ok(conn) = rusqlite::Connection::open(&conn) {
                    let now = chrono::Utc::now().timestamp();
                    let event_payload = serde_json::json!({
                        "user_id": user_id,
                        "username": user_name,
                        "followed_at": followed_at
                    });

                    if let Err(e) = conn.execute(
                        "INSERT INTO twitch_events (event_type, event_data, user_id, username, timestamp)
                         VALUES (?1, ?2, ?3, ?4, ?5)",
                        rusqlite::params!["twitch.follow", event_payload.to_string(), user_id, user_name, now],
                    ) {
                        log::error!("[Twitch EventSub] Failed to queue follow event: {}", e);
                    } else {
                        log::info!("[Twitch EventSub] Queued follow event for {}", user_name);
                    }
                }
            }
        }
        _ => {
            // Queue all other event types
            log::info!("[Twitch EventSub] Received {} event", event_type);

            let conn = crate::core::database::get_database_path();
            if let Ok(conn) = rusqlite::Connection::open(&conn) {
                let now = chrono::Utc::now().timestamp();
                let event_name = format!("twitch.{}", event_type.replace('.', "_"));

                // Extract user info if available
                let user_id = event_data.get("user_id")
                    .or_else(|| event_data.get("from_broadcaster_user_id"))
                    .or_else(|| event_data.get("broadcaster_user_id"))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                let username = event_data.get("user_name")
                    .or_else(|| event_data.get("from_broadcaster_user_name"))
                    .or_else(|| event_data.get("broadcaster_user_name"))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                if let Err(e) = conn.execute(
                    "INSERT INTO twitch_events (event_type, event_data, user_id, username, timestamp)
                     VALUES (?1, ?2, ?3, ?4, ?5)",
                    rusqlite::params![
                        event_name,
                        event_data.to_string(),
                        user_id,
                        username,
                        now
                    ],
                ) {
                    log::error!("[Twitch EventSub] Failed to queue {} event: {}", event_type, e);
                } else {
                    log::info!("[Twitch EventSub] Queued {} event", event_type);
                }
            }
        }
    }
}
