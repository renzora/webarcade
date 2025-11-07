use crate::core::plugin_context::PluginContext;
use crate::core::plugin_router::PluginRouter;
use anyhow::Result;
use hyper::{Method, Request, Response, StatusCode, body::Incoming};
use hyper::body::Bytes;
use http_body_util::{Full, combinators::BoxBody};
use std::convert::Infallible;

pub async fn register_routes(ctx: &PluginContext) -> Result<()> {
    let mut router = PluginRouter::new();

    // GET /database/tables - List all tables
    router.route(Method::GET, "/tables", |_path, _query, _req| {
        Box::pin(async move {
            handle_get_tables().await
        })
    });

    // GET /database/schema?table=tablename - Get table schema
    router.route(Method::GET, "/schema", |_path, query, _req| {
        Box::pin(async move {
            handle_get_schema(query).await
        })
    });

    // POST /database/query - Execute SQL query
    router.route(Method::POST, "/query", |_path, _query, req| {
        Box::pin(async move {
            handle_execute_query(req).await
        })
    });

    // GET /database/todos/toggle - Get community tasks overlay enabled state
    router.route(Method::GET, "/todos/toggle", |_path, _query, _req| {
        Box::pin(async move {
            handle_get_community_tasks_state().await
        })
    });

    // POST /database/todos/toggle - Set community tasks overlay enabled state
    router.route(Method::POST, "/todos/toggle", |_path, _query, req| {
        Box::pin(async move {
            handle_set_community_tasks_state(req).await
        })
    });

    ctx.register_router("database", router).await;
    Ok(())
}

async fn handle_get_tables() -> Response<BoxBody<Bytes, Infallible>> {
    let db_path = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&db_path) {
        Ok(conn) => {
            let mut stmt = match conn.prepare(
                "SELECT name FROM sqlite_master WHERE type='table' ORDER BY name"
            ) {
                Ok(stmt) => stmt,
                Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            };

            let mapped = match stmt.query_map([], |row| row.get(0)) {
                Ok(m) => m,
                Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            };

            let tables: Result<Vec<String>, _> = mapped.collect();

            match tables {
                Ok(tables) => json_response(&tables),
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn handle_get_schema(query: String) -> Response<BoxBody<Bytes, Infallible>> {
    let table = parse_query_param(&query, "table");
    match table {
        Some(table_name) => {
            let db_path = crate::core::database::get_database_path();
            match rusqlite::Connection::open(&db_path) {
                Ok(conn) => {
                    let query_str = format!("SELECT sql FROM sqlite_master WHERE type='table' AND name=?");
                    match conn.query_row(&query_str, [&table_name], |row| row.get::<_, String>(0)) {
                        Ok(schema) => json_response(&serde_json::json!({"schema": schema})),
                        Err(e) => error_response(StatusCode::NOT_FOUND, &format!("Table not found: {}", e)),
                    }
                }
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        None => error_response(StatusCode::BAD_REQUEST, "Missing table parameter"),
    }
}

async fn handle_execute_query(req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    match read_json_body(req).await {
        Ok(body) => {
            let query = match body.get("query").and_then(|v| v.as_str()) {
                Some(q) => q,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing query parameter"),
            };

            let db_path = crate::core::database::get_database_path();
            match rusqlite::Connection::open(&db_path) {
                Ok(conn) => {
                    // Check if it's a SELECT query
                    let trimmed = query.trim_start().to_uppercase();
                    if trimmed.starts_with("SELECT") || trimmed.starts_with("PRAGMA") {
                        // Read query
                        match conn.prepare(query) {
                            Ok(mut stmt) => {
                                let columns: Vec<String> = stmt
                                    .column_names()
                                    .iter()
                                    .map(|s| s.to_string())
                                    .collect();

                                let mapped = match stmt.query_map([], |row| {
                                    let mut map = serde_json::Map::new();
                                    for (i, col) in columns.iter().enumerate() {
                                        let value: serde_json::Value = match row.get_ref(i) {
                                            Ok(val) => match val {
                                                rusqlite::types::ValueRef::Null => serde_json::Value::Null,
                                                rusqlite::types::ValueRef::Integer(i) => serde_json::json!(i),
                                                rusqlite::types::ValueRef::Real(f) => serde_json::json!(f),
                                                rusqlite::types::ValueRef::Text(s) => {
                                                    serde_json::Value::String(String::from_utf8_lossy(s).to_string())
                                                }
                                                rusqlite::types::ValueRef::Blob(b) => {
                                                    serde_json::Value::String(format!("<blob {} bytes>", b.len()))
                                                }
                                            },
                                            Err(_) => serde_json::Value::Null,
                                        };
                                        map.insert(col.clone(), value);
                                    }
                                    Ok(serde_json::Value::Object(map))
                                }) {
                                    Ok(m) => m,
                                    Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
                                };

                                let rows: Result<Vec<serde_json::Value>, _> = mapped.collect();

                                match rows {
                                    Ok(data) => json_response(&serde_json::json!({
                                        "success": true,
                                        "data": data,
                                        "count": data.len()
                                    })),
                                    Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
                                }
                            }
                            Err(e) => error_response(StatusCode::BAD_REQUEST, &e.to_string()),
                        }
                    } else {
                        // Write query (INSERT, UPDATE, DELETE, etc.)
                        match conn.execute(query, []) {
                            Ok(rows_affected) => json_response(&serde_json::json!({
                                "success": true,
                                "rows_affected": rows_affected,
                                "message": format!("{} row(s) affected", rows_affected)
                            })),
                            Err(e) => json_response(&serde_json::json!({
                                "success": false,
                                "error": e.to_string()
                            })),
                        }
                    }
                }
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::BAD_REQUEST, &e),
    }
}

async fn handle_get_community_tasks_state() -> Response<BoxBody<Bytes, Infallible>> {
    let db_path = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&db_path) {
        Ok(conn) => {
            // Try to get the setting, default to true if not found
            let enabled = conn.query_row(
                "SELECT value FROM settings WHERE key = 'community_tasks_enabled'",
                [],
                |row| row.get::<_, String>(0)
            )
            .ok()
            .and_then(|v| v.parse::<bool>().ok())
            .unwrap_or(true);

            json_response(&serde_json::json!({ "enabled": enabled }))
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn handle_set_community_tasks_state(req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    match read_json_body(req).await {
        Ok(body) => {
            let enabled = match body.get("enabled").and_then(|v| v.as_bool()) {
                Some(e) => e,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing enabled parameter"),
            };

            let db_path = crate::core::database::get_database_path();
            match rusqlite::Connection::open(&db_path) {
                Ok(conn) => {
                    // Ensure the settings table exists
                    if let Err(e) = conn.execute(
                        "CREATE TABLE IF NOT EXISTS settings (
                            key TEXT PRIMARY KEY,
                            value TEXT NOT NULL
                        )",
                        [],
                    ) {
                        return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string());
                    }

                    // Insert or update the setting
                    match conn.execute(
                        "INSERT OR REPLACE INTO settings (key, value) VALUES ('community_tasks_enabled', ?1)",
                        rusqlite::params![enabled.to_string()],
                    ) {
                        Ok(_) => json_response(&serde_json::json!({ "success": true, "enabled": enabled })),
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
async fn read_json_body(req: Request<Incoming>) -> std::result::Result<serde_json::Value, String> {
    use http_body_util::BodyExt;
    let whole_body = req.collect().await
        .map_err(|e| format!("Failed to read body: {}", e))?
        .to_bytes();

    serde_json::from_slice(&whole_body)
        .map_err(|e| format!("Invalid JSON: {}", e))
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
