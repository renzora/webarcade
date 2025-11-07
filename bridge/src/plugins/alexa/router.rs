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

    // GET /alexa/config - Get Alexa configuration
    router.route(Method::GET, "/config", |_path, _query, _req| {
        Box::pin(async move {
            handle_get_config().await
        })
    });

    // POST /alexa/config - Save Alexa configuration
    router.route(Method::POST, "/config", |_path, _query, req| {
        Box::pin(async move {
            handle_save_config(req).await
        })
    });

    // GET /alexa/commands - Get all Alexa commands
    router.route(Method::GET, "/commands", |_path, _query, _req| {
        Box::pin(async move {
            handle_get_commands().await
        })
    });

    // POST /alexa/commands - Create or update an Alexa command
    router.route(Method::POST, "/commands", |_path, _query, req| {
        Box::pin(async move {
            handle_save_command(req).await
        })
    });

    // DELETE /alexa/commands/:id - Delete an Alexa command
    router.route(Method::DELETE, "/commands/:id", |path, _query, _req| {
        Box::pin(async move {
            handle_delete_command(path).await
        })
    });

    // GET /alexa/obs/status - Get OBS status
    router.route(Method::GET, "/obs/status", |_path, _query, _req| {
        Box::pin(async move {
            handle_get_obs_status().await
        })
    });

    // POST /alexa/obs/connect - Connect to OBS
    router.route(Method::POST, "/obs/connect", |_path, _query, _req| {
        Box::pin(async move {
            handle_obs_connect().await
        })
    });

    // POST /alexa/obs/disconnect - Disconnect from OBS
    router.route(Method::POST, "/obs/disconnect", |_path, _query, _req| {
        Box::pin(async move {
            handle_obs_disconnect().await
        })
    });

    // GET /alexa/obs/scenes - Get OBS scenes
    router.route(Method::GET, "/obs/scenes", |_path, _query, _req| {
        Box::pin(async move {
            handle_get_obs_scenes().await
        })
    });

    ctx.register_router("alexa", router).await;
    Ok(())
}

async fn handle_get_config() -> Response<BoxBody<Bytes, Infallible>> {
    let conn = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&conn) {
        Ok(conn) => {
            let result = conn.query_row(
                "SELECT obs_host, obs_port, obs_password, skill_id, enabled FROM alexa_config LIMIT 1",
                [],
                |row| {
                    Ok(serde_json::json!({
                        "obs_host": row.get::<_, String>(0)?,
                        "obs_port": row.get::<_, i64>(1)?,
                        "obs_password": row.get::<_, Option<String>>(2)?,
                        "skill_id": row.get::<_, Option<String>>(3)?,
                        "enabled": row.get::<_, i64>(4)? == 1,
                    }))
                }
            ).optional();

            match result {
                Ok(Some(config)) => {
                    json_response(&serde_json::json!({
                        "success": true,
                        "content": config
                    }))
                }
                Ok(None) => {
                    // No config found, return defaults
                    json_response(&serde_json::json!({
                        "success": true,
                        "content": {
                            "obs_host": "localhost",
                            "obs_port": 4455,
                            "obs_password": null,
                            "skill_id": null,
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
            let obs_host = body.get("obs_host").and_then(|v| v.as_str()).unwrap_or("localhost");
            let obs_port = body.get("obs_port").and_then(|v| v.as_i64()).unwrap_or(4455);
            let obs_password = body.get("obs_password").and_then(|v| v.as_str());
            let skill_id = body.get("skill_id").and_then(|v| v.as_str());
            let enabled = body.get("enabled").and_then(|v| v.as_bool()).unwrap_or(false);

            let conn = crate::core::database::get_database_path();
            match rusqlite::Connection::open(&conn) {
                Ok(conn) => {
                    // Check if config exists
                    let exists: bool = conn.query_row(
                        "SELECT COUNT(*) > 0 FROM alexa_config",
                        [],
                        |row| row.get(0)
                    ).unwrap_or(false);

                    let result = if exists {
                        conn.execute(
                            "UPDATE alexa_config SET
                             obs_host = ?1,
                             obs_port = ?2,
                             obs_password = ?3,
                             skill_id = ?4,
                             enabled = ?5
                             WHERE id = 1",
                            rusqlite::params![obs_host, obs_port, obs_password, skill_id, enabled as i64],
                        )
                    } else {
                        conn.execute(
                            "INSERT INTO alexa_config (id, obs_host, obs_port, obs_password, skill_id, enabled)
                             VALUES (1, ?1, ?2, ?3, ?4, ?5)",
                            rusqlite::params![obs_host, obs_port, obs_password, skill_id, enabled as i64],
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

async fn handle_get_commands() -> Response<BoxBody<Bytes, Infallible>> {
    let conn = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&conn) {
        Ok(conn) => {
            let mut stmt = match conn.prepare(
                "SELECT id, name, intent_name, action_type, action_value, response_text, enabled
                 FROM alexa_commands
                 ORDER BY name"
            ) {
                Ok(stmt) => stmt,
                Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            };

            let mapped = match stmt.query_map([], |row| {
                Ok(serde_json::json!({
                    "id": row.get::<_, i64>(0)?,
                    "name": row.get::<_, String>(1)?,
                    "intent_name": row.get::<_, String>(2)?,
                    "action_type": row.get::<_, String>(3)?,
                    "action_value": row.get::<_, String>(4)?,
                    "response_text": row.get::<_, String>(5)?,
                    "enabled": row.get::<_, i64>(6)? == 1,
                }))
            }) {
                Ok(m) => m,
                Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            };

            let commands: Result<Vec<_>, _> = mapped.collect();

            match commands {
                Ok(commands) => json_response(&serde_json::json!({
                    "success": true,
                    "content": commands
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
            let intent_name = match body.get("intent_name").and_then(|v| v.as_str()) {
                Some(i) => i,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing intent_name"),
            };
            let action_type = match body.get("action_type").and_then(|v| v.as_str()) {
                Some(a) => a,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing action_type"),
            };
            let action_value = match body.get("action_value").and_then(|v| v.as_str()) {
                Some(a) => a,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing action_value"),
            };
            let response_text = match body.get("response_text").and_then(|v| v.as_str()) {
                Some(r) => r,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing response_text"),
            };
            let enabled = body.get("enabled").and_then(|v| v.as_bool()).unwrap_or(true);

            let conn = crate::core::database::get_database_path();
            match rusqlite::Connection::open(&conn) {
                Ok(conn) => {
                    let now = current_timestamp();

                    let result = if let Some(id) = id {
                        // Update existing command
                        conn.execute(
                            "UPDATE alexa_commands SET
                             name = ?1,
                             intent_name = ?2,
                             action_type = ?3,
                             action_value = ?4,
                             response_text = ?5,
                             enabled = ?6,
                             updated_at = ?7
                             WHERE id = ?8",
                            rusqlite::params![name, intent_name, action_type, action_value, response_text, enabled as i64, now, id],
                        )
                    } else {
                        // Insert new command
                        conn.execute(
                            "INSERT INTO alexa_commands (name, intent_name, action_type, action_value, response_text, enabled, created_at, updated_at)
                             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7)",
                            rusqlite::params![name, intent_name, action_type, action_value, response_text, enabled as i64, now],
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
            match conn.execute("DELETE FROM alexa_commands WHERE id = ?1", rusqlite::params![id]) {
                Ok(_) => json_response(&serde_json::json!({ "success": true })),
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn handle_get_obs_status() -> Response<BoxBody<Bytes, Infallible>> {
    // Placeholder for OBS status
    // In a full implementation, this would connect to OBS WebSocket and get status
    json_response(&serde_json::json!({
        "success": true,
        "content": {
            "connected": false,
            "streaming": false,
            "recording": false
        }
    }))
}

async fn handle_obs_connect() -> Response<BoxBody<Bytes, Infallible>> {
    // Placeholder for OBS connection
    // In a full implementation, this would:
    // 1. Get OBS config from database (host, port, password)
    // 2. Connect to OBS WebSocket
    // 3. Store connection state
    json_response(&serde_json::json!({
        "success": true,
        "message": "OBS connection functionality requires WebSocket implementation",
        "connected": false
    }))
}

async fn handle_obs_disconnect() -> Response<BoxBody<Bytes, Infallible>> {
    // Placeholder for OBS disconnection
    json_response(&serde_json::json!({
        "success": true,
        "message": "Disconnected from OBS",
        "connected": false
    }))
}

async fn handle_get_obs_scenes() -> Response<BoxBody<Bytes, Infallible>> {
    // Placeholder for OBS scenes list
    // In a full implementation, this would:
    // 1. Check if connected to OBS
    // 2. Fetch scene list from OBS WebSocket
    // 3. Return scene names
    json_response(&serde_json::json!({
        "success": true,
        "content": []
    }))
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
