use crate::core::plugin_context::PluginContext;
use crate::core::plugin_router::PluginRouter;
use anyhow::Result;
use hyper::{Method, Request, Response, StatusCode, body::Incoming};
use hyper::body::Bytes;
use http_body_util::{Full, combinators::BoxBody};
use std::convert::Infallible;
use rusqlite::OptionalExtension;
use once_cell::sync::Lazy;

static NGROK_MANAGER: Lazy<super::NgrokManager> = Lazy::new(|| super::NgrokManager::new());

pub async fn register_routes(ctx: &PluginContext) -> Result<()> {
    let mut router = PluginRouter::new();

    // GET /twitch/auth/url - Get OAuth authorization URL
    router.route(Method::GET, "/auth/url", |_path, query, _req| {
        Box::pin(async move {
            handle_get_auth_url(query).await
        })
    });

    // GET /twitch/auth/callback - OAuth callback endpoint
    router.route(Method::GET, "/auth/callback", |_path, query, _req| {
        Box::pin(async move {
            handle_auth_callback(query).await
        })
    });

    // POST /twitch/auth/refresh - Refresh access token
    router.route(Method::POST, "/auth/refresh", |_path, _query, _req| {
        Box::pin(async move {
            handle_refresh_token().await
        })
    });

    // POST /twitch/auth/validate - Validate access token
    router.route(Method::POST, "/auth/validate", |_path, _query, _req| {
        Box::pin(async move {
            handle_validate_token().await
        })
    });

    // POST /twitch/auth/revoke - Revoke access token
    router.route(Method::POST, "/auth/revoke", |_path, _query, _req| {
        Box::pin(async move {
            handle_revoke_token().await
        })
    });

    // GET /twitch/auth/status - Get authentication status
    router.route(Method::GET, "/auth/status", |_path, _query, _req| {
        Box::pin(async move {
            handle_auth_status().await
        })
    });

    // Bot authentication endpoints
    // GET /twitch/bot/auth/url - Get OAuth authorization URL for bot
    router.route(Method::GET, "/bot/auth/url", |_path, query, _req| {
        Box::pin(async move {
            handle_get_bot_auth_url(query).await
        })
    });

    // GET /twitch/bot/auth/callback - OAuth callback endpoint for bot
    router.route(Method::GET, "/bot/auth/callback", |_path, query, _req| {
        Box::pin(async move {
            handle_bot_auth_callback(query).await
        })
    });

    // POST /twitch/bot/auth/revoke - Revoke bot access token
    router.route(Method::POST, "/bot/auth/revoke", |_path, _query, _req| {
        Box::pin(async move {
            handle_revoke_bot_token().await
        })
    });

    // GET /twitch/bot/auth/status - Get bot authentication status
    router.route(Method::GET, "/bot/auth/status", |_path, _query, _req| {
        Box::pin(async move {
            handle_bot_auth_status().await
        })
    });

    // POST /twitch/eventsub/webhook - EventSub webhook callback
    router.route(Method::POST, "/eventsub/webhook", |_path, _query, req| {
        Box::pin(async move {
            handle_eventsub_webhook(req).await
        })
    });

    // OPTIONS handlers for CORS preflight
    router.route(Method::OPTIONS, "/eventsub/subscriptions", |_path, _query, _req| {
        Box::pin(async move {
            cors_preflight_response()
        })
    });

    router.route(Method::OPTIONS, "/eventsub/auto-setup", |_path, _query, _req| {
        Box::pin(async move {
            cors_preflight_response()
        })
    });

    // GET /twitch/eventsub/subscriptions - List EventSub subscriptions
    router.route(Method::GET, "/eventsub/subscriptions", |_path, _query, _req| {
        Box::pin(async move {
            handle_list_eventsub_subscriptions().await
        })
    });

    // POST /twitch/eventsub/subscriptions - Create EventSub subscription
    router.route(Method::POST, "/eventsub/subscriptions", |_path, _query, req| {
        Box::pin(async move {
            handle_create_eventsub_subscription(req).await
        })
    });

    // DELETE /twitch/eventsub/subscriptions - Delete all EventSub subscriptions
    router.route(Method::DELETE, "/eventsub/subscriptions", |_path, _query, _req| {
        Box::pin(async move {
            handle_delete_all_eventsub_subscriptions().await
        })
    });

    // POST /twitch/eventsub/auto-setup - Auto-setup EventSub subscriptions
    router.route(Method::POST, "/eventsub/auto-setup", |_path, _query, req| {
        Box::pin(async move {
            handle_auto_setup_eventsub(req).await
        })
    });

    // GET /twitch/channel/info/:channel_name - Get channel info
    router.route(Method::GET, "/channel/info/:channel_name", |path, _query, _req| {
        Box::pin(async move {
            handle_get_channel_info(path).await
        })
    });

    // POST /twitch/channel/update - Update channel info
    router.route(Method::POST, "/channel/update", |_path, _query, req| {
        Box::pin(async move {
            handle_update_channel_info(req).await
        })
    });

    // GET /twitch/messages/:channel - Get recent messages
    router.route(Method::GET, "/messages/:channel", |path, query, _req| {
        Box::pin(async move {
            handle_get_messages(path, query).await
        })
    });

    // POST /twitch/messages/send - Send chat message
    // Note: This endpoint logs only. Use twitch.send_message event for actual sending.
    router.route(Method::POST, "/messages/send", |_path, _query, req| {
        Box::pin(async move {
            handle_send_message(req).await
        })
    });

    // GET /twitch/config - Get Twitch configuration
    router.route(Method::GET, "/config", |_path, _query, _req| {
        Box::pin(async move {
            handle_get_config().await
        })
    });

    // POST /twitch/config - Save Twitch configuration
    router.route(Method::POST, "/config", |_path, _query, req| {
        Box::pin(async move {
            handle_save_config(req).await
        })
    });

    // GET /twitch/commands - Get all registered commands
    router.route(Method::GET, "/commands", |_path, _query, _req| {
        Box::pin(async move {
            handle_get_commands().await
        })
    });

    // POST /twitch/commands/register - Register a new command
    router.route(Method::POST, "/commands/register", |_path, _query, req| {
        Box::pin(async move {
            handle_register_command(req).await
        })
    });

    // GET /twitch/schedule - Get Twitch schedule
    router.route(Method::GET, "/schedule", |_path, _query, _req| {
        Box::pin(async move {
            handle_get_schedule().await
        })
    });

    // POST /twitch/schedule/sync - Sync schedule from Twitch
    router.route(Method::POST, "/schedule/sync", |_path, _query, _req| {
        Box::pin(async move {
            handle_sync_schedule().await
        })
    });

    // DELETE /twitch/schedule/segment - Delete schedule segment
    router.route(Method::DELETE, "/schedule/segment", |_path, query, _req| {
        Box::pin(async move {
            handle_delete_schedule_segment(query).await
        })
    });

    // GET /twitch/text-commands - Get all text commands for a channel
    router.route(Method::GET, "/text-commands", |_path, query, _req| {
        Box::pin(async move {
            handle_get_text_commands(query).await
        })
    });

    // POST /twitch/text-commands/add - Add a text command
    router.route(Method::POST, "/text-commands/add", |_path, _query, req| {
        Box::pin(async move {
            handle_add_text_command(req).await
        })
    });

    // POST /twitch/text-commands/edit - Edit a text command
    router.route(Method::POST, "/text-commands/edit", |_path, _query, req| {
        Box::pin(async move {
            handle_edit_text_command(req).await
        })
    });

    // DELETE /twitch/text-commands - Delete a text command
    router.route(Method::DELETE, "/text-commands", |_path, _query, req| {
        Box::pin(async move {
            handle_delete_text_command(req).await
        })
    });

    // Ngrok management endpoints
    // POST /twitch/ngrok/start - Start ngrok tunnel
    router.route(Method::POST, "/ngrok/start", |_path, _query, _req| {
        Box::pin(async move {
            handle_ngrok_start().await
        })
    });

    // POST /twitch/ngrok/stop - Stop ngrok tunnel
    router.route(Method::POST, "/ngrok/stop", |_path, _query, _req| {
        Box::pin(async move {
            handle_ngrok_stop().await
        })
    });

    // GET /twitch/ngrok/status - Get ngrok status
    router.route(Method::GET, "/ngrok/status", |_path, _query, _req| {
        Box::pin(async move {
            handle_ngrok_status().await
        })
    });

    ctx.register_router("twitch", router).await;
    Ok(())
}

async fn handle_get_auth_url(query: String) -> Response<BoxBody<Bytes, Infallible>> {
    let conn = crate::core::database::get_database_path();
    let conn = match rusqlite::Connection::open(&conn) {
        Ok(c) => c,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    // Get client_id and client_secret from config
    let client_id: Option<String> = match conn.query_row(
        "SELECT value FROM twitch_config WHERE key = 'client_id'",
        [],
        |row| row.get(0)
    ).optional() {
        Ok(v) => v,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    let client_secret: Option<String> = match conn.query_row(
        "SELECT value FROM twitch_config WHERE key = 'client_secret'",
        [],
        |row| row.get(0)
    ).optional() {
        Ok(v) => v,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    if client_id.is_none() || client_secret.is_none() {
        return error_response(StatusCode::BAD_REQUEST, "Client ID and Secret not configured");
    }

    // Parse redirect_uri from query or use default
    let redirect_uri = parse_query_param(&query, "redirect_uri")
        .unwrap_or_else(|| "http://localhost:3001/twitch/auth/callback".to_string());

    // Parse scopes from query or use defaults with ALL EventSub scopes
    let scopes_str = parse_query_param(&query, "scopes")
        .unwrap_or_else(|| "chat:read chat:edit \
            channel:read:subscriptions channel:read:redemptions channel:manage:redemptions \
            channel:read:polls channel:manage:polls channel:read:predictions channel:manage:predictions \
            channel:read:hype_train channel:read:charity channel:read:goals channel:manage:broadcast \
            channel:read:vips channel:manage:vips channel:bot \
            moderator:read:followers moderator:read:automod_settings moderator:manage:automod \
            moderator:read:chat_messages moderator:read:chatters moderator:read:chat_settings moderator:manage:chat_settings \
            moderator:read:banned_users moderator:manage:banned_users moderator:read:moderators \
            moderator:read:vips moderator:manage:announcements moderator:read:shield_mode moderator:manage:shield_mode \
            moderator:read:shoutouts moderator:manage:shoutouts moderator:read:unban_requests moderator:manage:unban_requests \
            moderator:read:suspicious_users moderator:read:warnings moderator:manage:warnings \
            user:read:chat user:write:chat user:bot bits:read".to_string());
    let scopes: Vec<&str> = scopes_str.split_whitespace().collect();

    let state = parse_query_param(&query, "state")
        .unwrap_or_else(|| "twitch_oauth".to_string());

    let auth = super::auth::TwitchAuth::new(
        client_id.unwrap(),
        client_secret.unwrap(),
        redirect_uri,
    );

    let auth_url = auth.get_authorization_url(&scopes, &state);

    json_response(&serde_json::json!({ "url": auth_url }))
}

async fn handle_auth_callback(query: String) -> Response<BoxBody<Bytes, Infallible>> {
    let code = match parse_query_param(&query, "code") {
        Some(c) => c,
        None => return error_response(StatusCode::BAD_REQUEST, "Missing code parameter"),
    };

    let conn = crate::core::database::get_database_path();
    let conn = match rusqlite::Connection::open(&conn) {
        Ok(c) => c,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    // Get client_id and client_secret from config
    let client_id: Option<String> = match conn.query_row(
        "SELECT value FROM twitch_config WHERE key = 'client_id'",
        [],
        |row| row.get(0)
    ).optional() {
        Ok(v) => v,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    let client_secret: Option<String> = match conn.query_row(
        "SELECT value FROM twitch_config WHERE key = 'client_secret'",
        [],
        |row| row.get(0)
    ).optional() {
        Ok(v) => v,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    if client_id.is_none() || client_secret.is_none() {
        return error_response(StatusCode::BAD_REQUEST, "Client ID and Secret not configured");
    }

    let redirect_uri = parse_query_param(&query, "redirect_uri")
        .unwrap_or_else(|| "http://localhost:3001/twitch/auth/callback".to_string());

    let auth = super::auth::TwitchAuth::new(
        client_id.unwrap(),
        client_secret.unwrap(),
        redirect_uri,
    );

    // Exchange code for token
    let token_response = match auth.exchange_code_for_token(&code).await {
        Ok(t) => t,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to exchange code: {}", e)),
    };

    // Get user info to store user_id and username
    let api_client = super::api::TwitchApiClient::new(
        auth.client_id.clone(),
        token_response.access_token.clone(),
    );

    let user_info = match api_client.get_authenticated_user().await {
        Ok(info) => info,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to get user info: {}", e)),
    };

    // Extract user data
    let user_data = match user_info.get("data").and_then(|d| d.as_array()).and_then(|a| a.first()) {
        Some(u) => u,
        None => return error_response(StatusCode::INTERNAL_SERVER_ERROR, "Invalid user data"),
    };

    let user_id = match user_data.get("id").and_then(|v| v.as_str()) {
        Some(id) => id,
        None => return error_response(StatusCode::INTERNAL_SERVER_ERROR, "Missing user ID"),
    };

    let username = match user_data.get("login").and_then(|v| v.as_str()) {
        Some(name) => name,
        None => return error_response(StatusCode::INTERNAL_SERVER_ERROR, "Missing username"),
    };

    let profile_image_url = user_data.get("profile_image_url").and_then(|v| v.as_str());

    // Store auth in database
    let now = current_timestamp();
    let expires_at = now + token_response.expires_in;
    let scopes_json = serde_json::to_string(&token_response.scope).unwrap_or_default();

    if let Err(e) = conn.execute(
        "INSERT OR REPLACE INTO twitch_auth (user_id, username, access_token, refresh_token, expires_at, scopes, profile_image_url, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        rusqlite::params![user_id, username, token_response.access_token, token_response.refresh_token, expires_at, scopes_json, profile_image_url, now, now],
    ) {
        return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string());
    }

    // Auto-configure channel from authenticated user
    let channels_json = serde_json::to_string(&vec![username]).unwrap_or_default();
    if let Err(e) = conn.execute(
        "INSERT OR REPLACE INTO twitch_config (key, value, updated_at) VALUES ('channels', ?1, ?2)",
        rusqlite::params![channels_json, now],
    ) {
        log::warn!("[Twitch Auth] Failed to auto-set channel: {}", e);
    }

    // Also store the broadcaster username for reference
    if let Err(e) = conn.execute(
        "INSERT OR REPLACE INTO twitch_config (key, value, updated_at) VALUES ('broadcaster_username', ?1, ?2)",
        rusqlite::params![username, now],
    ) {
        log::warn!("[Twitch Auth] Failed to store broadcaster username: {}", e);
    }

    log::info!("[Twitch Auth] Successfully authenticated {} and auto-configured channel", username);

    json_response(&serde_json::json!({
        "success": true,
        "user_id": user_id,
        "username": username,
        "expires_in": token_response.expires_in
    }))
}

async fn handle_refresh_token() -> Response<BoxBody<Bytes, Infallible>> {
    let conn = crate::core::database::get_database_path();
    let conn = match rusqlite::Connection::open(&conn) {
        Ok(c) => c,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    // Get current auth
    let auth_data: Option<(String, String, String)> = match conn.query_row(
        "SELECT user_id, username, refresh_token FROM twitch_auth LIMIT 1",
        [],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?))
    ).optional() {
        Ok(v) => v,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    let (user_id, username, refresh_token) = match auth_data {
        Some(data) => data,
        None => return error_response(StatusCode::BAD_REQUEST, "No authentication found"),
    };

    // Get client config
    let client_id: Option<String> = match conn.query_row(
        "SELECT value FROM twitch_config WHERE key = 'client_id'",
        [],
        |row| row.get(0)
    ).optional() {
        Ok(v) => v,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    let client_secret: Option<String> = match conn.query_row(
        "SELECT value FROM twitch_config WHERE key = 'client_secret'",
        [],
        |row| row.get(0)
    ).optional() {
        Ok(v) => v,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    if client_id.is_none() || client_secret.is_none() {
        return error_response(StatusCode::BAD_REQUEST, "Client ID and Secret not configured");
    }

    let auth = super::auth::TwitchAuth::new(
        client_id.unwrap(),
        client_secret.unwrap(),
        "".to_string(),
    );

    // Refresh the token
    let token_response = match auth.refresh_access_token(&refresh_token).await {
        Ok(t) => t,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to refresh token: {}", e)),
    };

    // Update database
    let now = current_timestamp();
    let expires_at = now + token_response.expires_in;
    let scopes_json = serde_json::to_string(&token_response.scope).unwrap_or_default();

    if let Err(e) = conn.execute(
        "UPDATE twitch_auth SET access_token = ?1, refresh_token = ?2, expires_at = ?3, scopes = ?4, updated_at = ?5
         WHERE user_id = ?6",
        rusqlite::params![token_response.access_token, token_response.refresh_token, expires_at, scopes_json, now, user_id],
    ) {
        return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string());
    }

    json_response(&serde_json::json!({
        "success": true,
        "user_id": user_id,
        "username": username,
        "expires_in": token_response.expires_in
    }))
}

async fn handle_validate_token() -> Response<BoxBody<Bytes, Infallible>> {
    let conn = crate::core::database::get_database_path();
    let conn = match rusqlite::Connection::open(&conn) {
        Ok(c) => c,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    // Get current auth
    let access_token: Option<String> = match conn.query_row(
        "SELECT access_token FROM twitch_auth LIMIT 1",
        [],
        |row| row.get(0)
    ).optional() {
        Ok(v) => v,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    let access_token = match access_token {
        Some(token) => token,
        None => return error_response(StatusCode::BAD_REQUEST, "No authentication found"),
    };

    // Get client config for validation
    let client_id: Option<String> = match conn.query_row(
        "SELECT value FROM twitch_config WHERE key = 'client_id'",
        [],
        |row| row.get(0)
    ).optional() {
        Ok(v) => v,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    let client_secret: Option<String> = match conn.query_row(
        "SELECT value FROM twitch_config WHERE key = 'client_secret'",
        [],
        |row| row.get(0)
    ).optional() {
        Ok(v) => v,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    if client_id.is_none() || client_secret.is_none() {
        return error_response(StatusCode::BAD_REQUEST, "Client ID and Secret not configured");
    }

    let auth = super::auth::TwitchAuth::new(
        client_id.unwrap(),
        client_secret.unwrap(),
        "".to_string(),
    );

    let is_valid = match auth.validate_token(&access_token).await {
        Ok(valid) => valid,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to validate token: {}", e)),
    };

    json_response(&serde_json::json!({ "valid": is_valid }))
}

async fn handle_revoke_token() -> Response<BoxBody<Bytes, Infallible>> {
    let conn = crate::core::database::get_database_path();
    let conn = match rusqlite::Connection::open(&conn) {
        Ok(c) => c,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    // Get current auth
    let access_token: Option<String> = match conn.query_row(
        "SELECT access_token FROM twitch_auth LIMIT 1",
        [],
        |row| row.get(0)
    ).optional() {
        Ok(v) => v,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    let access_token = match access_token {
        Some(token) => token,
        None => return error_response(StatusCode::BAD_REQUEST, "No authentication found"),
    };

    // Get client config
    let client_id: Option<String> = match conn.query_row(
        "SELECT value FROM twitch_config WHERE key = 'client_id'",
        [],
        |row| row.get(0)
    ).optional() {
        Ok(v) => v,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    let client_secret: Option<String> = match conn.query_row(
        "SELECT value FROM twitch_config WHERE key = 'client_secret'",
        [],
        |row| row.get(0)
    ).optional() {
        Ok(v) => v,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    if client_id.is_none() || client_secret.is_none() {
        return error_response(StatusCode::BAD_REQUEST, "Client ID and Secret not configured");
    }

    let auth = super::auth::TwitchAuth::new(
        client_id.unwrap(),
        client_secret.unwrap(),
        "".to_string(),
    );

    // Revoke the token (ignore errors if token is already invalid)
    match auth.revoke_token(&access_token).await {
        Ok(_) => log::info!("[Twitch Auth] Token revoked successfully"),
        Err(e) => log::warn!("[Twitch Auth] Could not revoke token (may already be invalid): {}", e),
    }

    // Delete from database regardless of revoke result
    if let Err(e) = conn.execute("DELETE FROM twitch_auth", []) {
        return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string());
    }

    json_response(&serde_json::json!({ "success": true }))
}

async fn handle_auth_status() -> Response<BoxBody<Bytes, Infallible>> {
    let conn = crate::core::database::get_database_path();
    let conn = match rusqlite::Connection::open(&conn) {
        Ok(c) => c,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    let auth_data: Option<(String, String, i64, String, Option<String>)> = match conn.query_row(
        "SELECT user_id, username, expires_at, scopes, profile_image_url FROM twitch_auth LIMIT 1",
        [],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?))
    ).optional() {
        Ok(v) => v,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    match auth_data {
        Some((user_id, username, expires_at, scopes_json, profile_image_url)) => {
            let now = current_timestamp();
            let is_expired = expires_at <= now;
            let scopes: Vec<String> = serde_json::from_str(&scopes_json).unwrap_or_default();

            json_response(&serde_json::json!({
                "authenticated": true,
                "user_id": user_id,
                "username": username,
                "expires_at": expires_at,
                "is_expired": is_expired,
                "scopes": scopes,
                "profile_image_url": profile_image_url,
                "connected_channels": vec![username.clone()]
            }))
        }
        None => {
            json_response(&serde_json::json!({ "authenticated": false }))
        }
    }
}

async fn handle_get_channel_info(path: String) -> Response<BoxBody<Bytes, Infallible>> {
    let channel_name = extract_path_param(&path, "/channel/info/");
    if channel_name.is_empty() {
        return error_response(StatusCode::BAD_REQUEST, "Missing channel_name");
    }

    let conn = crate::core::database::get_database_path();
    let conn = match rusqlite::Connection::open(&conn) {
        Ok(c) => c,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    let channel: Option<serde_json::Value> = match conn.query_row(
        "SELECT channel_id, channel_name, display_name, is_live, game_name, title, viewer_count, started_at
         FROM twitch_channels WHERE channel_name = ?1",
        rusqlite::params![channel_name],
        |row| {
            Ok(serde_json::json!({
                "channel_id": row.get::<_, String>(0)?,
                "channel_name": row.get::<_, String>(1)?,
                "display_name": row.get::<_, String>(2)?,
                "is_live": row.get::<_, i64>(3)? != 0,
                "game_name": row.get::<_, Option<String>>(4)?,
                "title": row.get::<_, Option<String>>(5)?,
                "viewer_count": row.get::<_, Option<i64>>(6)?,
                "started_at": row.get::<_, Option<i64>>(7)?,
            }))
        }
    ).optional() {
        Ok(v) => v,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    json_response(&serde_json::json!({ "channel": channel }))
}

async fn handle_update_channel_info(req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    let body = match read_json_body(req).await {
        Ok(b) => b,
        Err(e) => return error_response(StatusCode::BAD_REQUEST, &e),
    };

    let channel_id = match body.get("channel_id").and_then(|v| v.as_str()) {
        Some(id) => id,
        None => return error_response(StatusCode::BAD_REQUEST, "Missing channel_id"),
    };
    let channel_name = match body.get("channel_name").and_then(|v| v.as_str()) {
        Some(name) => name,
        None => return error_response(StatusCode::BAD_REQUEST, "Missing channel_name"),
    };
    let display_name = match body.get("display_name").and_then(|v| v.as_str()) {
        Some(name) => name,
        None => return error_response(StatusCode::BAD_REQUEST, "Missing display_name"),
    };
    let is_live = body.get("is_live").and_then(|v| v.as_bool()).unwrap_or(false);
    let game_name = body.get("game_name").and_then(|v| v.as_str());
    let title = body.get("title").and_then(|v| v.as_str());
    let viewer_count = body.get("viewer_count").and_then(|v| v.as_i64());
    let started_at = body.get("started_at").and_then(|v| v.as_i64());

    let conn = crate::core::database::get_database_path();
    let conn = match rusqlite::Connection::open(&conn) {
        Ok(c) => c,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    let now = current_timestamp();
    if let Err(e) = conn.execute(
        "INSERT OR REPLACE INTO twitch_channels (channel_id, channel_name, display_name, is_live, game_name, title, viewer_count, started_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        rusqlite::params![channel_id, channel_name, display_name, is_live as i64, game_name, title, viewer_count, started_at, now],
    ) {
        return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string());
    }

    json_response(&serde_json::json!({ "success": true }))
}

async fn handle_get_messages(path: String, query: String) -> Response<BoxBody<Bytes, Infallible>> {
    let channel = extract_path_param(&path, "/messages/");
    if channel.is_empty() {
        return error_response(StatusCode::BAD_REQUEST, "Missing channel");
    }

    let limit = parse_query_param(&query, "limit")
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(100);

    let conn = crate::core::database::get_database_path();
    let conn = match rusqlite::Connection::open(&conn) {
        Ok(c) => c,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    let mut stmt = match conn.prepare(
        "SELECT username, message, timestamp FROM twitch_chat_messages
         WHERE channel = ?1 ORDER BY timestamp DESC LIMIT ?2"
    ) {
        Ok(stmt) => stmt,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    let mapped = match stmt.query_map(
        rusqlite::params![channel, limit],
        |row| {
            Ok(serde_json::json!({
                "username": row.get::<_, String>(0)?,
                "message": row.get::<_, String>(1)?,
                "timestamp": row.get::<_, i64>(2)?,
            }))
        }
    ) {
        Ok(m) => m,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    let messages_result: Result<Vec<_>, _> = mapped.collect();

    match messages_result {
        Ok(msgs) => json_response(&serde_json::json!({ "messages": msgs })),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn handle_send_message(req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    let body = match read_json_body(req).await {
        Ok(b) => b,
        Err(e) => return error_response(StatusCode::BAD_REQUEST, &e),
    };

    let channel = match body.get("channel").and_then(|v| v.as_str()) {
        Some(c) => c,
        None => return error_response(StatusCode::BAD_REQUEST, "Missing channel"),
    };
    let message = match body.get("message").and_then(|v| v.as_str()) {
        Some(m) => m,
        None => return error_response(StatusCode::BAD_REQUEST, "Missing message"),
    };

    log::info!("[Twitch Router] HTTP send message request: {} -> {}", channel, message);
    log::info!("[Twitch Router] NOTE: Use twitch.send_message event for actual IRC sending");

    json_response(&serde_json::json!({ "success": true }))
}

async fn handle_get_config() -> Response<BoxBody<Bytes, Infallible>> {
    let conn = crate::core::database::get_database_path();
    let conn = match rusqlite::Connection::open(&conn) {
        Ok(c) => c,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    // Get all config values
    let client_id: Option<String> = match conn.query_row(
        "SELECT value FROM twitch_config WHERE key = 'client_id'",
        [],
        |row| row.get(0)
    ).optional() {
        Ok(v) => v,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    let client_secret: Option<String> = match conn.query_row(
        "SELECT value FROM twitch_config WHERE key = 'client_secret'",
        [],
        |row| row.get(0)
    ).optional() {
        Ok(v) => v,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    let broadcaster_username: Option<String> = match conn.query_row(
        "SELECT value FROM twitch_config WHERE key = 'broadcaster_username'",
        [],
        |row| row.get(0)
    ).optional() {
        Ok(v) => v,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    let channels_json: Option<String> = match conn.query_row(
        "SELECT value FROM twitch_config WHERE key = 'channels'",
        [],
        |row| row.get(0)
    ).optional() {
        Ok(v) => v,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    let channels: Vec<String> = if let Some(json) = channels_json {
        serde_json::from_str(&json).unwrap_or_else(|_| vec![])
    } else {
        vec![]
    };

    // Check if we have an auth token
    let has_token = match conn.query_row(
        "SELECT COUNT(*) FROM twitch_auth",
        [],
        |row| row.get::<_, i64>(0)
    ) {
        Ok(count) => count > 0,
        Err(_) => false,
    };

    json_response(&serde_json::json!({
        "client_id": client_id.unwrap_or_default(),
        "client_secret": client_secret.map(|_| "***").unwrap_or_default(), // Don't send actual secret
        "broadcaster_username": broadcaster_username,
        "channels": channels,
        "has_token": has_token
    }))
}

async fn handle_save_config(req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    let body = match read_json_body(req).await {
        Ok(b) => b,
        Err(e) => return error_response(StatusCode::BAD_REQUEST, &e),
    };

    let conn = crate::core::database::get_database_path();
    let conn = match rusqlite::Connection::open(&conn) {
        Ok(c) => c,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    let now = current_timestamp();

    // Save client_id
    if let Some(client_id) = body.get("client_id").and_then(|v| v.as_str()) {
        if let Err(e) = conn.execute(
            "INSERT OR REPLACE INTO twitch_config (key, value, updated_at) VALUES ('client_id', ?1, ?2)",
            rusqlite::params![client_id, now],
        ) {
            return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string());
        }
    }

    // Save client_secret (only if provided and not the masked version)
    if let Some(client_secret) = body.get("client_secret").and_then(|v| v.as_str()) {
        if !client_secret.is_empty() && client_secret != "***" {
            if let Err(e) = conn.execute(
                "INSERT OR REPLACE INTO twitch_config (key, value, updated_at) VALUES ('client_secret', ?1, ?2)",
                rusqlite::params![client_secret, now],
            ) {
                return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string());
            }
        }
    }

    // Note: channels are now auto-configured from authenticated user
    // This can still be saved if explicitly provided for backward compatibility
    if let Some(channels) = body.get("channels").and_then(|v| v.as_array()) {
        let channels_json = match serde_json::to_string(&channels) {
            Ok(json) => json,
            Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
        };
        if let Err(e) = conn.execute(
            "INSERT OR REPLACE INTO twitch_config (key, value, updated_at) VALUES ('channels', ?1, ?2)",
            rusqlite::params![channels_json, now],
        ) {
            return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string());
        }
    }

    json_response(&serde_json::json!({ "success": true }))
}

async fn handle_get_commands() -> Response<BoxBody<Bytes, Infallible>> {
    let conn = crate::core::database::get_database_path();
    let conn = match rusqlite::Connection::open(&conn) {
        Ok(c) => c,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    let mut stmt = match conn.prepare(
        "SELECT id, command, handler_plugin, handler_method, permission_level, cooldown_seconds, enabled
         FROM twitch_commands ORDER BY command"
    ) {
        Ok(stmt) => stmt,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    let mapped = match stmt.query_map([], |row| {
        Ok(serde_json::json!({
            "id": row.get::<_, i64>(0)?,
            "command": row.get::<_, String>(1)?,
            "handler_plugin": row.get::<_, String>(2)?,
            "handler_method": row.get::<_, String>(3)?,
            "permission_level": row.get::<_, String>(4)?,
            "cooldown_seconds": row.get::<_, i64>(5)?,
            "enabled": row.get::<_, i64>(6)? != 0,
        }))
    }) {
        Ok(m) => m,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    let commands_result: Result<Vec<_>, _> = mapped.collect();

    match commands_result {
        Ok(cmds) => json_response(&serde_json::json!({ "commands": cmds })),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn handle_register_command(req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    let body = match read_json_body(req).await {
        Ok(b) => b,
        Err(e) => return error_response(StatusCode::BAD_REQUEST, &e),
    };

    let command = match body.get("command").and_then(|v| v.as_str()) {
        Some(c) => c,
        None => return error_response(StatusCode::BAD_REQUEST, "Missing command"),
    };
    let handler_plugin = match body.get("handler_plugin").and_then(|v| v.as_str()) {
        Some(h) => h,
        None => return error_response(StatusCode::BAD_REQUEST, "Missing handler_plugin"),
    };
    let handler_method = match body.get("handler_method").and_then(|v| v.as_str()) {
        Some(h) => h,
        None => return error_response(StatusCode::BAD_REQUEST, "Missing handler_method"),
    };
    let permission_level = body.get("permission_level").and_then(|v| v.as_str()).unwrap_or("everyone");
    let cooldown_seconds = body.get("cooldown_seconds").and_then(|v| v.as_i64()).unwrap_or(0);

    let conn = crate::core::database::get_database_path();
    let conn = match rusqlite::Connection::open(&conn) {
        Ok(c) => c,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    let now = current_timestamp();
    if let Err(e) = conn.execute(
        "INSERT OR REPLACE INTO twitch_commands (command, handler_plugin, handler_method, permission_level, cooldown_seconds, enabled, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, 1, ?6)",
        rusqlite::params![command, handler_plugin, handler_method, permission_level, cooldown_seconds, now],
    ) {
        return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string());
    }

    json_response(&serde_json::json!({ "success": true }))
}

// Helper functions
fn extract_path_param(path: &str, prefix: &str) -> String {
    path.strip_prefix(prefix)
        .map(|s| urlencoding::decode(s).unwrap_or_default().to_string())
        .unwrap_or_default()
}

fn parse_query_param(query: &str, key: &str) -> Option<String> {
    query.split('&')
        .find_map(|pair| {
            let mut parts = pair.split('=');
            if parts.next()? == key {
                Some(urlencoding::decode(parts.next()?).ok()?.into_owned())
            } else {
                None
            }
        })
}

async fn read_json_body(req: Request<Incoming>) -> std::result::Result<serde_json::Value, String> {
    use http_body_util::BodyExt;
    let whole_body = req.collect().await
        .map_err(|e| format!("Failed to read body: {}", e))?
        .to_bytes();

    serde_json::from_slice(&whole_body)
        .map_err(|e| format!("Invalid JSON: {}", e))
}

fn json_response<T: serde::Serialize>(data: &T) -> Response<BoxBody<Bytes, Infallible>> {
    let json = serde_json::to_string(data).unwrap_or_else(|_| "{}".to_string());
    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .header("Access-Control-Allow-Origin", "*")
        .body(full_body(&json))
        .unwrap()
}

fn error_response(status: StatusCode, message: &str) -> Response<BoxBody<Bytes, Infallible>> {
    let json = serde_json::json!({"error": message}).to_string();
    Response::builder()
        .status(status)
        .header("Content-Type", "application/json")
        .header("Access-Control-Allow-Origin", "*")
        .body(full_body(&json))
        .unwrap()
}

fn full_body(s: &str) -> BoxBody<Bytes, Infallible> {
    use http_body_util::combinators::BoxBody;
    use http_body_util::BodyExt;
    BoxBody::new(Full::new(Bytes::from(s.to_string())).map_err(|err: Infallible| match err {}))
}

fn current_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

fn cors_preflight_response() -> Response<BoxBody<Bytes, std::convert::Infallible>> {
    Response::builder()
        .status(StatusCode::OK)
        .header("Access-Control-Allow-Origin", "*")
        .header("Access-Control-Allow-Methods", "GET, POST, DELETE, OPTIONS")
        .header("Access-Control-Allow-Headers", "Content-Type, Authorization")
        .header("Access-Control-Max-Age", "86400")
        .body(full_body(""))
        .unwrap()
}

// Schedule handlers
async fn handle_get_schedule() -> Response<BoxBody<Bytes, Infallible>> {
    // Get auth token
    let conn = crate::core::database::get_database_path();
    let conn = match rusqlite::Connection::open(&conn) {
        Ok(c) => c,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    let (user_id, access_token, client_id): (String, String, String) = match conn.query_row(
        "SELECT a.user_id, a.access_token, c.value
         FROM twitch_auth a
         LEFT JOIN twitch_config c ON c.key = 'client_id'
         LIMIT 1",
        [],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?))
    ) {
        Ok(data) => data,
        Err(_) => return error_response(StatusCode::UNAUTHORIZED, "Not authenticated"),
    };

    // Fetch schedule from Twitch API
    let api_client = super::api::TwitchApiClient::new(client_id, access_token);
    match api_client.get_schedule(&user_id).await {
        Ok(schedule) => json_response(&schedule),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn handle_sync_schedule() -> Response<BoxBody<Bytes, Infallible>> {
    // Same as get_schedule for now - could add database caching later
    handle_get_schedule().await
}

async fn handle_delete_schedule_segment(query: String) -> Response<BoxBody<Bytes, Infallible>> {
    // Extract segment ID from query
    let id = parse_query_param(&query, "id").unwrap_or_default();

    if id.is_empty() {
        return error_response(StatusCode::BAD_REQUEST, "Missing segment id");
    }

    // For now, return success - actual deletion would require Twitch API call
    // This is a stub that can be implemented when needed
    json_response(&serde_json::json!({
        "success": true,
        "message": "Schedule segment deletion not yet implemented"
    }))
}

// Text Commands handlers
async fn handle_get_text_commands(query: String) -> Response<BoxBody<Bytes, Infallible>> {
    let channel = parse_query_param(&query, "channel").unwrap_or_default();
    if channel.is_empty() {
        return error_response(StatusCode::BAD_REQUEST, "Missing channel parameter");
    }

    let conn = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&conn) {
        Ok(conn) => {
            // Check which columns exist in the table
            let has_auto_post = conn.query_row(
                "SELECT COUNT(*) FROM pragma_table_info('text_commands') WHERE name = 'auto_post'",
                [],
                |row| row.get::<_, i64>(0)
            ).unwrap_or(0) > 0;

            let has_interval = conn.query_row(
                "SELECT COUNT(*) FROM pragma_table_info('text_commands') WHERE name = 'interval_minutes'",
                [],
                |row| row.get::<_, i64>(0)
            ).unwrap_or(0) > 0;

            let query_sql = if has_auto_post && has_interval {
                "SELECT id, command, response, auto_post, interval_minutes FROM text_commands WHERE channel = ?1 ORDER BY command ASC"
            } else {
                "SELECT id, command, response FROM text_commands WHERE channel = ?1 ORDER BY command ASC"
            };

            let mut stmt = match conn.prepare(query_sql) {
                Ok(stmt) => stmt,
                Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            };

            let commands_result: Result<Vec<_>, _> = stmt.query_map([&channel], |row| {
                let mut cmd = serde_json::json!({
                    "id": row.get::<_, i64>(0)?,
                    "command": row.get::<_, String>(1)?,
                    "response": row.get::<_, String>(2)?,
                });

                if has_auto_post && has_interval {
                    if let Ok(auto_post) = row.get::<_, i64>(3) {
                        cmd["auto_post"] = serde_json::json!(auto_post != 0);
                    }
                    if let Ok(interval) = row.get::<_, i64>(4) {
                        cmd["interval_minutes"] = serde_json::json!(interval);
                    }
                }

                Ok(cmd)
            }).and_then(|rows| rows.collect());

            match commands_result {
                Ok(commands) => json_response(&commands),
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn handle_add_text_command(req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    match read_json_body(req).await {
        Ok(body) => {
            let channel = match body.get("channel").and_then(|v| v.as_str()) {
                Some(c) => c,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing channel"),
            };
            let command = match body.get("command").and_then(|v| v.as_str()) {
                Some(cmd) => cmd,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing command"),
            };
            let response_text = match body.get("response").and_then(|v| v.as_str()) {
                Some(r) => r,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing response"),
            };

            let conn = crate::core::database::get_database_path();
            match rusqlite::Connection::open(&conn) {
                Ok(conn) => {
                    let now = current_timestamp();
                    match conn.execute(
                        "INSERT INTO text_commands (channel, command, response, auto_post, interval_minutes, created_at)
                         VALUES (?1, ?2, ?3, 0, 10, ?4)
                         ON CONFLICT(channel, command) DO UPDATE SET
                           response = ?3",
                        rusqlite::params![channel, command, response_text, now],
                    ) {
                        Ok(_) => json_response(&serde_json::json!({ "success": true })),
                        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
                    }
                }
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::BAD_REQUEST, &e),
    }
}

async fn handle_edit_text_command(req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    // Same as add - uses UPSERT
    handle_add_text_command(req).await
}

async fn handle_delete_text_command(req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    match read_json_body(req).await {
        Ok(body) => {
            let channel = match body.get("channel").and_then(|v| v.as_str()) {
                Some(c) => c,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing channel"),
            };
            let command = match body.get("command").and_then(|v| v.as_str()) {
                Some(cmd) => cmd,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing command"),
            };

            let conn = crate::core::database::get_database_path();
            match rusqlite::Connection::open(&conn) {
                Ok(conn) => {
                    match conn.execute(
                        "DELETE FROM text_commands WHERE channel = ?1 AND command = ?2",
                        rusqlite::params![channel, command],
                    ) {
                        Ok(_) => json_response(&serde_json::json!({ "success": true })),
                        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
                    }
                }
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::BAD_REQUEST, &e),
    }
}

// ==================== EventSub Handlers ====================

async fn handle_eventsub_webhook(req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    // Get EventSub configuration from database
    let conn = crate::core::database::get_database_path();
    let conn = match rusqlite::Connection::open(&conn) {
        Ok(c) => c,
        Err(e) => {
            log::error!("[Twitch EventSub] Failed to open database: {}", e);
            return error_response(StatusCode::INTERNAL_SERVER_ERROR, "Database error");
        }
    };

    // Get auth data and webhook secret
    let auth_data: Option<(String, String, String)> = match conn.query_row(
        "SELECT a.access_token, c1.value, c2.value
         FROM twitch_auth a
         LEFT JOIN twitch_config c1 ON c1.key = 'client_id'
         LEFT JOIN twitch_config c2 ON c2.key = 'eventsub_secret'
         LIMIT 1",
        [],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?))
    ).optional() {
        Ok(v) => v,
        Err(e) => {
            log::error!("[Twitch EventSub] Database query failed: {}", e);
            return error_response(StatusCode::INTERNAL_SERVER_ERROR, "Database error");
        }
    };

    let (access_token, client_id, webhook_secret) = match auth_data {
        Some(data) => data,
        None => {
            log::warn!("[Twitch EventSub] No authentication configured");
            return error_response(StatusCode::UNAUTHORIZED, "Not authenticated");
        }
    };

    // Use stored webhook secret or generate one
    let webhook_secret = if webhook_secret.is_empty() {
        log::warn!("[Twitch EventSub] No webhook secret found - this webhook may fail verification");
        String::new()
    } else {
        webhook_secret
    };

    drop(conn);

    // Create EventSub manager
    let callback_url = "http://localhost:3001/twitch/eventsub/webhook".to_string();
    let eventsub_manager = std::sync::Arc::new(super::eventsub::EventSubManager::new(
        client_id,
        access_token,
        webhook_secret,
        callback_url,
    ));

    // Get plugin context from the global plugin manager
    // For now, we'll handle the webhook without emitting events
    // The proper way would be to pass PluginContext through the router

    // Convert the hyper response to our response type
    match super::eventsub_webhook::handle_eventsub_webhook_simple(req, eventsub_manager).await {
        Ok(response) => {
            // Convert Full<Bytes> response to BoxBody response
            use http_body_util::BodyExt;
            let (parts, body) = response.into_parts();
            let boxed = BoxBody::new(body.map_err(|err: std::convert::Infallible| match err {}));
            Response::from_parts(parts, boxed)
        }
        Err(e) => {
            log::error!("[Twitch EventSub] Webhook handler error: {:?}", e);
            error_response(StatusCode::INTERNAL_SERVER_ERROR, "Webhook handler error")
        }
    }
}

async fn handle_list_eventsub_subscriptions() -> Response<BoxBody<Bytes, Infallible>> {
    let conn = crate::core::database::get_database_path();
    let conn = match rusqlite::Connection::open(&conn) {
        Ok(c) => c,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    // Get client credentials
    let creds_result = conn.query_row(
        "SELECT c1.value, c2.value
         FROM twitch_config c1
         LEFT JOIN twitch_config c2 ON c2.key = 'client_secret'
         WHERE c1.key = 'client_id'
         LIMIT 1",
        [],
        |row| {
            let client_id: String = row.get(0)?;
            let client_secret: Option<String> = row.get(1).ok();
            Ok((client_id, client_secret))
        }
    ).optional();

    let (client_id, client_secret) = match creds_result {
        Ok(Some((cid, Some(cs)))) => (cid, cs),
        Ok(Some((_, None))) => return error_response(StatusCode::BAD_REQUEST, "Client Secret not configured"),
        Ok(None) => return error_response(StatusCode::BAD_REQUEST, "Client ID not configured"),
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    // Generate app access token
    let http_client = reqwest::Client::new();
    let token_response = match http_client
        .post("https://id.twitch.tv/oauth2/token")
        .form(&[
            ("client_id", client_id.as_str()),
            ("client_secret", client_secret.as_str()),
            ("grant_type", "client_credentials"),
        ])
        .send()
        .await
    {
        Ok(resp) => resp,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to get app token: {}", e)),
    };

    if !token_response.status().is_success() {
        return error_response(StatusCode::INTERNAL_SERVER_ERROR, "Failed to get app token");
    }

    let token_data: serde_json::Value = match token_response.json().await {
        Ok(data) => data,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to parse token: {}", e)),
    };

    let app_access_token = match token_data.get("access_token").and_then(|v| v.as_str()) {
        Some(token) => token.to_string(),
        None => return error_response(StatusCode::INTERNAL_SERVER_ERROR, "No access token in response"),
    };

    let callback_url = "http://localhost:3001/twitch/eventsub/webhook".to_string();
    let manager = super::eventsub::EventSubManager::new(
        client_id,
        app_access_token,
        String::new(), // Not needed for listing
        callback_url,
    );

    match manager.list_subscriptions().await {
        Ok(subs) => json_response(&serde_json::json!({ "subscriptions": subs })),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn handle_create_eventsub_subscription(req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    let body = match read_json_body(req).await {
        Ok(b) => b,
        Err(e) => return error_response(StatusCode::BAD_REQUEST, &e),
    };

    let subscription_type = match body.get("type").and_then(|v| v.as_str()) {
        Some(t) => t,
        None => return error_response(StatusCode::BAD_REQUEST, "Missing 'type' field"),
    };

    let version = match body.get("version").and_then(|v| v.as_str()) {
        Some(v) => v,
        None => "1",
    };

    let condition = match body.get("condition") {
        Some(c) => c.clone(),
        None => return error_response(StatusCode::BAD_REQUEST, "Missing 'condition' field"),
    };

    let conn = crate::core::database::get_database_path();
    let conn = match rusqlite::Connection::open(&conn) {
        Ok(c) => c,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    // Get auth data and client config
    let auth_data: Option<(String, String, String)> = match conn.query_row(
        "SELECT a.access_token, c1.value, c2.value
         FROM twitch_auth a
         LEFT JOIN twitch_config c1 ON c1.key = 'client_id'
         LEFT JOIN twitch_config c2 ON c2.key = 'eventsub_secret'
         LIMIT 1",
        [],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?))
    ).optional() {
        Ok(v) => v,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    let (access_token, client_id, webhook_secret) = match auth_data {
        Some(data) => data,
        None => return error_response(StatusCode::UNAUTHORIZED, "Not authenticated"),
    };

    // Use stored webhook secret or generate one if not exists
    let webhook_secret = if webhook_secret.is_empty() {
        use rand::Rng;
        let secret: String = rand::thread_rng()
            .sample_iter(&rand::distributions::Alphanumeric)
            .take(32)
            .map(char::from)
            .collect();

        // Store it
        let now = current_timestamp();
        if let Err(e) = conn.execute(
            "INSERT OR REPLACE INTO twitch_config (key, value, updated_at) VALUES ('eventsub_secret', ?1, ?2)",
            rusqlite::params![secret.clone(), now],
        ) {
            log::error!("Failed to store EventSub secret: {}", e);
        }

        secret
    } else {
        webhook_secret
    };

    let callback_url = body.get("callback_url")
        .and_then(|v| v.as_str())
        .unwrap_or("http://localhost:3001/twitch/eventsub/webhook")
        .to_string();

    let manager = super::eventsub::EventSubManager::new(
        client_id,
        access_token,
        webhook_secret,
        callback_url,
    );

    match manager.create_subscription(subscription_type, version, condition).await {
        Ok(sub) => {
            // Store subscription in database
            let now = current_timestamp();
            if let Err(e) = conn.execute(
                "INSERT INTO twitch_eventsub_subscriptions (subscription_id, subscription_type, condition, status, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                rusqlite::params![
                    sub.id,
                    sub.subscription_type,
                    sub.condition.to_string(),
                    sub.status,
                    now
                ],
            ) {
                log::error!("Failed to store EventSub subscription: {}", e);
            }

            json_response(&sub)
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn handle_delete_all_eventsub_subscriptions() -> Response<BoxBody<Bytes, Infallible>> {
    let conn = crate::core::database::get_database_path();
    let conn = match rusqlite::Connection::open(&conn) {
        Ok(c) => c,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    // Get client credentials
    let creds_result = conn.query_row(
        "SELECT c1.value, c2.value
         FROM twitch_config c1
         LEFT JOIN twitch_config c2 ON c2.key = 'client_secret'
         WHERE c1.key = 'client_id'
         LIMIT 1",
        [],
        |row| {
            let client_id: String = row.get(0)?;
            let client_secret: Option<String> = row.get(1).ok();
            Ok((client_id, client_secret))
        }
    ).optional();

    let (client_id, client_secret) = match creds_result {
        Ok(Some((cid, Some(cs)))) => (cid, cs),
        Ok(Some((_, None))) => return error_response(StatusCode::BAD_REQUEST, "Client Secret not configured"),
        Ok(None) => return error_response(StatusCode::BAD_REQUEST, "Client ID not configured"),
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    // Generate app access token
    let http_client = reqwest::Client::new();
    let token_response = match http_client
        .post("https://id.twitch.tv/oauth2/token")
        .form(&[
            ("client_id", client_id.as_str()),
            ("client_secret", client_secret.as_str()),
            ("grant_type", "client_credentials"),
        ])
        .send()
        .await
    {
        Ok(resp) => resp,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to get app token: {}", e)),
    };

    if !token_response.status().is_success() {
        return error_response(StatusCode::INTERNAL_SERVER_ERROR, "Failed to get app token");
    }

    let token_data: serde_json::Value = match token_response.json().await {
        Ok(data) => data,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to parse token: {}", e)),
    };

    let app_access_token = match token_data.get("access_token").and_then(|v| v.as_str()) {
        Some(token) => token.to_string(),
        None => return error_response(StatusCode::INTERNAL_SERVER_ERROR, "No access token in response"),
    };

    let callback_url = "http://localhost:3001/twitch/eventsub/webhook".to_string();
    let manager = super::eventsub::EventSubManager::new(
        client_id,
        app_access_token,
        String::new(),
        callback_url,
    );

    match manager.delete_all_subscriptions().await {
        Ok(_) => {
            // Clear from database
            if let Err(e) = conn.execute("DELETE FROM twitch_eventsub_subscriptions", []) {
                log::error!("Failed to clear EventSub subscriptions from database: {}", e);
            }
            json_response(&serde_json::json!({ "success": true }))
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn handle_auto_setup_eventsub(req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    let body = match read_json_body(req).await {
        Ok(b) => b,
        Err(e) => return error_response(StatusCode::BAD_REQUEST, &e),
    };

    let callback_url = match body.get("callback_url").and_then(|v| v.as_str()) {
        Some(url) => url.to_string(),
        None => return error_response(StatusCode::BAD_REQUEST, "Missing callback_url"),
    };

    let conn = crate::core::database::get_database_path();
    let conn = match rusqlite::Connection::open(&conn) {
        Ok(c) => c,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    // Get user ID and config - we need BOTH tokens for EventSub
    let auth_result = conn.query_row(
        "SELECT a.user_id, c1.value, c2.value, c3.value
         FROM twitch_auth a
         LEFT JOIN twitch_config c1 ON c1.key = 'client_id'
         LEFT JOIN twitch_config c2 ON c2.key = 'client_secret'
         LEFT JOIN twitch_config c3 ON c3.key = 'eventsub_secret'
         LIMIT 1",
        [],
        |row| {
            let user_id: String = row.get(0)?;
            let client_id: Option<String> = row.get(1).ok();
            let client_secret: Option<String> = row.get(2).ok();
            let webhook_secret: Option<String> = row.get(3).ok();
            Ok((user_id, client_id, client_secret, webhook_secret))
        }
    ).optional();

    let (user_id, client_id, client_secret, webhook_secret) = match auth_result {
        Ok(Some((uid, Some(cid), Some(cs), secret))) => (uid, cid, cs, secret.unwrap_or_default()),
        Ok(Some((_, None, _, _))) => return error_response(StatusCode::BAD_REQUEST, "Client ID not configured"),
        Ok(Some((_, _, None, _))) => return error_response(StatusCode::BAD_REQUEST, "Client Secret not configured"),
        Ok(None) => return error_response(StatusCode::UNAUTHORIZED, "Not authenticated"),
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    // EventSub webhooks REQUIRE app access token (not user token)
    // But the user must have granted the necessary scopes (verified by their user_id in conditions)
    // Generate app access token using client credentials flow
    log::info!("[Twitch EventSub] Generating app access token for webhook subscriptions");
    let http_client = reqwest::Client::new();
    let token_response = match http_client
        .post("https://id.twitch.tv/oauth2/token")
        .form(&[
            ("client_id", client_id.as_str()),
            ("client_secret", client_secret.as_str()),
            ("grant_type", "client_credentials"),
        ])
        .send()
        .await
    {
        Ok(resp) => resp,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to get app token: {}", e)),
    };

    if !token_response.status().is_success() {
        let error_text = token_response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to get app token: {}", error_text));
    }

    let token_data: serde_json::Value = match token_response.json().await {
        Ok(data) => data,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to parse token response: {}", e)),
    };

    let app_access_token = match token_data.get("access_token").and_then(|v| v.as_str()) {
        Some(token) => token.to_string(),
        None => return error_response(StatusCode::INTERNAL_SERVER_ERROR, "No access token in response"),
    };

    // Use stored webhook secret or generate one
    let webhook_secret = if webhook_secret.is_empty() {
        use rand::Rng;
        let secret: String = rand::thread_rng()
            .sample_iter(&rand::distributions::Alphanumeric)
            .take(32)
            .map(char::from)
            .collect();

        // Store it
        let now = current_timestamp();
        if let Err(e) = conn.execute(
            "INSERT OR REPLACE INTO twitch_config (key, value, updated_at) VALUES ('eventsub_secret', ?1, ?2)",
            rusqlite::params![secret.clone(), now],
        ) {
            log::error!("Failed to store EventSub secret: {}", e);
        }

        secret
    } else {
        webhook_secret
    };

    let manager = super::eventsub::EventSubManager::new(
        client_id.clone(),
        app_access_token,
        webhook_secret,
        callback_url.clone(),
    );

    // Define ALL EventSub event subscriptions to create (80+ events)
    let subscriptions_to_create = vec![
        // ==================== AUTOMOD ====================
        ("automod.message.hold", "1", serde_json::json!({
            "broadcaster_user_id": user_id.clone(),
            "moderator_user_id": user_id.clone()
        })),
        ("automod.message.update", "1", serde_json::json!({
            "broadcaster_user_id": user_id.clone(),
            "moderator_user_id": user_id.clone()
        })),
        ("automod.settings.update", "1", serde_json::json!({
            "broadcaster_user_id": user_id.clone(),
            "moderator_user_id": user_id.clone()
        })),
        ("automod.terms.update", "1", serde_json::json!({
            "broadcaster_user_id": user_id.clone(),
            "moderator_user_id": user_id.clone()
        })),

        // ==================== CHANNEL - BASIC ====================
        ("channel.update", "2", serde_json::json!({
            "broadcaster_user_id": user_id.clone()
        })),
        ("channel.follow", "2", serde_json::json!({
            "broadcaster_user_id": user_id.clone(),
            "moderator_user_id": user_id.clone()
        })),
        // channel.ad_break.begin - Requires Partner status, skip

        // ==================== CHANNEL - CHAT ====================
        ("channel.chat.clear", "1", serde_json::json!({
            "broadcaster_user_id": user_id.clone(),
            "user_id": user_id.clone()
        })),
        ("channel.chat.clear_user_messages", "1", serde_json::json!({
            "broadcaster_user_id": user_id.clone(),
            "user_id": user_id.clone()
        })),
        ("channel.chat.message", "1", serde_json::json!({
            "broadcaster_user_id": user_id.clone(),
            "user_id": user_id.clone()
        })),
        ("channel.chat.message_delete", "1", serde_json::json!({
            "broadcaster_user_id": user_id.clone(),
            "user_id": user_id.clone()
        })),
        ("channel.chat.notification", "1", serde_json::json!({
            "broadcaster_user_id": user_id.clone(),
            "user_id": user_id.clone()
        })),
        ("channel.chat_settings.update", "1", serde_json::json!({
            "broadcaster_user_id": user_id.clone(),
            "user_id": user_id.clone()
        })),
        ("channel.chat.user_message_hold", "1", serde_json::json!({
            "broadcaster_user_id": user_id.clone(),
            "user_id": user_id.clone()
        })),
        ("channel.chat.user_message_update", "1", serde_json::json!({
            "broadcaster_user_id": user_id.clone(),
            "user_id": user_id.clone()
        })),

        // ==================== CHANNEL - SHARED CHAT ====================
        ("channel.shared_chat.begin", "1", serde_json::json!({
            "broadcaster_user_id": user_id.clone()
        })),
        ("channel.shared_chat.update", "1", serde_json::json!({
            "broadcaster_user_id": user_id.clone()
        })),
        ("channel.shared_chat.end", "1", serde_json::json!({
            "broadcaster_user_id": user_id.clone()
        })),

        // ==================== CHANNEL - SUBSCRIPTIONS ====================
        ("channel.subscribe", "1", serde_json::json!({
            "broadcaster_user_id": user_id.clone()
        })),
        ("channel.subscription.end", "1", serde_json::json!({
            "broadcaster_user_id": user_id.clone()
        })),
        ("channel.subscription.gift", "1", serde_json::json!({
            "broadcaster_user_id": user_id.clone()
        })),
        ("channel.subscription.message", "1", serde_json::json!({
            "broadcaster_user_id": user_id.clone()
        })),

        // ==================== CHANNEL - ENGAGEMENT ====================
        ("channel.cheer", "1", serde_json::json!({
            "broadcaster_user_id": user_id.clone()
        })),
        ("channel.raid", "1", serde_json::json!({
            "to_broadcaster_user_id": user_id.clone()
        })),

        // ==================== CHANNEL - MODERATION ====================
        ("channel.ban", "1", serde_json::json!({
            "broadcaster_user_id": user_id.clone()
        })),
        ("channel.unban", "1", serde_json::json!({
            "broadcaster_user_id": user_id.clone()
        })),
        ("channel.unban_request.create", "1", serde_json::json!({
            "broadcaster_user_id": user_id.clone(),
            "moderator_user_id": user_id.clone()
        })),
        ("channel.unban_request.resolve", "1", serde_json::json!({
            "broadcaster_user_id": user_id.clone(),
            "moderator_user_id": user_id.clone()
        })),
        // channel.moderate v1 - Use v2 instead (includes warnings)
        ("channel.moderate", "2", serde_json::json!({
            "broadcaster_user_id": user_id.clone(),
            "moderator_user_id": user_id.clone()
        })),
        // channel.moderator.add/remove - May require specific app permissions, skip for now
        ("channel.suspicious_user.message", "1", serde_json::json!({
            "broadcaster_user_id": user_id.clone(),
            "moderator_user_id": user_id.clone()
        })),
        ("channel.suspicious_user.update", "1", serde_json::json!({
            "broadcaster_user_id": user_id.clone(),
            "moderator_user_id": user_id.clone()
        })),
        ("channel.vip.add", "1", serde_json::json!({
            "broadcaster_user_id": user_id.clone()
        })),
        ("channel.vip.remove", "1", serde_json::json!({
            "broadcaster_user_id": user_id.clone()
        })),
        ("channel.warning.acknowledge", "1", serde_json::json!({
            "broadcaster_user_id": user_id.clone(),
            "moderator_user_id": user_id.clone()
        })),
        ("channel.warning.send", "1", serde_json::json!({
            "broadcaster_user_id": user_id.clone(),
            "moderator_user_id": user_id.clone()
        })),

        // ==================== CHANNEL POINTS ====================
        ("channel.channel_points_automatic_reward_redemption.add", "1", serde_json::json!({
            "broadcaster_user_id": user_id.clone()
        })),
        ("channel.channel_points_custom_reward.add", "1", serde_json::json!({
            "broadcaster_user_id": user_id.clone()
        })),
        ("channel.channel_points_custom_reward.update", "1", serde_json::json!({
            "broadcaster_user_id": user_id.clone()
        })),
        ("channel.channel_points_custom_reward.remove", "1", serde_json::json!({
            "broadcaster_user_id": user_id.clone()
        })),
        ("channel.channel_points_custom_reward_redemption.add", "1", serde_json::json!({
            "broadcaster_user_id": user_id.clone()
        })),
        ("channel.channel_points_custom_reward_redemption.update", "1", serde_json::json!({
            "broadcaster_user_id": user_id.clone()
        })),

        // ==================== POLLS ====================
        ("channel.poll.begin", "1", serde_json::json!({
            "broadcaster_user_id": user_id.clone()
        })),
        ("channel.poll.progress", "1", serde_json::json!({
            "broadcaster_user_id": user_id.clone()
        })),
        ("channel.poll.end", "1", serde_json::json!({
            "broadcaster_user_id": user_id.clone()
        })),

        // ==================== PREDICTIONS ====================
        ("channel.prediction.begin", "1", serde_json::json!({
            "broadcaster_user_id": user_id.clone()
        })),
        ("channel.prediction.progress", "1", serde_json::json!({
            "broadcaster_user_id": user_id.clone()
        })),
        ("channel.prediction.lock", "1", serde_json::json!({
            "broadcaster_user_id": user_id.clone()
        })),
        ("channel.prediction.end", "1", serde_json::json!({
            "broadcaster_user_id": user_id.clone()
        })),

        // ==================== GOALS ====================
        // Goals may require specific channel features enabled
        ("channel.goal.begin", "1", serde_json::json!({
            "broadcaster_user_id": user_id.clone()
        })),
        ("channel.goal.progress", "1", serde_json::json!({
            "broadcaster_user_id": user_id.clone()
        })),
        ("channel.goal.end", "1", serde_json::json!({
            "broadcaster_user_id": user_id.clone()
        })),

        // ==================== HYPE TRAIN ====================
        // Use v2 (beta) for hype train events
        ("channel.hype_train.begin", "2", serde_json::json!({
            "broadcaster_user_id": user_id.clone()
        })),
        ("channel.hype_train.progress", "2", serde_json::json!({
            "broadcaster_user_id": user_id.clone()
        })),
        ("channel.hype_train.end", "2", serde_json::json!({
            "broadcaster_user_id": user_id.clone()
        })),

        // ==================== SHIELD MODE ====================
        ("channel.shield_mode.begin", "1", serde_json::json!({
            "broadcaster_user_id": user_id.clone(),
            "moderator_user_id": user_id.clone()
        })),
        ("channel.shield_mode.end", "1", serde_json::json!({
            "broadcaster_user_id": user_id.clone(),
            "moderator_user_id": user_id.clone()
        })),

        // ==================== SHOUTOUTS ====================
        ("channel.shoutout.create", "1", serde_json::json!({
            "broadcaster_user_id": user_id.clone(),
            "moderator_user_id": user_id.clone()
        })),
        ("channel.shoutout.receive", "1", serde_json::json!({
            "broadcaster_user_id": user_id.clone(),
            "moderator_user_id": user_id.clone()
        })),

        // ==================== CHARITY ====================
        ("channel.charity_campaign.donate", "1", serde_json::json!({
            "broadcaster_user_id": user_id.clone()
        })),
        ("channel.charity_campaign.start", "1", serde_json::json!({
            "broadcaster_user_id": user_id.clone()
        })),
        ("channel.charity_campaign.progress", "1", serde_json::json!({
            "broadcaster_user_id": user_id.clone()
        })),
        ("channel.charity_campaign.stop", "1", serde_json::json!({
            "broadcaster_user_id": user_id.clone()
        })),

        // ==================== STREAM ====================
        ("stream.online", "1", serde_json::json!({
            "broadcaster_user_id": user_id.clone()
        })),
        ("stream.offline", "1", serde_json::json!({
            "broadcaster_user_id": user_id.clone()
        })),

        // Note: drop.entitlement.grant requires is_batching_enabled=true and is rarely used
        // Note: User authorization and extension events require different setup
    ];

    let mut created_subs = Vec::new();
    let mut errors = Vec::new();

    // Create all subscriptions with delay to avoid rate limiting
    for (i, (event_type, version, condition)) in subscriptions_to_create.iter().enumerate() {
        // Add 100ms delay between requests to avoid rate limiting
        if i > 0 {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }

        match manager.create_subscription(event_type, version, condition.clone()).await {
            Ok(sub) => {
                // Store subscription in database
                let now = current_timestamp();
                if let Err(e) = conn.execute(
                    "INSERT INTO twitch_eventsub_subscriptions (subscription_id, subscription_type, condition, status, created_at)
                     VALUES (?1, ?2, ?3, ?4, ?5)",
                    rusqlite::params![
                        sub.id,
                        sub.subscription_type,
                        sub.condition.to_string(),
                        sub.status,
                        now
                    ],
                ) {
                    log::error!("Failed to store EventSub subscription: {}", e);
                }

                log::info!("[Twitch EventSub] ✅ Created subscription: {}", event_type);
                created_subs.push(sub);
            }
            Err(e) => {
                log::error!("[Twitch EventSub] ❌ Failed to create {} subscription: {}", event_type, e);
                errors.push(format!("{}: {}", event_type, e));
            }
        }
    }

    if !created_subs.is_empty() {
        json_response(&serde_json::json!({
            "success": true,
            "subscriptions": created_subs,
            "count": created_subs.len(),
            "errors": errors,
            "message": format!("Created {} EventSub subscriptions", created_subs.len())
        }))
    } else {
        error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to create any subscriptions. Errors: {}", errors.join(", ")))
    }
}

// ==================== Bot Authentication Handlers ====================

async fn handle_get_bot_auth_url(query: String) -> Response<BoxBody<Bytes, Infallible>> {
    let conn = crate::core::database::get_database_path();
    let conn = match rusqlite::Connection::open(&conn) {
        Ok(c) => c,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    // Get client_id and client_secret from config
    let client_id: Option<String> = match conn.query_row(
        "SELECT value FROM twitch_config WHERE key = 'client_id'",
        [],
        |row| row.get(0)
    ).optional() {
        Ok(v) => v,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    let client_secret: Option<String> = match conn.query_row(
        "SELECT value FROM twitch_config WHERE key = 'client_secret'",
        [],
        |row| row.get(0)
    ).optional() {
        Ok(v) => v,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    if client_id.is_none() || client_secret.is_none() {
        return error_response(StatusCode::BAD_REQUEST, "Client ID and Secret not configured");
    }

    // Bot uses different callback endpoint
    let redirect_uri = parse_query_param(&query, "redirect_uri")
        .unwrap_or_else(|| "http://localhost:3001/twitch/bot/auth/callback".to_string());

    // Bot only needs chat scopes
    let scopes_str = parse_query_param(&query, "scopes")
        .unwrap_or_else(|| "chat:read chat:edit".to_string());
    let scopes: Vec<&str> = scopes_str.split(' ').collect();

    let state = parse_query_param(&query, "state")
        .unwrap_or_else(|| "twitch_bot_oauth".to_string());

    let auth = super::auth::TwitchAuth::new(
        client_id.unwrap(),
        client_secret.unwrap(),
        redirect_uri,
    );

    let auth_url = auth.get_authorization_url(&scopes, &state);

    json_response(&serde_json::json!({ "url": auth_url }))
}

async fn handle_bot_auth_callback(query: String) -> Response<BoxBody<Bytes, Infallible>> {
    let code = match parse_query_param(&query, "code") {
        Some(c) => c,
        None => return error_response(StatusCode::BAD_REQUEST, "Missing code parameter"),
    };

    let conn = crate::core::database::get_database_path();
    let conn = match rusqlite::Connection::open(&conn) {
        Ok(c) => c,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    // Get client_id and client_secret from config
    let client_id: Option<String> = match conn.query_row(
        "SELECT value FROM twitch_config WHERE key = 'client_id'",
        [],
        |row| row.get(0)
    ).optional() {
        Ok(v) => v,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    let client_secret: Option<String> = match conn.query_row(
        "SELECT value FROM twitch_config WHERE key = 'client_secret'",
        [],
        |row| row.get(0)
    ).optional() {
        Ok(v) => v,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    if client_id.is_none() || client_secret.is_none() {
        return error_response(StatusCode::BAD_REQUEST, "Client ID and Secret not configured");
    }

    let redirect_uri = parse_query_param(&query, "redirect_uri")
        .unwrap_or_else(|| "http://localhost:3001/twitch/bot/auth/callback".to_string());

    let auth = super::auth::TwitchAuth::new(
        client_id.unwrap(),
        client_secret.unwrap(),
        redirect_uri,
    );

    // Exchange code for token
    let token_response = match auth.exchange_code_for_token(&code).await {
        Ok(t) => t,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to exchange code: {}", e)),
    };

    // Get user info to store user_id and username
    let api_client = super::api::TwitchApiClient::new(
        auth.client_id.clone(),
        token_response.access_token.clone(),
    );

    let user_info = match api_client.get_authenticated_user().await {
        Ok(info) => info,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to get user info: {}", e)),
    };

    // Extract user data
    let user_data = match user_info.get("data").and_then(|d| d.as_array()).and_then(|a| a.first()) {
        Some(u) => u,
        None => return error_response(StatusCode::INTERNAL_SERVER_ERROR, "Invalid user data"),
    };

    let user_id = match user_data.get("id").and_then(|v| v.as_str()) {
        Some(id) => id,
        None => return error_response(StatusCode::INTERNAL_SERVER_ERROR, "Missing user ID"),
    };

    let username = match user_data.get("login").and_then(|v| v.as_str()) {
        Some(name) => name,
        None => return error_response(StatusCode::INTERNAL_SERVER_ERROR, "Missing username"),
    };

    let profile_image_url = user_data.get("profile_image_url").and_then(|v| v.as_str());

    // Store bot auth in twitch_bot_auth table
    let now = current_timestamp();
    let expires_at = now + token_response.expires_in;
    let scopes_json = serde_json::to_string(&token_response.scope).unwrap_or_default();

    if let Err(e) = conn.execute(
        "INSERT OR REPLACE INTO twitch_bot_auth (user_id, username, access_token, refresh_token, expires_at, scopes, profile_image_url, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        rusqlite::params![user_id, username, token_response.access_token, token_response.refresh_token, expires_at, scopes_json, profile_image_url, now, now],
    ) {
        return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string());
    }

    log::info!("[Twitch Bot Auth] Successfully authenticated bot account: {}", username);

    json_response(&serde_json::json!({
        "success": true,
        "user_id": user_id,
        "username": username,
        "scopes": token_response.scope
    }))
}

async fn handle_revoke_bot_token() -> Response<BoxBody<Bytes, Infallible>> {
    let conn = crate::core::database::get_database_path();
    let conn = match rusqlite::Connection::open(&conn) {
        Ok(c) => c,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    // Get current bot token
    let access_token: String = match conn.query_row(
        "SELECT access_token FROM twitch_bot_auth LIMIT 1",
        [],
        |row| row.get(0)
    ).optional() {
        Ok(Some(t)) => t,
        Ok(None) => return error_response(StatusCode::NOT_FOUND, "No bot authentication found"),
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    // Get client_id to revoke token
    let client_id: Option<String> = match conn.query_row(
        "SELECT value FROM twitch_config WHERE key = 'client_id'",
        [],
        |row| row.get(0)
    ).optional() {
        Ok(v) => v,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    let auth = super::auth::TwitchAuth::new(
        client_id.unwrap_or_default(),
        "".to_string(),
        "".to_string(),
    );

    // Revoke the token (ignore errors if token is already invalid)
    match auth.revoke_token(&access_token).await {
        Ok(_) => log::info!("[Twitch Bot Auth] Bot token revoked successfully"),
        Err(e) => log::warn!("[Twitch Bot Auth] Could not revoke bot token (may already be invalid): {}", e),
    }

    // Delete from database regardless of revoke result
    if let Err(e) = conn.execute("DELETE FROM twitch_bot_auth", []) {
        return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string());
    }

    json_response(&serde_json::json!({ "success": true }))
}

async fn handle_bot_auth_status() -> Response<BoxBody<Bytes, Infallible>> {
    let conn = crate::core::database::get_database_path();
    let conn = match rusqlite::Connection::open(&conn) {
        Ok(c) => c,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    let auth_data: Option<(String, String, i64, String, Option<String>)> = match conn.query_row(
        "SELECT user_id, username, expires_at, scopes, profile_image_url FROM twitch_bot_auth LIMIT 1",
        [],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?))
    ).optional() {
        Ok(v) => v,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    match auth_data {
        Some((user_id, username, expires_at, scopes_json, profile_image_url)) => {
            let now = current_timestamp();
            let is_expired = expires_at <= now;
            let scopes: Vec<String> = serde_json::from_str(&scopes_json).unwrap_or_default();

            json_response(&serde_json::json!({
                "authenticated": true,
                "user_id": user_id,
                "username": username,
                "expires_at": expires_at,
                "is_expired": is_expired,
                "scopes": scopes,
                "profile_image_url": profile_image_url
            }))
        }
        None => {
            json_response(&serde_json::json!({
                "authenticated": false
            }))
        }
    }
}

// Ngrok management handlers
async fn handle_ngrok_start() -> Response<BoxBody<Bytes, Infallible>> {
    match NGROK_MANAGER.start(3001).await {
        Ok(_) => {
            // Wait a bit for ngrok to fully start
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            
            match NGROK_MANAGER.get_public_url().await {
                Ok(url) => json_response(&serde_json::json!({
                    "success": true,
                    "public_url": url,
                    "message": "Ngrok tunnel started successfully"
                })),
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, 
                    &format!("Ngrok started but failed to get URL: {}", e))
            }
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, 
            &format!("Failed to start ngrok: {}", e))
    }
}

async fn handle_ngrok_stop() -> Response<BoxBody<Bytes, Infallible>> {
    match NGROK_MANAGER.stop().await {
        Ok(_) => json_response(&serde_json::json!({
            "success": true,
            "message": "Ngrok tunnel stopped"
        })),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, 
            &format!("Failed to stop ngrok: {}", e))
    }
}

async fn handle_ngrok_status() -> Response<BoxBody<Bytes, Infallible>> {
    match NGROK_MANAGER.get_status().await {
        Ok(status) => json_response(&status),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, 
            &format!("Failed to get ngrok status: {}", e))
    }
}
