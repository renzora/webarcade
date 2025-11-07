use crate::core::plugin_context::PluginContext;
use crate::core::plugin_router::PluginRouter;
use anyhow::Result;
use hyper::{Method, Request, Response, StatusCode, body::Incoming};
use hyper::body::Bytes;
use http_body_util::{Full, combinators::BoxBody};
use std::convert::Infallible;

pub async fn register_routes(ctx: &PluginContext) -> Result<()> {
    let mut router = PluginRouter::new();

    // GET /watchtime/all - Get all watchtime records with pagination
    router.route(Method::GET, "/all", |_path, query, _req| {
        Box::pin(async move {
            handle_get_all(query).await
        })
    });

    // GET /watchtime/search - Search watchtime by username
    router.route(Method::GET, "/search", |_path, query, _req| {
        Box::pin(async move {
            handle_search(query).await
        })
    });

    // GET /watchtime/period - Get watchtime by period (for stats)
    router.route(Method::GET, "/period", |_path, query, _req| {
        Box::pin(async move {
            handle_get_by_period(query).await
        })
    });

    // GET /watchtime/by-period - Get watchtime by period (alias)
    router.route(Method::GET, "/by-period", |_path, query, _req| {
        Box::pin(async move {
            handle_get_by_period(query).await
        })
    });

    // POST /watchtime/add - Add or update watchtime
    router.route(Method::POST, "/add", |_path, _query, req| {
        Box::pin(async move {
            handle_add_time(req).await
        })
    });

    ctx.register_router("watchtime", router).await;
    Ok(())
}

async fn handle_get_all(query: String) -> Response<BoxBody<Bytes, Infallible>> {
    let channel = match parse_query_param(&query, "channel") {
        Some(ch) => ch,
        None => return error_response(StatusCode::BAD_REQUEST, "Missing channel parameter"),
    };
    let limit = parse_query_param(&query, "limit")
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(50);
    let offset = parse_query_param(&query, "offset")
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(0);

    let conn = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&conn) {
        Ok(conn) => {
            // Get total count
            let total: i64 = match conn.query_row(
                "SELECT COUNT(*) FROM users WHERE channel = ?1",
                [&channel],
                |row| row.get(0)
            ) {
                Ok(count) => count,
                Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            };

            // Get watchers
            let mut stmt = match conn.prepare(
                "SELECT username, total_minutes, last_seen
                 FROM users
                 WHERE channel = ?1
                 ORDER BY total_minutes DESC
                 LIMIT ?2 OFFSET ?3"
            ) {
                Ok(stmt) => stmt,
                Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            };

            let mapped = match stmt.query_map(
                rusqlite::params![&channel, limit, offset],
                |row| {
                    Ok(serde_json::json!({
                        "username": row.get::<_, String>(0)?,
                        "total_minutes": row.get::<_, i64>(1)?,
                        "last_seen": row.get::<_, i64>(2)?
                    }))
                }
            ) {
                Ok(m) => m,
                Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            };

            let watchers: Result<Vec<_>, _> = mapped.collect();

            match watchers {
                Ok(watchers) => json_response(&serde_json::json!({
                    "watchers": watchers,
                    "total": total
                })),
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn handle_search(query: String) -> Response<BoxBody<Bytes, Infallible>> {
    let channel = match parse_query_param(&query, "channel") {
        Some(ch) => ch,
        None => return error_response(StatusCode::BAD_REQUEST, "Missing channel parameter"),
    };
    let search = match parse_query_param(&query, "search") {
        Some(s) => s,
        None => return error_response(StatusCode::BAD_REQUEST, "Missing search parameter"),
    };

    let conn = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&conn) {
        Ok(conn) => {
            let search_pattern = format!("%{}%", search);

            let mut stmt = match conn.prepare(
                "SELECT username, total_minutes, last_seen
                 FROM users
                 WHERE channel = ?1 AND username LIKE ?2
                 ORDER BY total_minutes DESC
                 LIMIT 100"
            ) {
                Ok(stmt) => stmt,
                Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            };

            let mapped = match stmt.query_map(
                rusqlite::params![&channel, &search_pattern],
                |row| {
                    Ok(serde_json::json!({
                        "username": row.get::<_, String>(0)?,
                        "total_minutes": row.get::<_, i64>(1)?,
                        "last_seen": row.get::<_, i64>(2)?
                    }))
                }
            ) {
                Ok(m) => m,
                Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            };

            let watchers: Result<Vec<_>, _> = mapped.collect();

            match watchers {
                Ok(watchers) => json_response(&watchers),
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn handle_get_by_period(query: String) -> Response<BoxBody<Bytes, Infallible>> {
    let channel = match parse_query_param(&query, "channel") {
        Some(ch) => ch,
        None => return error_response(StatusCode::BAD_REQUEST, "Missing channel parameter"),
    };
    let period = parse_query_param(&query, "period").unwrap_or_else(|| "all".to_string());
    let limit = parse_query_param(&query, "limit")
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(50);
    let offset = parse_query_param(&query, "offset")
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(0);

    let conn = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&conn) {
        Ok(conn) => {
            let now = current_timestamp();
            let cutoff_time = match period.as_str() {
                "day" => now - 86400,
                "week" => now - 604800,
                "month" => now - 2592000,
                _ => 0, // "all" or anything else
            };

            // Get total count for period
            let total: i64 = if cutoff_time > 0 {
                match conn.query_row(
                    "SELECT COUNT(*) FROM users WHERE channel = ?1 AND last_seen >= ?2",
                    rusqlite::params![&channel, cutoff_time],
                    |row| row.get(0)
                ) {
                    Ok(count) => count,
                    Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
                }
            } else {
                match conn.query_row(
                    "SELECT COUNT(*) FROM users WHERE channel = ?1",
                    [&channel],
                    |row| row.get(0)
                ) {
                    Ok(count) => count,
                    Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
                }
            };

            // Get watchers for period
            let watchers: Result<Vec<_>, _> = if cutoff_time > 0 {
                let mut stmt = match conn.prepare(
                    "SELECT username, total_minutes, last_seen
                     FROM users
                     WHERE channel = ?1 AND last_seen >= ?2
                     ORDER BY total_minutes DESC
                     LIMIT ?3 OFFSET ?4"
                ) {
                    Ok(stmt) => stmt,
                    Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
                };

                let mapped = match stmt.query_map(
                    rusqlite::params![&channel, cutoff_time, limit, offset],
                    |row| {
                        Ok(serde_json::json!({
                            "username": row.get::<_, String>(0)?,
                            "total_minutes": row.get::<_, i64>(1)?,
                            "last_seen": row.get::<_, i64>(2)?
                        }))
                    }
                ) {
                    Ok(m) => m,
                    Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
                };
                mapped.collect()
            } else {
                let mut stmt = match conn.prepare(
                    "SELECT username, total_minutes, last_seen
                     FROM users
                     WHERE channel = ?1
                     ORDER BY total_minutes DESC
                     LIMIT ?2 OFFSET ?3"
                ) {
                    Ok(stmt) => stmt,
                    Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
                };

                let mapped = match stmt.query_map(
                    rusqlite::params![&channel, limit, offset],
                    |row| {
                        Ok(serde_json::json!({
                            "username": row.get::<_, String>(0)?,
                            "total_minutes": row.get::<_, i64>(1)?,
                            "last_seen": row.get::<_, i64>(2)?
                        }))
                    }
                ) {
                    Ok(m) => m,
                    Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
                };
                mapped.collect()
            };

            match watchers {
                Ok(watchers) => json_response(&serde_json::json!({
                    "watchers": watchers,
                    "total": total
                })),
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn handle_add_time(req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    match read_json_body(req).await {
        Ok(body) => {
            let channel = match body.get("channel").and_then(|v| v.as_str()) {
                Some(ch) => ch,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing channel"),
            };
            let user_id = match body.get("user_id").and_then(|v| v.as_str()) {
                Some(id) => id,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing user_id"),
            };
            let username = match body.get("username").and_then(|v| v.as_str()) {
                Some(name) => name,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing username"),
            };
            let minutes = body.get("minutes").and_then(|v| v.as_i64()).unwrap_or(1);

            let conn = crate::core::database::get_database_path();
            match rusqlite::Connection::open(&conn) {
                Ok(conn) => {
                    let now = current_timestamp();

                    match conn.execute(
                        "INSERT INTO users (channel, username, total_minutes, last_seen, created_at)
                         VALUES (?1, ?2, ?3, ?4, ?4)
                         ON CONFLICT(channel, username) DO UPDATE SET
                           total_minutes = total_minutes + ?3,
                           last_seen = ?4",
                        rusqlite::params![channel, username, minutes, now],
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
