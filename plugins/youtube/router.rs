use crate::core::plugin_context::PluginContext;
use crate::core::plugin_router::PluginRouter;
use anyhow::Result;
use hyper::{Method, Response, StatusCode};
use hyper::body::Bytes;
use http_body_util::{Full, combinators::BoxBody, BodyExt};
use std::convert::Infallible;
use rusqlite::OptionalExtension;
use serde_json::json;

pub async fn register_routes(ctx: &PluginContext) -> Result<()> {
    let mut router = PluginRouter::new();

    // GET /youtube/auth/url - Get OAuth authorization URL
    router.route(Method::GET, "/auth/url", |_path, query, _req| {
        Box::pin(async move {
            handle_get_auth_url(query).await
        })
    });

    // GET /youtube/auth/callback - OAuth callback endpoint
    router.route(Method::GET, "/auth/callback", |_path, query, _req| {
        Box::pin(async move {
            handle_auth_callback(query).await
        })
    });

    // POST /youtube/auth/refresh - Refresh access token
    router.route(Method::POST, "/auth/refresh", |_path, _query, _req| {
        Box::pin(async move {
            handle_refresh_token().await
        })
    });

    // POST /youtube/auth/revoke - Revoke access token
    router.route(Method::POST, "/auth/revoke", |_path, _query, _req| {
        Box::pin(async move {
            handle_revoke_token().await
        })
    });

    // GET /youtube/auth/status - Get authentication status
    router.route(Method::GET, "/auth/status", |_path, _query, _req| {
        Box::pin(async move {
            handle_auth_status().await
        })
    });

    // GET /youtube/channels - Get authenticated user's channels
    router.route(Method::GET, "/channels", |_path, _query, _req| {
        Box::pin(async move {
            handle_get_channels().await
        })
    });

    // GET /youtube/channels/:id - Get specific channel
    router.route(Method::GET, "/channels/:id", |path, _query, _req| {
        Box::pin(async move {
            handle_get_channel(path).await
        })
    });

    // GET /youtube/analytics/:channel_id - Get channel analytics
    router.route(Method::GET, "/analytics/:channel_id", |path, query, _req| {
        Box::pin(async move {
            handle_get_analytics(path, query).await
        })
    });

    // GET /youtube/analytics/:channel_id/report - Get detailed analytics report
    router.route(Method::GET, "/analytics/:channel_id/report", |path, query, _req| {
        Box::pin(async move {
            handle_get_analytics_report(path, query).await
        })
    });

    ctx.register_router("youtube", router).await;

    log::info!("[YouTube] Routes registered");
    Ok(())
}

fn parse_query(query: String) -> std::collections::HashMap<String, String> {
    query
        .split('&')
        .filter_map(|pair| {
            let mut parts = pair.splitn(2, '=');
            let key = parts.next()?.to_string();
            let value = parts.next().unwrap_or("").to_string();
            Some((key, urlencoding::decode(&value).ok()?.to_string()))
        })
        .collect()
}

fn parse_path(path: String) -> std::collections::HashMap<String, String> {
    let parts: Vec<&str> = path.split('/').collect();
    let mut params = std::collections::HashMap::new();

    // Extract path parameters based on position
    if parts.len() >= 3 {
        params.insert("id".to_string(), parts[2].to_string());
    }
    if parts.len() >= 4 {
        params.insert("channel_id".to_string(), parts[2].to_string());
    }

    params
}

async fn handle_get_auth_url(query: String) -> Response<BoxBody<Bytes, Infallible>> {
    let query_params = parse_query(query);
    let db_path = crate::core::database::get_database_path();
    let conn = match rusqlite::Connection::open(db_path) {
        Ok(c) => c,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    // Get client_id from config
    let client_id: Option<String> = match conn
        .query_row(
            "SELECT value FROM config WHERE key = ?",
            ["youtube_client_id"],
            |row| row.get(0),
        )
        .optional()
    {
        Ok(id) => id,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    if client_id.is_none() {
        return error_response(StatusCode::BAD_REQUEST, "YouTube client_id not configured");
    }

    let state = query_params.get("state").cloned();
    let auth_url = super::auth::get_auth_url(&client_id.unwrap(), state);

    json_response(&json!({ "url": auth_url }))
}

async fn handle_auth_callback(query: String) -> Response<BoxBody<Bytes, Infallible>> {
    let query_params = parse_query(query);
    let code = match query_params.get("code") {
        Some(c) => c,
        None => return error_response(StatusCode::BAD_REQUEST, "Missing authorization code"),
    };

    let db_path = crate::core::database::get_database_path();
    let conn = match rusqlite::Connection::open(&db_path) {
        Ok(c) => c,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    // Get credentials from config
    let client_id: Option<String> = match conn
        .query_row(
            "SELECT value FROM config WHERE key = ?",
            ["youtube_client_id"],
            |row| row.get(0),
        )
        .optional()
    {
        Ok(id) => id,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    let client_secret: Option<String> = match conn
        .query_row(
            "SELECT value FROM config WHERE key = ?",
            ["youtube_client_secret"],
            |row| row.get(0),
        )
        .optional()
    {
        Ok(secret) => secret,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    if client_id.is_none() || client_secret.is_none() {
        return error_response(StatusCode::BAD_REQUEST, "YouTube credentials not configured");
    }

    drop(conn);

    // Exchange code for token
    let token_response = match super::auth::exchange_code_for_token(
        code,
        &client_id.unwrap(),
        &client_secret.unwrap(),
    )
    .await
    {
        Ok(t) => t,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    // Get user info
    let user_info = match super::auth::get_user_info(&token_response.access_token).await {
        Ok(u) => u,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    // Calculate expiry time
    let expires_at = super::auth::current_timestamp() + token_response.expires_in;

    // Store auth in database
    let conn = match rusqlite::Connection::open(&db_path) {
        Ok(c) => c,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };
    let now = super::auth::current_timestamp();

    if let Err(e) = conn.execute(
        "INSERT OR REPLACE INTO youtube_auth (
            user_id, access_token, refresh_token, expires_at, scopes, created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?)",
        rusqlite::params![
            user_info.id,
            token_response.access_token,
            token_response.refresh_token.unwrap_or_default(),
            expires_at,
            token_response.scope,
            now,
            now,
        ],
    ) {
        return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string());
    }

    // Return HTML page that closes the popup
    let html = format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>YouTube Authentication Success</title>
    <style>
        body {{
            font-family: system-ui, -apple-system, sans-serif;
            display: flex;
            align-items: center;
            justify-content: center;
            height: 100vh;
            margin: 0;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
        }}
        .container {{
            background: white;
            padding: 2rem;
            border-radius: 8px;
            box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
            text-align: center;
        }}
        .success {{
            color: #10b981;
            font-size: 3rem;
            margin-bottom: 1rem;
        }}
        h1 {{
            color: #1f2937;
            margin: 0 0 0.5rem 0;
        }}
        p {{
            color: #6b7280;
            margin: 0;
        }}
    </style>
</head>
<body>
    <div class="container">
        <div class="success">✓</div>
        <h1>Authentication Successful!</h1>
        <p>You can close this window now.</p>
    </div>
    <script>
        // Close the popup after 2 seconds
        setTimeout(() => {{
            window.close();
        }}, 2000);
    </script>
</body>
</html>"#
    );

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/html")
        .header("Access-Control-Allow-Origin", "*")
        .body(Full::new(Bytes::from(html)).map_err(|_| unreachable!()).boxed())
        .unwrap()
}

async fn handle_refresh_token() -> Response<BoxBody<Bytes, Infallible>> {
    let db_path = crate::core::database::get_database_path();
    let conn = match rusqlite::Connection::open(&db_path) {
        Ok(c) => c,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    // Get current auth
    let auth: Option<(String, String)> = match conn
        .query_row(
            "SELECT user_id, refresh_token FROM youtube_auth LIMIT 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()
    {
        Ok(a) => a,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    if auth.is_none() {
        return error_response(StatusCode::UNAUTHORIZED, "Not authenticated");
    }

    let (user_id, refresh_token) = auth.unwrap();

    // Get credentials
    let client_id: Option<String> = match conn
        .query_row(
            "SELECT value FROM config WHERE key = ?",
            ["youtube_client_id"],
            |row| row.get(0),
        )
        .optional()
    {
        Ok(id) => id,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    let client_secret: Option<String> = match conn
        .query_row(
            "SELECT value FROM config WHERE key = ?",
            ["youtube_client_secret"],
            |row| row.get(0),
        )
        .optional()
    {
        Ok(secret) => secret,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    drop(conn);

    if client_id.is_none() || client_secret.is_none() {
        return error_response(StatusCode::BAD_REQUEST, "YouTube credentials not configured");
    }

    // Refresh token
    let token_response = match super::auth::refresh_access_token(
        &refresh_token,
        &client_id.unwrap(),
        &client_secret.unwrap(),
    )
    .await
    {
        Ok(t) => t,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    let expires_at = super::auth::current_timestamp() + token_response.expires_in;
    let now = super::auth::current_timestamp();

    // Update auth in database
    let conn = match rusqlite::Connection::open(&db_path) {
        Ok(c) => c,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    if let Err(e) = conn.execute(
        "UPDATE youtube_auth SET access_token = ?, expires_at = ?, updated_at = ? WHERE user_id = ?",
        rusqlite::params![token_response.access_token, expires_at, now, user_id],
    ) {
        return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string());
    }

    json_response(&json!({ "success": true }))
}

async fn handle_revoke_token() -> Response<BoxBody<Bytes, Infallible>> {
    let db_path = crate::core::database::get_database_path();
    let conn = match rusqlite::Connection::open(&db_path) {
        Ok(c) => c,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    let access_token: Option<String> = match conn
        .query_row(
            "SELECT access_token FROM youtube_auth LIMIT 1",
            [],
            |row| row.get(0),
        )
        .optional()
    {
        Ok(t) => t,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    if let Some(token) = access_token {
        drop(conn);
        if let Err(e) = super::auth::revoke_token(&token).await {
            return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string());
        }

        let conn = match rusqlite::Connection::open(&db_path) {
            Ok(c) => c,
            Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
        };
        if let Err(e) = conn.execute("DELETE FROM youtube_auth", []) {
            return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string());
        }
    }

    json_response(&json!({ "success": true }))
}

async fn handle_auth_status() -> Response<BoxBody<Bytes, Infallible>> {
    let db_path = crate::core::database::get_database_path();
    let conn = match rusqlite::Connection::open(db_path) {
        Ok(c) => c,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    let auth: Option<(String, i64)> = match conn
        .query_row(
            "SELECT user_id, expires_at FROM youtube_auth LIMIT 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()
    {
        Ok(a) => a,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    match auth {
        Some((user_id, expires_at)) => {
            let is_expired = super::auth::is_token_expired(expires_at);
            json_response(&json!({
                "authenticated": !is_expired,
                "user_id": user_id,
                "expires_at": expires_at,
            }))
        }
        None => json_response(&json!({ "authenticated": false })),
    }
}

async fn handle_get_channels() -> Response<BoxBody<Bytes, Infallible>> {
    let access_token = match get_valid_access_token().await {
        Ok(Some(token)) => token,
        Ok(None) => return error_response(StatusCode::UNAUTHORIZED, "Not authenticated"),
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    let channels = match super::api::get_my_channels(&access_token).await {
        Ok(c) => c,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    // Store channels in database
    let db_path = crate::core::database::get_database_path();
    let conn = match rusqlite::Connection::open(db_path) {
        Ok(c) => c,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };
    let now = super::auth::current_timestamp();

    for channel in &channels {
        if let Err(e) = conn.execute(
            "INSERT OR REPLACE INTO youtube_channels (
                channel_id, channel_title, description, custom_url, thumbnail_url,
                subscriber_count, video_count, view_count, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
            rusqlite::params![
                channel.id,
                channel.title,
                channel.description,
                channel.custom_url,
                channel.thumbnail_url,
                channel.subscriber_count,
                channel.video_count,
                channel.view_count,
                now,
            ],
        ) {
            return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string());
        }
    }

    json_response(&json!({ "channels": channels }))
}

async fn handle_get_channel(path: String) -> Response<BoxBody<Bytes, Infallible>> {
    let path_params = parse_path(path);
    let channel_id = match path_params.get("id") {
        Some(id) => id,
        None => return error_response(StatusCode::BAD_REQUEST, "Missing channel ID"),
    };

    let access_token = match get_valid_access_token().await {
        Ok(Some(token)) => token,
        Ok(None) => return error_response(StatusCode::UNAUTHORIZED, "Not authenticated"),
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    let channel = match super::api::get_channel_by_id(&access_token, channel_id).await {
        Ok(c) => c,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    json_response(&json!({ "channel": channel }))
}

async fn handle_get_analytics(path: String, query: String) -> Response<BoxBody<Bytes, Infallible>> {
    let path_params = parse_path(path);
    let query_params = parse_query(query);

    let channel_id = match path_params.get("id").or(path_params.get("channel_id")) {
        Some(id) => id,
        None => return error_response(StatusCode::BAD_REQUEST, "Missing channel ID"),
    };

    let start_date = query_params.get("start_date").cloned().unwrap_or_else(|| {
        // Default to last 30 days
        chrono::Utc::now()
            .checked_sub_signed(chrono::Duration::days(30))
            .unwrap()
            .format("%Y-%m-%d")
            .to_string()
    });

    let end_date = query_params.get("end_date").cloned().unwrap_or_else(|| {
        chrono::Utc::now().format("%Y-%m-%d").to_string()
    });

    let access_token = match get_valid_access_token().await {
        Ok(Some(token)) => token,
        Ok(None) => return error_response(StatusCode::UNAUTHORIZED, "Not authenticated"),
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    let analytics = match super::api::get_channel_analytics(&access_token, channel_id, &start_date, &end_date).await {
        Ok(a) => a,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    json_response(&json!({ "analytics": analytics }))
}

async fn handle_get_analytics_report(path: String, query: String) -> Response<BoxBody<Bytes, Infallible>> {
    let path_params = parse_path(path);
    let query_params = parse_query(query);

    let channel_id = match path_params.get("id").or(path_params.get("channel_id")) {
        Some(id) => id,
        None => return error_response(StatusCode::BAD_REQUEST, "Missing channel ID"),
    };

    let start_date = match query_params.get("start_date") {
        Some(d) => d.clone(),
        None => return error_response(StatusCode::BAD_REQUEST, "Missing start_date"),
    };

    let end_date = match query_params.get("end_date") {
        Some(d) => d.clone(),
        None => return error_response(StatusCode::BAD_REQUEST, "Missing end_date"),
    };

    let metrics = match query_params.get("metrics") {
        Some(m) => m.split(',').map(|s| s.to_string()).collect(),
        None => return error_response(StatusCode::BAD_REQUEST, "Missing metrics"),
    };

    let dimensions = query_params.get("dimensions").map(|d| {
        d.split(',').map(|s| s.to_string()).collect()
    });

    let analytics_query = super::api::AnalyticsQuery {
        start_date,
        end_date,
        metrics,
        dimensions,
    };

    let access_token = match get_valid_access_token().await {
        Ok(Some(token)) => token,
        Ok(None) => return error_response(StatusCode::UNAUTHORIZED, "Not authenticated"),
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    let report = match super::api::get_analytics_report(&access_token, channel_id, &analytics_query).await {
        Ok(r) => r,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    json_response(&report)
}

// Helper function to get a valid access token
async fn get_valid_access_token() -> Result<Option<String>> {
    let db_path = crate::core::database::get_database_path();
    let conn = rusqlite::Connection::open(&db_path)?;

    let auth: Option<(String, String, i64)> = conn
        .query_row(
            "SELECT user_id, access_token, expires_at FROM youtube_auth LIMIT 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .optional()?;

    match auth {
        Some((user_id, access_token, expires_at)) => {
            if super::auth::is_token_expired(expires_at) {
                // Token expired, need to refresh
                drop(conn);
                // Trigger refresh
                let _ = handle_refresh_token().await;

                // Get new token
                let conn = rusqlite::Connection::open(&db_path)?;
                let new_token: String = conn.query_row(
                    "SELECT access_token FROM youtube_auth WHERE user_id = ?",
                    [&user_id],
                    |row| row.get(0),
                )?;
                Ok(Some(new_token))
            } else {
                Ok(Some(access_token))
            }
        }
        None => Ok(None),
    }
}

fn json_response<T: serde::Serialize>(body: &T) -> Response<BoxBody<Bytes, Infallible>> {
    let json = serde_json::to_string(body).unwrap();
    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .header("Access-Control-Allow-Origin", "*")
        .header("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE, OPTIONS")
        .header("Access-Control-Allow-Headers", "Content-Type, Authorization")
        .body(Full::new(Bytes::from(json)).map_err(|_| unreachable!()).boxed())
        .unwrap()
}

fn error_response(status: StatusCode, message: &str) -> Response<BoxBody<Bytes, Infallible>> {
    let json = json!({ "error": message });
    let json_str = serde_json::to_string(&json).unwrap();
    Response::builder()
        .status(status)
        .header("Content-Type", "application/json")
        .header("Access-Control-Allow-Origin", "*")
        .header("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE, OPTIONS")
        .header("Access-Control-Allow-Headers", "Content-Type, Authorization")
        .body(Full::new(Bytes::from(json_str)).map_err(|_| unreachable!()).boxed())
        .unwrap()
}
