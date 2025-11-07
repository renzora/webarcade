use crate::core::plugin_context::PluginContext;
use crate::core::plugin_router::PluginRouter;
use anyhow::Result;
use hyper::{Method, Request, Response, StatusCode, body::Incoming};
use hyper::body::Bytes;
use http_body_util::{Full, combinators::BoxBody};
use std::convert::Infallible;

pub async fn register_routes(ctx: &PluginContext) -> Result<()> {
    let mut router = PluginRouter::new();

    // POST /household/tasks - Create a new task
    router.route(Method::POST, "/tasks", |_path, _query, req| {
        Box::pin(async move {
            handle_create_task(req).await
        })
    });

    // GET /household/tasks - Get all tasks (with optional filters)
    router.route(Method::GET, "/tasks", |_path, query, _req| {
        Box::pin(async move {
            handle_get_tasks(query).await
        })
    });

    // DELETE /household/tasks/:task_id - Delete a task
    router.route(Method::DELETE, "/tasks/:task_id", |path, _query, _req| {
        Box::pin(async move {
            handle_delete_task(path).await
        })
    });

    // POST /household/tasks/:task_id/complete - Complete a task
    router.route(Method::POST, "/tasks/:task_id/complete", |path, _query, req| {
        Box::pin(async move {
            handle_complete_task(path, req).await
        })
    });

    // GET /household/completions - Get completion history
    router.route(Method::GET, "/completions", |_path, query, _req| {
        Box::pin(async move {
            handle_get_completions(query).await
        })
    });

    // POST /household/members - Add a household member
    router.route(Method::POST, "/members", |_path, _query, req| {
        Box::pin(async move {
            handle_add_member(req).await
        })
    });

    // GET /household/members - Get all household members
    router.route(Method::GET, "/members", |_path, _query, _req| {
        Box::pin(async move {
            handle_get_members().await
        })
    });

    // GET /household/stats - Get household statistics
    router.route(Method::GET, "/stats", |_path, query, _req| {
        Box::pin(async move {
            handle_get_stats(query).await
        })
    });

    ctx.register_router("household", router).await;
    Ok(())
}

async fn handle_create_task(req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    match read_json_body(req).await {
        Ok(body) => {
            let task_name = match body.get("task_name").and_then(|v| v.as_str()) {
                Some(name) => name,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing task_name"),
            };
            let created_by = match body.get("created_by").and_then(|v| v.as_str()) {
                Some(by) => by,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing created_by"),
            };

            let description = body.get("description").and_then(|v| v.as_str());
            let category = body.get("category").and_then(|v| v.as_str());
            let priority = body.get("priority").and_then(|v| v.as_str()).unwrap_or("medium");
            let recurrence = body.get("recurrence").and_then(|v| v.as_str());
            let recurrence_interval = body.get("recurrence_interval").and_then(|v| v.as_i64());
            let assigned_to = body.get("assigned_to").and_then(|v| v.as_str());

            let conn = crate::core::database::get_database_path();
            match rusqlite::Connection::open(&conn) {
                Ok(conn) => {
                    let now = current_timestamp();
                    let result = conn.execute(
                        "INSERT INTO household_tasks (task_name, description, category, priority, recurrence, recurrence_interval, assigned_to, created_by, created_at, updated_at)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?9)",
                        rusqlite::params![task_name, description, category, priority, recurrence, recurrence_interval, assigned_to, created_by, now],
                    );

                    match result {
                        Ok(_) => {
                            let task_id = conn.last_insert_rowid();
                            json_response(&serde_json::json!({ "id": task_id, "success": true }))
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

async fn handle_get_tasks(query: String) -> Response<BoxBody<Bytes, Infallible>> {
    let assigned_to = parse_query_param(&query, "assigned_to");
    let category = parse_query_param(&query, "category");

    let conn = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&conn) {
        Ok(conn) => {
            let mut sql = "SELECT id, task_name, description, category, priority, recurrence, recurrence_interval, assigned_to, created_at
                         FROM household_tasks WHERE 1=1".to_string();
            let mut params: Vec<String> = Vec::new();

            if let Some(ref assigned) = assigned_to {
                sql.push_str(" AND assigned_to = ?");
                params.push(assigned.clone());
            }

            if let Some(ref cat) = category {
                sql.push_str(" AND category = ?");
                params.push(cat.clone());
            }

            sql.push_str(" ORDER BY priority DESC, created_at DESC");

            let mut stmt = match conn.prepare(&sql) {
                Ok(stmt) => stmt,
                Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            };

            let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p as &dyn rusqlite::ToSql).collect();

            let mapped = match stmt.query_map(param_refs.as_slice(), |row| {
                Ok(serde_json::json!({
                    "id": row.get::<_, i64>(0)?,
                    "task_name": row.get::<_, String>(1)?,
                    "description": row.get::<_, Option<String>>(2)?,
                    "category": row.get::<_, Option<String>>(3)?,
                    "priority": row.get::<_, String>(4)?,
                    "recurrence": row.get::<_, Option<String>>(5)?,
                    "recurrence_interval": row.get::<_, Option<i64>>(6)?,
                    "assigned_to": row.get::<_, Option<String>>(7)?,
                    "created_at": row.get::<_, i64>(8)?,
                }))
            }) {
                Ok(m) => m,
                Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            };

            let tasks_result: Result<Vec<_>, _> = mapped.collect();

            match tasks_result {
                Ok(tasks) => json_response(&serde_json::json!({ "tasks": tasks })),
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn handle_delete_task(path: String) -> Response<BoxBody<Bytes, Infallible>> {
    let task_id_str = extract_path_param(&path, "/tasks/");
    let task_id = match task_id_str.parse::<i64>() {
        Ok(id) => id,
        Err(_) => return error_response(StatusCode::BAD_REQUEST, "Invalid task_id"),
    };

    let conn = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&conn) {
        Ok(conn) => {
            match conn.execute("DELETE FROM household_tasks WHERE id = ?1", rusqlite::params![task_id]) {
                Ok(_) => json_response(&serde_json::json!({ "success": true })),
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn handle_complete_task(path: String, req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    let task_id_str = extract_path_param(&path, "/tasks/");
    let parts: Vec<&str> = task_id_str.split('/').collect();
    let task_id = match parts.first().and_then(|s| s.parse::<i64>().ok()) {
        Some(id) => id,
        None => return error_response(StatusCode::BAD_REQUEST, "Invalid task_id"),
    };

    match read_json_body(req).await {
        Ok(body) => {
            let completed_by = match body.get("completed_by").and_then(|v| v.as_str()) {
                Some(by) => by,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing completed_by"),
            };
            let completion_notes = body.get("completion_notes").and_then(|v| v.as_str());

            let conn = crate::core::database::get_database_path();
            match rusqlite::Connection::open(&conn) {
                Ok(conn) => {
                    // Get task name
                    let task_name: Result<String, rusqlite::Error> = conn.query_row(
                        "SELECT task_name FROM household_tasks WHERE id = ?1",
                        rusqlite::params![task_id],
                        |row| row.get(0),
                    );

                    match task_name {
                        Ok(task_name) => {
                            let now = current_timestamp();
                            let result = conn.execute(
                                "INSERT INTO household_completions (task_id, task_name, completed_by, completion_notes, completed_at)
                                 VALUES (?1, ?2, ?3, ?4, ?5)",
                                rusqlite::params![task_id, task_name, completed_by, completion_notes, now],
                            );

                            match result {
                                Ok(_) => {
                                    let completion_id = conn.last_insert_rowid();
                                    json_response(&serde_json::json!({ "success": true, "completion_id": completion_id }))
                                }
                                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
                            }
                        }
                        Err(e) => error_response(StatusCode::NOT_FOUND, &format!("Task not found: {}", e)),
                    }
                }
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::BAD_REQUEST, &e),
    }
}

async fn handle_get_completions(query: String) -> Response<BoxBody<Bytes, Infallible>> {
    let task_id = parse_query_param(&query, "task_id").and_then(|s| s.parse::<i64>().ok());
    let completed_by = parse_query_param(&query, "completed_by");
    let limit = parse_query_param(&query, "limit")
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(50);

    let conn = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&conn) {
        Ok(conn) => {
            let mut sql = "SELECT id, task_id, task_name, completed_by, completion_notes, completed_at
                         FROM household_completions WHERE 1=1".to_string();
            let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

            if let Some(tid) = task_id {
                sql.push_str(" AND task_id = ?");
                params.push(Box::new(tid));
            }

            if let Some(ref by) = completed_by {
                sql.push_str(" AND completed_by = ?");
                params.push(Box::new(by.clone()));
            }

            sql.push_str(" ORDER BY completed_at DESC LIMIT ?");
            params.push(Box::new(limit));

            let mut stmt = match conn.prepare(&sql) {
                Ok(stmt) => stmt,
                Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            };

            let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

            let mapped = match stmt.query_map(param_refs.as_slice(), |row| {
                Ok(serde_json::json!({
                    "id": row.get::<_, i64>(0)?,
                    "task_id": row.get::<_, i64>(1)?,
                    "task_name": row.get::<_, String>(2)?,
                    "completed_by": row.get::<_, String>(3)?,
                    "completion_notes": row.get::<_, Option<String>>(4)?,
                    "completed_at": row.get::<_, i64>(5)?,
                }))
            }) {
                Ok(m) => m,
                Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            };

            let completions_result: Result<Vec<_>, _> = mapped.collect();

            match completions_result {
                Ok(completions) => json_response(&serde_json::json!({ "completions": completions })),
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn handle_add_member(req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    match read_json_body(req).await {
        Ok(body) => {
            let user_id = match body.get("user_id").and_then(|v| v.as_str()) {
                Some(id) => id,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing user_id"),
            };
            let username = match body.get("username").and_then(|v| v.as_str()) {
                Some(name) => name,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing username"),
            };
            let role = body.get("role").and_then(|v| v.as_str()).unwrap_or("member");

            let conn = crate::core::database::get_database_path();
            match rusqlite::Connection::open(&conn) {
                Ok(conn) => {
                    let now = current_timestamp();
                    let result = conn.execute(
                        "INSERT OR IGNORE INTO household_members (user_id, username, role, joined_at)
                         VALUES (?1, ?2, ?3, ?4)",
                        rusqlite::params![user_id, username, role, now],
                    );

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

async fn handle_get_members() -> Response<BoxBody<Bytes, Infallible>> {
    let conn = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&conn) {
        Ok(conn) => {
            let mut stmt = match conn.prepare(
                "SELECT user_id, username, role, joined_at FROM household_members ORDER BY joined_at"
            ) {
                Ok(stmt) => stmt,
                Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            };

            let mapped = match stmt.query_map([], |row| {
                Ok(serde_json::json!({
                    "user_id": row.get::<_, String>(0)?,
                    "username": row.get::<_, String>(1)?,
                    "role": row.get::<_, String>(2)?,
                    "joined_at": row.get::<_, i64>(3)?,
                }))
            }) {
                Ok(m) => m,
                Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            };

            let members_result: Result<Vec<_>, _> = mapped.collect();

            match members_result {
                Ok(members) => json_response(&serde_json::json!({ "members": members })),
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn handle_get_stats(query: String) -> Response<BoxBody<Bytes, Infallible>> {
    let user_id = parse_query_param(&query, "user_id");

    let conn = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&conn) {
        Ok(conn) => {
            let total_tasks: Result<i64, rusqlite::Error> = conn.query_row(
                "SELECT COUNT(*) FROM household_tasks",
                [],
                |row| row.get(0),
            );

            let completions_count: Result<i64, rusqlite::Error> = if let Some(ref uid) = user_id {
                conn.query_row(
                    "SELECT COUNT(*) FROM household_completions WHERE completed_by = ?1",
                    rusqlite::params![uid],
                    |row| row.get(0),
                )
            } else {
                conn.query_row("SELECT COUNT(*) FROM household_completions", [], |row| row.get(0))
            };

            match (total_tasks, completions_count) {
                (Ok(total_tasks), Ok(completions_count)) => {
                    json_response(&serde_json::json!({
                        "total_tasks": total_tasks,
                        "total_completions": completions_count
                    }))
                }
                (Err(e), _) | (_, Err(e)) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
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
