use crate::core::plugin_context::PluginContext;
use crate::core::plugin_router::PluginRouter;
use anyhow::Result;
use hyper::{Method, Request, Response, StatusCode, body::Incoming};
use hyper::body::Bytes;
use http_body_util::{Full, combinators::BoxBody};
use std::convert::Infallible;

pub async fn register_routes(ctx: &PluginContext) -> Result<()> {
    let mut router = PluginRouter::new();

    // GET /wheel/options - Get all wheel options for a channel
    router.route(Method::GET, "/options", |_path, query, _req| {
        Box::pin(async move {
            handle_get_options(query).await
        })
    });

    // POST /wheel/options - Add a new wheel option
    router.route(Method::POST, "/options", |_path, _query, req| {
        Box::pin(async move {
            handle_add_option(req).await
        })
    });

    // PUT /wheel/options/:id - Update an existing wheel option
    router.route(Method::PUT, "/options/:id", |path, _query, req| {
        Box::pin(async move {
            handle_update_option(path, req).await
        })
    });

    // DELETE /wheel/options/:id - Delete a wheel option
    router.route(Method::DELETE, "/options/:id", |path, _query, _req| {
        Box::pin(async move {
            handle_delete_option(path).await
        })
    });

    // POST /wheel/options/:id/toggle - Toggle option enabled/disabled
    router.route(Method::POST, "/options/:id/toggle", |path, _query, _req| {
        Box::pin(async move {
            handle_toggle_option(path).await
        })
    });

    // POST /wheel/spin - Spin the wheel
    router.route(Method::POST, "/spin", |_path, _query, req| {
        Box::pin(async move {
            handle_spin_wheel(req).await
        })
    });

    ctx.register_router("wheel", router).await;
    Ok(())
}

async fn handle_get_options(query: String) -> Response<BoxBody<Bytes, Infallible>> {
    let channel = parse_query_param(&query, "channel")
        .unwrap_or_else(|| "global".to_string());

    let conn = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&conn) {
        Ok(conn) => {
            let mut stmt = match conn.prepare(
                "SELECT id, channel, option_text, color, weight, chance_percentage, enabled, prize_type, prize_data
                 FROM wheel_options WHERE channel = ?1 ORDER BY id ASC"
            ) {
                Ok(stmt) => stmt,
                Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            };

            let options: Result<Vec<serde_json::Value>, _> = stmt.query_map([&channel], |row| {
                Ok(serde_json::json!({
                    "id": row.get::<_, i64>(0)?,
                    "channel": row.get::<_, String>(1)?,
                    "option_text": row.get::<_, String>(2)?,
                    "color": row.get::<_, String>(3)?,
                    "weight": row.get::<_, i64>(4)?,
                    "chance_percentage": row.get::<_, Option<f64>>(5)?,
                    "enabled": row.get::<_, i64>(6)?,
                    "prize_type": row.get::<_, Option<String>>(7)?,
                    "prize_data": row.get::<_, Option<String>>(8)?
                }))
            }).and_then(|rows| rows.collect());

            match options {
                Ok(opts) => json_response(&opts),
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn handle_add_option(req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    match read_json_body(req).await {
        Ok(body) => {
            let channel = body.get("channel")
                .and_then(|v| v.as_str())
                .unwrap_or("global")
                .to_string();
            let option_text = match body.get("option_text").and_then(|v| v.as_str()) {
                Some(text) => text,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing option_text"),
            };
            let color = match body.get("color").and_then(|v| v.as_str()) {
                Some(c) => c,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing color"),
            };
            let weight = body.get("weight").and_then(|v| v.as_i64()).unwrap_or(1);
            let chance_percentage = body.get("chance_percentage").and_then(|v| v.as_f64());
            let prize_type = body.get("prize_type").and_then(|v| v.as_str());
            let prize_data = body.get("prize_data").and_then(|v| v.as_str());

            let conn = crate::core::database::get_database_path();
            match rusqlite::Connection::open(&conn) {
                Ok(conn) => {
                    match conn.execute(
                        "INSERT INTO wheel_options (channel, option_text, color, weight, chance_percentage, enabled, prize_type, prize_data)
                         VALUES (?1, ?2, ?3, ?4, ?5, 1, ?6, ?7)",
                        rusqlite::params![channel, option_text, color, weight, chance_percentage, prize_type, prize_data],
                    ) {
                        Ok(_) => json_response(&serde_json::json!({
                            "id": conn.last_insert_rowid(),
                            "success": true
                        })),
                        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
                    }
                }
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::BAD_REQUEST, &e),
    }
}

async fn handle_update_option(path: String, req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    let id_str = extract_path_param(&path, "/options/");
    let id = match id_str.parse::<i64>() {
        Ok(id) => id,
        Err(_) => return error_response(StatusCode::BAD_REQUEST, "Invalid option ID"),
    };

    match read_json_body(req).await {
        Ok(body) => {
            let option_text = match body.get("option_text").and_then(|v| v.as_str()) {
                Some(text) => text,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing option_text"),
            };
            let color = match body.get("color").and_then(|v| v.as_str()) {
                Some(c) => c,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing color"),
            };
            let weight = body.get("weight").and_then(|v| v.as_i64()).unwrap_or(1);
            let chance_percentage = body.get("chance_percentage").and_then(|v| v.as_f64());
            let prize_type = body.get("prize_type").and_then(|v| v.as_str());
            let prize_data = body.get("prize_data").and_then(|v| v.as_str());

            let conn = crate::core::database::get_database_path();
            match rusqlite::Connection::open(&conn) {
                Ok(conn) => {
                    match conn.execute(
                        "UPDATE wheel_options
                         SET option_text = ?1, color = ?2, weight = ?3, chance_percentage = ?4, prize_type = ?5, prize_data = ?6
                         WHERE id = ?7",
                        rusqlite::params![option_text, color, weight, chance_percentage, prize_type, prize_data, id],
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

async fn handle_delete_option(path: String) -> Response<BoxBody<Bytes, Infallible>> {
    let id_str = extract_path_param(&path, "/options/");
    let id = match id_str.parse::<i64>() {
        Ok(id) => id,
        Err(_) => return error_response(StatusCode::BAD_REQUEST, "Invalid option ID"),
    };

    let conn = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&conn) {
        Ok(conn) => {
            match conn.execute("DELETE FROM wheel_options WHERE id = ?1", [id]) {
                Ok(_) => json_response(&serde_json::json!({ "success": true })),
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn handle_toggle_option(path: String) -> Response<BoxBody<Bytes, Infallible>> {
    let id_str = extract_path_param(&path, "/options/");
    let id = match id_str.parse::<i64>() {
        Ok(id) => id,
        Err(_) => return error_response(StatusCode::BAD_REQUEST, "Invalid option ID"),
    };

    let conn = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&conn) {
        Ok(conn) => {
            match conn.execute(
                "UPDATE wheel_options SET enabled = 1 - enabled WHERE id = ?1",
                [id]
            ) {
                Ok(_) => json_response(&serde_json::json!({ "success": true })),
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn handle_spin_wheel(req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    match read_json_body(req).await {
        Ok(body) => {
            let channel = body.get("channel")
                .and_then(|v| v.as_str())
                .unwrap_or("global")
                .to_string();
            let user_id = body.get("user_id").and_then(|v| v.as_str());
            let username = body.get("username").and_then(|v| v.as_str());

            let conn = crate::core::database::get_database_path();
            match rusqlite::Connection::open(&conn) {
                Ok(conn) => {
                    // Get all enabled options with their data
                    let mut stmt = match conn.prepare(
                        "SELECT id, option_text, color, weight, chance_percentage, prize_type, prize_data
                         FROM wheel_options WHERE channel = ?1 AND enabled = 1"
                    ) {
                        Ok(stmt) => stmt,
                        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
                    };

                    struct WheelOption {
                        id: i64,
                        text: String,
                        color: String,
                        weight: i64,
                        chance_percentage: Option<f64>,
                        prize_type: Option<String>,
                        prize_data: Option<String>,
                    }

                    let options: Result<Vec<WheelOption>, _> = stmt.query_map([&channel], |row| {
                        Ok(WheelOption {
                            id: row.get(0)?,
                            text: row.get(1)?,
                            color: row.get(2)?,
                            weight: row.get(3)?,
                            chance_percentage: row.get(4)?,
                            prize_type: row.get(5)?,
                            prize_data: row.get(6)?,
                        })
                    }).and_then(|rows| rows.collect());

                    let options = match options {
                        Ok(opts) => opts,
                        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
                    };

                    if options.is_empty() {
                        return error_response(StatusCode::BAD_REQUEST, "No wheel options available");
                    }

                    // Weighted random selection
                    use rand::Rng;
                    let total_weight: i64 = options.iter().map(|o| o.weight).sum();
                    let mut rng = rand::thread_rng();
                    let mut roll = rng.gen_range(0..total_weight);

                    let mut winner_option = &options[0];
                    for option in &options {
                        if roll < option.weight {
                            winner_option = option;
                            break;
                        }
                        roll -= option.weight;
                    }

                    // Prepare all options for overlay display
                    let display_options: Vec<serde_json::Value> = options.iter().map(|opt| {
                        serde_json::json!({
                            "text": opt.text,
                            "color": opt.color
                        })
                    }).collect();

                    // Record spin history if user info provided
                    if let (Some(uid), Some(uname)) = (user_id, username) {
                        let timestamp = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs() as i64;

                        let _ = conn.execute(
                            "INSERT INTO wheel_spins (channel, user_id, username, result, prize_type, prize_data, created_at)
                             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                            rusqlite::params![
                                channel,
                                uid,
                                uname,
                                winner_option.text,
                                winner_option.prize_type,
                                winner_option.prize_data,
                                timestamp
                            ],
                        );
                    }

                    json_response(&serde_json::json!({
                        "winner": winner_option.text,
                        "prize_type": winner_option.prize_type,
                        "prize_data": winner_option.prize_data,
                        "options": display_options
                    }))
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
