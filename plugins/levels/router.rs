use crate::core::plugin_context::PluginContext;
use crate::core::plugin_router::PluginRouter;
use anyhow::Result;
use hyper::{Method, Request, Response, StatusCode, body::Incoming};
use hyper::body::Bytes;
use http_body_util::{Full, combinators::BoxBody};
use std::convert::Infallible;

pub async fn register_routes(ctx: &PluginContext) -> Result<()> {
    let mut router = PluginRouter::new();

    // GET /levels/user/:channel/:username - Get user level information
    router.route(Method::GET, "/user/:channel/:username", |path, _query, _req| {
        Box::pin(async move {
            handle_get_level(path).await
        })
    });

    // POST /levels/add_xp - Add XP to user
    router.route(Method::POST, "/add_xp", |_path, _query, req| {
        Box::pin(async move {
            handle_add_xp(req).await
        })
    });

    // GET /levels/leaderboard - Get XP leaderboard
    router.route(Method::GET, "/leaderboard", |_path, query, _req| {
        Box::pin(async move {
            handle_get_leaderboard(query).await
        })
    });

    ctx.register_router("levels", router).await;
    Ok(())
}

async fn handle_get_level(path: String) -> Response<BoxBody<Bytes, Infallible>> {
    // Path format: /user/:channel/:username
    let parts: Vec<&str> = path.trim_start_matches("/user/").split('/').collect();
    if parts.len() != 2 {
        return error_response(StatusCode::BAD_REQUEST, "Invalid path format, expected /user/:channel/:username");
    }

    let channel = urlencoding::decode(parts[0]).unwrap_or_default().to_string();
    let username = urlencoding::decode(parts[1]).unwrap_or_default().to_string();

    let conn = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&conn) {
        Ok(conn) => {
            match super::database::get_user_level(&conn, &channel, &username) {
                Ok(Some(user_level)) => json_response(&user_level),
                Ok(None) => error_response(StatusCode::NOT_FOUND, "User not found"),
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn handle_add_xp(req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    match read_json_body(req).await {
        Ok(body) => {
            let channel = match body.get("channel").and_then(|v| v.as_str()) {
                Some(c) => c,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing channel"),
            };
            let username = match body.get("username").and_then(|v| v.as_str()) {
                Some(name) => name,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing username"),
            };
            let amount = match body.get("amount").and_then(|v| v.as_i64()) {
                Some(amt) => amt,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing amount"),
            };
            let reason = body.get("reason").and_then(|v| v.as_str());

            let conn = crate::core::database::get_database_path();
            match rusqlite::Connection::open(&conn) {
                Ok(conn) => {
                    match super::database::add_xp(&conn, channel, username, amount, reason) {
                        Ok((old_level, new_level)) => {
                            json_response(&serde_json::json!({
                                "old_level": old_level,
                                "new_level": new_level,
                                "leveled_up": new_level > old_level
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

async fn handle_get_leaderboard(query: String) -> Response<BoxBody<Bytes, Infallible>> {
    let channel = match parse_query_param(&query, "channel") {
        Some(c) => c,
        None => return error_response(StatusCode::BAD_REQUEST, "Missing channel parameter"),
    };

    let limit = parse_query_param(&query, "limit")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(10);

    let conn = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&conn) {
        Ok(conn) => {
            match super::database::get_leaderboard(&conn, &channel, limit) {
                Ok(leaderboard) => json_response(&serde_json::json!({ "leaderboard": leaderboard })),
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
