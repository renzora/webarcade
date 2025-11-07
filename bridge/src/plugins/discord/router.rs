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

    // GET /discord/config - Get Discord configuration
    router.route(Method::GET, "/config", |_path, _query, _req| {
        Box::pin(async move {
            handle_get_config().await
        })
    });

    // POST /discord/config - Save Discord configuration
    router.route(Method::POST, "/config", |_path, _query, req| {
        Box::pin(async move {
            handle_save_config(req).await
        })
    });

    // GET /discord/stats - Get Discord bot stats
    router.route(Method::GET, "/stats", |_path, _query, _req| {
        Box::pin(async move {
            handle_get_stats().await
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
                "SELECT bot_token, channel_id, enabled, command_prefix, max_song_length, max_queue_size FROM discord_config LIMIT 1",
                [],
                |row| {
                    Ok(serde_json::json!({
                        "bot_token": row.get::<_, Option<String>>(0)?,
                        "channel_id": row.get::<_, Option<String>>(1)?,
                        "enabled": row.get::<_, bool>(2)?,
                        "command_prefix": row.get::<_, String>(3)?,
                        "max_song_length": row.get::<_, i64>(4)?,
                        "max_queue_size": row.get::<_, i64>(5)?,
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
                            "channel_id": null,
                            "enabled": false,
                            "command_prefix": "!sr",
                            "max_song_length": 600,
                            "max_queue_size": 50
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
            let channel_id = body.get("channel_id").and_then(|v| v.as_str());
            let enabled = body.get("enabled").and_then(|v| v.as_bool()).unwrap_or(false);
            let command_prefix = body.get("command_prefix").and_then(|v| v.as_str()).unwrap_or("!sr");
            let max_song_length = body.get("max_song_length").and_then(|v| v.as_i64()).unwrap_or(600);
            let max_queue_size = body.get("max_queue_size").and_then(|v| v.as_i64()).unwrap_or(50);

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
                             channel_id = ?2,
                             enabled = ?3,
                             command_prefix = ?4,
                             max_song_length = ?5,
                             max_queue_size = ?6,
                             updated_at = ?7
                             WHERE id = 1",
                            rusqlite::params![bot_token, channel_id, enabled, command_prefix, max_song_length, max_queue_size, now],
                        )
                    } else {
                        conn.execute(
                            "INSERT INTO discord_config (id, bot_token, channel_id, enabled, command_prefix, max_song_length, max_queue_size, created_at, updated_at)
                             VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7)",
                            rusqlite::params![bot_token, channel_id, enabled, command_prefix, max_song_length, max_queue_size, now],
                        )
                    };

                    match result {
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

async fn handle_get_stats() -> Response<BoxBody<Bytes, Infallible>> {
    // For now, return placeholder stats
    // In a full implementation, this would check the Discord bot status
    json_response(&serde_json::json!({
        "success": true,
        "data": {
            "connected": false,
            "uptime": 0,
            "server_count": 0,
            "commands_processed": 0
        }
    }))
}

async fn handle_start_bot() -> Response<BoxBody<Bytes, Infallible>> {
    // Placeholder for starting the Discord bot
    // In a full implementation, this would start the Discord bot connection
    json_response(&serde_json::json!({
        "success": false,
        "error": "Discord bot functionality not yet implemented"
    }))
}

async fn handle_stop_bot() -> Response<BoxBody<Bytes, Infallible>> {
    // Placeholder for stopping the Discord bot
    json_response(&serde_json::json!({
        "success": false,
        "error": "Discord bot functionality not yet implemented"
    }))
}

async fn handle_get_commands() -> Response<BoxBody<Bytes, Infallible>> {
    let conn = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&conn) {
        Ok(conn) => {
            let mut stmt = match conn.prepare(
                "SELECT id, name, aliases, response, description, permission, cooldown, enabled
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
                    "aliases": row.get::<_, Option<String>>(2)?,
                    "response": row.get::<_, String>(3)?,
                    "description": row.get::<_, Option<String>>(4)?,
                    "permission": row.get::<_, String>(5)?,
                    "cooldown": row.get::<_, i64>(6)?,
                    "enabled": row.get::<_, i64>(7)? == 1,
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
            let aliases = body.get("aliases").and_then(|v| v.as_str());
            let description = body.get("description").and_then(|v| v.as_str());
            let permission = body.get("permission").and_then(|v| v.as_str()).unwrap_or("Everyone");
            let cooldown = body.get("cooldown").and_then(|v| v.as_i64()).unwrap_or(0);
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
                             aliases = ?2,
                             response = ?3,
                             description = ?4,
                             permission = ?5,
                             cooldown = ?6,
                             enabled = ?7,
                             updated_at = ?8
                             WHERE id = ?9",
                            rusqlite::params![name, aliases, response, description, permission, cooldown, enabled as i64, now, id],
                        )
                    } else {
                        // Insert new command
                        conn.execute(
                            "INSERT INTO discord_commands (name, aliases, response, description, permission, cooldown, enabled, created_at, updated_at)
                             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?8)",
                            rusqlite::params![name, aliases, response, description, permission, cooldown, enabled as i64, now],
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
