use crate::core::plugin_context::PluginContext;
use crate::core::plugin_router::PluginRouter;
use anyhow::Result;
use hyper::{Method, Request, Response, StatusCode, body::Incoming};
use hyper::body::Bytes;
use http_body_util::{Full, combinators::BoxBody};
use std::convert::Infallible;

pub async fn register_routes(ctx: &PluginContext) -> Result<()> {
    let mut router = PluginRouter::new();

    // POST /ticker/messages - Add a ticker message
    router.route(Method::POST, "/messages", |_path, _query, req| {
        Box::pin(async move {
            handle_add_message(req).await
        })
    });

    // GET /ticker/messages - Get all ticker messages
    router.route(Method::GET, "/messages", |_path, _query, _req| {
        Box::pin(async move {
            handle_get_messages().await
        })
    });

    // DELETE /ticker/messages/:id - Delete a ticker message
    router.route(Method::DELETE, "/messages/:id", |path, _query, _req| {
        Box::pin(async move {
            handle_delete_message(path).await
        })
    });

    // GET /ticker/events - Get all ticker events
    router.route(Method::GET, "/events", |_path, _query, _req| {
        Box::pin(async move {
            handle_get_events().await
        })
    });

    // POST /ticker/events - Add a ticker event
    router.route(Method::POST, "/events", |_path, _query, req| {
        Box::pin(async move {
            handle_add_event(req).await
        })
    });

    // GET /ticker/segments - Get all ticker segments
    router.route(Method::GET, "/segments", |_path, _query, _req| {
        Box::pin(async move {
            handle_get_segments().await
        })
    });

    // POST /ticker/segments - Add a ticker segment
    router.route(Method::POST, "/segments", |_path, _query, req| {
        Box::pin(async move {
            handle_add_segment(req).await
        })
    });

    // PUT /ticker/segments/:id - Update a ticker segment
    router.route(Method::PUT, "/segments/:id", |path, _query, req| {
        Box::pin(async move {
            handle_update_segment(path, req).await
        })
    });

    // DELETE /ticker/segments/:id - Delete a ticker segment
    router.route(Method::DELETE, "/segments/:id", |path, _query, _req| {
        Box::pin(async move {
            handle_delete_segment(path).await
        })
    });

    // POST /ticker/segments/reorder - Reorder segments
    router.route(Method::POST, "/segments/reorder", |_path, _query, req| {
        Box::pin(async move {
            handle_reorder_segments(req).await
        })
    });

    // GET /ticker/events/config - Get events configuration
    router.route(Method::GET, "/events/config", |_path, _query, _req| {
        Box::pin(async move {
            handle_get_events_config().await
        })
    });

    // POST /ticker/events/config - Update events configuration
    router.route(Method::POST, "/events/config", |_path, _query, req| {
        Box::pin(async move {
            handle_update_events_config(req).await
        })
    });

    // GET /ticker/messages/enabled - Get only enabled messages
    router.route(Method::GET, "/messages/enabled", |_path, _query, _req| {
        Box::pin(async move {
            handle_get_enabled_messages().await
        })
    });

    // POST /ticker/messages/toggle - Toggle message enabled status
    router.route(Method::POST, "/messages/toggle", |_path, _query, req| {
        Box::pin(async move {
            handle_toggle_message(req).await
        })
    });

    // POST /ticker/messages/toggle-sticky - Toggle message sticky status
    router.route(Method::POST, "/messages/toggle-sticky", |_path, _query, req| {
        Box::pin(async move {
            handle_toggle_sticky(req).await
        })
    });

    ctx.register_router("ticker", router).await;
    Ok(())
}

async fn handle_add_message(req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    match read_json_body(req).await {
        Ok(body) => {
            let message = match body.get("message").and_then(|v| v.as_str()) {
                Some(msg) => msg,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing message"),
            };
            let is_sticky = body.get("is_sticky").and_then(|v| v.as_bool()).unwrap_or(false);

            let conn = crate::core::database::get_database_path();
            match rusqlite::Connection::open(&conn) {
                Ok(conn) => {
                    let now = current_timestamp();
                    match conn.execute(
                        "INSERT INTO ticker_messages (message, enabled, is_sticky, created_at, updated_at)
                         VALUES (?1, 1, ?2, ?3, ?3)",
                        rusqlite::params![message, is_sticky as i64, now],
                    ) {
                        Ok(_) => {
                            let id = conn.last_insert_rowid();
                            json_response(&serde_json::json!({ "id": id }))
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

async fn handle_get_messages() -> Response<BoxBody<Bytes, Infallible>> {
    let conn = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&conn) {
        Ok(conn) => {
            let mut stmt = match conn.prepare(
                "SELECT id, message, enabled, is_sticky, created_at, updated_at
                 FROM ticker_messages WHERE enabled = 1 ORDER BY created_at DESC"
            ) {
                Ok(stmt) => stmt,
                Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            };

            let messages_result: Result<Vec<_>, _> = stmt.query_map([], |row| {
                Ok(serde_json::json!({
                    "id": row.get::<_, i64>(0)?,
                    "message": row.get::<_, String>(1)?,
                    "enabled": row.get::<_, i64>(2)? != 0,
                    "is_sticky": row.get::<_, i64>(3)? != 0,
                    "created_at": row.get::<_, i64>(4)?,
                    "updated_at": row.get::<_, i64>(5)?,
                }))
            }).and_then(|rows| rows.collect());

            match messages_result {
                Ok(msgs) => json_response(&serde_json::json!({ "messages": msgs })),
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn handle_delete_message(path: String) -> Response<BoxBody<Bytes, Infallible>> {
    let id_str = extract_path_param(&path, "/messages/");
    if id_str.is_empty() {
        return error_response(StatusCode::BAD_REQUEST, "Missing message id");
    }

    let id: i64 = match id_str.parse() {
        Ok(id) => id,
        Err(_) => return error_response(StatusCode::BAD_REQUEST, "Invalid message id"),
    };

    let conn = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&conn) {
        Ok(conn) => {
            match conn.execute("DELETE FROM ticker_messages WHERE id = ?1", rusqlite::params![id]) {
                Ok(_) => json_response(&serde_json::json!({ "success": true })),
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn handle_get_events() -> Response<BoxBody<Bytes, Infallible>> {
    let conn = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&conn) {
        Ok(conn) => {
            let mut stmt = match conn.prepare(
                "SELECT id, event_type, event_data, display_text, is_sticky, created_at
                 FROM ticker_events ORDER BY created_at DESC LIMIT 100"
            ) {
                Ok(stmt) => stmt,
                Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            };

            let events_result: Result<Vec<_>, _> = stmt.query_map([], |row| {
                Ok(serde_json::json!({
                    "id": row.get::<_, i64>(0)?,
                    "event_type": row.get::<_, String>(1)?,
                    "event_data": row.get::<_, String>(2)?,
                    "display_text": row.get::<_, String>(3)?,
                    "is_sticky": row.get::<_, i64>(4)? != 0,
                    "created_at": row.get::<_, i64>(5)?,
                }))
            }).and_then(|rows| rows.collect());

            match events_result {
                Ok(evts) => json_response(&evts),
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn handle_add_event(req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    match read_json_body(req).await {
        Ok(body) => {
            let event_type = match body.get("event_type").and_then(|v| v.as_str()) {
                Some(et) => et,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing event_type"),
            };
            let event_data = match body.get("event_data").and_then(|v| v.as_str()) {
                Some(ed) => ed,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing event_data"),
            };
            let display_text = match body.get("display_text").and_then(|v| v.as_str()) {
                Some(dt) => dt,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing display_text"),
            };
            let is_sticky = body.get("is_sticky").and_then(|v| v.as_bool()).unwrap_or(false);

            let conn = crate::core::database::get_database_path();
            match rusqlite::Connection::open(&conn) {
                Ok(conn) => {
                    let now = current_timestamp();
                    match conn.execute(
                        "INSERT INTO ticker_events (event_type, event_data, display_text, is_sticky, created_at)
                         VALUES (?1, ?2, ?3, ?4, ?5)",
                        rusqlite::params![event_type, event_data, display_text, is_sticky as i64, now],
                    ) {
                        Ok(_) => {
                            let id = conn.last_insert_rowid();
                            json_response(&serde_json::json!({ "id": id }))
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

// Segment handlers
async fn handle_get_segments() -> Response<BoxBody<Bytes, Infallible>> {
    let conn = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&conn) {
        Ok(conn) => {
            let mut stmt = match conn.prepare(
                "SELECT id, type, enabled, content, position, created_at, updated_at
                 FROM ticker_segments ORDER BY position ASC"
            ) {
                Ok(stmt) => stmt,
                Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            };

            let segments_result: Result<Vec<_>, _> = stmt.query_map([], |row| {
                let content_str: String = row.get(3)?;
                let content: serde_json::Value = serde_json::from_str(&content_str).unwrap_or(serde_json::json!({}));

                Ok(serde_json::json!({
                    "id": row.get::<_, i64>(0)?,
                    "type": row.get::<_, String>(1)?,
                    "enabled": row.get::<_, i64>(2)? != 0,
                    "content": content,
                    "position": row.get::<_, i64>(4)?,
                    "created_at": row.get::<_, i64>(5)?,
                    "updated_at": row.get::<_, i64>(6)?,
                }))
            }).and_then(|rows| rows.collect());

            match segments_result {
                Ok(segs) => json_response(&segs),
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn handle_add_segment(req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    match read_json_body(req).await {
        Ok(body) => {
            let segment_type = match body.get("type").and_then(|v| v.as_str()) {
                Some(t) => t,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing type"),
            };
            let enabled = body.get("enabled").and_then(|v| v.as_bool()).unwrap_or(true);
            let content = body.get("content").cloned().unwrap_or(serde_json::json!({}));
            let content_str = serde_json::to_string(&content).unwrap_or_default();

            let conn = crate::core::database::get_database_path();
            match rusqlite::Connection::open(&conn) {
                Ok(conn) => {
                    // Get max position
                    let position: i64 = conn.query_row(
                        "SELECT COALESCE(MAX(position), -1) + 1 FROM ticker_segments",
                        [],
                        |row| row.get(0)
                    ).unwrap_or(0);

                    let now = current_timestamp();
                    match conn.execute(
                        "INSERT INTO ticker_segments (type, enabled, content, position, created_at, updated_at)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?5)",
                        rusqlite::params![segment_type, enabled as i64, content_str, position, now],
                    ) {
                        Ok(_) => {
                            let id = conn.last_insert_rowid();
                            json_response(&serde_json::json!({ "id": id }))
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

async fn handle_update_segment(path: String, req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    let id_str = extract_path_param(&path, "/segments/");
    if id_str.is_empty() {
        return error_response(StatusCode::BAD_REQUEST, "Missing segment id");
    }

    let id: i64 = match id_str.parse() {
        Ok(id) => id,
        Err(_) => return error_response(StatusCode::BAD_REQUEST, "Invalid segment id"),
    };

    match read_json_body(req).await {
        Ok(body) => {
            let conn = crate::core::database::get_database_path();
            match rusqlite::Connection::open(&conn) {
                Ok(conn) => {
                    let now = current_timestamp();
                    let mut updates = vec![];
                    let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![];

                    if let Some(enabled) = body.get("enabled").and_then(|v| v.as_bool()) {
                        updates.push("enabled = ?");
                        params.push(Box::new(enabled as i64));
                    }
                    if let Some(content) = body.get("content") {
                        updates.push("content = ?");
                        params.push(Box::new(serde_json::to_string(content).unwrap_or_default()));
                    }

                    if updates.is_empty() {
                        return json_response(&serde_json::json!({ "success": true }));
                    }

                    updates.push("updated_at = ?");
                    params.push(Box::new(now));
                    params.push(Box::new(id));

                    let query = format!("UPDATE ticker_segments SET {} WHERE id = ?", updates.join(", "));
                    let params_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

                    match conn.execute(&query, params_refs.as_slice()) {
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

async fn handle_delete_segment(path: String) -> Response<BoxBody<Bytes, Infallible>> {
    let id_str = extract_path_param(&path, "/segments/");
    if id_str.is_empty() {
        return error_response(StatusCode::BAD_REQUEST, "Missing segment id");
    }

    let id: i64 = match id_str.parse() {
        Ok(id) => id,
        Err(_) => return error_response(StatusCode::BAD_REQUEST, "Invalid segment id"),
    };

    let conn = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&conn) {
        Ok(conn) => {
            match conn.execute("DELETE FROM ticker_segments WHERE id = ?1", rusqlite::params![id]) {
                Ok(_) => json_response(&serde_json::json!({ "success": true })),
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn handle_reorder_segments(req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    match read_json_body(req).await {
        Ok(body) => {
            let segments = match body.get("segments").and_then(|v| v.as_array()) {
                Some(segs) => segs,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing segments array"),
            };

            let conn = crate::core::database::get_database_path();
            match rusqlite::Connection::open(&conn) {
                Ok(conn) => {
                    for (index, segment) in segments.iter().enumerate() {
                        let id = match segment.get("id").and_then(|v| v.as_i64()) {
                            Some(id) => id,
                            None => continue,
                        };

                        if let Err(e) = conn.execute(
                            "UPDATE ticker_segments SET position = ?1 WHERE id = ?2",
                            rusqlite::params![index as i64, id],
                        ) {
                            return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string());
                        }
                    }
                    json_response(&serde_json::json!({ "success": true }))
                }
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::BAD_REQUEST, &e),
    }
}

// Events config handlers
async fn handle_get_events_config() -> Response<BoxBody<Bytes, Infallible>> {
    let conn = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&conn) {
        Ok(conn) => {
            let config_json: String = conn.query_row(
                "SELECT value FROM ticker_config WHERE key = 'events_config'",
                [],
                |row| row.get(0)
            ).unwrap_or_else(|_| serde_json::json!({
                "show_followers": true,
                "show_subscribers": true,
                "show_raids": true,
                "show_donations": true,
                "show_gifted_subs": true,
                "show_cheers": true
            }).to_string());

            let config: serde_json::Value = serde_json::from_str(&config_json).unwrap_or(serde_json::json!({}));
            json_response(&config)
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn handle_update_events_config(req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    match read_json_body(req).await {
        Ok(body) => {
            let config_str = serde_json::to_string(&body).unwrap_or_default();

            let conn = crate::core::database::get_database_path();
            match rusqlite::Connection::open(&conn) {
                Ok(conn) => {
                    // Create config table if it doesn't exist
                    let _ = conn.execute(
                        "CREATE TABLE IF NOT EXISTS ticker_config (
                            key TEXT PRIMARY KEY,
                            value TEXT NOT NULL
                        )",
                        [],
                    );

                    match conn.execute(
                        "INSERT OR REPLACE INTO ticker_config (key, value) VALUES ('events_config', ?1)",
                        rusqlite::params![config_str],
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

// Message handlers
async fn handle_get_enabled_messages() -> Response<BoxBody<Bytes, Infallible>> {
    let conn = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&conn) {
        Ok(conn) => {
            let mut stmt = match conn.prepare(
                "SELECT id, message, enabled, is_sticky, created_at, updated_at
                 FROM ticker_messages WHERE enabled = 1 ORDER BY created_at DESC"
            ) {
                Ok(stmt) => stmt,
                Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            };

            let messages_result: Result<Vec<_>, _> = stmt.query_map([], |row| {
                Ok(serde_json::json!({
                    "id": row.get::<_, i64>(0)?,
                    "message": row.get::<_, String>(1)?,
                    "enabled": row.get::<_, i64>(2)? != 0,
                    "is_sticky": row.get::<_, i64>(3)? != 0,
                    "created_at": row.get::<_, i64>(4)?,
                    "updated_at": row.get::<_, i64>(5)?,
                }))
            }).and_then(|rows| rows.collect());

            match messages_result {
                Ok(msgs) => json_response(&msgs),
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn handle_toggle_message(req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    match read_json_body(req).await {
        Ok(body) => {
            let id = match body.get("id").and_then(|v| v.as_i64()) {
                Some(id) => id,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing id"),
            };

            let conn = crate::core::database::get_database_path();
            match rusqlite::Connection::open(&conn) {
                Ok(conn) => {
                    match conn.execute(
                        "UPDATE ticker_messages SET enabled = NOT enabled, updated_at = ?1 WHERE id = ?2",
                        rusqlite::params![current_timestamp(), id],
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

async fn handle_toggle_sticky(req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    match read_json_body(req).await {
        Ok(body) => {
            let id = match body.get("id").and_then(|v| v.as_i64()) {
                Some(id) => id,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing id"),
            };

            let conn = crate::core::database::get_database_path();
            match rusqlite::Connection::open(&conn) {
                Ok(conn) => {
                    match conn.execute(
                        "UPDATE ticker_messages SET is_sticky = NOT is_sticky, updated_at = ?1 WHERE id = ?2",
                        rusqlite::params![current_timestamp(), id],
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
