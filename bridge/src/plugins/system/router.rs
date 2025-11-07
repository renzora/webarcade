use crate::core::plugin_context::PluginContext;
use crate::core::plugin_router::PluginRouter;
use anyhow::Result;
use hyper::{Method, Request, Response, StatusCode, body::Incoming};
use hyper::body::Bytes;
use http_body_util::{Full, combinators::BoxBody};
use std::convert::Infallible;
use sysinfo::System;
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
struct BuildProgress {
    state: String,
    progress: u8,
    message: String,
    timestamp: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    errors: Option<Vec<String>>,
}

lazy_static::lazy_static! {
    static ref BUILD_PROGRESS: Arc<Mutex<BuildProgress>> = Arc::new(Mutex::new(BuildProgress {
        state: "idle".to_string(),
        progress: 0,
        message: "Ready".to_string(),
        timestamp: 0,
        errors: None,
    }));
}

pub async fn register_routes(ctx: &PluginContext) -> Result<()> {
    let mut router = PluginRouter::new();

    // GET /system/stats - Get all system statistics (CPU cores + memory)
    router.route(Method::GET, "/stats", |_path, _query, _req| {
        Box::pin(async move {
            handle_get_stats().await
        })
    });

    // GET /system/cpu - Get CPU information
    router.route(Method::GET, "/cpu", |_path, _query, _req| {
        Box::pin(async move {
            handle_get_cpu().await
        })
    });

    // GET /system/memory - Get memory information
    router.route(Method::GET, "/memory", |_path, _query, _req| {
        Box::pin(async move {
            handle_get_memory().await
        })
    });

    // GET /system/settings?key=xxx - Get a specific setting by key
    router.route(Method::GET, "/settings", |_path, query, _req| {
        Box::pin(async move {
            handle_get_settings(query).await
        })
    });

    // GET /system/build-progress - Get current build progress
    router.route(Method::GET, "/build-progress", |_path, _query, _req| {
        Box::pin(async move {
            handle_get_build_progress().await
        })
    });

    // POST /system/build-progress - Update build progress (from rspack plugin)
    router.route(Method::POST, "/build-progress", |_path, _query, req| {
        Box::pin(async move {
            handle_post_build_progress(req).await
        })
    });

    // POST /system/trigger-rebuild - Trigger frontend rebuild by touching entry file
    router.route(Method::POST, "/trigger-rebuild", |_path, _query, req| {
        Box::pin(async move {
            handle_trigger_rebuild(req).await
        })
    });

    ctx.register_router("system", router).await;
    Ok(())
}

async fn handle_get_stats() -> Response<BoxBody<Bytes, Infallible>> {
    let mut sys = System::new_all();
    sys.refresh_all();

    // Calculate CPU usage
    let cpu_count = sys.cpus().len();
    let cpu_usage: f64 = sys.cpus().iter().map(|cpu| cpu.cpu_usage() as f64).sum::<f64>() / cpu_count as f64;

    // Calculate memory usage
    let total_memory = sys.total_memory();
    let used_memory = sys.used_memory();
    let memory_usage = (used_memory as f64 / total_memory as f64) * 100.0;

    let response_data = serde_json::json!({
        "cpu_usage": cpu_usage,
        "memory_usage": memory_usage,
        "cpu": {
            "cores": cpu_count,
            "usage_percent": cpu_usage,
        },
        "memory": {
            "total": total_memory,
            "used": used_memory,
            "usage_percent": memory_usage,
        },
    });

    json_response(&response_data)
}

async fn handle_get_cpu() -> Response<BoxBody<Bytes, Infallible>> {
    let sys = System::new_all();

    let response_data = serde_json::json!({
        "cores": sys.cpus().len(),
    });

    json_response(&response_data)
}

async fn handle_get_memory() -> Response<BoxBody<Bytes, Infallible>> {
    let mut sys = System::new_all();
    sys.refresh_memory();

    let total_memory = sys.total_memory();
    let used_memory = sys.used_memory();
    let memory_usage = (used_memory as f64 / total_memory as f64) * 100.0;

    let response_data = serde_json::json!({
        "total": total_memory,
        "used": used_memory,
        "usage_percent": memory_usage,
    });

    json_response(&response_data)
}

async fn handle_get_settings(query: String) -> Response<BoxBody<Bytes, Infallible>> {
    let key = match parse_query_param(&query, "key") {
        Some(k) => k,
        None => return error_response(StatusCode::BAD_REQUEST, "Missing key parameter"),
    };

    let db_path = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&db_path) {
        Ok(conn) => {
            let result: std::result::Result<String, _> = conn.query_row(
                "SELECT value FROM system_settings WHERE key = ?1",
                rusqlite::params![key],
                |row| row.get(0),
            );

            match result {
                Ok(value) => json_response(&serde_json::json!({ "key": key, "value": value })),
                Err(rusqlite::Error::QueryReturnedNoRows) => {
                    json_response(&serde_json::json!({ "key": key, "value": null }))
                }
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn handle_get_build_progress() -> Response<BoxBody<Bytes, Infallible>> {
    let progress = BUILD_PROGRESS.lock().unwrap();
    json_response(&*progress)
}

async fn handle_post_build_progress(req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    match read_json_body(req).await {
        Ok(body) => {
            match serde_json::from_value::<BuildProgress>(body) {
                Ok(new_progress) => {
                    let mut progress = BUILD_PROGRESS.lock().unwrap();
                    *progress = new_progress;
                    json_response(&serde_json::json!({ "success": true }))
                }
                Err(e) => error_response(StatusCode::BAD_REQUEST, &format!("Invalid build progress data: {}", e)),
            }
        }
        Err(e) => error_response(StatusCode::BAD_REQUEST, &e),
    }
}

async fn handle_trigger_rebuild(_req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    use std::fs::OpenOptions;
    use std::path::Path;

    // Touch the entry file to trigger rspack rebuild
    // Path is relative to the workspace root (one level up from bridge/)
    let entry_file = Path::new("../src/entry-client.jsx");

    if !entry_file.exists() {
        return error_response(StatusCode::NOT_FOUND, "Entry file not found");
    }

    match OpenOptions::new().write(true).append(true).open(entry_file) {
        Ok(file) => {
            // Set the file's modified time to now by opening it
            drop(file);

            // Also try to use filetime to ensure the timestamp updates
            if let Err(e) = filetime::set_file_mtime(
                entry_file,
                filetime::FileTime::now()
            ) {
                log::warn!("[System] Failed to update file time: {}", e);
            }

            log::info!("[System] Triggered rebuild by touching entry file");
            json_response(&serde_json::json!({
                "success": true,
                "message": "Rebuild triggered"
            }))
        }
        Err(e) => {
            error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to touch file: {}", e))
        }
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
