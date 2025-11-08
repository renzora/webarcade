use crate::core::plugin_context::PluginContext;
use crate::core::plugin_router::PluginRouter;
use anyhow::Result;
use hyper::{Method, Request, Response, StatusCode, body::Incoming};
use hyper::body::Bytes;
use http_body_util::{Full, combinators::BoxBody};
use std::convert::Infallible;
use std::fs;

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

    // GET /database/export - Export entire database
    router.route(Method::GET, "/export", |_path, _query, _req| {
        Box::pin(async move {
            handle_export_database().await
        })
    });

    // POST /database/import - Import database from file
    router.route(Method::POST, "/import", |_path, _query, req| {
        Box::pin(async move {
            handle_import_database(req).await
        })
    });

    // GET /database/config - Get all config values or specific key
    router.route(Method::GET, "/config", |_path, query, _req| {
        Box::pin(async move {
            handle_get_config(query).await
        })
    });

    // POST /database/config - Save config values
    router.route(Method::POST, "/config", |_path, _query, req| {
        Box::pin(async move {
            handle_post_config(req).await
        })
    });

    // OPTIONS /database/config - CORS preflight for config
    router.route(Method::OPTIONS, "/config", |_path, _query, _req| {
        Box::pin(async move {
            handle_cors_preflight().await
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

fn bytes_body(bytes: Vec<u8>) -> BoxBody<Bytes, Infallible> {
    use http_body_util::combinators::BoxBody;
    use http_body_util::BodyExt;
    BoxBody::new(Full::new(Bytes::from(bytes)).map_err(|err: Infallible| match err {}))
}

async fn handle_export_database() -> Response<BoxBody<Bytes, Infallible>> {
    let db_path = crate::core::database::get_database_path();

    match fs::read(&db_path) {
        Ok(data) => {
            Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "application/octet-stream")
                .header("Content-Disposition", "attachment; filename=\"database_backup.db\"")
                .header("Access-Control-Allow-Origin", "*")
                .body(bytes_body(data))
                .unwrap()
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to read database: {}", e)),
    }
}

async fn handle_import_database(req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    use http_body_util::BodyExt;

    // Extract boundary before consuming the request
    let boundary = match req.headers()
        .get("content-type")
        .and_then(|ct| ct.to_str().ok())
        .and_then(|ct| ct.split("boundary=").nth(1))
        .map(|s| s.to_string())
    {
        Some(b) => b,
        None => return error_response(StatusCode::BAD_REQUEST, "Missing multipart boundary"),
    };

    // Read the entire body
    let whole_body = match req.collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(e) => return error_response(StatusCode::BAD_REQUEST, &format!("Failed to read body: {}", e)),
    };

    // Simple multipart parsing - look for the file data
    let body_str = String::from_utf8_lossy(&whole_body);
    let parts: Vec<&str> = body_str.split(&format!("--{}", boundary)).collect();

    let mut file_data: Option<Vec<u8>> = None;

    for part in parts {
        if part.contains("Content-Disposition") && part.contains("filename") {
            // Find the start of actual file data (after headers)
            if let Some(data_start) = part.find("\r\n\r\n") {
                let data = &part[data_start + 4..];
                // Remove trailing boundary markers
                let data = data.trim_end_matches("\r\n");
                file_data = Some(data.as_bytes().to_vec());
                break;
            }
        }
    }

    match file_data {
        Some(data) => {
            // Validate it's a SQLite database
            if data.len() < 16 || &data[0..16] != b"SQLite format 3\0" {
                return error_response(StatusCode::BAD_REQUEST, "Invalid SQLite database file");
            }

            let db_path = crate::core::database::get_database_path();

            // Create backup of current database
            let backup_path = format!("{}.backup", db_path.display());
            if let Err(e) = fs::copy(&db_path, &backup_path) {
                return error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to create backup: {}", e));
            }

            // Write new database
            match fs::write(&db_path, &data) {
                Ok(_) => json_response(&serde_json::json!({
                    "success": true,
                    "message": "Database imported successfully"
                })),
                Err(e) => {
                    // Restore from backup on failure
                    let _ = fs::copy(&backup_path, &db_path);
                    error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to write database: {}", e))
                }
            }
        }
        None => error_response(StatusCode::BAD_REQUEST, "No database file found in request"),
    }
}

async fn handle_get_config(query: String) -> Response<BoxBody<Bytes, Infallible>> {
    let db_path = crate::core::database::get_database_path();

    match rusqlite::Connection::open(&db_path) {
        Ok(conn) => {
            // Ensure config table exists
            if let Err(e) = conn.execute(
                "CREATE TABLE IF NOT EXISTS config (
                    key TEXT PRIMARY KEY,
                    value TEXT NOT NULL
                )",
                [],
            ) {
                return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string());
            }

            // Check if a specific key was requested
            if let Some(key) = parse_query_param(&query, "key") {
                match conn.query_row(
                    "SELECT value FROM config WHERE key = ?1",
                    rusqlite::params![key],
                    |row| row.get::<_, String>(0),
                ) {
                    Ok(value) => json_response(&serde_json::json!({ key: value })),
                    Err(rusqlite::Error::QueryReturnedNoRows) => {
                        json_response(&serde_json::json!({ key: serde_json::Value::Null }))
                    }
                    Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
                }
            } else {
                // Return all config values
                let mut stmt = match conn.prepare("SELECT key, value FROM config") {
                    Ok(s) => s,
                    Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
                };

                let mapped = match stmt.query_map([], |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
                }) {
                    Ok(m) => m,
                    Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
                };

                let mut config = serde_json::Map::new();
                for result in mapped {
                    match result {
                        Ok((key, value)) => {
                            config.insert(key, serde_json::Value::String(value));
                        }
                        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
                    }
                }

                json_response(&serde_json::Value::Object(config))
            }
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn handle_post_config(req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    match read_json_body(req).await {
        Ok(body) => {
            let db_path = crate::core::database::get_database_path();

            match rusqlite::Connection::open(&db_path) {
                Ok(conn) => {
                    // Ensure config table exists
                    if let Err(e) = conn.execute(
                        "CREATE TABLE IF NOT EXISTS config (
                            key TEXT PRIMARY KEY,
                            value TEXT NOT NULL
                        )",
                        [],
                    ) {
                        return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string());
                    }

                    // Save each key-value pair
                    if let Some(obj) = body.as_object() {
                        for (key, value) in obj {
                            let value_str = match value {
                                serde_json::Value::String(s) => s.clone(),
                                _ => value.to_string(),
                            };

                            if let Err(e) = conn.execute(
                                "INSERT OR REPLACE INTO config (key, value) VALUES (?1, ?2)",
                                rusqlite::params![key, value_str],
                            ) {
                                return error_response(
                                    StatusCode::INTERNAL_SERVER_ERROR,
                                    &format!("Failed to save config key '{}': {}", key, e)
                                );
                            }
                        }

                        json_response(&serde_json::json!({ "success": true }))
                    } else {
                        error_response(StatusCode::BAD_REQUEST, "Request body must be a JSON object")
                    }
                }
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::BAD_REQUEST, &e),
    }
}

async fn handle_cors_preflight() -> Response<BoxBody<Bytes, Infallible>> {
    Response::builder()
        .status(StatusCode::OK)
        .header("Access-Control-Allow-Origin", "*")
        .header("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE, OPTIONS")
        .header("Access-Control-Allow-Headers", "Content-Type, Authorization")
        .header("Access-Control-Max-Age", "86400")
        .body(full_body(""))
        .unwrap()
}
