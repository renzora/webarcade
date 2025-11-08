use crate::core::plugin_context::PluginContext;
use crate::core::plugin_router::PluginRouter;
use anyhow::Result;
use hyper::{Method, Request, Response, StatusCode, body::Incoming};
use hyper::body::Bytes;
use http_body_util::{Full, combinators::BoxBody};
use std::convert::Infallible;
use rusqlite::OptionalExtension;
use crate::plugins::twitch::TwitchApiClient;

pub async fn register_routes(ctx: &PluginContext) -> Result<()> {
    let mut router = PluginRouter::new();

    // GET /goals/list?channel=xxx - Get all goals for a channel (query param)
    router.route(Method::GET, "/list", |_path, query, _req| {
        Box::pin(async move {
            handle_get_goals_query(query).await
        })
    });

    // GET /goals/goal/:goal_id - Get a specific goal by ID
    router.route(Method::GET, "/goal/:goal_id", |path, _query, _req| {
        Box::pin(async move {
            handle_get_goal(path).await
        })
    });

    // GET /goals/:channel - Get all goals for a channel (path param)
    router.route(Method::GET, "/:channel", |path, _query, _req| {
        Box::pin(async move {
            handle_get_goals(path).await
        })
    });

    // POST /goals/create - Create a new goal
    router.route(Method::POST, "/create", |_path, _query, req| {
        Box::pin(async move {
            handle_create_goal(req).await
        })
    });

    // POST /goals/update - Update a goal
    router.route(Method::POST, "/update", |_path, _query, req| {
        Box::pin(async move {
            handle_update_goal(req).await
        })
    });

    // POST /goals/progress - Update goal progress (increment)
    router.route(Method::POST, "/progress", |_path, _query, req| {
        Box::pin(async move {
            handle_update_progress(req).await
        })
    });

    // POST /goals/set-progress - Set goal progress (absolute value)
    router.route(Method::POST, "/set-progress", |_path, _query, req| {
        Box::pin(async move {
            handle_set_progress(req).await
        })
    });

    // DELETE /goals/:goal_id - Delete a goal
    router.route(Method::DELETE, "/:goal_id", |path, _query, _req| {
        Box::pin(async move {
            handle_delete_goal(path).await
        })
    });

    // POST /goals/sync-twitch - Sync goal with Twitch data
    router.route(Method::POST, "/sync-twitch", |_path, _query, req| {
        Box::pin(async move {
            handle_sync_twitch(req).await
        })
    });

    // POST /goals/:id/sync-twitch - Sync specific goal with Twitch data
    router.route(Method::POST, "/:id/sync-twitch", |path, _query, _req| {
        Box::pin(async move {
            handle_sync_twitch_by_id(path).await
        })
    });

    ctx.register_router("goals", router).await;
    Ok(())
}

async fn handle_get_goals_query(query: String) -> Response<BoxBody<Bytes, Infallible>> {
    let channel = extract_query_param(&query, "channel");
    if channel.is_empty() {
        return error_response(StatusCode::BAD_REQUEST, "Missing channel parameter");
    }

    get_goals_for_channel(&channel).await
}

async fn handle_get_goals(path: String) -> Response<BoxBody<Bytes, Infallible>> {
    let channel = extract_path_param(&path, "/");
    if channel.is_empty() {
        return error_response(StatusCode::BAD_REQUEST, "Missing channel");
    }

    get_goals_for_channel(&channel).await
}

async fn get_goals_for_channel(channel: &str) -> Response<BoxBody<Bytes, Infallible>> {
    let conn = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&conn) {
        Ok(conn) => {
            match super::database::get_goals(&conn, channel) {
                Ok(goals) => json_response(&goals),
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn handle_get_goal(path: String) -> Response<BoxBody<Bytes, Infallible>> {
    let goal_id_str = extract_path_param(&path, "/goal/");
    let goal_id = match goal_id_str.parse::<i64>() {
        Ok(id) => id,
        Err(_) => return error_response(StatusCode::BAD_REQUEST, "Invalid goal_id"),
    };

    let conn = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&conn) {
        Ok(conn) => {
            match super::database::get_goal(&conn, goal_id) {
                Ok(Some(goal)) => json_response(&goal),
                Ok(None) => error_response(StatusCode::NOT_FOUND, "Goal not found"),
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn handle_create_goal(req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    match read_json_body(req).await {
        Ok(body) => {
            let channel = match body.get("channel").and_then(|v| v.as_str()) {
                Some(c) => c,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing channel"),
            };
            let title = match body.get("title").and_then(|v| v.as_str()) {
                Some(t) => t,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing title"),
            };
            let description = body.get("description").and_then(|v| v.as_str());
            let goal_type = match body.get("type").and_then(|v| v.as_str()) {
                Some(t) => t,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing type"),
            };
            let target = match body.get("target").and_then(|v| v.as_i64()) {
                Some(t) => t,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing target"),
            };
            let is_sub_goal = body.get("is_sub_goal").and_then(|v| v.as_bool()).unwrap_or(false);

            let conn = crate::core::database::get_database_path();
            match rusqlite::Connection::open(&conn) {
                Ok(conn) => {
                    match super::database::create_goal(&conn, channel, title, description, goal_type, target, is_sub_goal) {
                        Ok(goal_id) => json_response(&serde_json::json!({ "goal_id": goal_id })),
                        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
                    }
                }
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::BAD_REQUEST, &e),
    }
}

async fn handle_update_goal(req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    match read_json_body(req).await {
        Ok(body) => {
            let goal_id = match body.get("goal_id").and_then(|v| v.as_i64()) {
                Some(id) => id,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing goal_id"),
            };
            let title = body.get("title").and_then(|v| v.as_str());
            let description = body.get("description").and_then(|v| v.as_str());
            let goal_type = body.get("type").and_then(|v| v.as_str());
            let target = body.get("target").and_then(|v| v.as_i64());
            let current = body.get("current").and_then(|v| v.as_i64());
            let is_sub_goal = body.get("is_sub_goal").and_then(|v| v.as_bool());

            let conn = crate::core::database::get_database_path();
            match rusqlite::Connection::open(&conn) {
                Ok(conn) => {
                    match super::database::update_goal(&conn, goal_id, title, description, goal_type, target, current, is_sub_goal) {
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

async fn handle_update_progress(req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    match read_json_body(req).await {
        Ok(body) => {
            let goal_id = match body.get("goal_id").and_then(|v| v.as_i64()) {
                Some(id) => id,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing goal_id"),
            };
            let amount = match body.get("amount").and_then(|v| v.as_i64()) {
                Some(amt) => amt,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing amount"),
            };

            let conn = crate::core::database::get_database_path();
            match rusqlite::Connection::open(&conn) {
                Ok(conn) => {
                    match super::database::update_progress(&conn, goal_id, amount) {
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

async fn handle_set_progress(req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    match read_json_body(req).await {
        Ok(body) => {
            let goal_id = match body.get("goal_id").and_then(|v| v.as_i64()) {
                Some(id) => id,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing goal_id"),
            };
            let current = match body.get("current").and_then(|v| v.as_i64()) {
                Some(cur) => cur,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing current"),
            };

            let conn = crate::core::database::get_database_path();
            match rusqlite::Connection::open(&conn) {
                Ok(conn) => {
                    match super::database::set_progress(&conn, goal_id, current) {
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

async fn handle_delete_goal(path: String) -> Response<BoxBody<Bytes, Infallible>> {
    let goal_id_str = extract_path_param(&path, "/");
    let goal_id = match goal_id_str.parse::<i64>() {
        Ok(id) => id,
        Err(_) => return error_response(StatusCode::BAD_REQUEST, "Invalid goal_id"),
    };

    let conn = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&conn) {
        Ok(conn) => {
            match super::database::delete_goal(&conn, goal_id) {
                Ok(_) => json_response(&serde_json::json!({ "success": true })),
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn handle_sync_twitch(req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    match read_json_body(req).await {
        Ok(body) => {
            let goal_id = match body.get("goal_id").and_then(|v| v.as_i64()) {
                Some(id) => id,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing goal_id"),
            };

            let conn = crate::core::database::get_database_path();
            match rusqlite::Connection::open(&conn) {
                Ok(conn) => {
                    // Get the goal to check its type
                    match super::database::get_goal(&conn, goal_id) {
                        Ok(Some(goal)) => {
                            // Check if goal type supports Twitch sync
                            match goal.goal_type.as_str() {
                                "follower" | "subscriber" => {
                                    error_response(StatusCode::NOT_IMPLEMENTED, "Twitch sync not implemented yet")
                                }
                                _ => {
                                    error_response(StatusCode::BAD_REQUEST, "Goal type does not support Twitch sync")
                                }
                            }
                        }
                        Ok(None) => error_response(StatusCode::NOT_FOUND, "Goal not found"),
                        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
                    }
                }
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::BAD_REQUEST, &e),
    }
}

async fn handle_sync_twitch_by_id(path: String) -> Response<BoxBody<Bytes, Infallible>> {
    let goal_id_str = extract_path_param(&path, "/");
    let goal_id_str = goal_id_str.trim_end_matches("/sync-twitch");

    let goal_id = match goal_id_str.parse::<i64>() {
        Ok(id) => id,
        Err(_) => return error_response(StatusCode::BAD_REQUEST, "Invalid goal_id"),
    };

    let db_path = crate::core::database::get_database_path();

    // Get goal info first (synchronously)
    let goal = {
        let conn = match rusqlite::Connection::open(&db_path) {
            Ok(c) => c,
            Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
        };

        match super::database::get_goal(&conn, goal_id) {
            Ok(Some(g)) => g,
            Ok(None) => return error_response(StatusCode::NOT_FOUND, "Goal not found"),
            Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
        }
    };

    // Check if goal type supports Twitch sync
    match goal.goal_type.as_str() {
        "follower" => {
            // Try to get follower count from Twitch
            match get_twitch_followers(&db_path, &goal.channel).await {
                Ok(follower_count) => {
                    // Update the goal's current progress
                    let conn = match rusqlite::Connection::open(&db_path) {
                        Ok(c) => c,
                        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
                    };

                    match super::database::set_progress(&conn, goal_id, follower_count) {
                        Ok(_) => json_response(&serde_json::json!({
                            "success": true,
                            "current": follower_count,
                            "message": format!("Synced {} followers from Twitch", follower_count)
                        })),
                        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
                    }
                }
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to sync with Twitch: {}", e)),
            }
        }
        "subscriber" => {
            // Try to get subscriber count from Twitch
            match get_twitch_subscribers(&db_path, &goal.channel).await {
                Ok(subscriber_count) => {
                    // Update the goal's current progress
                    let conn = match rusqlite::Connection::open(&db_path) {
                        Ok(c) => c,
                        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
                    };

                    match super::database::set_progress(&conn, goal_id, subscriber_count) {
                        Ok(_) => json_response(&serde_json::json!({
                            "success": true,
                            "current": subscriber_count,
                            "message": format!("Synced {} subscribers from Twitch", subscriber_count)
                        })),
                        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
                    }
                }
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to sync with Twitch: {}", e)),
            }
        }
        _ => {
            error_response(StatusCode::BAD_REQUEST, "Goal type does not support Twitch sync")
        }
    }
}

async fn get_twitch_followers(db_path: &std::path::PathBuf, channel: &str) -> Result<i64, String> {
    // Get all needed data from database first
    let (broadcaster_id, client_id, access_token) = {
        let conn = rusqlite::Connection::open(db_path)
            .map_err(|e| format!("Failed to open database: {}", e))?;

        let broadcaster_id = get_broadcaster_id_from_channel(&conn, channel)?;
        let (client_id, access_token) = get_twitch_credentials(&conn)?;
        (broadcaster_id, client_id, access_token)
    };

    // Call Twitch API (now no database connection held)
    let api_client = TwitchApiClient::new(client_id, access_token);
    match api_client.get_channel_followers(&broadcaster_id).await {
        Ok(count) => Ok(count),
        Err(e) => Err(format!("Failed to fetch followers from Twitch: {}", e)),
    }
}

async fn get_twitch_subscribers(db_path: &std::path::PathBuf, channel: &str) -> Result<i64, String> {
    // Get all needed data from database first
    let (broadcaster_id, client_id, access_token) = {
        let conn = rusqlite::Connection::open(db_path)
            .map_err(|e| format!("Failed to open database: {}", e))?;

        let broadcaster_id = get_broadcaster_id_from_channel(&conn, channel)?;
        let (client_id, access_token) = get_twitch_credentials(&conn)?;
        (broadcaster_id, client_id, access_token)
    };

    // Call Twitch API (now no database connection held)
    let api_client = TwitchApiClient::new(client_id, access_token);
    match api_client.get_broadcaster_subscriptions(&broadcaster_id).await {
        Ok(count) => Ok(count),
        Err(e) => Err(format!("Failed to fetch subscribers from Twitch: {}", e)),
    }
}

fn get_broadcaster_id_from_channel(conn: &rusqlite::Connection, channel: &str) -> Result<String, String> {
    // First try to get from twitch_auth (if they're the authenticated user)
    match conn.query_row(
        "SELECT user_id FROM twitch_auth WHERE username = ?1",
        rusqlite::params![channel],
        |row| row.get::<_, String>(0)
    ).optional() {
        Ok(Some(id)) => return Ok(id),
        Ok(None) => {},
        Err(e) => return Err(format!("Database error: {}", e)),
    }

    // If not found, check twitch_channels table
    match conn.query_row(
        "SELECT channel_id FROM twitch_channels WHERE channel_name = ?1",
        rusqlite::params![channel],
        |row| row.get::<_, String>(0)
    ).optional() {
        Ok(Some(id)) => Ok(id),
        Ok(None) => Err(format!("Broadcaster ID not found for channel: {}", channel)),
        Err(e) => Err(format!("Database error: {}", e)),
    }
}

fn get_twitch_credentials(conn: &rusqlite::Connection) -> Result<(String, String), String> {
    let client_id: Option<String> = conn.query_row(
        "SELECT value FROM twitch_config WHERE key = 'client_id'",
        [],
        |row| row.get(0)
    ).optional().map_err(|e| format!("Database error: {}", e))?;

    let access_token: Option<String> = conn.query_row(
        "SELECT access_token FROM twitch_auth LIMIT 1",
        [],
        |row| row.get(0)
    ).optional().map_err(|e| format!("Database error: {}", e))?;

    match (client_id, access_token) {
        (Some(id), Some(token)) => Ok((id, token)),
        _ => Err("Twitch API credentials not configured".to_string()),
    }
}

// Helper functions
fn extract_path_param(path: &str, prefix: &str) -> String {
    path.strip_prefix(prefix)
        .map(|s| urlencoding::decode(s).unwrap_or_default().to_string())
        .unwrap_or_default()
}

fn extract_query_param(query: &str, key: &str) -> String {
    query
        .split('&')
        .filter_map(|pair| {
            let mut parts = pair.split('=');
            match (parts.next(), parts.next()) {
                (Some(k), Some(v)) if k == key => {
                    Some(urlencoding::decode(v).unwrap_or_default().to_string())
                }
                _ => None,
            }
        })
        .next()
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
