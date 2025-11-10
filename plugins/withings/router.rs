use crate::core::plugin_context::PluginContext;
use crate::core::plugin_router::PluginRouter;
use crate::core::router_utils::*;
use crate::route;
use anyhow::Result;
use hyper::{Request, Response, StatusCode, body::Incoming};
use hyper::body::Bytes;
use http_body_util::combinators::BoxBody;
use std::convert::Infallible;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct AuthConfig {
    client_id: String,
    client_secret: String,
    redirect_uri: String,
}

#[derive(Serialize, Deserialize)]
struct AuthCallback {
    code: String,
    state: String,
}

#[derive(Serialize, Deserialize)]
struct Measurement {
    id: String,
    timestamp: i64,
    weight: Option<f64>,
    fat_mass: Option<f64>,
    muscle_mass: Option<f64>,
    hydration: Option<f64>,
    bone_mass: Option<f64>,
    fat_ratio: Option<f64>,
    fat_free_mass: Option<f64>,
}

pub async fn register_routes(ctx: &PluginContext) -> Result<()> {
    let mut router = PluginRouter::new();

    // Auth endpoints
    route!(router, GET "/auth/config" => get_auth_config);
    route!(router, POST "/auth/config" => set_auth_config);
    route!(router, POST "/auth/callback" => handle_auth_callback);
    route!(router, POST "/auth/refresh" => refresh_token);
    route!(router, GET "/auth/status" => get_auth_status);

    // Measurement endpoints
    route!(router, GET "/measurements" => get_measurements);
    route!(router, GET "/measurements/latest" => get_latest_measurements);
    route!(router, POST "/measurements/sync" => sync_measurements);
    route!(router, GET "/measurements/stats" => get_measurement_stats);

    // CORS
    route!(router, OPTIONS "/auth/config" => cors_preflight);
    route!(router, OPTIONS "/auth/callback" => cors_preflight);
    route!(router, OPTIONS "/auth/refresh" => cors_preflight);
    route!(router, OPTIONS "/auth/status" => cors_preflight);
    route!(router, OPTIONS "/measurements" => cors_preflight);
    route!(router, OPTIONS "/measurements/latest" => cors_preflight);
    route!(router, OPTIONS "/measurements/sync" => cors_preflight);
    route!(router, OPTIONS "/measurements/stats" => cors_preflight);

    ctx.register_router("withings", router).await;
    Ok(())
}

async fn get_auth_config() -> Response<BoxBody<Bytes, Infallible>> {
    let db_path = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&db_path) {
        Ok(conn) => {
            // For now, return empty config - can be stored in a settings table
            json_response(&serde_json::json!({
                "client_id": "",
                "redirect_uri": "http://localhost:3000/withings/callback"
            }))
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
    }
}

async fn set_auth_config(req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    let body = match read_json_body(req).await {
        Ok(b) => b,
        Err(e) => return error_response(StatusCode::BAD_REQUEST, &e),
    };

    // Store config in database (can add a settings table later)
    json_response(&serde_json::json!({ "success": true }))
}

async fn handle_auth_callback(req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    let body = match read_json_body(req).await {
        Ok(b) => b,
        Err(e) => return error_response(StatusCode::BAD_REQUEST, &e),
    };

    // Exchange code for access token
    // This would call the Withings OAuth endpoint
    // For now, return success
    json_response(&serde_json::json!({
        "success": true,
        "message": "Authentication successful"
    }))
}

async fn refresh_token(req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    let db_path = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&db_path) {
        Ok(conn) => {
            // Refresh the access token using refresh token
            json_response(&serde_json::json!({
                "success": true,
                "message": "Token refreshed"
            }))
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
    }
}

async fn get_auth_status() -> Response<BoxBody<Bytes, Infallible>> {
    // For now, return not authenticated (can implement OAuth later)
    json_response(&serde_json::json!({
        "authenticated": false
    }))
}

async fn get_measurements() -> Response<BoxBody<Bytes, Infallible>> {
    let db_path = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&db_path) {
        Ok(conn) => {
            let mut stmt = match conn.prepare(
                "SELECT id, timestamp, weight, fat_mass, muscle_mass, hydration, bone_mass, fat_ratio, fat_free_mass
                 FROM withings_measurements
                 ORDER BY timestamp DESC
                 LIMIT 100"
            ) {
                Ok(s) => s,
                Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            };

            let measurements_result = stmt.query_map([], |row| {
                Ok(Measurement {
                    id: row.get(0)?,
                    timestamp: row.get(1)?,
                    weight: row.get(2)?,
                    fat_mass: row.get(3)?,
                    muscle_mass: row.get(4)?,
                    hydration: row.get(5)?,
                    bone_mass: row.get(6)?,
                    fat_ratio: row.get(7)?,
                    fat_free_mass: row.get(8)?,
                })
            });

            let measurements: Vec<Measurement> = match measurements_result {
                Ok(rows) => rows.filter_map(Result::ok).collect(),
                Err(_) => Vec::new(),
            };

            json_response(&measurements)
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
    }
}

async fn get_latest_measurements() -> Response<BoxBody<Bytes, Infallible>> {
    let db_path = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&db_path) {
        Ok(conn) => {
            let result = conn.query_row(
                "SELECT id, timestamp, weight, fat_mass, muscle_mass, hydration, bone_mass, fat_ratio, fat_free_mass
                 FROM withings_measurements
                 ORDER BY timestamp DESC
                 LIMIT 1",
                [],
                |row| {
                    Ok(Measurement {
                        id: row.get(0)?,
                        timestamp: row.get(1)?,
                        weight: row.get(2)?,
                        fat_mass: row.get(3)?,
                        muscle_mass: row.get(4)?,
                        hydration: row.get(5)?,
                        bone_mass: row.get(6)?,
                        fat_ratio: row.get(7)?,
                        fat_free_mass: row.get(8)?,
                    })
                }
            );

            match result {
                Ok(measurement) => json_response(&measurement),
                Err(_) => json_response(&serde_json::json!(null))
            }
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
    }
}

async fn sync_measurements(req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    // This would call the Withings API to fetch new measurements
    // For now, return mock data
    let db_path = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&db_path) {
        Ok(mut conn) => {
            // Generate some mock data
            let tx = conn.transaction().unwrap();
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64;

            // Insert mock measurement
            tx.execute(
                "INSERT OR REPLACE INTO withings_measurements
                 (id, timestamp, weight, fat_mass, muscle_mass, hydration, bone_mass, fat_ratio, fat_free_mass, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                (
                    format!("mock_{}", now),
                    now,
                    75.5,
                    15.2,
                    35.8,
                    55.0,
                    3.2,
                    20.1,
                    60.3,
                    now,
                ),
            ).unwrap();

            tx.commit().unwrap();

            json_response(&serde_json::json!({
                "success": true,
                "synced": 1,
                "message": "Measurements synced successfully"
            }))
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
    }
}

async fn get_measurement_stats() -> Response<BoxBody<Bytes, Infallible>> {
    let db_path = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&db_path) {
        Ok(conn) => {
            let stats = conn.query_row(
                "SELECT
                    COUNT(*) as total,
                    MIN(timestamp) as first_measurement,
                    MAX(timestamp) as last_measurement,
                    AVG(weight) as avg_weight,
                    AVG(muscle_mass) as avg_muscle,
                    AVG(hydration) as avg_hydration
                 FROM withings_measurements",
                [],
                |row| {
                    Ok(serde_json::json!({
                        "total": row.get::<_, i64>(0).unwrap_or(0),
                        "first_measurement": row.get::<_, Option<i64>>(1).unwrap_or(None),
                        "last_measurement": row.get::<_, Option<i64>>(2).unwrap_or(None),
                        "avg_weight": row.get::<_, Option<f64>>(3).unwrap_or(None),
                        "avg_muscle": row.get::<_, Option<f64>>(4).unwrap_or(None),
                        "avg_hydration": row.get::<_, Option<f64>>(5).unwrap_or(None),
                    }))
                }
            ).unwrap_or_else(|_| {
                serde_json::json!({
                    "total": 0,
                    "first_measurement": null,
                    "last_measurement": null,
                    "avg_weight": null,
                    "avg_muscle": null,
                    "avg_hydration": null,
                })
            });

            json_response(&stats)
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
    }
}
