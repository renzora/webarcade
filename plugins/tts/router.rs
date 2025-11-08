use crate::core::plugin_context::PluginContext;
use crate::core::plugin_router::PluginRouter;
use anyhow::Result;
use hyper::{Method, Request, Response, StatusCode, body::Incoming};
use hyper::body::Bytes;
use http_body_util::{Full, combinators::BoxBody};
use std::convert::Infallible;

pub async fn register_routes(ctx: &PluginContext) -> Result<()> {
    let mut router = PluginRouter::new();

    // POST /tts/speak - Add to TTS queue
    router.route(Method::POST, "/speak", |_path, _query, req| {
        Box::pin(async move {
            handle_speak(req).await
        })
    });

    // GET /tts/next - Get next from queue
    router.route(Method::GET, "/next", |_path, _query, _req| {
        Box::pin(async move {
            handle_get_next().await
        })
    });

    // POST /tts/complete - Mark TTS complete
    router.route(Method::POST, "/complete", |_path, _query, req| {
        Box::pin(async move {
            handle_complete(req).await
        })
    });

    // GET /tts/queue/status - Get queue status
    router.route(Method::GET, "/queue/status", |_path, _query, _req| {
        Box::pin(async move {
            handle_get_queue_status().await
        })
    });

    // GET /tts/voices - Get available voices
    router.route(Method::GET, "/voices", |_path, _query, _req| {
        Box::pin(async move {
            handle_get_voices().await
        })
    });

    // POST /tts/settings - Update TTS settings
    router.route(Method::POST, "/settings", |_path, _query, req| {
        Box::pin(async move {
            handle_update_setting(req).await
        })
    });

    // GET /tts/settings?channel=xxx - Get TTS settings with query param
    router.route(Method::GET, "/settings", |_path, query, _req| {
        Box::pin(async move {
            handle_get_settings_query(query).await
        })
    });

    // GET /tts/whitelist?channel=xxx - Get whitelist users with query param
    router.route(Method::GET, "/whitelist", |_path, query, _req| {
        Box::pin(async move {
            handle_get_whitelist_query(query).await
        })
    });

    // GET /tts/whitelist/:channel - Get whitelist users (path param)
    router.route(Method::GET, "/whitelist/:channel", |path, _query, _req| {
        Box::pin(async move {
            handle_get_whitelist_users(path).await
        })
    });

    // POST /tts/whitelist/add - Add whitelist user
    router.route(Method::POST, "/whitelist/add", |_path, _query, req| {
        Box::pin(async move {
            handle_add_whitelist_user(req).await
        })
    });

    // POST /tts/whitelist/remove - Remove whitelist user
    router.route(Method::POST, "/whitelist/remove", |_path, _query, req| {
        Box::pin(async move {
            handle_remove_whitelist_user(req).await
        })
    });

    // GET /tts/channel/:channel/settings - Get channel settings
    router.route(Method::GET, "/channel/:channel/settings", |path, _query, _req| {
        Box::pin(async move {
            handle_get_channel_settings(path).await
        })
    });

    // POST /tts/channel/settings - Update channel settings
    router.route(Method::POST, "/channel/settings", |_path, _query, req| {
        Box::pin(async move {
            handle_update_channel_settings(req).await
        })
    });

    ctx.register_router("tts", router).await;
    Ok(())
}

async fn handle_speak(req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    match read_json_body(req).await {
        Ok(body) => {
            let text = match body.get("text").and_then(|v| v.as_str()) {
                Some(t) => t,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing text"),
            };
            let voice = body.get("voice").and_then(|v| v.as_str()).unwrap_or("default");
            let priority = body.get("priority").and_then(|v| v.as_i64()).unwrap_or(0);
            let requested_by = body.get("requested_by").and_then(|v| v.as_str());

            let conn = crate::core::database::get_database_path();
            match rusqlite::Connection::open(&conn) {
                Ok(conn) => {
                    let now = super::current_timestamp();
                    match conn.execute(
                        "INSERT INTO tts_queue (text, voice, priority, status, requested_by, created_at)
                         VALUES (?1, ?2, ?3, 'pending', ?4, ?5)",
                        rusqlite::params![text, voice, priority, requested_by, now],
                    ) {
                        Ok(_) => {
                            let id = conn.last_insert_rowid();
                            json_response(&serde_json::json!({ "id": id, "success": true }))
                        }
                        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
                    }
                }
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::BAD_REQUEST, &e),
    }
}

async fn handle_get_next() -> Response<BoxBody<Bytes, Infallible>> {
    use rusqlite::OptionalExtension;

    let conn = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&conn) {
        Ok(conn) => {
            let result: Result<Option<serde_json::Value>, _> = conn.query_row(
                "SELECT id, text, voice, requested_by FROM tts_queue
                 WHERE status = 'pending'
                 ORDER BY priority DESC, created_at ASC
                 LIMIT 1",
                [],
                |row| {
                    Ok(serde_json::json!({
                        "id": row.get::<_, i64>(0)?,
                        "text": row.get::<_, String>(1)?,
                        "voice": row.get::<_, String>(2)?,
                        "requested_by": row.get::<_, Option<String>>(3)?,
                    }))
                }
            ).optional();

            match result {
                Ok(item) => {
                    // Mark as processing
                    if let Some(ref i) = item {
                        if let Some(id) = i["id"].as_i64() {
                            let now = super::current_timestamp();
                            let _ = conn.execute(
                                "UPDATE tts_queue SET status = 'processing', started_at = ?1 WHERE id = ?2",
                                rusqlite::params![now, id],
                            );
                        }
                    }
                    json_response(&serde_json::json!({ "item": item }))
                }
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn handle_complete(req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    match read_json_body(req).await {
        Ok(body) => {
            let id = match body.get("id").and_then(|v| v.as_i64()) {
                Some(i) => i,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing id"),
            };
            let duration_ms = body.get("duration_ms").and_then(|v| v.as_i64());

            let conn = crate::core::database::get_database_path();
            match rusqlite::Connection::open(&conn) {
                Ok(conn) => {
                    let now = super::current_timestamp();

                    // Get item details before updating
                    let item_result: Result<(String, String, Option<String>), _> = conn.query_row(
                        "SELECT text, voice, requested_by FROM tts_queue WHERE id = ?1",
                        rusqlite::params![id],
                        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
                    );

                    match item_result {
                        Ok((text, voice, requested_by)) => {
                            // Mark as complete
                            if let Err(e) = conn.execute(
                                "UPDATE tts_queue SET status = 'completed', completed_at = ?1 WHERE id = ?2",
                                rusqlite::params![now, id],
                            ) {
                                return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string());
                            }

                            // Add to history
                            if let Err(e) = conn.execute(
                                "INSERT INTO tts_history (text, voice, requested_by, duration_ms, created_at)
                                 VALUES (?1, ?2, ?3, ?4, ?5)",
                                rusqlite::params![text, voice, requested_by, duration_ms, now],
                            ) {
                                return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string());
                            }

                            // Clean up old queue items (completed > 1 hour ago)
                            let one_hour_ago = now - 3600;
                            let _ = conn.execute(
                                "DELETE FROM tts_queue WHERE status = 'completed' AND completed_at < ?1",
                                rusqlite::params![one_hour_ago],
                            );

                            json_response(&serde_json::json!({ "success": true }))
                        }
                        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
                    }
                }
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::BAD_REQUEST, &e),
    }
}

async fn handle_get_queue_status() -> Response<BoxBody<Bytes, Infallible>> {
    let conn = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&conn) {
        Ok(conn) => {
            let pending: Result<i64, _> = conn.query_row(
                "SELECT COUNT(*) FROM tts_queue WHERE status = 'pending'",
                [],
                |row| row.get(0),
            );

            let processing: Result<i64, _> = conn.query_row(
                "SELECT COUNT(*) FROM tts_queue WHERE status = 'processing'",
                [],
                |row| row.get(0),
            );

            match (pending, processing) {
                (Ok(p), Ok(pr)) => json_response(&serde_json::json!({
                    "pending": p,
                    "processing": pr,
                    "total": p + pr
                })),
                _ => error_response(StatusCode::INTERNAL_SERVER_ERROR, "Failed to get queue status"),
            }
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn handle_get_voices() -> Response<BoxBody<Bytes, Infallible>> {
    let conn = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&conn) {
        Ok(conn) => {
            let mut stmt = match conn.prepare(
                "SELECT voice_id, voice_name, language, engine FROM tts_voices WHERE enabled = 1"
            ) {
                Ok(stmt) => stmt,
                Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            };

            let mapped = match stmt.query_map([], |row| {
                Ok(serde_json::json!({
                    "voice_id": row.get::<_, String>(0)?,
                    "voice_name": row.get::<_, String>(1)?,
                    "language": row.get::<_, String>(2)?,
                    "engine": row.get::<_, String>(3)?,
                }))
            }) {
                Ok(m) => m,
                Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            };

            let voices_result: Result<Vec<_>, _> = mapped.collect();

            match voices_result {
                Ok(v) => json_response(&serde_json::json!({ "voices": v })),
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn handle_update_setting(req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    match read_json_body(req).await {
        Ok(body) => {
            // Log the received body for debugging
            log::info!("[TTS] POST /settings received body: {}", serde_json::to_string(&body).unwrap_or_default());

            // Check if this is a channel settings update (has channel, enabled, mode)
            if body.get("channel").is_some() {
                // This is a channel settings update
                let channel = match body.get("channel").and_then(|v| v.as_str()) {
                    Some(c) => c,
                    None => return error_response(StatusCode::BAD_REQUEST, "Missing channel"),
                };

                // Handle enabled as boolean, number, or string
                let enabled = match body.get("enabled") {
                    Some(v) => {
                        if let Some(b) = v.as_bool() {
                            b
                        } else if let Some(n) = v.as_i64() {
                            n != 0
                        } else if let Some(s) = v.as_str() {
                            s == "true" || s == "1"
                        } else {
                            return error_response(StatusCode::BAD_REQUEST, "Invalid enabled value");
                        }
                    }
                    None => return error_response(StatusCode::BAD_REQUEST, "Missing enabled"),
                };

                let mode = match body.get("mode").and_then(|v| v.as_str()) {
                    Some(m) => m,
                    None => return error_response(StatusCode::BAD_REQUEST, "Missing mode"),
                };

                let conn = crate::core::database::get_database_path();
                match rusqlite::Connection::open(&conn) {
                    Ok(conn) => {
                        match conn.execute(
                            "INSERT OR REPLACE INTO tts_channel_settings (channel, enabled, mode)
                             VALUES (?1, ?2, ?3)",
                            rusqlite::params![channel, enabled as i64, mode],
                        ) {
                            Ok(_) => json_response(&serde_json::json!({ "success": true })),
                            Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
                        }
                    }
                    Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
                }
            } else {
                // This is a global settings update (key-value)
                let key = match body.get("key").and_then(|v| v.as_str()) {
                    Some(k) => k,
                    None => return error_response(StatusCode::BAD_REQUEST, "Missing key"),
                };
                let value = match body.get("value").and_then(|v| v.as_str()) {
                    Some(v) => v,
                    None => return error_response(StatusCode::BAD_REQUEST, "Missing value"),
                };

                let conn = crate::core::database::get_database_path();
                match rusqlite::Connection::open(&conn) {
                    Ok(conn) => {
                        let now = super::current_timestamp();
                        match conn.execute(
                            "INSERT OR REPLACE INTO tts_settings (key, value, updated_at) VALUES (?1, ?2, ?3)",
                            rusqlite::params![key, value, now],
                        ) {
                            Ok(_) => json_response(&serde_json::json!({ "success": true })),
                            Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
                        }
                    }
                    Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
                }
            }
        }
        Err(e) => error_response(StatusCode::BAD_REQUEST, &e),
    }
}

async fn handle_get_settings() -> Response<BoxBody<Bytes, Infallible>> {
    let conn = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&conn) {
        Ok(conn) => {
            let mut stmt = match conn.prepare("SELECT key, value FROM tts_settings") {
                Ok(stmt) => stmt,
                Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            };

            let mapped = match stmt.query_map([], |row| {
                Ok((row.get(0)?, row.get(1)?))
            }) {
                Ok(m) => m,
                Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            };

            let settings_result: Result<Vec<_>, _> = mapped.collect();

            match settings_result {
                Ok(s) => {
                    let mut settings_map = serde_json::Map::new();
                    for (key, value) in s {
                        settings_map.insert(key, serde_json::Value::String(value));
                    }
                    json_response(&serde_json::json!({ "settings": settings_map }))
                }
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn handle_get_whitelist_users(path: String) -> Response<BoxBody<Bytes, Infallible>> {
    let channel = extract_path_param(&path, "/whitelist/");
    if channel.is_empty() {
        return error_response(StatusCode::BAD_REQUEST, "Missing channel");
    }

    let conn = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&conn) {
        Ok(conn) => {
            let mut stmt = match conn.prepare(
                "SELECT username FROM tts_whitelist WHERE channel = ?1 ORDER BY username ASC"
            ) {
                Ok(stmt) => stmt,
                Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            };

            let mapped = match stmt.query_map([&channel], |row| {
                row.get::<_, String>(0)
            }) {
                Ok(m) => m,
                Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            };

            let users_result: Result<Vec<_>, _> = mapped.collect();

            match users_result {
                Ok(u) => json_response(&serde_json::json!(u)),
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn handle_add_whitelist_user(req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    match read_json_body(req).await {
        Ok(body) => {
            let channel = match body.get("channel").and_then(|v| v.as_str()) {
                Some(c) => c,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing channel"),
            };
            let username = match body.get("username").and_then(|v| v.as_str()) {
                Some(u) => u,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing username"),
            };

            let conn = crate::core::database::get_database_path();
            match rusqlite::Connection::open(&conn) {
                Ok(conn) => {
                    let now = super::current_timestamp();
                    match conn.execute(
                        "INSERT OR IGNORE INTO tts_whitelist (channel, username, created_at) VALUES (?1, ?2, ?3)",
                        rusqlite::params![channel, username, now],
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

async fn handle_remove_whitelist_user(req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    match read_json_body(req).await {
        Ok(body) => {
            let channel = match body.get("channel").and_then(|v| v.as_str()) {
                Some(c) => c,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing channel"),
            };
            let username = match body.get("username").and_then(|v| v.as_str()) {
                Some(u) => u,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing username"),
            };

            let conn = crate::core::database::get_database_path();
            match rusqlite::Connection::open(&conn) {
                Ok(conn) => {
                    match conn.execute(
                        "DELETE FROM tts_whitelist WHERE channel = ?1 AND username = ?2",
                        rusqlite::params![channel, username],
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

async fn handle_get_channel_settings(path: String) -> Response<BoxBody<Bytes, Infallible>> {
    use rusqlite::OptionalExtension;

    let channel = extract_path_param(&path, "/channel/");
    let channel = channel.strip_suffix("/settings").unwrap_or(&channel);

    if channel.is_empty() {
        return error_response(StatusCode::BAD_REQUEST, "Missing channel");
    }

    let conn = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&conn) {
        Ok(conn) => {
            let result: Result<Option<(i64, String)>, _> = conn.query_row(
                "SELECT enabled, mode FROM tts_channel_settings WHERE channel = ?1",
                [&channel],
                |row| Ok((row.get(0)?, row.get(1)?))
            ).optional();

            match result {
                Ok(Some((enabled, mode))) => json_response(&serde_json::json!({
                    "enabled": enabled != 0,
                    "mode": mode
                })),
                Ok(None) => json_response(&serde_json::json!({
                    "enabled": false,
                    "mode": "broadcaster"
                })),
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn handle_update_channel_settings(req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    match read_json_body(req).await {
        Ok(body) => {
            let channel = match body.get("channel").and_then(|v| v.as_str()) {
                Some(c) => c,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing channel"),
            };
            let enabled = match body.get("enabled").and_then(|v| v.as_bool()) {
                Some(e) => e,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing enabled"),
            };
            let mode = match body.get("mode").and_then(|v| v.as_str()) {
                Some(m) => m,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing mode"),
            };

            let conn = crate::core::database::get_database_path();
            match rusqlite::Connection::open(&conn) {
                Ok(conn) => {
                    match conn.execute(
                        "INSERT OR REPLACE INTO tts_channel_settings (channel, enabled, mode)
                         VALUES (?1, ?2, ?3)",
                        rusqlite::params![channel, enabled as i64, mode],
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

// Helper functions
fn extract_path_param(path: &str, prefix: &str) -> String {
    path.strip_prefix(prefix)
        .map(|s| urlencoding::decode(s).unwrap_or_default().to_string())
        .unwrap_or_default()
}

async fn read_json_body(req: Request<Incoming>) -> std::result::Result<serde_json::Value, String> {
    use http_body_util::BodyExt;
    let whole_body = req.collect().await
        .map_err(|e| format!("Failed to read body: {}", e))?
        .to_bytes();

    serde_json::from_slice(&whole_body)
        .map_err(|e| format!("Invalid JSON: {}", e))
}

// Query parameter versions
async fn handle_get_settings_query(query: String) -> Response<BoxBody<Bytes, Infallible>> {
    let channel = parse_query_param(&query, "channel").unwrap_or_default();

    // If channel is provided, try to get channel-specific settings
    if !channel.is_empty() {
        return handle_get_channel_settings_by_name(&channel).await;
    }

    // Otherwise return global settings
    handle_get_settings().await
}

async fn handle_get_whitelist_query(query: String) -> Response<BoxBody<Bytes, Infallible>> {
    let channel = parse_query_param(&query, "channel").unwrap_or_default();
    if channel.is_empty() {
        return error_response(StatusCode::BAD_REQUEST, "Missing channel parameter");
    }

    let conn = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&conn) {
        Ok(conn) => {
            let mut stmt = match conn.prepare(
                "SELECT username FROM tts_whitelist WHERE channel = ?1 ORDER BY username ASC"
            ) {
                Ok(stmt) => stmt,
                Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            };

            let mapped = match stmt.query_map([&channel], |row| {
                row.get::<_, String>(0)
            }) {
                Ok(m) => m,
                Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            };

            let users_result: Result<Vec<_>, _> = mapped.collect();

            match users_result {
                Ok(u) => json_response(&u),
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn handle_get_channel_settings_by_name(channel: &str) -> Response<BoxBody<Bytes, Infallible>> {
    use rusqlite::OptionalExtension;

    let conn = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&conn) {
        Ok(conn) => {
            let result: Result<Option<(i64, String)>, _> = conn.query_row(
                "SELECT enabled, mode FROM tts_channel_settings WHERE channel = ?1",
                [channel],
                |row| Ok((row.get(0)?, row.get(1)?))
            ).optional();

            match result {
                Ok(Some((enabled, mode))) => json_response(&serde_json::json!({
                    "enabled": enabled != 0,
                    "mode": mode
                })),
                Ok(None) => json_response(&serde_json::json!({
                    "enabled": false,
                    "mode": "broadcaster"
                })),
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
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
