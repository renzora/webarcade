use crate::core::plugin_context::PluginContext;
use crate::core::plugin_router::PluginRouter;
use anyhow::Result;
use hyper::{Method, Request, Response, StatusCode, body::Incoming};
use hyper::body::Bytes;
use http_body_util::{Full, combinators::BoxBody};
use std::convert::Infallible;

pub async fn register_routes(ctx: &PluginContext) -> Result<()> {
    let mut router = PluginRouter::new();

    // GET /status/config - Get all status configuration
    router.route(Method::GET, "/config", |_path, _query, _req| {
        Box::pin(async move {
            handle_get_config().await
        })
    });

    // POST /status/start-date - Update stream start date
    router.route(Method::POST, "/start-date", |_path, _query, req| {
        Box::pin(async move {
            handle_update_start_date(req).await
        })
    });

    // POST /status/ticker-speed - Update ticker speed
    router.route(Method::POST, "/ticker-speed", |_path, _query, req| {
        Box::pin(async move {
            handle_update_ticker_speed(req).await
        })
    });

    // POST /status/max-ticker-items - Update max ticker items
    router.route(Method::POST, "/max-ticker-items", |_path, _query, req| {
        Box::pin(async move {
            handle_update_max_ticker_items(req).await
        })
    });

    // POST /status/segment-duration - Update segment duration
    router.route(Method::POST, "/segment-duration", |_path, _query, req| {
        Box::pin(async move {
            handle_update_segment_duration(req).await
        })
    });

    // POST /status/breaking-news - Update breaking news
    router.route(Method::POST, "/breaking-news", |_path, _query, req| {
        Box::pin(async move {
            handle_update_breaking_news(req).await
        })
    });

    ctx.register_router("status", router).await;
    Ok(())
}

async fn handle_get_config() -> Response<BoxBody<Bytes, Infallible>> {
    let conn = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&conn) {
        Ok(conn) => {
            let stream_start_date = get_config_value(&conn, "stream_start_date");
            let ticker_speed = get_config_value(&conn, "ticker_speed").and_then(|v| v.parse::<i64>().ok()).unwrap_or(30);
            let max_ticker_items = get_config_value(&conn, "max_ticker_items").and_then(|v| v.parse::<i64>().ok()).unwrap_or(20);
            let segment_duration = get_config_value(&conn, "segment_duration").and_then(|v| v.parse::<i64>().ok()).unwrap_or(15);
            let breaking_news_active = get_config_value(&conn, "breaking_news_active").map(|v| v == "true").unwrap_or(false);
            let breaking_news_message = get_config_value(&conn, "breaking_news_message");

            let config = serde_json::json!({
                "stream_start_date": stream_start_date,
                "ticker_speed": ticker_speed,
                "max_ticker_items": max_ticker_items,
                "segment_duration": segment_duration,
                "breaking_news_active": breaking_news_active,
                "breaking_news_message": breaking_news_message,
            });

            json_response(&config)
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn handle_update_start_date(req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    match read_json_body(req).await {
        Ok(body) => {
            let start_date = body.get("start_date").and_then(|v| v.as_str()).unwrap_or("");

            let conn = crate::core::database::get_database_path();
            match rusqlite::Connection::open(&conn) {
                Ok(conn) => {
                    match set_config_value(&conn, "stream_start_date", start_date) {
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

async fn handle_update_ticker_speed(req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    match read_json_body(req).await {
        Ok(body) => {
            let speed = match body.get("speed").and_then(|v| v.as_i64()) {
                Some(s) => s,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing speed"),
            };

            let conn = crate::core::database::get_database_path();
            match rusqlite::Connection::open(&conn) {
                Ok(conn) => {
                    match set_config_value(&conn, "ticker_speed", &speed.to_string()) {
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

async fn handle_update_max_ticker_items(req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    match read_json_body(req).await {
        Ok(body) => {
            let max_items = match body.get("max_items").and_then(|v| v.as_i64()) {
                Some(m) => m,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing max_items"),
            };

            let conn = crate::core::database::get_database_path();
            match rusqlite::Connection::open(&conn) {
                Ok(conn) => {
                    match set_config_value(&conn, "max_ticker_items", &max_items.to_string()) {
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

async fn handle_update_segment_duration(req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    match read_json_body(req).await {
        Ok(body) => {
            let duration = match body.get("duration").and_then(|v| v.as_i64()) {
                Some(d) => d,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing duration"),
            };

            let conn = crate::core::database::get_database_path();
            match rusqlite::Connection::open(&conn) {
                Ok(conn) => {
                    match set_config_value(&conn, "segment_duration", &duration.to_string()) {
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

async fn handle_update_breaking_news(req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    match read_json_body(req).await {
        Ok(body) => {
            let active = body.get("active").and_then(|v| v.as_bool()).unwrap_or(false);
            let message = body.get("message").and_then(|v| v.as_str()).unwrap_or("");

            let conn = crate::core::database::get_database_path();
            match rusqlite::Connection::open(&conn) {
                Ok(conn) => {
                    if let Err(e) = set_config_value(&conn, "breaking_news_active", if active { "true" } else { "false" }) {
                        return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string());
                    }
                    if let Err(e) = set_config_value(&conn, "breaking_news_message", message) {
                        return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string());
                    }
                    json_response(&serde_json::json!({ "success": true }))
                }
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::BAD_REQUEST, &e),
    }
}

// Helper functions
fn get_config_value(conn: &rusqlite::Connection, key: &str) -> Option<String> {
    conn.query_row(
        "SELECT value FROM status_config WHERE key = ?1",
        rusqlite::params![key],
        |row| row.get(0)
    ).ok()
}

fn set_config_value(conn: &rusqlite::Connection, key: &str, value: &str) -> rusqlite::Result<()> {
    let now = current_timestamp();
    conn.execute(
        "INSERT OR REPLACE INTO status_config (key, value, updated_at) VALUES (?1, ?2, ?3)",
        rusqlite::params![key, value, now],
    )?;
    Ok(())
}

fn current_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
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
