use crate::core::plugin_context::PluginContext;
use crate::core::plugin_router::PluginRouter;
use anyhow::Result;
use hyper::{Method, Request, Response, StatusCode, body::Incoming};
use hyper::body::Bytes;
use http_body_util::{Full, combinators::BoxBody};
use std::convert::Infallible;

pub async fn register_routes(ctx: &PluginContext) -> Result<()> {
    let mut router = PluginRouter::new();

    // GET /confessions - Get all confessions
    router.route(Method::GET, "/", |_path, query, _req| {
        Box::pin(async move {
            handle_get_all_confessions(query).await
        })
    });

    // DELETE /confessions/:id - Delete a confession
    router.route(Method::DELETE, "/:id", |path, _query, _req| {
        Box::pin(async move {
            handle_delete_confession(path).await
        })
    });

    // POST /confessions/submit - Submit a new confession
    router.route(Method::POST, "/submit", |_path, _query, req| {
        Box::pin(async move {
            handle_submit_confession(req).await
        })
    });

    // GET /confessions/pending - Get all pending confessions for moderation
    router.route(Method::GET, "/pending", |_path, _query, _req| {
        Box::pin(async move {
            handle_get_pending_confessions().await
        })
    });

    // POST /confessions/approve - Approve a pending confession
    router.route(Method::POST, "/approve", |_path, _query, req| {
        Box::pin(async move {
            handle_approve_confession(req).await
        })
    });

    // POST /confessions/reject - Reject a pending confession
    router.route(Method::POST, "/reject", |_path, _query, req| {
        Box::pin(async move {
            handle_reject_confession(req).await
        })
    });

    // GET /confessions/random - Get a random approved confession
    router.route(Method::GET, "/random", |_path, _query, _req| {
        Box::pin(async move {
            handle_get_random_confession().await
        })
    });

    // POST /confessions/reaction - Add a reaction to a confession
    router.route(Method::POST, "/reaction", |_path, _query, req| {
        Box::pin(async move {
            handle_add_reaction(req).await
        })
    });

    // GET /confessions/stats - Get confession statistics
    router.route(Method::GET, "/stats", |_path, _query, _req| {
        Box::pin(async move {
            handle_get_stats().await
        })
    });

    // POST /confessions/settings - Update confession settings
    router.route(Method::POST, "/settings", |_path, _query, req| {
        Box::pin(async move {
            handle_update_setting(req).await
        })
    });

    // OPTIONS for CORS preflight
    router.route(Method::OPTIONS, "/submit", |_path, _query, _req| {
        Box::pin(async move { cors_preflight_response() })
    });
    router.route(Method::OPTIONS, "/approve", |_path, _query, _req| {
        Box::pin(async move { cors_preflight_response() })
    });
    router.route(Method::OPTIONS, "/reject", |_path, _query, _req| {
        Box::pin(async move { cors_preflight_response() })
    });
    router.route(Method::OPTIONS, "/reaction", |_path, _query, _req| {
        Box::pin(async move { cors_preflight_response() })
    });
    router.route(Method::OPTIONS, "/settings", |_path, _query, _req| {
        Box::pin(async move { cors_preflight_response() })
    });

    ctx.register_router("confessions", router).await;
    Ok(())
}

async fn handle_get_all_confessions(query: String) -> Response<BoxBody<Bytes, Infallible>> {
    let channel_filter = parse_query_param(&query, "channel");
    let limit = parse_query_param(&query, "limit")
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(100);

    let conn = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&conn) {
        Ok(conn) => {
            let (query_str, params): (String, Vec<String>) = if let Some(channel) = channel_filter {
                (
                    format!("SELECT id, channel, username, message, created_at
                             FROM confessions
                             WHERE channel = ?
                             ORDER BY created_at DESC
                             LIMIT {}", limit),
                    vec![channel]
                )
            } else {
                (
                    format!("SELECT id, channel, username, message, created_at
                             FROM confessions
                             ORDER BY created_at DESC
                             LIMIT {}", limit),
                    vec![]
                )
            };

            let mut stmt = match conn.prepare(&query_str) {
                Ok(stmt) => stmt,
                Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            };

            let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p as &dyn rusqlite::ToSql).collect();

            let mapped = match stmt.query_map(param_refs.as_slice(), |row| {
                Ok(serde_json::json!({
                    "id": row.get::<_, i64>(0)?,
                    "channel": row.get::<_, String>(1)?,
                    "username": row.get::<_, String>(2)?,
                    "message": row.get::<_, String>(3)?,
                    "created_at": row.get::<_, i64>(4)?,
                }))
            }) {
                Ok(m) => m,
                Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            };

            let confessions_result: Result<Vec<_>, _> = mapped.collect();

            match confessions_result {
                Ok(confessions) => json_response(&confessions),
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn handle_delete_confession(path: String) -> Response<BoxBody<Bytes, Infallible>> {
    let id = extract_path_param(&path, "/");
    let id: i64 = match id.parse() {
        Ok(id) => id,
        Err(_) => return error_response(StatusCode::BAD_REQUEST, "Invalid confession ID"),
    };

    let conn = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&conn) {
        Ok(conn) => {
            // Delete associated reactions first
            let _ = conn.execute("DELETE FROM confession_reactions WHERE confession_id = ?1", rusqlite::params![id]);

            // Delete the confession
            match conn.execute("DELETE FROM confessions WHERE id = ?1", rusqlite::params![id]) {
                Ok(_) => json_response(&serde_json::json!({ "success": true })),
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn handle_submit_confession(req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    match read_json_body(req).await {
        Ok(body) => {
            let confession_text = match body.get("confession_text").and_then(|v| v.as_str()) {
                Some(text) => text.to_string(),
                None => return error_response(StatusCode::BAD_REQUEST, "Missing confession_text"),
            };
            let submitted_by = body.get("submitted_by").and_then(|v| v.as_str()).map(|s| s.to_string());

            let conn = crate::core::database::get_database_path();
            match rusqlite::Connection::open(&conn) {
                Ok(conn) => {
                    // Get settings
                    let require_moderation: bool = conn.query_row(
                        "SELECT value FROM confession_settings WHERE key = 'require_moderation'",
                        [],
                        |row| row.get::<_, String>(0),
                    ).map(|v| v == "true").unwrap_or(true);

                    let min_length: i64 = conn.query_row(
                        "SELECT value FROM confession_settings WHERE key = 'min_length'",
                        [],
                        |row| row.get::<_, String>(0),
                    ).ok().and_then(|v| v.parse().ok()).unwrap_or(10);

                    let max_length: i64 = conn.query_row(
                        "SELECT value FROM confession_settings WHERE key = 'max_length'",
                        [],
                        |row| row.get::<_, String>(0),
                    ).ok().and_then(|v| v.parse().ok()).unwrap_or(500);

                    // Validate length
                    let text_len = confession_text.len() as i64;
                    if text_len < min_length {
                        return error_response(StatusCode::BAD_REQUEST, &format!("Confession too short (minimum {} characters)", min_length));
                    }
                    if text_len > max_length {
                        return error_response(StatusCode::BAD_REQUEST, &format!("Confession too long (maximum {} characters)", max_length));
                    }

                    let now = current_timestamp();
                    let status = if require_moderation { "pending" } else { "approved" };

                    match conn.execute(
                        "INSERT INTO confessions (confession_text, status, submitted_by, submitted_at, approved_at)
                         VALUES (?1, ?2, ?3, ?4, ?5)",
                        rusqlite::params![
                            confession_text,
                            status,
                            submitted_by,
                            now,
                            if require_moderation { None } else { Some(now) }
                        ],
                    ) {
                        Ok(_) => {
                            let id = conn.last_insert_rowid();
                            json_response(&serde_json::json!({
                                "id": id,
                                "success": true,
                                "status": status
                            }))
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

async fn handle_get_pending_confessions() -> Response<BoxBody<Bytes, Infallible>> {
    let conn = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&conn) {
        Ok(conn) => {
            let mut stmt = match conn.prepare(
                "SELECT id, confession_text, submitted_at FROM confessions
                 WHERE status = 'pending'
                 ORDER BY submitted_at ASC"
            ) {
                Ok(stmt) => stmt,
                Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            };

            let mapped = match stmt.query_map([], |row| {
                Ok(serde_json::json!({
                    "id": row.get::<_, i64>(0)?,
                    "confession_text": row.get::<_, String>(1)?,
                    "submitted_at": row.get::<_, i64>(2)?,
                }))
            }) {
                Ok(m) => m,
                Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            };

            let confessions_result: Result<Vec<_>, _> = mapped.collect();

            match confessions_result {
                Ok(confessions) => json_response(&serde_json::json!({ "confessions": confessions })),
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn handle_approve_confession(req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    match read_json_body(req).await {
        Ok(body) => {
            let id = match body.get("id").and_then(|v| v.as_i64()) {
                Some(id) => id,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing id"),
            };
            let approved_by = match body.get("approved_by").and_then(|v| v.as_str()) {
                Some(user) => user,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing approved_by"),
            };

            let conn = crate::core::database::get_database_path();
            match rusqlite::Connection::open(&conn) {
                Ok(conn) => {
                    let now = current_timestamp();
                    match conn.execute(
                        "UPDATE confessions SET status = 'approved', approved_at = ?1, approved_by = ?2
                         WHERE id = ?3 AND status = 'pending'",
                        rusqlite::params![now, approved_by, id],
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

async fn handle_reject_confession(req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    match read_json_body(req).await {
        Ok(body) => {
            let id = match body.get("id").and_then(|v| v.as_i64()) {
                Some(id) => id,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing id"),
            };
            let rejected_by = match body.get("rejected_by").and_then(|v| v.as_str()) {
                Some(user) => user,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing rejected_by"),
            };
            let rejection_reason = body.get("rejection_reason").and_then(|v| v.as_str()).map(|s| s.to_string());

            let conn = crate::core::database::get_database_path();
            match rusqlite::Connection::open(&conn) {
                Ok(conn) => {
                    let now = current_timestamp();
                    match conn.execute(
                        "UPDATE confessions SET status = 'rejected', rejected_at = ?1, rejected_by = ?2, rejection_reason = ?3
                         WHERE id = ?4 AND status = 'pending'",
                        rusqlite::params![now, rejected_by, rejection_reason, id],
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

async fn handle_get_random_confession() -> Response<BoxBody<Bytes, Infallible>> {
    let conn = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&conn) {
        Ok(conn) => {
            use rusqlite::OptionalExtension;

            let confession: Option<serde_json::Value> = match conn.query_row(
                "SELECT id, confession_text, display_count FROM confessions
                 WHERE status = 'approved'
                 ORDER BY RANDOM()
                 LIMIT 1",
                [],
                |row| {
                    Ok(serde_json::json!({
                        "id": row.get::<_, i64>(0)?,
                        "confession_text": row.get::<_, String>(1)?,
                        "display_count": row.get::<_, i64>(2)?,
                    }))
                }
            ).optional() {
                Ok(conf) => conf,
                Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            };

            // Increment display count
            if let Some(ref conf) = confession {
                if let Some(id) = conf["id"].as_i64() {
                    let now = current_timestamp();
                    let _ = conn.execute(
                        "UPDATE confessions SET display_count = display_count + 1, displayed_at = ?1
                         WHERE id = ?2",
                        rusqlite::params![now, id],
                    );
                }
            }

            json_response(&serde_json::json!({ "confession": confession }))
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn handle_add_reaction(req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    match read_json_body(req).await {
        Ok(body) => {
            let confession_id = match body.get("confession_id").and_then(|v| v.as_i64()) {
                Some(id) => id,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing confession_id"),
            };
            let user_id = match body.get("user_id").and_then(|v| v.as_str()) {
                Some(id) => id,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing user_id"),
            };
            let reaction_type = match body.get("reaction_type").and_then(|v| v.as_str()) {
                Some(r_type) => r_type,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing reaction_type"),
            };

            let conn = crate::core::database::get_database_path();
            match rusqlite::Connection::open(&conn) {
                Ok(conn) => {
                    let now = current_timestamp();
                    match conn.execute(
                        "INSERT OR IGNORE INTO confession_reactions (confession_id, user_id, reaction_type, created_at)
                         VALUES (?1, ?2, ?3, ?4)",
                        rusqlite::params![confession_id, user_id, reaction_type, now],
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

async fn handle_get_stats() -> Response<BoxBody<Bytes, Infallible>> {
    let conn = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&conn) {
        Ok(conn) => {
            let total: i64 = match conn.query_row("SELECT COUNT(*) FROM confessions", [], |row| row.get(0)) {
                Ok(count) => count,
                Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            };
            let pending: i64 = match conn.query_row("SELECT COUNT(*) FROM confessions WHERE status = 'pending'", [], |row| row.get(0)) {
                Ok(count) => count,
                Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            };
            let approved: i64 = match conn.query_row("SELECT COUNT(*) FROM confessions WHERE status = 'approved'", [], |row| row.get(0)) {
                Ok(count) => count,
                Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            };
            let rejected: i64 = match conn.query_row("SELECT COUNT(*) FROM confessions WHERE status = 'rejected'", [], |row| row.get(0)) {
                Ok(count) => count,
                Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            };

            json_response(&serde_json::json!({
                "total": total,
                "pending": pending,
                "approved": approved,
                "rejected": rejected
            }))
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn handle_update_setting(req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    match read_json_body(req).await {
        Ok(body) => {
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
                    let now = current_timestamp();
                    match conn.execute(
                        "INSERT OR REPLACE INTO confession_settings (key, value, updated_at) VALUES (?1, ?2, ?3)",
                        rusqlite::params![key, value, now],
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

fn cors_preflight_response() -> Response<BoxBody<Bytes, Infallible>> {
    Response::builder()
        .status(StatusCode::OK)
        .header("Access-Control-Allow-Origin", "*")
        .header("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE, OPTIONS")
        .header("Access-Control-Allow-Headers", "Content-Type")
        .body(full_body(""))
        .unwrap()
}
