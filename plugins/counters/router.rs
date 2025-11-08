use crate::core::plugin_context::PluginContext;
use crate::core::plugin_router::PluginRouter;
use anyhow::Result;
use hyper::{Method, Request, Response, StatusCode, body::Incoming};
use hyper::body::Bytes;
use http_body_util::{Full, combinators::BoxBody};
use std::convert::Infallible;

pub async fn register_routes(ctx: &PluginContext) -> Result<()> {
    let mut router = PluginRouter::new();

    // GET /counters/list?channel=xxx - Get all counters for a channel (query param)
    router.route(Method::GET, "/list", |_path, query, _req| {
        Box::pin(async move {
            handle_get_counters_query(query).await
        })
    });

    // GET /counters/:channel - Get all counters for a channel (path param)
    router.route(Method::GET, "/:channel", |path, _query, _req| {
        Box::pin(async move {
            handle_get_counters(path).await
        })
    });

    // POST /counters/increment - Increment a counter
    router.route(Method::POST, "/increment", |_path, _query, req| {
        Box::pin(async move {
            handle_increment_counter(req).await
        })
    });

    // POST /counters/decrement - Decrement a counter
    router.route(Method::POST, "/decrement", |_path, _query, req| {
        Box::pin(async move {
            handle_decrement_counter(req).await
        })
    });

    // POST /counters/reset - Reset a counter to zero
    router.route(Method::POST, "/reset", |_path, _query, req| {
        Box::pin(async move {
            handle_reset_counter(req).await
        })
    });

    ctx.register_router("counters", router).await;
    Ok(())
}

async fn handle_get_counters_query(query: String) -> Response<BoxBody<Bytes, Infallible>> {
    let channel = extract_query_param(&query, "channel");
    if channel.is_empty() {
        return error_response(StatusCode::BAD_REQUEST, "Missing channel parameter");
    }

    get_counters_for_channel(&channel).await
}

async fn handle_get_counters(path: String) -> Response<BoxBody<Bytes, Infallible>> {
    let channel = extract_path_param(&path, "/");
    if channel.is_empty() {
        return error_response(StatusCode::BAD_REQUEST, "Missing channel");
    }

    get_counters_for_channel(&channel).await
}

async fn get_counters_for_channel(channel: &str) -> Response<BoxBody<Bytes, Infallible>> {
    let conn = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&conn) {
        Ok(conn) => {
            let mut stmt = match conn.prepare(
                "SELECT task, count FROM counters WHERE channel = ?1 ORDER BY task ASC"
            ) {
                Ok(stmt) => stmt,
                Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            };

            let counters: Result<Vec<serde_json::Value>, _> = stmt.query_map([channel], |row| {
                Ok(serde_json::json!({
                    "task": row.get::<_, String>(0)?,
                    "count": row.get::<_, i64>(1)?,
                }))
            }).and_then(|rows| rows.collect());

            match counters {
                Ok(counters) => json_response(&serde_json::json!(counters)),
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn handle_increment_counter(req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    match read_json_body(req).await {
        Ok(body) => {
            let channel = match body.get("channel").and_then(|v| v.as_str()) {
                Some(ch) => ch,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing channel"),
            };
            let task = match body.get("task").and_then(|v| v.as_str()) {
                Some(t) => t,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing task"),
            };

            let conn = crate::core::database::get_database_path();
            match rusqlite::Connection::open(&conn) {
                Ok(conn) => {
                    let now = current_timestamp();
                    match conn.execute(
                        "INSERT INTO counters (channel, task, count, last_updated)
                         VALUES (?1, ?2, 1, ?3)
                         ON CONFLICT(channel, task) DO UPDATE SET
                           count = count + 1,
                           last_updated = ?3",
                        rusqlite::params![channel, task, now],
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

async fn handle_decrement_counter(req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    match read_json_body(req).await {
        Ok(body) => {
            let channel = match body.get("channel").and_then(|v| v.as_str()) {
                Some(ch) => ch,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing channel"),
            };
            let task = match body.get("task").and_then(|v| v.as_str()) {
                Some(t) => t,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing task"),
            };

            let conn = crate::core::database::get_database_path();
            match rusqlite::Connection::open(&conn) {
                Ok(conn) => {
                    let now = current_timestamp();
                    match conn.execute(
                        "UPDATE counters
                         SET count = MAX(0, count - 1), last_updated = ?3
                         WHERE channel = ?1 AND task = ?2",
                        rusqlite::params![channel, task, now],
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

async fn handle_reset_counter(req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    match read_json_body(req).await {
        Ok(body) => {
            let channel = match body.get("channel").and_then(|v| v.as_str()) {
                Some(ch) => ch,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing channel"),
            };
            let task = match body.get("task").and_then(|v| v.as_str()) {
                Some(t) => t,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing task"),
            };

            let conn = crate::core::database::get_database_path();
            match rusqlite::Connection::open(&conn) {
                Ok(conn) => {
                    let now = current_timestamp();
                    match conn.execute(
                        "UPDATE counters
                         SET count = 0, last_updated = ?3
                         WHERE channel = ?1 AND task = ?2",
                        rusqlite::params![channel, task, now],
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

fn extract_query_param(query: &str, key: &str) -> String {
    query
        .split('&')
        .filter_map(|pair| {
            let mut parts = pair.split('=');
            match (parts.next(), parts.next()) {
                (Some(k), Some(v)) if k == key => {
                    Some(urlencoding::decode(v).unwrap_or_default().to_string())
                }
                _ => None,
            }
        })
        .next()
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
