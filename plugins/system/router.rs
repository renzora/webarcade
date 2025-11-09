use crate::core::plugin_context::PluginContext;
use crate::core::plugin_router::PluginRouter;
use crate::core::router_utils::*;
use crate::route;
use anyhow::Result;
use hyper::{Request, Response, StatusCode, body::Incoming};
use hyper::body::Bytes;
use http_body_util::combinators::BoxBody;
use std::convert::Infallible;
use sysinfo::System;
use std::sync::{Arc, Mutex};
use std::path::PathBuf;
use std::io::{Read, Write};
use zip::ZipArchive;

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
    route!(router, GET "/stats" => handle_get_stats);

    // GET /system/cpu - Get CPU information
    route!(router, GET "/cpu" => handle_get_cpu);

    // GET /system/memory - Get memory information
    route!(router, GET "/memory" => handle_get_memory);

    // GET /system/settings?key=xxx - Get a specific setting by key
    route!(router, GET "/settings", query => handle_get_settings);

    // GET /system/build-progress - Get current build progress
    route!(router, GET "/build-progress" => handle_get_build_progress);

    // POST /system/build-progress - Update build progress (from rspack plugin)
    route!(router, POST "/build-progress" => handle_post_build_progress);

    // POST /system/trigger-rebuild - Trigger frontend rebuild by touching entry file
    route!(router, POST "/trigger-rebuild" => handle_trigger_rebuild);

    // POST /system/plugins/upload - Upload and install a plugin from a zip file
    route!(router, POST "/plugins/upload" => handle_plugin_upload);

    // DELETE /system/plugins/:plugin_name - Remove an installed plugin
    route!(router, DELETE "/plugins/:plugin_name", path => handle_plugin_delete);

    // OPTIONS /system/plugins/:plugin_name - CORS preflight
    route!(router, OPTIONS "/plugins/:plugin_name" => cors_preflight);

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

async fn handle_plugin_upload(req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    use http_body_util::BodyExt;
    use multer::Multipart;
    use std::fs;
    use futures_util::stream;

    // Get the boundary from Content-Type header
    let content_type = match req.headers().get("content-type") {
        Some(ct) => match ct.to_str() {
            Ok(s) => s,
            Err(_) => return error_response(StatusCode::BAD_REQUEST, "Invalid content-type header"),
        },
        None => return error_response(StatusCode::BAD_REQUEST, "Missing content-type header"),
    };

    let boundary = match multer::parse_boundary(content_type) {
        Ok(b) => b,
        Err(_) => return error_response(StatusCode::BAD_REQUEST, "Invalid multipart boundary"),
    };

    // Collect the body
    let body_bytes = match req.collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(e) => return error_response(StatusCode::BAD_REQUEST, &format!("Failed to read body: {}", e)),
    };

    // Create a stream from the bytes
    let stream = stream::once(async move { Result::<_, std::io::Error>::Ok(body_bytes) });

    // Parse multipart
    let mut multipart = Multipart::new(stream, boundary);

    // Find the plugin file
    let mut plugin_data: Option<Vec<u8>> = None;

    while let Some(field) = multipart.next_field().await.ok().flatten() {
        if field.name() == Some("plugin") {
            match field.bytes().await {
                Ok(bytes) => {
                    plugin_data = Some(bytes.to_vec());
                    break;
                }
                Err(e) => return error_response(StatusCode::BAD_REQUEST, &format!("Failed to read file: {}", e)),
            }
        }
    }

    let plugin_data = match plugin_data {
        Some(data) => data,
        None => return error_response(StatusCode::BAD_REQUEST, "No plugin file found in request"),
    };

    // Extract the zip file
    let cursor = std::io::Cursor::new(plugin_data);
    let mut archive = match ZipArchive::new(cursor) {
        Ok(a) => a,
        Err(e) => return error_response(StatusCode::BAD_REQUEST, &format!("Invalid zip file: {}", e)),
    };

    // Determine plugin name from the zip structure
    // Expected structure: plugin_name/mod.rs, plugin_name/index.jsx, plugin_name/overlay.jsx, etc.
    let plugin_name = match determine_plugin_name(&mut archive) {
        Ok(name) => name,
        Err(e) => return error_response(StatusCode::BAD_REQUEST, &e),
    };

    // Extract to temporary location first
    let temp_dir = std::env::temp_dir().join(format!("webarcade_plugin_{}", plugin_name));
    if temp_dir.exists() {
        let _ = fs::remove_dir_all(&temp_dir);
    }

    if let Err(e) = fs::create_dir_all(&temp_dir) {
        return error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to create temp directory: {}", e));
    }

    // Extract all files
    for i in 0..archive.len() {
        let mut file = match archive.by_index(i) {
            Ok(f) => f,
            Err(e) => {
                let _ = fs::remove_dir_all(&temp_dir);
                return error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to read zip entry: {}", e));
            }
        };

        let outpath = match file.enclosed_name() {
            Some(path) => temp_dir.join(path),
            None => continue,
        };

        if file.name().ends_with('/') {
            let _ = fs::create_dir_all(&outpath);
        } else {
            if let Some(p) = outpath.parent() {
                let _ = fs::create_dir_all(p);
            }
            let mut outfile = match fs::File::create(&outpath) {
                Ok(f) => f,
                Err(e) => {
                    let _ = fs::remove_dir_all(&temp_dir);
                    return error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to create file: {}", e));
                }
            };
            let mut buffer = Vec::new();
            if let Err(e) = file.read_to_end(&mut buffer) {
                let _ = fs::remove_dir_all(&temp_dir);
                return error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to read file: {}", e));
            }
            if let Err(e) = outfile.write_all(&buffer) {
                let _ = fs::remove_dir_all(&temp_dir);
                return error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to write file: {}", e));
            }
        }
    }

    // Now copy the extracted files to the unified plugins directory
    // Working directory is bridge/, so paths are relative to that
    let workspace_root = PathBuf::from("..");

    let plugin_temp = temp_dir.join(&plugin_name);

    // Copy to unified plugins/plugin_name directory
    let plugin_dest = workspace_root.join("plugins").join(&plugin_name);
    if let Err(e) = copy_dir_all(&plugin_temp, &plugin_dest) {
        let _ = fs::remove_dir_all(&temp_dir);
        return error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to copy plugin: {}", e));
    }
    log::info!("[System] Installed plugin to: {:?}", plugin_dest);

    // Clean up temp directory
    let _ = fs::remove_dir_all(&temp_dir);

    log::info!("[System] Successfully installed plugin: {}", plugin_name);

    // Run discovery scripts to regenerate plugins.json and backend generated.rs
    log::info!("[System] Running plugin discovery scripts...");
    let workspace_root = PathBuf::from("..");

    // Run frontend discovery
    let frontend_script = workspace_root.join("scripts").join("discover-plugins.js");
    if frontend_script.exists() {
        match std::process::Command::new("bun")
            .arg("run")
            .arg(&frontend_script)
            .current_dir(&workspace_root)
            .output()
        {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);

                if output.status.success() {
                    log::info!("[System] Frontend plugin discovery completed:");
                    if !stdout.is_empty() {
                        log::info!("{}", stdout);
                    }
                } else {
                    log::warn!("[System] Frontend discovery error: {}", stderr);
                }
            }
            Err(e) => log::warn!("[System] Could not run frontend discovery: {}", e),
        }
    } else {
        log::warn!("[System] Frontend discovery script not found at {:?}", frontend_script);
    }

    // Run backend discovery
    let backend_script = workspace_root.join("scripts").join("discover-backend-plugins.js");
    if backend_script.exists() {
        match std::process::Command::new("bun")
            .arg("run")
            .arg(&backend_script)
            .current_dir(&workspace_root)
            .output()
        {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);

                if output.status.success() {
                    log::info!("[System] Backend plugin discovery completed:");
                    if !stdout.is_empty() {
                        log::info!("{}", stdout);
                    }
                    log::info!("[System] Cargo watch will auto-rebuild the backend");
                } else {
                    log::warn!("[System] Backend discovery error: {}", stderr);
                }
            }
            Err(e) => log::warn!("[System] Could not run backend discovery: {}", e),
        }
    } else {
        log::warn!("[System] Backend discovery script not found at {:?}", backend_script);
    }

    json_response(&serde_json::json!({
        "success": true,
        "pluginName": plugin_name,
        "message": "Plugin installed successfully. The frontend will auto-reload to show the new plugin."
    }))
}

async fn handle_plugin_delete(path: String) -> Response<BoxBody<Bytes, Infallible>> {
    use std::fs;

    // Extract plugin name from path (format: "/plugins/plugin_name")
    let plugin_name = path.trim_start_matches('/').trim_start_matches("plugins/");

    if plugin_name.is_empty() {
        return error_response(StatusCode::BAD_REQUEST, "Plugin name is required");
    }

    log::info!("[System] Removing plugin: {}", plugin_name);

    // Working directory is bridge/, so paths are relative to that
    let workspace_root = PathBuf::from("..");

    // Remove from unified plugins directory
    let plugin_path = workspace_root.join("plugins").join(plugin_name);
    if !plugin_path.exists() {
        return error_response(StatusCode::NOT_FOUND, &format!("Plugin '{}' not found", plugin_name));
    }

    match fs::remove_dir_all(&plugin_path) {
        Ok(_) => {
            log::info!("[System] Removed plugin from: {:?}", plugin_path);
        }
        Err(e) => {
            log::warn!("[System] Failed to remove plugin: {}", e);
            return error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to remove plugin: {}", e));
        }
    }

    // Run discovery scripts to regenerate plugins.json and backend generated.rs
    log::info!("[System] Running plugin discovery scripts...");

    // Run frontend discovery
    let frontend_script = workspace_root.join("scripts").join("discover-plugins.js");
    if frontend_script.exists() {
        match std::process::Command::new("bun")
            .arg("run")
            .arg(&frontend_script)
            .current_dir(&workspace_root)
            .output()
        {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);

                if output.status.success() {
                    log::info!("[System] Frontend plugin discovery completed:");
                    if !stdout.is_empty() {
                        log::info!("{}", stdout);
                    }
                } else {
                    log::warn!("[System] Frontend discovery error: {}", stderr);
                }
            }
            Err(e) => log::warn!("[System] Could not run frontend discovery: {}", e),
        }
    } else {
        log::warn!("[System] Frontend discovery script not found at {:?}", frontend_script);
    }

    // Run backend discovery
    let backend_script = workspace_root.join("scripts").join("discover-backend-plugins.js");
    if backend_script.exists() {
        match std::process::Command::new("bun")
            .arg("run")
            .arg(&backend_script)
            .current_dir(&workspace_root)
            .output()
        {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);

                if output.status.success() {
                    log::info!("[System] Backend plugin discovery completed:");
                    if !stdout.is_empty() {
                        log::info!("{}", stdout);
                    }
                    log::info!("[System] Cargo watch will auto-rebuild the backend");
                } else {
                    log::warn!("[System] Backend discovery error: {}", stderr);
                }
            }
            Err(e) => log::warn!("[System] Could not run backend discovery: {}", e),
        }
    } else {
        log::warn!("[System] Backend discovery script not found at {:?}", backend_script);
    }

    log::info!("[System] Successfully removed plugin: {}", plugin_name);

    json_response(&serde_json::json!({
        "success": true,
        "pluginName": plugin_name,
        "message": "Plugin removed successfully. The frontend will auto-reload."
    }))
}

fn determine_plugin_name(archive: &mut ZipArchive<std::io::Cursor<Vec<u8>>>) -> Result<String, String> {
    // Look for the first directory in the zip
    for i in 0..archive.len() {
        let file = archive.by_index(i).map_err(|e| format!("Failed to read zip: {}", e))?;
        if let Some(path) = file.enclosed_name() {
            if let Some(first_component) = path.components().next() {
                if let Some(name) = first_component.as_os_str().to_str() {
                    if name != "." && name != ".." {
                        return Ok(name.to_string());
                    }
                }
            }
        }
    }
    Err("Could not determine plugin name from zip structure".to_string())
}

fn copy_dir_all(src: &PathBuf, dst: &PathBuf) -> std::io::Result<()> {
    use std::fs;

    if !dst.exists() {
        fs::create_dir_all(dst)?;
    }

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if ty.is_dir() {
            copy_dir_all(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

// Helper functions are now imported from router_utils
