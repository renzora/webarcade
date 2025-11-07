use crate::core::plugin_context::PluginContext;
use crate::core::plugin_router::PluginRouter;
use anyhow::Result;
use hyper::{Method, Request, Response, StatusCode, body::Incoming};
use hyper::body::Bytes;
use http_body_util::{Full, combinators::BoxBody};
use std::convert::Infallible;

pub async fn register_routes(ctx: &PluginContext) -> Result<()> {
    let mut router = PluginRouter::new();

    // GET /notes - Get all notes
    router.route(Method::GET, "/", |_path, query, _req| {
        Box::pin(async move {
            handle_get_notes(query).await
        })
    });

    // GET /notes/categories - Get all distinct categories
    router.route(Method::GET, "/categories", |_path, _query, _req| {
        Box::pin(async move {
            handle_get_categories().await
        })
    });

    // GET /notes/:id - Get note by ID
    router.route(Method::GET, "/:id", |path, _query, _req| {
        Box::pin(async move {
            handle_get_note(path).await
        })
    });

    // POST /notes - Create new note
    router.route(Method::POST, "/", |_path, _query, req| {
        Box::pin(async move {
            handle_create_note(req).await
        })
    });

    // PUT /notes/:id - Update note
    router.route(Method::PUT, "/:id", |path, _query, req| {
        Box::pin(async move {
            handle_update_note(path, req).await
        })
    });

    // DELETE /notes/:id - Delete note
    router.route(Method::DELETE, "/:id", |path, _query, _req| {
        Box::pin(async move {
            handle_delete_note(path).await
        })
    });

    // GET /notes/category/:category - Get notes by category
    router.route(Method::GET, "/category/:category", |path, query, _req| {
        Box::pin(async move {
            handle_get_notes_by_category(path, query).await
        })
    });

    ctx.register_router("notes", router).await;
    Ok(())
}

async fn handle_get_notes(query: String) -> Response<BoxBody<Bytes, Infallible>> {
    let limit = parse_query_param(&query, "limit")
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(100);

    let conn = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&conn) {
        Ok(conn) => {
            let mut stmt = match conn.prepare(
                "SELECT id, title, content, category, created_at, updated_at
                 FROM notes ORDER BY updated_at DESC LIMIT ?1"
            ) {
                Ok(stmt) => stmt,
                Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            };

            let notes: Result<Vec<serde_json::Value>, _> = stmt.query_map([limit], |row| {
                Ok(serde_json::json!({
                    "id": row.get::<_, i64>(0)?,
                    "title": row.get::<_, String>(1)?,
                    "content": row.get::<_, String>(2)?,
                    "category": row.get::<_, Option<String>>(3)?,
                    "created_at": row.get::<_, i64>(4)?,
                    "updated_at": row.get::<_, i64>(5)?,
                }))
            }).and_then(|rows| rows.collect());

            match notes {
                Ok(notes) => json_response(&serde_json::json!({ "notes": notes })),
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn handle_get_categories() -> Response<BoxBody<Bytes, Infallible>> {
    let conn = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&conn) {
        Ok(conn) => {
            let mut stmt = match conn.prepare(
                "SELECT DISTINCT category FROM notes WHERE category IS NOT NULL ORDER BY category"
            ) {
                Ok(stmt) => stmt,
                Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            };

            let categories: Result<Vec<String>, _> = stmt.query_map([], |row| {
                row.get::<_, String>(0)
            }).and_then(|rows| rows.collect());

            match categories {
                Ok(categories) => json_response(&categories),
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn handle_get_note(path: String) -> Response<BoxBody<Bytes, Infallible>> {
    let id = extract_path_param(&path, "/");
    if id.is_empty() {
        return error_response(StatusCode::BAD_REQUEST, "Missing note id");
    }

    let note_id = match id.parse::<i64>() {
        Ok(id) => id,
        Err(_) => return error_response(StatusCode::BAD_REQUEST, "Invalid note id"),
    };

    let conn = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&conn) {
        Ok(conn) => {
            let mut stmt = match conn.prepare(
                "SELECT id, title, content, category, created_at, updated_at
                 FROM notes WHERE id = ?1"
            ) {
                Ok(stmt) => stmt,
                Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            };

            match stmt.query_row([note_id], |row| {
                Ok(serde_json::json!({
                    "id": row.get::<_, i64>(0)?,
                    "title": row.get::<_, String>(1)?,
                    "content": row.get::<_, String>(2)?,
                    "category": row.get::<_, Option<String>>(3)?,
                    "created_at": row.get::<_, i64>(4)?,
                    "updated_at": row.get::<_, i64>(5)?,
                }))
            }) {
                Ok(note) => json_response(&note),
                Err(rusqlite::Error::QueryReturnedNoRows) => {
                    error_response(StatusCode::NOT_FOUND, "Note not found")
                }
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn handle_create_note(req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    match read_json_body(req).await {
        Ok(body) => {
            let title = match body.get("title").and_then(|v| v.as_str()) {
                Some(t) => t,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing title"),
            };
            let content = match body.get("content").and_then(|v| v.as_str()) {
                Some(c) => c,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing content"),
            };
            let category = body.get("category").and_then(|v| v.as_str());

            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64;

            let conn = crate::core::database::get_database_path();
            match rusqlite::Connection::open(&conn) {
                Ok(conn) => {
                    match conn.execute(
                        "INSERT INTO notes (title, content, category, created_at, updated_at)
                         VALUES (?1, ?2, ?3, ?4, ?5)",
                        rusqlite::params![title, content, category, now, now],
                    ) {
                        Ok(_) => {
                            let note_id = conn.last_insert_rowid();
                            json_response(&serde_json::json!({
                                "success": true,
                                "id": note_id
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

async fn handle_update_note(path: String, req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    let id = extract_path_param(&path, "/");
    if id.is_empty() {
        return error_response(StatusCode::BAD_REQUEST, "Missing note id");
    }

    let note_id = match id.parse::<i64>() {
        Ok(id) => id,
        Err(_) => return error_response(StatusCode::BAD_REQUEST, "Invalid note id"),
    };

    match read_json_body(req).await {
        Ok(body) => {
            let title = body.get("title").and_then(|v| v.as_str());
            let content = body.get("content").and_then(|v| v.as_str());
            let category = body.get("category").and_then(|v| v.as_str());

            if title.is_none() && content.is_none() && category.is_none() {
                return error_response(StatusCode::BAD_REQUEST, "No fields to update");
            }

            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64;

            let conn = crate::core::database::get_database_path();
            match rusqlite::Connection::open(&conn) {
                Ok(conn) => {
                    // Build dynamic update query
                    let mut updates = Vec::new();
                    let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

                    if let Some(t) = title {
                        updates.push("title = ?");
                        params.push(Box::new(t.to_string()));
                    }
                    if let Some(c) = content {
                        updates.push("content = ?");
                        params.push(Box::new(c.to_string()));
                    }
                    if let Some(cat) = category {
                        updates.push("category = ?");
                        params.push(Box::new(Some(cat.to_string())));
                    }
                    updates.push("updated_at = ?");
                    params.push(Box::new(now));

                    params.push(Box::new(note_id));

                    let query = format!(
                        "UPDATE notes SET {} WHERE id = ?",
                        updates.join(", ")
                    );

                    let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter()
                        .map(|p| p.as_ref())
                        .collect();

                    match conn.execute(&query, param_refs.as_slice()) {
                        Ok(updated) => {
                            if updated == 0 {
                                error_response(StatusCode::NOT_FOUND, "Note not found")
                            } else {
                                json_response(&serde_json::json!({ "success": true }))
                            }
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

async fn handle_delete_note(path: String) -> Response<BoxBody<Bytes, Infallible>> {
    let id = extract_path_param(&path, "/");
    if id.is_empty() {
        return error_response(StatusCode::BAD_REQUEST, "Missing note id");
    }

    let note_id = match id.parse::<i64>() {
        Ok(id) => id,
        Err(_) => return error_response(StatusCode::BAD_REQUEST, "Invalid note id"),
    };

    let conn = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&conn) {
        Ok(conn) => {
            match conn.execute("DELETE FROM notes WHERE id = ?1", [note_id]) {
                Ok(deleted) => {
                    if deleted == 0 {
                        error_response(StatusCode::NOT_FOUND, "Note not found")
                    } else {
                        json_response(&serde_json::json!({ "success": true }))
                    }
                }
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn handle_get_notes_by_category(path: String, query: String) -> Response<BoxBody<Bytes, Infallible>> {
    let category = extract_path_param(&path, "/category/");
    if category.is_empty() {
        return error_response(StatusCode::BAD_REQUEST, "Missing category");
    }

    let limit = parse_query_param(&query, "limit")
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(100);

    let conn = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&conn) {
        Ok(conn) => {
            let mut stmt = match conn.prepare(
                "SELECT id, title, content, category, created_at, updated_at
                 FROM notes WHERE category = ?1 ORDER BY updated_at DESC LIMIT ?2"
            ) {
                Ok(stmt) => stmt,
                Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            };

            let notes: Result<Vec<serde_json::Value>, _> = stmt.query_map(
                rusqlite::params![&category, limit],
                |row| {
                    Ok(serde_json::json!({
                        "id": row.get::<_, i64>(0)?,
                        "title": row.get::<_, String>(1)?,
                        "content": row.get::<_, String>(2)?,
                        "category": row.get::<_, Option<String>>(3)?,
                        "created_at": row.get::<_, i64>(4)?,
                        "updated_at": row.get::<_, i64>(5)?,
                    }))
                }
            ).and_then(|rows| rows.collect());

            match notes {
                Ok(notes) => json_response(&serde_json::json!({ "notes": notes })),
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
