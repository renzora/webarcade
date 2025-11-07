use crate::core::plugin_context::PluginContext;
use crate::core::plugin_router::PluginRouter;
use anyhow::Result;
use hyper::{Method, Request, Response, StatusCode, body::Incoming};
use hyper::body::Bytes;
use http_body_util::{Full, combinators::BoxBody};
use std::convert::Infallible;

pub async fn register_routes(ctx: &PluginContext) -> Result<()> {
    let mut router = PluginRouter::new();

    // GET /files/read/:path - Read file
    router.route(Method::GET, "/read/*", |path, _query, _req| {
        Box::pin(async move {
            handle_read_file(path).await
        })
    });

    // GET /files/file/:path - Get file (alias for read)
    router.route(Method::GET, "/file/*", |path, _query, _req| {
        Box::pin(async move {
            handle_read_file(path).await
        })
    });

    // POST /files/write/:path - Write file
    router.route(Method::POST, "/write/*", |path, _query, req| {
        Box::pin(async move {
            handle_write_file(path, req).await
        })
    });

    // POST /files/write-binary/:path - Write binary file
    router.route(Method::POST, "/write-binary/*", |path, _query, req| {
        Box::pin(async move {
            handle_write_file(path, req).await
        })
    });

    // DELETE /files/delete/:path - Delete file
    router.route(Method::DELETE, "/delete/*", |path, _query, _req| {
        Box::pin(async move {
            handle_delete_file(path).await
        })
    });

    // GET /files/list/:path - List directory
    router.route(Method::GET, "/list/*", |path, _query, _req| {
        Box::pin(async move {
            handle_list_files(path).await
        })
    });

    ctx.register_router("files", router).await;
    Ok(())
}

async fn handle_read_file(path: String) -> Response<BoxBody<Bytes, Infallible>> {
    // Extract file path from /read/* or /file/*
    let file_path = if path.starts_with("/read/") {
        urlencoding::decode(&path[6..]).unwrap_or_default().to_string()
    } else if path.starts_with("/file/") {
        urlencoding::decode(&path[6..]).unwrap_or_default().to_string()
    } else {
        return error_response(StatusCode::BAD_REQUEST, "Invalid path");
    };

    // Validate path
    if let Err(e) = super::validate_file_path(&file_path) {
        return error_response(StatusCode::FORBIDDEN, &e.to_string());
    }

    // Read file
    match std::fs::read_to_string(&file_path) {
        Ok(content) => json_response(&serde_json::json!({ "content": content })),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn handle_write_file(path: String, req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    // Extract file path from /write/* or /write-binary/*
    let file_path = if path.starts_with("/write/") {
        urlencoding::decode(&path[7..]).unwrap_or_default().to_string()
    } else if path.starts_with("/write-binary/") {
        urlencoding::decode(&path[14..]).unwrap_or_default().to_string()
    } else {
        return error_response(StatusCode::BAD_REQUEST, "Invalid path");
    };

    // Validate path
    if let Err(e) = super::validate_file_path(&file_path) {
        return error_response(StatusCode::FORBIDDEN, &e.to_string());
    }
    if let Err(e) = super::validate_file_extension(&file_path) {
        return error_response(StatusCode::FORBIDDEN, &e.to_string());
    }

    match read_json_body(req).await {
        Ok(body) => {
            let content = body.get("content").and_then(|v| v.as_str()).unwrap_or("");

            // Ensure parent directory exists
            if let Some(parent) = std::path::Path::new(&file_path).parent() {
                if let Err(e) = std::fs::create_dir_all(parent) {
                    return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string());
                }
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

async fn handle_delete_file(path: String) -> Response<BoxBody<Bytes, Infallible>> {
    // Extract file path from /delete/*
    let file_path = if path.starts_with("/delete/") {
        urlencoding::decode(&path[8..]).unwrap_or_default().to_string()
    } else {
        return error_response(StatusCode::BAD_REQUEST, "Invalid path");
    };

    // Validate path
    if let Err(e) = super::validate_file_path(&file_path) {
        return error_response(StatusCode::FORBIDDEN, &e.to_string());
    }

    // Delete file
    match std::fs::remove_file(&file_path) {
        Ok(_) => json_response(&serde_json::json!({ "success": true })),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn handle_list_files(path: String) -> Response<BoxBody<Bytes, Infallible>> {
    // Extract directory path from /list/*
    let dir_path = if path.starts_with("/list/") {
        urlencoding::decode(&path[6..]).unwrap_or_default().to_string()
    } else {
        return error_response(StatusCode::BAD_REQUEST, "Invalid path");
    };

    // Validate path
    if let Err(e) = super::validate_file_path(&dir_path) {
        return error_response(StatusCode::FORBIDDEN, &e.to_string());
    }

    // List files
    match std::fs::read_dir(&dir_path) {
        Ok(entries) => {
            let files: Vec<serde_json::Value> = entries
                .filter_map(|entry| {
                    let entry = entry.ok()?;
                    let path = entry.path();
                    let metadata = entry.metadata().ok()?;

                    Some(serde_json::json!({
                        "name": path.file_name()?.to_string_lossy(),
                        "path": path.to_string_lossy(),
                        "is_dir": metadata.is_dir(),
                        "size": metadata.len(),
                    }))
                })
                .collect();

            json_response(&serde_json::json!({ "files": files }))
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
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
