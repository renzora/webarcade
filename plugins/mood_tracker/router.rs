use crate::core::plugin_context::PluginContext;
use crate::core::plugin_router::PluginRouter;
use anyhow::Result;
use hyper::{Method, Request, Response, StatusCode, body::Incoming};
use hyper::body::Bytes;
use http_body_util::{Full, combinators::BoxBody};
use std::convert::Infallible;

pub async fn register_routes(ctx: &PluginContext) -> Result<()> {
    let mut router = PluginRouter::new();

    // OPTIONS /mood-tracker/data - CORS preflight
    router.route(Method::OPTIONS, "/data", |_path, _query, _req| {
        Box::pin(async move {
            cors_preflight_response()
        })
    });

    // GET /mood-tracker/data - Get current mood tracker data
    router.route(Method::GET, "/data", |_path, _query, _req| {
        Box::pin(async move {
            handle_get_data().await
        })
    });

    // POST /mood-tracker/data - Update mood tracker data
    router.route(Method::POST, "/data", |_path, _query, req| {
        Box::pin(async move {
            handle_update_data(req).await
        })
    });

    // OPTIONS /mood-tracker/withings/weight - CORS preflight
    router.route(Method::OPTIONS, "/withings/weight", |_path, _query, _req| {
        Box::pin(async move {
            cors_preflight_response()
        })
    });

    // GET /mood-tracker/withings/weight - Fetch weight from Withings (placeholder)
    router.route(Method::GET, "/withings/weight", |_path, _query, _req| {
        Box::pin(async move {
            handle_get_withings_weight().await
        })
    });

    ctx.register_router("mood-tracker", router).await;
    Ok(())
}

async fn handle_get_data() -> Response<BoxBody<Bytes, Infallible>> {
    let conn = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&conn) {
        Ok(conn) => {
            match conn.query_row(
                "SELECT id, mood, show_background, sleep, updated_at, water, weight
                 FROM mood_ticker_data WHERE id = 1",
                [],
                |row| {
                    Ok(serde_json::json!({
                        "id": row.get::<_, i64>(0)?,
                        "mood": row.get::<_, i64>(1)?,
                        "show_background": row.get::<_, i64>(2)? != 0,
                        "sleep": row.get::<_, Option<f64>>(3)?,
                        "updated_at": row.get::<_, i64>(4)?,
                        "water": row.get::<_, i64>(5)?,
                        "weight": row.get::<_, Option<f64>>(6)?,
                    }))
                }
            ) {
                Ok(data) => json_response(&data),
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn handle_update_data(req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    match read_json_body(req).await {
        Ok(body) => {
            let mood = body.get("mood").and_then(|v| v.as_i64()).unwrap_or(5);
            let show_background = body.get("show_background").and_then(|v| v.as_bool()).unwrap_or(true);
            let sleep = body.get("sleep").and_then(|v| v.as_f64());
            let water = body.get("water").and_then(|v| v.as_i64()).unwrap_or(0);
            let weight = body.get("weight").and_then(|v| v.as_f64());
            let updated_at = body.get("updated_at").and_then(|v| v.as_i64()).unwrap_or_else(current_timestamp);

            let conn = crate::core::database::get_database_path();
            match rusqlite::Connection::open(&conn) {
                Ok(conn) => {
                    match conn.execute(
                        "UPDATE mood_ticker_data
                         SET mood = ?1, show_background = ?2, sleep = ?3, updated_at = ?4, water = ?5, weight = ?6
                         WHERE id = 1",
                        rusqlite::params![
                            mood,
                            show_background as i64,
                            sleep,
                            updated_at,
                            water,
                            weight
                        ],
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

async fn handle_get_withings_weight() -> Response<BoxBody<Bytes, Infallible>> {
    // Placeholder - would integrate with Withings API
    error_response(
        StatusCode::NOT_IMPLEMENTED,
        "Withings integration not yet configured"
    )
}

// Helper functions
fn cors_preflight_response() -> Response<BoxBody<Bytes, Infallible>> {
    Response::builder()
        .status(StatusCode::OK)
        .header("Access-Control-Allow-Origin", "*")
        .header("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE, OPTIONS")
        .header("Access-Control-Allow-Headers", "Content-Type, Authorization")
        .header("Access-Control-Max-Age", "86400")
        .body(full_body(""))
        .unwrap()
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
