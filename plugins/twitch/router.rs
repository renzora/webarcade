use crate::core::plugin_context::PluginContext;
use crate::core::plugin_router::PluginRouter;
use crate::core::router_utils::*;
use crate::route;
use hyper::{Request, Response, StatusCode, body::Incoming};
use hyper::body::Bytes;
use http_body_util::combinators::BoxBody;
use std::convert::Infallible;
use anyhow::Result;

use super::twitch_api;
use super::twitch_irc;

pub async fn register_routes(ctx: &PluginContext) -> Result<()> {
    let mut router = PluginRouter::new();

    // Account management routes
    route!(router, GET "/accounts" => handle_get_accounts);
    route!(router, GET "/auth/start", query => handle_auth_start);
    route!(router, GET "/auth/callback", query => handle_auth_callback);
    route!(router, POST "/auth/refresh" => handle_auth_refresh_post);
    route!(router, DELETE "/accounts/:type", path => handle_delete_account);

    // IRC routes
    route!(router, GET "/irc/status" => handle_irc_status);
    route!(router, POST "/irc/send" => handle_irc_send);
    route!(router, GET "/irc/messages" => handle_get_messages);
    route!(router, POST "/irc/connect" => handle_irc_connect_post);
    route!(router, POST "/irc/disconnect" => handle_irc_disconnect_post);

    // EventSub routes
    route!(router, GET "/eventsub/subscriptions" => handle_get_subscriptions);
    route!(router, POST "/eventsub/subscribe" => handle_create_subscription);
    route!(router, DELETE "/eventsub/subscribe/:id", path => handle_delete_subscription);
    route!(router, GET "/eventsub/events" => handle_get_events);

    // Channel info routes
    route!(router, GET "/channel/info", query => handle_get_channel_info);
    route!(router, GET "/user/info", query => handle_get_user_info);

    // Settings routes
    route!(router, GET "/settings" => handle_get_settings);
    route!(router, POST "/settings" => handle_update_settings);

    // Setup routes
    route!(router, GET "/setup/status" => handle_setup_status);
    route!(router, POST "/setup" => handle_setup);

    // Test routes
    route!(router, POST "/test/irc" => handle_test_irc);
    route!(router, POST "/test/eventsub" => handle_test_eventsub);

    // CORS preflight
    route!(router, OPTIONS "/auth/start" => cors_preflight);
    route!(router, OPTIONS "/auth/callback" => cors_preflight);
    route!(router, OPTIONS "/irc/send" => cors_preflight);
    route!(router, OPTIONS "/eventsub/subscribe" => cors_preflight);
    route!(router, OPTIONS "/settings" => cors_preflight);

    ctx.register_router("twitch", router).await;
    Ok(())
}

// Account management handlers
async fn handle_get_accounts() -> Response<BoxBody<Bytes, Infallible>> {
    let db_path = crate::core::database::get_database_path();
    let conn = match rusqlite::Connection::open(&db_path) {
        Ok(c) => c,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
    };

    let mut stmt = match conn.prepare(
        "SELECT account_type, username, user_id, scopes, created_at, updated_at FROM twitch_accounts"
    ) {
        Ok(s) => s,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
    };

    let accounts: Vec<serde_json::Value> = stmt.query_map([], |row| {
        Ok(serde_json::json!({
            "account_type": row.get::<_, String>(0)?,
            "username": row.get::<_, Option<String>>(1)?,
            "user_id": row.get::<_, Option<String>>(2)?,
            "scopes": row.get::<_, Option<String>>(3)?,
            "created_at": row.get::<_, i64>(4)?,
            "updated_at": row.get::<_, i64>(5)?
        }))
    }).unwrap()
      .collect::<Result<Vec<_>, _>>()
      .unwrap_or_default();

    json_response(&accounts)
}

async fn handle_auth_start(query: String) -> Response<BoxBody<Bytes, Infallible>> {
    let account_type = match parse_query_param(&query, "type") {
        Some(t) if t == "broadcaster" || t == "bot" => t,
        _ => return error_response(StatusCode::BAD_REQUEST, "Invalid account_type. Must be 'broadcaster' or 'bot'")
    };

    // Generate OAuth URL for Twitch
    match twitch_api::generate_auth_url(&account_type) {
        Ok(auth_url) => json_response(&serde_json::json!({
            "auth_url": auth_url,
            "account_type": account_type
        })),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
    }
}

async fn handle_auth_callback(query: String) -> Response<BoxBody<Bytes, Infallible>> {
    let code = match parse_query_param(&query, "code") {
        Some(c) => c,
        None => return error_response(StatusCode::BAD_REQUEST, "Missing 'code' parameter")
    };

    let account_type = match parse_query_param(&query, "state") {
        Some(t) if t == "broadcaster" || t == "bot" => t,
        _ => return error_response(StatusCode::BAD_REQUEST, "Invalid state parameter")
    };

    // Exchange code for tokens
    match twitch_api::exchange_code_for_token(&code, &account_type).await {
        Ok(account_data) => json_response(&account_data),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
    }
}

async fn handle_auth_refresh_post(req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    match twitch_api::refresh_all_tokens().await {
        Ok(result) => json_response(&result),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
    }
}

async fn handle_delete_account(path: String) -> Response<BoxBody<Bytes, Infallible>> {
    let account_type = path.trim_start_matches("/accounts/");

    if account_type != "broadcaster" && account_type != "bot" {
        return error_response(StatusCode::BAD_REQUEST, "Invalid account_type");
    }

    let db_path = crate::core::database::get_database_path();
    let conn = match rusqlite::Connection::open(&db_path) {
        Ok(c) => c,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
    };

    match conn.execute(
        "DELETE FROM twitch_accounts WHERE account_type = ?1",
        [account_type]
    ) {
        Ok(_) => json_response(&serde_json::json!({
            "success": true,
            "account_type": account_type
        })),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
    }
}

// IRC handlers
async fn handle_irc_status() -> Response<BoxBody<Bytes, Infallible>> {
    match twitch_irc::get_irc_status().await {
        Ok(status) => json_response(&status),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
    }
}

async fn handle_irc_send(req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    let body = match read_json_body(req).await {
        Ok(b) => b,
        Err(e) => return error_response(StatusCode::BAD_REQUEST, &e)
    };

    let channel = body.get("channel").and_then(|v| v.as_str()).unwrap_or("");
    let message = body.get("message").and_then(|v| v.as_str()).unwrap_or("");

    if channel.is_empty() || message.is_empty() {
        return error_response(StatusCode::BAD_REQUEST, "Missing channel or message");
    }

    // This will be implemented in twitch_irc module
    json_response(&serde_json::json!({
        "success": true,
        "message": "Message queued for sending"
    }))
}

async fn handle_get_messages() -> Response<BoxBody<Bytes, Infallible>> {
    let db_path = crate::core::database::get_database_path();
    let conn = match rusqlite::Connection::open(&db_path) {
        Ok(c) => c,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
    };

    let mut stmt = match conn.prepare(
        "SELECT id, channel, username, message, timestamp, is_action, color, display_name
         FROM twitch_irc_messages
         ORDER BY timestamp DESC
         LIMIT 100"
    ) {
        Ok(s) => s,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
    };

    let messages: Vec<serde_json::Value> = stmt.query_map([], |row| {
        Ok(serde_json::json!({
            "id": row.get::<_, i64>(0)?,
            "channel": row.get::<_, String>(1)?,
            "username": row.get::<_, String>(2)?,
            "message": row.get::<_, String>(3)?,
            "timestamp": row.get::<_, i64>(4)?,
            "is_action": row.get::<_, i64>(5)? == 1,
            "color": row.get::<_, Option<String>>(6)?,
            "display_name": row.get::<_, Option<String>>(7)?
        }))
    }).unwrap()
      .collect::<Result<Vec<_>, _>>()
      .unwrap_or_default();

    json_response(&messages)
}

async fn handle_irc_connect_post(req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    json_response(&serde_json::json!({
        "success": true,
        "message": "IRC connection initiated"
    }))
}

async fn handle_irc_disconnect_post(req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    json_response(&serde_json::json!({
        "success": true,
        "message": "IRC disconnection initiated"
    }))
}

// EventSub handlers
async fn handle_get_subscriptions() -> Response<BoxBody<Bytes, Infallible>> {
    let db_path = crate::core::database::get_database_path();
    let conn = match rusqlite::Connection::open(&db_path) {
        Ok(c) => c,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
    };

    let mut stmt = match conn.prepare(
        "SELECT subscription_id, subscription_type, version, status, created_at, cost
         FROM twitch_eventsub_subscriptions"
    ) {
        Ok(s) => s,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
    };

    let subs: Vec<serde_json::Value> = stmt.query_map([], |row| {
        Ok(serde_json::json!({
            "subscription_id": row.get::<_, Option<String>>(0)?,
            "subscription_type": row.get::<_, String>(1)?,
            "version": row.get::<_, String>(2)?,
            "status": row.get::<_, String>(3)?,
            "created_at": row.get::<_, i64>(4)?,
            "cost": row.get::<_, i64>(5)?
        }))
    }).unwrap()
      .collect::<Result<Vec<_>, _>>()
      .unwrap_or_default();

    json_response(&subs)
}

async fn handle_create_subscription(req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    let body = match read_json_body(req).await {
        Ok(b) => b,
        Err(e) => return error_response(StatusCode::BAD_REQUEST, &e)
    };

    let subscription_type = body.get("type").and_then(|v| v.as_str()).unwrap_or("");

    if subscription_type.is_empty() {
        return error_response(StatusCode::BAD_REQUEST, "Missing subscription type");
    }

    // This will be implemented in twitch_eventsub module
    json_response(&serde_json::json!({
        "success": true,
        "message": "Subscription created"
    }))
}

async fn handle_delete_subscription(path: String) -> Response<BoxBody<Bytes, Infallible>> {
    let subscription_id = path.trim_start_matches("/eventsub/subscribe/");

    json_response(&serde_json::json!({
        "success": true,
        "subscription_id": subscription_id
    }))
}

async fn handle_get_events() -> Response<BoxBody<Bytes, Infallible>> {
    let db_path = crate::core::database::get_database_path();
    let conn = match rusqlite::Connection::open(&db_path) {
        Ok(c) => c,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
    };

    let mut stmt = match conn.prepare(
        "SELECT id, event_type, event_data, timestamp
         FROM twitch_events
         ORDER BY timestamp DESC
         LIMIT 50"
    ) {
        Ok(s) => s,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
    };

    let events: Vec<serde_json::Value> = stmt.query_map([], |row| {
        let event_data_str = row.get::<_, String>(2)?;
        let event_data: serde_json::Value = serde_json::from_str(&event_data_str)
            .unwrap_or(serde_json::json!({}));

        Ok(serde_json::json!({
            "id": row.get::<_, i64>(0)?,
            "event_type": row.get::<_, String>(1)?,
            "event_data": event_data,
            "timestamp": row.get::<_, i64>(3)?
        }))
    }).unwrap()
      .collect::<Result<Vec<_>, _>>()
      .unwrap_or_default();

    json_response(&events)
}

// Channel info handlers
async fn handle_get_channel_info(query: String) -> Response<BoxBody<Bytes, Infallible>> {
    let channel = match parse_query_param(&query, "channel") {
        Some(c) => c,
        None => return error_response(StatusCode::BAD_REQUEST, "Missing 'channel' parameter")
    };

    // Placeholder - will be implemented in twitch_api module
    json_response(&serde_json::json!({
        "channel": channel,
        "status": "offline"
    }))
}

async fn handle_get_user_info(query: String) -> Response<BoxBody<Bytes, Infallible>> {
    let username = match parse_query_param(&query, "username") {
        Some(u) => u,
        None => return error_response(StatusCode::BAD_REQUEST, "Missing 'username' parameter")
    };

    json_response(&serde_json::json!({
        "username": username
    }))
}

// Settings handlers
async fn handle_get_settings() -> Response<BoxBody<Bytes, Infallible>> {
    let db_path = crate::core::database::get_database_path();
    let conn = match rusqlite::Connection::open(&db_path) {
        Ok(c) => c,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
    };

    let mut stmt = match conn.prepare("SELECT key, value FROM twitch_settings") {
        Ok(s) => s,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
    };

    let mut settings = serde_json::Map::new();
    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?
        ))
    }).unwrap();

    for row in rows {
        if let Ok((key, value)) = row {
            if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&value) {
                settings.insert(key, json_value);
            }
        }
    }

    json_response(&serde_json::Value::Object(settings))
}

async fn handle_update_settings(req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    let body = match read_json_body(req).await {
        Ok(b) => b,
        Err(e) => return error_response(StatusCode::BAD_REQUEST, &e)
    };

    let db_path = crate::core::database::get_database_path();
    let conn = match rusqlite::Connection::open(&db_path) {
        Ok(c) => c,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
    };

    let timestamp = current_timestamp();

    if let Some(obj) = body.as_object() {
        for (key, value) in obj {
            let value_str = value.to_string();
            let _ = conn.execute(
                "INSERT OR REPLACE INTO twitch_settings (key, value, updated_at) VALUES (?1, ?2, ?3)",
                rusqlite::params![key, value_str, timestamp]
            );
        }
    }

    json_response(&serde_json::json!({
        "success": true
    }))
}

async fn handle_setup_status() -> Response<BoxBody<Bytes, Infallible>> {
    let db_path = crate::core::database::get_database_path();
    let conn = match rusqlite::Connection::open(&db_path) {
        Ok(c) => c,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
    };

    let has_client_id: bool = conn.query_row(
        "SELECT COUNT(*) FROM twitch_settings WHERE key = 'client_id' AND value != ''",
        [],
        |row| {
            let count: i64 = row.get(0)?;
            Ok(count > 0)
        }
    ).unwrap_or(false);

    let has_client_secret: bool = conn.query_row(
        "SELECT COUNT(*) FROM twitch_settings WHERE key = 'client_secret' AND value != ''",
        [],
        |row| {
            let count: i64 = row.get(0)?;
            Ok(count > 0)
        }
    ).unwrap_or(false);

    let is_configured = has_client_id && has_client_secret;

    json_response(&serde_json::json!({
        "is_configured": is_configured,
        "has_client_id": has_client_id,
        "has_client_secret": has_client_secret
    }))
}

async fn handle_setup(req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    let body = match read_json_body(req).await {
        Ok(b) => b,
        Err(e) => return error_response(StatusCode::BAD_REQUEST, &e)
    };

    let client_id = body.get("client_id").and_then(|v| v.as_str()).unwrap_or("");
    let client_secret = body.get("client_secret").and_then(|v| v.as_str()).unwrap_or("");

    if client_id.is_empty() || client_secret.is_empty() {
        return error_response(StatusCode::BAD_REQUEST, "client_id and client_secret are required");
    }

    let db_path = crate::core::database::get_database_path();
    let conn = match rusqlite::Connection::open(&db_path) {
        Ok(c) => c,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
    };

    let timestamp = current_timestamp();

    // Save client ID
    if let Err(e) = conn.execute(
        "INSERT OR REPLACE INTO twitch_settings (key, value, updated_at) VALUES ('client_id', ?1, ?2)",
        rusqlite::params![client_id, timestamp]
    ) {
        return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string());
    }

    // Save client secret
    if let Err(e) = conn.execute(
        "INSERT OR REPLACE INTO twitch_settings (key, value, updated_at) VALUES ('client_secret', ?1, ?2)",
        rusqlite::params![client_secret, timestamp]
    ) {
        return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string());
    }

    json_response(&serde_json::json!({
        "success": true,
        "message": "Twitch app credentials configured successfully"
    }))
}

// Test handlers
async fn handle_test_irc(req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    let body = match read_json_body(req).await {
        Ok(b) => b,
        Err(e) => return error_response(StatusCode::BAD_REQUEST, &e)
    };

    let test_type = body.get("type").and_then(|v| v.as_str()).unwrap_or("message");

    match test_type {
        "message" => {
            // Test sending a message
            let channel = body.get("channel").and_then(|v| v.as_str()).unwrap_or("webarcade");
            let message = body.get("message")
                .and_then(|v| v.as_str())
                .unwrap_or("Test message from WebArcade Twitch plugin!");

            // Store test message in database
            let db_path = crate::core::database::get_database_path();
            let conn = match rusqlite::Connection::open(&db_path) {
                Ok(c) => c,
                Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
            };

            let timestamp = current_timestamp();

            match conn.execute(
                "INSERT INTO twitch_irc_messages
                 (channel, username, user_id, message, timestamp, is_action, color, display_name)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                rusqlite::params![
                    channel,
                    "test_user",
                    "12345",
                    message,
                    timestamp,
                    0,
                    "#FF0000",
                    "TestUser"
                ]
            ) {
                Ok(_) => json_response(&serde_json::json!({
                    "success": true,
                    "type": "irc_test_message",
                    "message": format!("Test message stored: {}", message)
                })),
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
            }
        }
        "connection" => {
            // Test IRC connection status
            json_response(&serde_json::json!({
                "success": true,
                "type": "irc_connection_test",
                "message": "IRC connection test - check /irc/status endpoint",
                "test_endpoint": "/twitch/irc/status"
            }))
        }
        "bulk_messages" => {
            // Insert multiple test messages
            let db_path = crate::core::database::get_database_path();
            let conn = match rusqlite::Connection::open(&db_path) {
                Ok(c) => c,
                Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
            };

            let test_messages = vec![
                ("user1", "Hello chat!"),
                ("user2", "How's everyone doing?"),
                ("user3", "This is a test message"),
                ("user4", "Testing IRC functionality"),
                ("user5", "WebArcade is awesome!"),
            ];

            let timestamp = current_timestamp();
            let channel = body.get("channel").and_then(|v| v.as_str()).unwrap_or("webarcade");

            for (i, (username, message)) in test_messages.iter().enumerate() {
                let _ = conn.execute(
                    "INSERT INTO twitch_irc_messages
                     (channel, username, user_id, message, timestamp, is_action, color, display_name)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                    rusqlite::params![
                        channel,
                        username,
                        format!("{}", 10000 + i),
                        message,
                        timestamp + i as i64,
                        0,
                        format!("#FF{:02X}00", (i * 50) % 256),
                        username
                    ]
                );
            }

            json_response(&serde_json::json!({
                "success": true,
                "type": "bulk_messages",
                "count": test_messages.len(),
                "message": format!("Inserted {} test messages", test_messages.len())
            }))
        }
        _ => error_response(StatusCode::BAD_REQUEST, "Unknown test type. Use: message, connection, or bulk_messages")
    }
}

async fn handle_test_eventsub(req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    let body = match read_json_body(req).await {
        Ok(b) => b,
        Err(e) => return error_response(StatusCode::BAD_REQUEST, &e)
    };

    let test_type = body.get("type").and_then(|v| v.as_str()).unwrap_or("follow");

    match test_type {
        "follow" => {
            // Simulate a follow event
            let follower_name = body.get("follower_name")
                .and_then(|v| v.as_str())
                .unwrap_or("test_follower");

            let event_data = serde_json::json!({
                "user_id": "12345",
                "user_login": follower_name,
                "user_name": follower_name,
                "broadcaster_user_id": "67890",
                "broadcaster_user_login": "webarcade",
                "broadcaster_user_name": "WebArcade",
                "followed_at": chrono::Utc::now().to_rfc3339()
            });

            save_test_event("channel.follow", &event_data);

            json_response(&serde_json::json!({
                "success": true,
                "type": "follow_event",
                "event_data": event_data,
                "message": format!("Test follow event created for {}", follower_name)
            }))
        }
        "subscribe" => {
            // Simulate a subscription event
            let subscriber_name = body.get("subscriber_name")
                .and_then(|v| v.as_str())
                .unwrap_or("test_subscriber");

            let event_data = serde_json::json!({
                "user_id": "12345",
                "user_login": subscriber_name,
                "user_name": subscriber_name,
                "broadcaster_user_id": "67890",
                "broadcaster_user_login": "webarcade",
                "broadcaster_user_name": "WebArcade",
                "tier": "1000",
                "is_gift": false
            });

            save_test_event("channel.subscribe", &event_data);

            json_response(&serde_json::json!({
                "success": true,
                "type": "subscribe_event",
                "event_data": event_data,
                "message": format!("Test subscribe event created for {}", subscriber_name)
            }))
        }
        "cheer" => {
            // Simulate a cheer event
            let cheerer_name = body.get("cheerer_name")
                .and_then(|v| v.as_str())
                .unwrap_or("test_cheerer");
            let bits = body.get("bits").and_then(|v| v.as_i64()).unwrap_or(100);

            let event_data = serde_json::json!({
                "user_id": "12345",
                "user_login": cheerer_name,
                "user_name": cheerer_name,
                "broadcaster_user_id": "67890",
                "broadcaster_user_login": "webarcade",
                "broadcaster_user_name": "WebArcade",
                "bits": bits,
                "message": format!("Cheered {} bits!", bits)
            });

            save_test_event("channel.cheer", &event_data);

            json_response(&serde_json::json!({
                "success": true,
                "type": "cheer_event",
                "event_data": event_data,
                "message": format!("Test cheer event created: {} bits from {}", bits, cheerer_name)
            }))
        }
        "raid" => {
            // Simulate a raid event
            let raider_name = body.get("raider_name")
                .and_then(|v| v.as_str())
                .unwrap_or("test_raider");
            let viewers = body.get("viewers").and_then(|v| v.as_i64()).unwrap_or(50);

            let event_data = serde_json::json!({
                "from_broadcaster_user_id": "12345",
                "from_broadcaster_user_login": raider_name,
                "from_broadcaster_user_name": raider_name,
                "to_broadcaster_user_id": "67890",
                "to_broadcaster_user_login": "webarcade",
                "to_broadcaster_user_name": "WebArcade",
                "viewers": viewers
            });

            save_test_event("channel.raid", &event_data);

            json_response(&serde_json::json!({
                "success": true,
                "type": "raid_event",
                "event_data": event_data,
                "message": format!("Test raid event created: {} raiding with {} viewers", raider_name, viewers)
            }))
        }
        "bulk_events" => {
            // Create multiple test events
            let events = vec![
                ("channel.follow", serde_json::json!({"user_login": "follower1"})),
                ("channel.follow", serde_json::json!({"user_login": "follower2"})),
                ("channel.subscribe", serde_json::json!({"user_login": "subscriber1", "tier": "1000"})),
                ("channel.cheer", serde_json::json!({"user_login": "cheerer1", "bits": 500})),
                ("channel.raid", serde_json::json!({"from_broadcaster_user_login": "raider1", "viewers": 100})),
            ];

            for (event_type, event_data) in events {
                save_test_event(event_type, &event_data);
            }

            json_response(&serde_json::json!({
                "success": true,
                "type": "bulk_events",
                "count": 5,
                "message": "Created 5 test events (2 follows, 1 sub, 1 cheer, 1 raid)"
            }))
        }
        _ => error_response(StatusCode::BAD_REQUEST, "Unknown test type. Use: follow, subscribe, cheer, raid, or bulk_events")
    }
}

fn save_test_event(event_type: &str, event_data: &serde_json::Value) {
    let db_path = crate::core::database::get_database_path();
    if let Ok(conn) = rusqlite::Connection::open(&db_path) {
        let timestamp = current_timestamp();
        let event_data_str = event_data.to_string();

        let _ = conn.execute(
            "INSERT INTO twitch_events (event_type, event_data, timestamp)
             VALUES (?1, ?2, ?3)",
            rusqlite::params![event_type, event_data_str, timestamp]
        );
    }
}

fn current_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}
