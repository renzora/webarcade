use crate::core::plugin_context::PluginContext;
use crate::core::plugin_router::PluginRouter;
use anyhow::Result;
use hyper::{Method, Request, Response, StatusCode, body::Incoming};
use hyper::body::Bytes;
use http_body_util::{Full, combinators::BoxBody};
use rusqlite::OptionalExtension;
use std::convert::Infallible;

pub async fn register_routes(ctx: &PluginContext) -> Result<()> {
    let mut router = PluginRouter::new();

    // GET /discord/config - Get Discord bot configuration
    router.route(Method::GET, "/config", |_path, _query, _req| {
        Box::pin(async move {
            handle_get_config().await
        })
    });

    // POST /discord/config - Save Discord bot configuration
    router.route(Method::POST, "/config", |_path, _query, req| {
        Box::pin(async move {
            handle_save_config(req).await
        })
    });

    // GET /discord/status - Get Discord bot status
    router.route(Method::GET, "/status", |_path, _query, _req| {
        Box::pin(async move {
            handle_get_status().await
        })
    });

    // POST /discord/start - Start Discord bot
    router.route(Method::POST, "/start", |_path, _query, _req| {
        Box::pin(async move {
            handle_start_bot().await
        })
    });

    // POST /discord/stop - Stop Discord bot
    router.route(Method::POST, "/stop", |_path, _query, _req| {
        Box::pin(async move {
            handle_stop_bot().await
        })
    });

    // POST /discord/message - Send a message to a channel
    router.route(Method::POST, "/message", |_path, _query, req| {
        Box::pin(async move {
            handle_send_message(req).await
        })
    });

    // GET /discord/commands - Get all Discord commands
    router.route(Method::GET, "/commands", |_path, _query, _req| {
        Box::pin(async move {
            handle_get_commands().await
        })
    });

    // POST /discord/commands - Create or update a Discord command
    router.route(Method::POST, "/commands", |_path, _query, req| {
        Box::pin(async move {
            handle_save_command(req).await
        })
    });

    // DELETE /discord/commands/:id - Delete a Discord command
    router.route(Method::DELETE, "/commands/:id", |path, _query, _req| {
        Box::pin(async move {
            handle_delete_command(path).await
        })
    });

    ctx.register_router("discord", router).await;
    Ok(())
}

async fn handle_get_config() -> Response<BoxBody<Bytes, Infallible>> {
    let conn = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&conn) {
        Ok(conn) => {
            let result = conn.query_row(
                "SELECT bot_token, command_prefix, enabled FROM discord_config WHERE id = 1",
                [],
                |row| {
                    Ok(serde_json::json!({
                        "bot_token": row.get::<_, Option<String>>(0)?,
                        "command_prefix": row.get::<_, String>(1)?,
                        "enabled": row.get::<_, bool>(2)?,
                    }))
                }
            ).optional();

            match result {
                Ok(Some(config)) => {
                    json_response(&serde_json::json!({
                        "success": true,
                        "data": config
                    }))
                }
                Ok(None) => {
                    // No config found, return defaults
                    json_response(&serde_json::json!({
                        "success": true,
                        "data": {
                            "bot_token": null,
                            "command_prefix": "!",
                            "enabled": false
                        }
                    }))
                }
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn handle_save_config(req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    match read_json_body(req).await {
        Ok(body) => {
            let bot_token = body.get("bot_token").and_then(|v| v.as_str());
            let command_prefix = body.get("command_prefix").and_then(|v| v.as_str()).unwrap_or("!");
            let enabled = body.get("enabled").and_then(|v| v.as_bool()).unwrap_or(false);

            let conn = crate::core::database::get_database_path();
            match rusqlite::Connection::open(&conn) {
                Ok(conn) => {
                    let now = current_timestamp();

                    // Check if config exists
                    let exists: bool = conn.query_row(
                        "SELECT COUNT(*) > 0 FROM discord_config",
                        [],
                        |row| row.get(0)
                    ).unwrap_or(false);

                    let result = if exists {
                        conn.execute(
                            "UPDATE discord_config SET
                             bot_token = ?1,
                             command_prefix = ?2,
                             enabled = ?3,
                             updated_at = ?4
                             WHERE id = 1",
                            rusqlite::params![bot_token, command_prefix, enabled, now],
                        )
                    } else {
                        conn.execute(
                            "INSERT INTO discord_config (id, bot_token, command_prefix, enabled, created_at, updated_at)
                             VALUES (1, ?1, ?2, ?3, ?4, ?4)",
                            rusqlite::params![bot_token, command_prefix, enabled, now],
                        )
                    };

                    match result {
                        Ok(_) => {
                            // If enabled, try to start the bot
                            if enabled {
                                if let Some(token) = bot_token {
                                    if let Err(e) = super::start_bot(token.to_string()).await {
                                        return error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to start bot: {}", e));
                                    }
                                }
                            } else {
                                // If disabled, stop the bot
                                let _ = super::stop_bot().await;
                            }

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

async fn handle_get_status() -> Response<BoxBody<Bytes, Infallible>> {
    let state = super::BOT_STATE.read().await;

    json_response(&serde_json::json!({
        "success": true,
        "data": {
            "is_running": state.is_running,
            "command_prefix": state.config.command_prefix,
        }
    }))
}

async fn handle_start_bot() -> Response<BoxBody<Bytes, Infallible>> {
    let conn = crate::core::database::get_database_path();

    match rusqlite::Connection::open(&conn) {
        Ok(conn) => {
            if let Ok(token) = conn.query_row(
                "SELECT bot_token FROM discord_config WHERE id = 1",
                [],
                |row| row.get::<_, Option<String>>(0),
            ) {
                if let Some(token) = token {
                    match super::start_bot(token).await {
                        Ok(_) => json_response(&serde_json::json!({ "success": true })),
                        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
                    }
                } else {
                    error_response(StatusCode::BAD_REQUEST, "No bot token configured")
                }
            } else {
                error_response(StatusCode::BAD_REQUEST, "No bot configuration found")
            }
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn handle_stop_bot() -> Response<BoxBody<Bytes, Infallible>> {
    match super::stop_bot().await {
        Ok(_) => json_response(&serde_json::json!({ "success": true })),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn handle_send_message(req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    match read_json_body(req).await {
        Ok(body) => {
            let channel_id = match body.get("channel_id").and_then(|v| v.as_u64()) {
                Some(id) => id,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing channel_id"),
            };
            let content = match body.get("content").and_then(|v| v.as_str()) {
                Some(c) => c.to_string(),
                None => return error_response(StatusCode::BAD_REQUEST, "Missing content"),
            };

            match super::send_message(channel_id, content).await {
                Ok(_) => json_response(&serde_json::json!({ "success": true })),
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::BAD_REQUEST, &e),
    }
}

async fn handle_get_commands() -> Response<BoxBody<Bytes, Infallible>> {
    let conn = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&conn) {
        Ok(conn) => {
            let mut stmt = match conn.prepare(
                "SELECT id, name, description, response, enabled
                 FROM discord_commands
                 ORDER BY name"
            ) {
                Ok(stmt) => stmt,
                Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            };

            let mapped = match stmt.query_map([], |row| {
                Ok(serde_json::json!({
                    "id": row.get::<_, i64>(0)?,
                    "name": row.get::<_, String>(1)?,
                    "description": row.get::<_, Option<String>>(2)?,
                    "response": row.get::<_, String>(3)?,
                    "enabled": row.get::<_, i64>(4)? == 1,
                }))
            }) {
                Ok(m) => m,
                Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            };

            let commands: Result<Vec<_>, _> = mapped.collect();

            match commands {
                Ok(commands) => json_response(&serde_json::json!({
                    "success": true,
                    "data": commands
                })),
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn handle_save_command(req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    match read_json_body(req).await {
        Ok(body) => {
            let id = body.get("id").and_then(|v| v.as_i64());
            let name = match body.get("name").and_then(|v| v.as_str()) {
                Some(n) => n,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing name"),
            };
            let response = match body.get("response").and_then(|v| v.as_str()) {
                Some(r) => r,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing response"),
            };
            let description = body.get("description").and_then(|v| v.as_str());
            let enabled = body.get("enabled").and_then(|v| v.as_bool()).unwrap_or(true);

            let conn = crate::core::database::get_database_path();
            match rusqlite::Connection::open(&conn) {
                Ok(conn) => {
                    let now = current_timestamp();

                    let result = if let Some(id) = id {
                        // Update existing command
                        conn.execute(
                            "UPDATE discord_commands SET
                             name = ?1,
                             description = ?2,
                             response = ?3,
                             enabled = ?4,
                             updated_at = ?5
                             WHERE id = ?6",
                            rusqlite::params![name, description, response, enabled as i64, now, id],
                        )
                    } else {
                        // Insert new command
                        conn.execute(
                            "INSERT INTO discord_commands (name, description, response, enabled, created_at, updated_at)
                             VALUES (?1, ?2, ?3, ?4, ?5, ?5)",
                            rusqlite::params![name, description, response, enabled as i64, now],
                        )
                    };

                    match result {
                        Ok(_) => json_response(&serde_json::json!({
                            "success": true,
                            "id": conn.last_insert_rowid()
                        })),
                        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
                    }
                }
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::BAD_REQUEST, &e),
    }
}

async fn handle_delete_command(path: String) -> Response<BoxBody<Bytes, Infallible>> {
    let id = extract_path_param(&path, "/commands/");
    let id: i64 = match id.parse() {
        Ok(id) => id,
        Err(_) => return error_response(StatusCode::BAD_REQUEST, "Invalid command ID"),
    };

    let conn = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&conn) {
        Ok(conn) => {
            match conn.execute("DELETE FROM discord_commands WHERE id = ?1", rusqlite::params![id]) {
                Ok(_) => json_response(&serde_json::json!({ "success": true })),
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
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
