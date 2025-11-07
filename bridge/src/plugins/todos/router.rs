use crate::core::plugin_context::PluginContext;
use crate::core::plugin_router::PluginRouter;
use anyhow::Result;
use hyper::{Method, Request, Response, StatusCode, body::Incoming};
use hyper::body::Bytes;
use http_body_util::{Full, combinators::BoxBody};
use std::convert::Infallible;

pub async fn register_routes(ctx: &PluginContext) -> Result<()> {
    let mut router = PluginRouter::new();

    // POST /todos/create - Create a new todo
    router.route(Method::POST, "/create", |_path, _query, req| {
        Box::pin(async move {
            handle_create_todo(req).await
        })
    });

    // GET /todos/list - Get todos for a channel
    router.route(Method::GET, "/list", |_path, query, _req| {
        Box::pin(async move {
            handle_get_todos(query).await
        })
    });

    // POST /todos/toggle - Toggle todo completion status
    router.route(Method::POST, "/toggle", |_path, _query, req| {
        Box::pin(async move {
            handle_toggle_todo(req).await
        })
    });

    // DELETE /todos/:todo_id - Delete a specific todo
    router.route(Method::DELETE, "/:todo_id", |path, _query, _req| {
        Box::pin(async move {
            handle_delete_todo(path).await
        })
    });

    // DELETE /todos/completed - Delete all completed todos for a channel
    router.route(Method::DELETE, "/completed", |_path, query, _req| {
        Box::pin(async move {
            handle_delete_completed_todos(query).await
        })
    });

    ctx.register_router("todos", router).await;
    Ok(())
}

async fn handle_create_todo(req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    match read_json_body(req).await {
        Ok(body) => {
            let channel = match body.get("channel").and_then(|v| v.as_str()) {
                Some(c) => c,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing channel"),
            };
            let username = match body.get("username").and_then(|v| v.as_str()) {
                Some(u) => u,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing username"),
            };
            let task = match body.get("task").and_then(|v| v.as_str()) {
                Some(t) => t,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing task"),
            };

            let conn = crate::core::database::get_database_path();
            match rusqlite::Connection::open(&conn) {
                Ok(conn) => {
                    match super::database::create_todo(&conn, channel, username, task) {
                        Ok(todo_id) => json_response(&serde_json::json!({ "todo_id": todo_id })),
                        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
                    }
                }
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::BAD_REQUEST, &e),
    }
}

async fn handle_get_todos(query: String) -> Response<BoxBody<Bytes, Infallible>> {
    let channel = match parse_query_param(&query, "channel") {
        Some(c) => c,
        None => return error_response(StatusCode::BAD_REQUEST, "Missing channel parameter"),
    };

    let completed = parse_query_param(&query, "completed")
        .and_then(|s| match s.as_str() {
            "true" => Some(true),
            "false" => Some(false),
            _ => None,
        });

    let conn = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&conn) {
        Ok(conn) => {
            match super::database::get_todos(&conn, &channel, completed) {
                Ok(todos) => json_response(&todos),
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn handle_toggle_todo(req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    match read_json_body(req).await {
        Ok(body) => {
            let todo_id = match body.get("todo_id").and_then(|v| v.as_i64()) {
                Some(id) => id,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing todo_id"),
            };

            let conn = crate::core::database::get_database_path();
            match rusqlite::Connection::open(&conn) {
                Ok(conn) => {
                    match super::database::toggle_todo(&conn, todo_id) {
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

async fn handle_delete_todo(path: String) -> Response<BoxBody<Bytes, Infallible>> {
    let todo_id = extract_path_param(&path, "/");
    let todo_id = match todo_id.parse::<i64>() {
        Ok(id) => id,
        Err(_) => return error_response(StatusCode::BAD_REQUEST, "Invalid todo_id"),
    };

    let conn = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&conn) {
        Ok(conn) => {
            match super::database::delete_todo(&conn, todo_id) {
                Ok(_) => json_response(&serde_json::json!({ "success": true })),
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn handle_delete_completed_todos(query: String) -> Response<BoxBody<Bytes, Infallible>> {
    let channel = match parse_query_param(&query, "channel") {
        Some(c) => c,
        None => return error_response(StatusCode::BAD_REQUEST, "Missing channel parameter"),
    };

    let conn = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&conn) {
        Ok(conn) => {
            match super::database::delete_completed_todos(&conn, &channel) {
                Ok(count) => json_response(&serde_json::json!({
                    "success": true,
                    "deleted_count": count
                })),
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
