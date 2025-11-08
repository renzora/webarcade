use crate::core::plugin_context::PluginContext;
use crate::core::plugin_router::PluginRouter;
use anyhow::Result;
use hyper::{Method, Request, Response, StatusCode, body::Incoming};
use hyper::body::Bytes;
use http_body_util::{Full, combinators::BoxBody};
use std::convert::Infallible;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

#[derive(Clone, Debug, serde::Serialize)]
struct BuildStatus {
    is_building: bool,
    success: Option<bool>,
    message: String,
    timestamp: u64,
}

lazy_static::lazy_static! {
    static ref BUILD_STATUS: Arc<Mutex<BuildStatus>> = Arc::new(Mutex::new(BuildStatus {
        is_building: false,
        success: None,
        message: "Ready".to_string(),
        timestamp: SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs(),
    }));
}

pub async fn register_routes(ctx: &PluginContext) -> Result<()> {
    let mut router = PluginRouter::new();

    // GET /overlay-manager/files - List all overlay files
    router.route(Method::GET, "/files", |_path, _query, _req| {
        Box::pin(async move {
            handle_get_files().await
        })
    });

    // GET /overlay-manager/files/:filename - Get specific overlay file content
    router.route(Method::GET, "/files/*", |path, _query, _req| {
        Box::pin(async move {
            handle_get_file(path).await
        })
    });

    // POST /overlay-manager/files/:filename - Save overlay file
    router.route(Method::POST, "/files/*", |path, _query, req| {
        Box::pin(async move {
            handle_save_file(path, req).await
        })
    });

    // POST /overlay-manager/rebuild - Rebuild overlays
    router.route(Method::POST, "/rebuild", |_path, _query, _req| {
        Box::pin(async move {
            handle_rebuild().await
        })
    });

    // GET /overlay-manager/build-status - Get build status
    router.route(Method::GET, "/build-status", |_path, _query, _req| {
        Box::pin(async move {
            handle_build_status().await
        })
    });

    // OPTIONS for CORS preflight
    router.route(Method::OPTIONS, "/files/*", |_path, _query, _req| {
        Box::pin(async move { cors_preflight_response() })
    });
    router.route(Method::OPTIONS, "/rebuild", |_path, _query, _req| {
        Box::pin(async move { cors_preflight_response() })
    });

    ctx.register_router("overlay-manager", router).await;
    Ok(())
}

async fn handle_get_files() -> Response<BoxBody<Bytes, Infallible>> {
    // Path is relative to the workspace root (one level up from bridge/)
    let plugins_dir = Path::new("../plugins");

    match std::fs::read_dir(plugins_dir) {
        Ok(entries) => {
            let mut files: Vec<serde_json::Value> = Vec::new();

            for entry in entries.filter_map(|e| e.ok()) {
                let plugin_path = entry.path();
                if !plugin_path.is_dir() {
                    continue;
                }

                // Look for overlay.jsx in this plugin directory
                let overlay_file = plugin_path.join("overlay.jsx");
                if overlay_file.exists() {
                    if let Some(plugin_name) = plugin_path.file_name() {
                        files.push(serde_json::json!({
                            "name": plugin_name.to_string_lossy(),
                            "path": overlay_file.to_string_lossy(),
                        }));
                    }
                }
            }

            json_response(&files)
        }
        Err(e) => {
            log::warn!("[OverlayManager] Failed to read plugins directory: {}", e);
            // Return empty array if directory doesn't exist
            json_response(&Vec::<serde_json::Value>::new())
        }
    }
}

async fn handle_get_file(path: String) -> Response<BoxBody<Bytes, Infallible>> {
    // Extract plugin name from /files/*
    let plugin_name = if let Some(name) = path.strip_prefix("/files/") {
        urlencoding::decode(name).unwrap_or_default().to_string()
    } else {
        return error_response(StatusCode::BAD_REQUEST, "Invalid path");
    };

    // Path is relative to the workspace root (one level up from bridge/)
    let file_path = format!("../plugins/{}/overlay.jsx", plugin_name);

    match std::fs::read_to_string(&file_path) {
        Ok(content) => {
            // Return plain text, not JSON
            Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "text/plain; charset=utf-8")
                .header("Access-Control-Allow-Origin", "*")
                .body(full_body(&content))
                .unwrap()
        }
        Err(e) => error_response(StatusCode::NOT_FOUND, &format!("File not found: {}", e)),
    }
}

async fn handle_save_file(path: String, req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    // Extract plugin name from /files/*
    let plugin_name = if let Some(name) = path.strip_prefix("/files/") {
        urlencoding::decode(name).unwrap_or_default().to_string()
    } else {
        return error_response(StatusCode::BAD_REQUEST, "Invalid path");
    };

    // Path is relative to the workspace root (one level up from bridge/)
    let plugin_dir = format!("../plugins/{}", plugin_name);
    let file_path = format!("{}/overlay.jsx", plugin_dir);

    match read_json_body(req).await {
        Ok(body) => {
            let content = match body.get("content").and_then(|v| v.as_str()) {
                Some(c) => c,
                None => return error_response(StatusCode::BAD_REQUEST, "Missing content"),
            };

            // Ensure plugin directory exists
            if let Err(e) = std::fs::create_dir_all(&plugin_dir) {
                return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string());
            }

            // Write file
            match std::fs::write(&file_path, content) {
                Ok(_) => json_response(&serde_json::json!({ "success": true })),
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::BAD_REQUEST, &e),
    }
}

async fn handle_rebuild() -> Response<BoxBody<Bytes, Infallible>> {
    // Check if already building
    {
        let status = BUILD_STATUS.lock().unwrap();
        if status.is_building {
            return error_response(StatusCode::CONFLICT, "Build already in progress");
        }
    }

    // Update status to building
    {
        let mut status = BUILD_STATUS.lock().unwrap();
        status.is_building = true;
        status.success = None;
        status.message = "Building overlays...".to_string();
        status.timestamp = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
    }

    // Spawn build task in background
    tokio::spawn(async move {
        use std::process::Command;

        let result = Command::new("bun")
            .args(&["run", "build:overlays"])
            .output();

        let mut status = BUILD_STATUS.lock().unwrap();
        status.is_building = false;
        status.timestamp = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();

        match result {
            Ok(output) => {
                if output.status.success() {
                    status.success = Some(true);
                    status.message = "Build completed successfully".to_string();
                    log::info!("[OverlayManager] Rebuild completed successfully");
                } else {
                    status.success = Some(false);
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    status.message = format!("Build failed: {}", stderr);
                    log::error!("[OverlayManager] Rebuild failed: {}", stderr);
                }
            }
            Err(e) => {
                status.success = Some(false);
                status.message = format!("Failed to run build: {}", e);
                log::error!("[OverlayManager] Failed to run build: {}", e);
            }
        }
    });

    // Return immediately
    json_response(&serde_json::json!({
        "success": true,
        "message": "Build started"
    }))
}

async fn handle_build_status() -> Response<BoxBody<Bytes, Infallible>> {
    let status = BUILD_STATUS.lock().unwrap();
    json_response(&*status)
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

fn cors_preflight_response() -> Response<BoxBody<Bytes, Infallible>> {
    Response::builder()
        .status(StatusCode::OK)
        .header("Access-Control-Allow-Origin", "*")
        .header("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE, OPTIONS")
        .header("Access-Control-Allow-Headers", "Content-Type")
        .body(full_body(""))
        .unwrap()
}
