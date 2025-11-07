use crate::core::plugin_context::PluginContext;
use crate::core::plugin_router::PluginRouter;
use anyhow::Result;
use hyper::{Method, Request, Response, StatusCode, body::Incoming};
use hyper::body::Bytes;
use http_body_util::{Full, combinators::BoxBody};
use rusqlite::OptionalExtension;
use std::convert::Infallible;

pub async fn register_routes(ctx: &PluginContext) -> Result<()> {
    let mut router = PluginRouter::new();

    // POST /withings/weight - Record weight measurement
    router.route(Method::POST, "/weight", |_path, _query, req| {
        Box::pin(async move {
            handle_record_weight(req).await
        })
    });

    // GET /withings/weight/:user_id - Get weight history
    router.route(Method::GET, "/weight/:user_id", |path, query, _req| {
        Box::pin(async move {
            handle_get_weight_history(path, query).await
        })
    });

    // GET /withings/weight/latest/:user_id - Get latest weight
    router.route(Method::GET, "/weight/latest/:user_id", |path, _query, _req| {
        Box::pin(async move {
            handle_get_latest_weight(path).await
        })
    });

    // POST /withings/activity - Record activity
    router.route(Method::POST, "/activity", |_path, _query, req| {
        Box::pin(async move {
            handle_record_activity(req).await
        })
    });

    // GET /withings/activity/:user_id - Get activity history
    router.route(Method::GET, "/activity/:user_id", |path, query, _req| {
        Box::pin(async move {
            handle_get_activity_history(path, query).await
        })
    });

    // POST /withings/sleep - Record sleep data
    router.route(Method::POST, "/sleep", |_path, _query, req| {
        Box::pin(async move {
            handle_record_sleep(req).await
        })
    });

    // GET /withings/sleep/:user_id - Get sleep history
    router.route(Method::GET, "/sleep/:user_id", |path, query, _req| {
        Box::pin(async move {
            handle_get_sleep_history(path, query).await
        })
    });

    // POST /withings/goals - Set goal
    router.route(Method::POST, "/goals", |_path, _query, req| {
        Box::pin(async move {
            handle_set_goal(req).await
        })
    });

    // GET /withings/goals/:user_id - Get goals
    router.route(Method::GET, "/goals/:user_id", |path, _query, _req| {
        Box::pin(async move {
            handle_get_goals(path).await
        })
    });

    // GET /withings/stats/:user_id - Get stats
    router.route(Method::GET, "/stats/:user_id", |path, _query, _req| {
        Box::pin(async move {
            handle_get_stats(path).await
        })
    });

    // GET /withings/auth-url - Get OAuth authorization URL
    router.route(Method::GET, "/auth-url", |_path, _query, _req| {
        Box::pin(async move {
            handle_get_auth_url().await
        })
    });

    // POST /withings/sync - Sync data from Withings API
    router.route(Method::POST, "/sync", |_path, _query, _req| {
        Box::pin(async move {
            handle_sync_data().await
        })
    });

    // GET /withings/callback - OAuth callback handler
    router.route(Method::GET, "/callback", |_path, query, _req| {
        Box::pin(async move {
            handle_oauth_callback(query).await
        })
    });

    // GET /withings/latest - Get latest measurement (simplified endpoint)
    router.route(Method::GET, "/latest", |_path, _query, _req| {
        Box::pin(async move {
            handle_get_latest().await
        })
    });

    // GET /withings/history - Get measurement history (simplified endpoint)
    router.route(Method::GET, "/history", |_path, query, _req| {
        Box::pin(async move {
            handle_get_history(query).await
        })
    });

    // GET /withings/config - Get Withings configuration
    router.route(Method::GET, "/config", |_path, _query, _req| {
        Box::pin(async move {
            handle_get_config().await
        })
    });

    // POST /withings/config - Save Withings configuration
    router.route(Method::POST, "/config", |_path, _query, req| {
        Box::pin(async move {
            handle_save_config(req).await
        })
    });

    ctx.register_router("withings", router).await;
    Ok(())
}

async fn handle_record_weight(req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    match read_json_body(req).await {
        Ok(body) => {
            let user_id = match body.get("user_id").and_then(|v| v.as_str()) {
                Some(id) => id,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing user_id"),
            };
            let weight_kg = match body.get("weight_kg").and_then(|v| v.as_f64()) {
                Some(w) => w,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing weight_kg"),
            };
            let fat_mass_kg = body.get("fat_mass_kg").and_then(|v| v.as_f64());
            let muscle_mass_kg = body.get("muscle_mass_kg").and_then(|v| v.as_f64());
            let bone_mass_kg = body.get("bone_mass_kg").and_then(|v| v.as_f64());
            let water_percentage = body.get("water_percentage").and_then(|v| v.as_f64());
            let measured_at = match body.get("measured_at").and_then(|v| v.as_i64()) {
                Some(t) => t,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing measured_at"),
            };

            let conn = crate::core::database::get_database_path();
            match rusqlite::Connection::open(&conn) {
                Ok(conn) => {
                    let now = current_timestamp();
                    match conn.execute(
                        "INSERT INTO withings_weight_measurements (user_id, weight_kg, fat_mass_kg, muscle_mass_kg, bone_mass_kg, water_percentage, measured_at, synced_at)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                        rusqlite::params![user_id, weight_kg, fat_mass_kg, muscle_mass_kg, bone_mass_kg, water_percentage, measured_at, now],
                    ) {
                        Ok(_) => json_response(&serde_json::json!({ "id": conn.last_insert_rowid(), "success": true })),
                        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
                    }
                }
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::BAD_REQUEST, &e),
    }
}

async fn handle_get_weight_history(path: String, query: String) -> Response<BoxBody<Bytes, Infallible>> {
    let user_id = extract_path_param(&path, "/weight/");
    if user_id.is_empty() {
        return error_response(StatusCode::BAD_REQUEST, "Missing user_id");
    }

    let limit = parse_query_param(&query, "limit")
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(30);

    let conn = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&conn) {
        Ok(conn) => {
            let mut stmt = match conn.prepare(
                "SELECT id, weight_kg, fat_mass_kg, muscle_mass_kg, bone_mass_kg, water_percentage, measured_at
                 FROM withings_weight_measurements
                 WHERE user_id = ?1
                 ORDER BY measured_at DESC
                 LIMIT ?2"
            ) {
                Ok(stmt) => stmt,
                Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            };

            let mapped = match stmt.query_map(
                rusqlite::params![user_id, limit],
                |row| {
                    Ok(serde_json::json!({
                        "id": row.get::<_, i64>(0)?,
                        "weight_kg": row.get::<_, f64>(1)?,
                        "fat_mass_kg": row.get::<_, Option<f64>>(2)?,
                        "muscle_mass_kg": row.get::<_, Option<f64>>(3)?,
                        "bone_mass_kg": row.get::<_, Option<f64>>(4)?,
                        "water_percentage": row.get::<_, Option<f64>>(5)?,
                        "measured_at": row.get::<_, i64>(6)?,
                    }))
                }
            ) {
                Ok(m) => m,
                Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            };

            let measurements: Result<Vec<_>, _> = mapped.collect();

            match measurements {
                Ok(measurements) => json_response(&serde_json::json!({ "measurements": measurements })),
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn handle_get_latest_weight(path: String) -> Response<BoxBody<Bytes, Infallible>> {
    let user_id = extract_path_param(&path, "/weight/latest/");
    if user_id.is_empty() {
        return error_response(StatusCode::BAD_REQUEST, "Missing user_id");
    }

    let conn = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&conn) {
        Ok(conn) => {
            let weight = conn.query_row(
                "SELECT weight_kg, measured_at FROM withings_weight_measurements
                 WHERE user_id = ?1
                 ORDER BY measured_at DESC
                 LIMIT 1",
                rusqlite::params![user_id],
                |row| {
                    Ok(serde_json::json!({
                        "weight_kg": row.get::<_, f64>(0)?,
                        "measured_at": row.get::<_, i64>(1)?,
                    }))
                }
            ).optional();

            match weight {
                Ok(weight) => json_response(&serde_json::json!({ "latest_weight": weight })),
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn handle_record_activity(req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    match read_json_body(req).await {
        Ok(body) => {
            let user_id = match body.get("user_id").and_then(|v| v.as_str()) {
                Some(id) => id,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing user_id"),
            };
            let date = match body.get("date").and_then(|v| v.as_str()) {
                Some(d) => d,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing date"),
            };
            let steps = match body.get("steps").and_then(|v| v.as_i64()) {
                Some(s) => s,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing steps"),
            };
            let distance_meters = body.get("distance_meters").and_then(|v| v.as_i64());
            let calories = body.get("calories").and_then(|v| v.as_i64());
            let active_minutes = body.get("active_minutes").and_then(|v| v.as_i64());

            let conn = crate::core::database::get_database_path();
            match rusqlite::Connection::open(&conn) {
                Ok(conn) => {
                    let now = current_timestamp();
                    match conn.execute(
                        "INSERT OR REPLACE INTO withings_activity (user_id, date, steps, distance_meters, calories, active_minutes, synced_at)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                        rusqlite::params![user_id, date, steps, distance_meters, calories, active_minutes, now],
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

async fn handle_get_activity_history(path: String, query: String) -> Response<BoxBody<Bytes, Infallible>> {
    let user_id = extract_path_param(&path, "/activity/");
    if user_id.is_empty() {
        return error_response(StatusCode::BAD_REQUEST, "Missing user_id");
    }

    let limit = parse_query_param(&query, "limit")
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(30);

    let conn = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&conn) {
        Ok(conn) => {
            let mut stmt = match conn.prepare(
                "SELECT id, date, steps, distance_meters, calories, active_minutes
                 FROM withings_activity
                 WHERE user_id = ?1
                 ORDER BY date DESC
                 LIMIT ?2"
            ) {
                Ok(stmt) => stmt,
                Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            };

            let mapped = match stmt.query_map(
                rusqlite::params![user_id, limit],
                |row| {
                    Ok(serde_json::json!({
                        "id": row.get::<_, i64>(0)?,
                        "date": row.get::<_, String>(1)?,
                        "steps": row.get::<_, i64>(2)?,
                        "distance_meters": row.get::<_, Option<i64>>(3)?,
                        "calories": row.get::<_, Option<i64>>(4)?,
                        "active_minutes": row.get::<_, Option<i64>>(5)?,
                    }))
                }
            ) {
                Ok(m) => m,
                Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            };

            let activities: Result<Vec<_>, _> = mapped.collect();

            match activities {
                Ok(activities) => json_response(&serde_json::json!({ "activities": activities })),
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn handle_record_sleep(req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    match read_json_body(req).await {
        Ok(body) => {
            let user_id = match body.get("user_id").and_then(|v| v.as_str()) {
                Some(id) => id,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing user_id"),
            };
            let date = match body.get("date").and_then(|v| v.as_str()) {
                Some(d) => d,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing date"),
            };
            let total_sleep_minutes = match body.get("total_sleep_minutes").and_then(|v| v.as_i64()) {
                Some(t) => t,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing total_sleep_minutes"),
            };
            let deep_sleep_minutes = body.get("deep_sleep_minutes").and_then(|v| v.as_i64());
            let light_sleep_minutes = body.get("light_sleep_minutes").and_then(|v| v.as_i64());
            let rem_sleep_minutes = body.get("rem_sleep_minutes").and_then(|v| v.as_i64());
            let awake_minutes = body.get("awake_minutes").and_then(|v| v.as_i64());
            let sleep_score = body.get("sleep_score").and_then(|v| v.as_i64());

            let conn = crate::core::database::get_database_path();
            match rusqlite::Connection::open(&conn) {
                Ok(conn) => {
                    let now = current_timestamp();
                    match conn.execute(
                        "INSERT OR REPLACE INTO withings_sleep (user_id, date, total_sleep_minutes, deep_sleep_minutes, light_sleep_minutes, rem_sleep_minutes, awake_minutes, sleep_score, synced_at)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                        rusqlite::params![user_id, date, total_sleep_minutes, deep_sleep_minutes, light_sleep_minutes, rem_sleep_minutes, awake_minutes, sleep_score, now],
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

async fn handle_get_sleep_history(path: String, query: String) -> Response<BoxBody<Bytes, Infallible>> {
    let user_id = extract_path_param(&path, "/sleep/");
    if user_id.is_empty() {
        return error_response(StatusCode::BAD_REQUEST, "Missing user_id");
    }

    let limit = parse_query_param(&query, "limit")
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(30);

    let conn = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&conn) {
        Ok(conn) => {
            let mut stmt = match conn.prepare(
                "SELECT id, date, total_sleep_minutes, deep_sleep_minutes, light_sleep_minutes, rem_sleep_minutes, awake_minutes, sleep_score
                 FROM withings_sleep
                 WHERE user_id = ?1
                 ORDER BY date DESC
                 LIMIT ?2"
            ) {
                Ok(stmt) => stmt,
                Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            };

            let mapped = match stmt.query_map(
                rusqlite::params![user_id, limit],
                |row| {
                    Ok(serde_json::json!({
                        "id": row.get::<_, i64>(0)?,
                        "date": row.get::<_, String>(1)?,
                        "total_sleep_minutes": row.get::<_, i64>(2)?,
                        "deep_sleep_minutes": row.get::<_, Option<i64>>(3)?,
                        "light_sleep_minutes": row.get::<_, Option<i64>>(4)?,
                        "rem_sleep_minutes": row.get::<_, Option<i64>>(5)?,
                        "awake_minutes": row.get::<_, Option<i64>>(6)?,
                        "sleep_score": row.get::<_, Option<i64>>(7)?,
                    }))
                }
            ) {
                Ok(m) => m,
                Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            };

            let sleep_data: Result<Vec<_>, _> = mapped.collect();

            match sleep_data {
                Ok(sleep_data) => json_response(&serde_json::json!({ "sleep_data": sleep_data })),
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn handle_set_goal(req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    match read_json_body(req).await {
        Ok(body) => {
            let user_id = match body.get("user_id").and_then(|v| v.as_str()) {
                Some(id) => id,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing user_id"),
            };
            let goal_type = match body.get("goal_type").and_then(|v| v.as_str()) {
                Some(t) => t,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing goal_type"),
            };
            let goal_value = match body.get("goal_value").and_then(|v| v.as_f64()) {
                Some(v) => v,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing goal_value"),
            };

            let conn = crate::core::database::get_database_path();
            match rusqlite::Connection::open(&conn) {
                Ok(conn) => {
                    let now = current_timestamp();
                    match conn.execute(
                        "INSERT OR REPLACE INTO withings_goals (user_id, goal_type, goal_value, created_at, updated_at)
                         VALUES (?1, ?2, ?3, ?4, ?4)",
                        rusqlite::params![user_id, goal_type, goal_value, now],
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

async fn handle_get_goals(path: String) -> Response<BoxBody<Bytes, Infallible>> {
    let user_id = extract_path_param(&path, "/goals/");
    if user_id.is_empty() {
        return error_response(StatusCode::BAD_REQUEST, "Missing user_id");
    }

    let conn = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&conn) {
        Ok(conn) => {
            let mut stmt = match conn.prepare(
                "SELECT id, goal_type, goal_value, created_at FROM withings_goals WHERE user_id = ?1"
            ) {
                Ok(stmt) => stmt,
                Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            };

            let mapped = match stmt.query_map(rusqlite::params![user_id], |row| {
                Ok(serde_json::json!({
                    "id": row.get::<_, i64>(0)?,
                    "goal_type": row.get::<_, String>(1)?,
                    "goal_value": row.get::<_, f64>(2)?,
                    "created_at": row.get::<_, i64>(3)?,
                }))
            }) {
                Ok(m) => m,
                Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            };

            let goals: Result<Vec<_>, _> = mapped.collect();

            match goals {
                Ok(goals) => json_response(&serde_json::json!({ "goals": goals })),
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn handle_get_stats(path: String) -> Response<BoxBody<Bytes, Infallible>> {
    let user_id = extract_path_param(&path, "/stats/");
    if user_id.is_empty() {
        return error_response(StatusCode::BAD_REQUEST, "Missing user_id");
    }

    let conn = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&conn) {
        Ok(conn) => {
            let weight_count: Result<i64, _> = conn.query_row(
                "SELECT COUNT(*) FROM withings_weight_measurements WHERE user_id = ?1",
                rusqlite::params![user_id],
                |row| row.get(0),
            );

            let activity_count: Result<i64, _> = conn.query_row(
                "SELECT COUNT(*) FROM withings_activity WHERE user_id = ?1",
                rusqlite::params![user_id],
                |row| row.get(0),
            );

            let sleep_count: Result<i64, _> = conn.query_row(
                "SELECT COUNT(*) FROM withings_sleep WHERE user_id = ?1",
                rusqlite::params![user_id],
                |row| row.get(0),
            );

            match (weight_count, activity_count, sleep_count) {
                (Ok(weight), Ok(activity), Ok(sleep)) => {
                    json_response(&serde_json::json!({
                        "weight_measurements": weight,
                        "activity_days": activity,
                        "sleep_days": sleep
                    }))
                }
                _ => error_response(StatusCode::INTERNAL_SERVER_ERROR, "Failed to retrieve stats"),
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

async fn handle_get_latest() -> Response<BoxBody<Bytes, Infallible>> {
    let conn = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&conn) {
        Ok(conn) => {
            // Get the latest weight measurement
            let result: std::result::Result<(f64, i64, Option<f64>, Option<f64>), _> = conn.query_row(
                "SELECT weight_kg, measured_at, fat_mass_kg, muscle_mass_kg FROM withings_weight_measurements
                 ORDER BY measured_at DESC LIMIT 1",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
            );

            match result {
                Ok((weight, timestamp, fat_mass, muscle_mass)) => {
                    json_response(&serde_json::json!({
                        "success": true,
                        "data": {
                            "weight": weight,
                            "date": timestamp,
                            "fat_mass": fat_mass,
                            "muscle_mass": muscle_mass
                        }
                    }))
                }
                Err(_) => {
                    // No data found, return empty
                    json_response(&serde_json::json!({
                        "success": false,
                        "data": null
                    }))
                }
            }
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn handle_get_history(query: String) -> Response<BoxBody<Bytes, Infallible>> {
    let start_date = parse_query_param(&query, "start_date")
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(0);
    let limit = parse_query_param(&query, "limit")
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(100);

    let conn = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&conn) {
        Ok(conn) => {
            let mut stmt = match conn.prepare(
                "SELECT weight_kg, measured_at, fat_mass_kg, muscle_mass_kg FROM withings_weight_measurements
                 WHERE measured_at >= ?1
                 ORDER BY measured_at DESC LIMIT ?2"
            ) {
                Ok(stmt) => stmt,
                Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            };

            let measurements: std::result::Result<Vec<_>, _> = stmt.query_map(
                rusqlite::params![start_date, limit],
                |row| {
                    Ok(serde_json::json!({
                        "weight": row.get::<_, f64>(0)?,
                        "date": row.get::<_, i64>(1)?,
                        "fat_mass": row.get::<_, Option<f64>>(2)?,
                        "muscle_mass": row.get::<_, Option<f64>>(3)?,
                    }))
                }
            ).and_then(|rows| rows.collect());

            match measurements {
                Ok(data) => json_response(&serde_json::json!({
                    "success": true,
                    "data": data
                })),
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn handle_get_config() -> Response<BoxBody<Bytes, Infallible>> {
    let conn = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&conn) {
        Ok(conn) => {
            // Try to get config from a withings_config table
            let result: std::result::Result<(Option<String>, Option<String>), _> = conn.query_row(
                "SELECT client_id, client_secret FROM withings_config LIMIT 1",
                [],
                |row| Ok((row.get(0)?, row.get(1)?))
            );

            match result {
                Ok((client_id, client_secret)) => {
                    let configured = client_id.is_some() && !client_id.as_ref().unwrap().is_empty();
                    json_response(&serde_json::json!({
                        "success": true,
                        "data": {
                            "client_id": client_id,
                            "client_secret": client_secret,
                            "configured": configured
                        }
                    }))
                }
                Err(_) => {
                    // No config found, return empty
                    json_response(&serde_json::json!({
                        "success": true,
                        "data": {
                            "configured": false
                        }
                    }))
                }
            }
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn handle_save_config(req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    match read_json_body(req).await {
        Ok(body) => {
            let client_id = body.get("client_id").and_then(|v| v.as_str()).unwrap_or("");
            let client_secret = body.get("client_secret").and_then(|v| v.as_str()).unwrap_or("");

            let conn = crate::core::database::get_database_path();
            match rusqlite::Connection::open(&conn) {
                Ok(conn) => {
                    let now = current_timestamp();

                    // Insert or update config
                    match conn.execute(
                        "INSERT INTO withings_config (id, client_id, client_secret, updated_at)
                         VALUES (1, ?1, ?2, ?3)
                         ON CONFLICT(id) DO UPDATE SET
                           client_id = ?1,
                           client_secret = ?2,
                           updated_at = ?3",
                        rusqlite::params![client_id, client_secret, now],
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

async fn handle_get_auth_url() -> Response<BoxBody<Bytes, Infallible>> {
    let conn = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&conn) {
        Ok(conn) => {
            // Get client_id from config
            let result = conn.query_row(
                "SELECT client_id FROM withings_config LIMIT 1",
                [],
                |row| row.get::<_, Option<String>>(0)
            ).optional();

            match result {
                Ok(Some(Some(client_id))) => {
                    // Construct OAuth URL
                    let redirect_uri = "http://localhost:3001/withings/callback";
                    let scope = "user.metrics";
                    let state = generate_state();

                    let auth_url = format!(
                        "https://account.withings.com/oauth2_user/authorize2?response_type=code&client_id={}&redirect_uri={}&scope={}&state={}",
                        urlencoding::encode(&client_id),
                        urlencoding::encode(redirect_uri),
                        urlencoding::encode(scope),
                        urlencoding::encode(&state)
                    );

                    json_response(&serde_json::json!({
                        "success": true,
                        "auth_url": auth_url,
                        "state": state
                    }))
                }
                Ok(Some(None)) | Ok(None) => {
                    error_response(StatusCode::BAD_REQUEST, "Withings client_id not configured")
                }
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn handle_sync_data() -> Response<BoxBody<Bytes, Infallible>> {
    // Placeholder for syncing data from Withings API
    // In a full implementation, this would:
    // 1. Check for valid access token
    // 2. Call Withings API to fetch measurements
    // 3. Store measurements in database
    // 4. Return sync status

    json_response(&serde_json::json!({
        "success": true,
        "message": "Sync functionality requires OAuth token implementation",
        "synced": 0
    }))
}

async fn handle_oauth_callback(query: String) -> Response<BoxBody<Bytes, Infallible>> {
    // Extract code and state from query parameters
    let code = extract_query_param(&query, "code");
    let state = extract_query_param(&query, "state");

    if code.is_empty() {
        // Check for error parameter
        let error = extract_query_param(&query, "error");
        if !error.is_empty() {
            return redirect_response(&format!("http://localhost:3000/withings?error={}", urlencoding::encode(&error)));
        }
        return error_response(StatusCode::BAD_REQUEST, "Missing authorization code");
    }

    // In a full implementation, this would:
    // 1. Validate the state parameter
    // 2. Exchange the authorization code for access/refresh tokens
    // 3. Store tokens in database
    // 4. Fetch initial data from Withings API

    // For now, just redirect back to frontend with success
    redirect_response(&format!("http://localhost:3000/withings?status=connected&code={}", urlencoding::encode(&code)))
}

fn generate_state() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    format!("withings_{}", timestamp)
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

fn redirect_response(location: &str) -> Response<BoxBody<Bytes, Infallible>> {
    Response::builder()
        .status(StatusCode::FOUND)
        .header("Location", location)
        .body(full_body(""))
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
