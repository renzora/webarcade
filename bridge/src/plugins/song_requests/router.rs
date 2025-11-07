use crate::core::plugin_context::PluginContext;
use crate::core::plugin_router::PluginRouter;
use anyhow::Result;
use hyper::{Method, Request, Response, StatusCode, body::Incoming};
use hyper::body::Bytes;
use http_body_util::{Full, combinators::BoxBody};
use std::convert::Infallible;

pub async fn register_routes(ctx: &PluginContext) -> Result<()> {
    let mut router = PluginRouter::new();

    // GET /song-requests/pending - Get pending song requests
    router.route(Method::GET, "/pending", |_path, _query, _req| {
        Box::pin(async move {
            handle_get_pending().await
        })
    });

    // GET /song-requests/all - Get all song requests with optional limit
    router.route(Method::GET, "/all", |_path, query, _req| {
        Box::pin(async move {
            handle_get_all(query).await
        })
    });

    // POST /song-requests/status - Update song request status
    router.route(Method::POST, "/status", |_path, _query, req| {
        Box::pin(async move {
            handle_update_status(req).await
        })
    });

    // DELETE /song-requests/:id - Delete a song request
    router.route(Method::DELETE, "/:id", |_path, _query, req| {
        Box::pin(async move {
            handle_delete_request(req).await
        })
    });

    // DELETE /song-requests/clear - Clear completed/skipped requests
    router.route(Method::DELETE, "/clear", |_path, _query, req| {
        Box::pin(async move {
            handle_clear_requests(req).await
        })
    });

    ctx.register_router("song_requests", router).await;
    Ok(())
}

async fn handle_get_pending() -> Response<BoxBody<Bytes, Infallible>> {
    let conn = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&conn) {
        Ok(conn) => {
            let mut stmt = match conn.prepare(
                "SELECT id, song_query, requester_name, requester_id, status, requested_at, played_at, created_at, updated_at
                 FROM song_requests
                 WHERE status = 'pending'
                 ORDER BY requested_at ASC"
            ) {
                Ok(s) => s,
                Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            };

            let requests: Vec<serde_json::Value> = match stmt.query_map([], |row| {
                Ok(serde_json::json!({
                    "id": row.get::<_, i64>(0)?,
                    "song_query": row.get::<_, String>(1)?,
                    "requester_name": row.get::<_, String>(2)?,
                    "requester_id": row.get::<_, Option<String>>(3)?,
                    "status": row.get::<_, String>(4)?,
                    "requested_at": row.get::<_, i64>(5)?,
                    "played_at": row.get::<_, Option<i64>>(6)?,
                    "created_at": row.get::<_, i64>(7)?,
                    "updated_at": row.get::<_, i64>(8)?,
                }))
            }) {
                Ok(rows) => rows.filter_map(|r| r.ok()).collect(),
                Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            };

            json_response(&serde_json::json!({
                "success": true,
                "data": requests
            }))
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn handle_get_all(query: String) -> Response<BoxBody<Bytes, Infallible>> {
    // Parse limit from query string
    let limit = parse_query_param(&query, "limit")
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(50);

    let conn = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&conn) {
        Ok(conn) => {
            let mut stmt = match conn.prepare(
                "SELECT id, song_query, requester_name, requester_id, status, requested_at, played_at, created_at, updated_at
                 FROM song_requests
                 ORDER BY requested_at DESC
                 LIMIT ?1"
            ) {
                Ok(s) => s,
                Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            };

            let requests: Vec<serde_json::Value> = match stmt.query_map([limit], |row| {
                Ok(serde_json::json!({
                    "id": row.get::<_, i64>(0)?,
                    "song_query": row.get::<_, String>(1)?,
                    "requester_name": row.get::<_, String>(2)?,
                    "requester_id": row.get::<_, Option<String>>(3)?,
                    "status": row.get::<_, String>(4)?,
                    "requested_at": row.get::<_, i64>(5)?,
                    "played_at": row.get::<_, Option<i64>>(6)?,
                    "created_at": row.get::<_, i64>(7)?,
                    "updated_at": row.get::<_, i64>(8)?,
                }))
            }) {
                Ok(rows) => rows.filter_map(|r| r.ok()).collect(),
                Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            };

            json_response(&serde_json::json!({
                "success": true,
                "data": requests
            }))
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn handle_update_status(req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    match read_json_body(req).await {
        Ok(body) => {
            let id = match body.get("id").and_then(|v| v.as_i64()) {
                Some(i) => i,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing id"),
            };

            let status = match body.get("status").and_then(|v| v.as_str()) {
                Some(s) => s,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing status"),
            };

            let conn = crate::core::database::get_database_path();
            match rusqlite::Connection::open(&conn) {
                Ok(conn) => {
                    let now = current_timestamp();

                    // If status is 'playing' or 'completed', set played_at
                    let played_at = if status == "playing" || status == "completed" {
                        Some(now)
                    } else {
                        None
                    };

                    let result = if let Some(played_at) = played_at {
                        conn.execute(
                            "UPDATE song_requests SET status = ?1, played_at = ?2, updated_at = ?3 WHERE id = ?4",
                            rusqlite::params![status, played_at, now, id],
                        )
                    } else {
                        conn.execute(
                            "UPDATE song_requests SET status = ?1, updated_at = ?2 WHERE id = ?3",
                            rusqlite::params![status, now, id],
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

async fn handle_delete_request(req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    match read_json_body(req).await {
        Ok(body) => {
            let id = match body.get("id").and_then(|v| v.as_i64()) {
                Some(i) => i,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing id"),
            };

            let conn = crate::core::database::get_database_path();
            match rusqlite::Connection::open(&conn) {
                Ok(conn) => {
                    match conn.execute(
                        "DELETE FROM song_requests WHERE id = ?1",
                        rusqlite::params![id],
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

async fn handle_clear_requests(req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    match read_json_body(req).await {
        Ok(body) => {
            let status = body.get("status")
                .and_then(|v| v.as_str())
                .unwrap_or("completed");

            let conn = crate::core::database::get_database_path();
            match rusqlite::Connection::open(&conn) {
                Ok(conn) => {
                    match conn.execute(
                        "DELETE FROM song_requests WHERE status = ?1",
                        rusqlite::params![status],
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
        .find(|param| param.starts_with(&format!("{}=", key)))
        .and_then(|param| param.split('=').nth(1))
        .map(|v| urlencoding::decode(v).unwrap_or_default().to_string())
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
