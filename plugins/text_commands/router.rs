use crate::core::plugin_context::PluginContext;
use crate::core::plugin_router::PluginRouter;
use anyhow::Result;
use hyper::{Method, Request, Response, StatusCode, body::Incoming};
use hyper::body::Bytes;
use http_body_util::{Full, combinators::BoxBody};
use std::convert::Infallible;

pub async fn register_routes(ctx: &PluginContext) -> Result<()> {
    let mut router = PluginRouter::new();

    // POST /text_commands/create - Create or update a text command
    router.route(Method::POST, "/create", |_path, _query, req| {
        Box::pin(async move {
            handle_create_command(req).await
        })
    });

    // GET /text_commands/all - Get all text commands for a channel
    router.route(Method::GET, "/all", |_path, query, _req| {
        Box::pin(async move {
            handle_get_all(query).await
        })
    });

    // GET /text_commands/:command - Get a specific command
    router.route(Method::GET, "/:command", |path, query, _req| {
        Box::pin(async move {
            handle_get_command(path, query).await
        })
    });

    // DELETE /text_commands/:command - Delete a text command
    router.route(Method::DELETE, "/:command", |path, query, _req| {
        Box::pin(async move {
            handle_delete_command(path, query).await
        })
    });

    ctx.register_router("text_commands", router).await;
    Ok(())
}

async fn handle_create_command(req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
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
            let response = match body.get("response").and_then(|v| v.as_str()) {
                Some(r) => r,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing response"),
            };

            let conn = crate::core::database::get_database_path();
            match rusqlite::Connection::open(&conn) {
                Ok(conn) => {
                    let now = current_timestamp();
                    match conn.execute(
                        "INSERT OR REPLACE INTO text_commands (channel, command, response, enabled, created_at, updated_at)
                         VALUES (?1, ?2, ?3, 1, ?4, ?4)",
                        rusqlite::params![channel, command, response, now],
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

async fn handle_get_all(query: String) -> Response<BoxBody<Bytes, Infallible>> {
    let channel = match parse_query_param(&query, "channel") {
        Some(c) => c,
        None => return error_response(StatusCode::BAD_REQUEST, "Missing channel parameter"),
    };

    let conn = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&conn) {
        Ok(conn) => {
            // Check if enabled column exists, otherwise use auto_post
            let has_enabled = match conn.query_row(
                "SELECT COUNT(*) FROM pragma_table_info('text_commands') WHERE name = 'enabled'",
                [],
                |row| row.get::<_, i64>(0)
            ) {
                Ok(count) => count > 0,
                Err(_) => false,
            };

            let (query_sql, _enabled_col) = if has_enabled {
                ("SELECT id, command, response, enabled FROM text_commands WHERE channel = ?1 ORDER BY command ASC", "enabled")
            } else {
                ("SELECT id, command, response, auto_post FROM text_commands WHERE channel = ?1 ORDER BY command ASC", "auto_post")
            };

            let mut stmt = match conn.prepare(query_sql) {
                Ok(stmt) => stmt,
                Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            };

            let mapped = match stmt.query_map([&channel], |row| {
                Ok(serde_json::json!({
                    "id": row.get::<_, i64>(0)?,
                    "command": row.get::<_, String>(1)?,
                    "response": row.get::<_, String>(2)?,
                    "enabled": row.get::<_, i64>(3).unwrap_or(1) == 1
                }))
            }) {
                Ok(m) => m,
                Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            };

            let commands: Result<Vec<_>, _> = mapped.collect();

            match commands {
                Ok(cmds) => json_response(&cmds),
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn handle_get_command(path: String, query: String) -> Response<BoxBody<Bytes, Infallible>> {
    let command = extract_path_param(&path, "/");
    if command.is_empty() {
        return error_response(StatusCode::BAD_REQUEST, "Missing command");
    }

    let channel = match parse_query_param(&query, "channel") {
        Some(c) => c,
        None => return error_response(StatusCode::BAD_REQUEST, "Missing channel parameter"),
    };

    let conn = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&conn) {
        Ok(conn) => {
            use rusqlite::OptionalExtension;
            match conn.query_row(
                "SELECT response FROM text_commands WHERE channel = ?1 AND command = ?2 AND enabled = 1",
                rusqlite::params![channel, command],
                |row| row.get::<_, String>(0),
            ).optional() {
                Ok(response) => json_response(&serde_json::json!({ "response": response })),
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn handle_delete_command(path: String, query: String) -> Response<BoxBody<Bytes, Infallible>> {
    let command = extract_path_param(&path, "/");
    if command.is_empty() {
        return error_response(StatusCode::BAD_REQUEST, "Missing command");
    }

    let channel = match parse_query_param(&query, "channel") {
        Some(c) => c,
        None => return error_response(StatusCode::BAD_REQUEST, "Missing channel parameter"),
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
