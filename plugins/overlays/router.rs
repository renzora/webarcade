use crate::core::plugin_context::PluginContext;
use crate::core::plugin_router::PluginRouter;
use anyhow::Result;
use hyper::{Method, Response, StatusCode};
use hyper::body::Bytes;
use http_body_util::{Full, combinators::BoxBody};
use std::convert::Infallible;

pub async fn register_routes(ctx: &PluginContext) -> Result<()> {
    let mut router = PluginRouter::new();

    // Static overlay files at /overlay/* (HTML, JS, CSS from dist/overlays/)
    // This needs to check for layout paths and skip them
    router.route(Method::GET, "/*", |path, _query, _req| {
        Box::pin(async move {
            // If it's a layout path, handle separately
            if path.starts_with("/layout/") {
                handle_serve_layout_html(path).await
            } else {
                handle_serve_overlay_file(path).await
            }
        })
    });

    ctx.register_router("overlay", router).await;

    Ok(())
}

async fn handle_serve_layout_html(path: String) -> Response<BoxBody<Bytes, Infallible>> {
    // Extract layout name from path (e.g., "/layout/Main" -> "Main")
    let layout_name = path.trim_start_matches("/layout/");

    if layout_name.is_empty() {
        return error_response(StatusCode::BAD_REQUEST, "Missing layout name");
    }

    // Decode URL-encoded name
    let layout_name = match urlencoding::decode(layout_name) {
        Ok(name) => name.to_string(),
        Err(_) => return error_response(StatusCode::BAD_REQUEST, "Invalid layout name"),
    };

    // Get layout from database
    let db_path = crate::core::database::get_database_path();
    let layout_data = match rusqlite::Connection::open(&db_path) {
        Ok(conn) => {
            match conn.query_row(
                "SELECT layout_data FROM layouts WHERE name = ?1",
                rusqlite::params![layout_name],
                |row| {
                    let layout_json: String = row.get(0)?;
                    Ok(layout_json)
                },
            ) {
                Ok(data) => data,
                Err(rusqlite::Error::QueryReturnedNoRows) => {
                    return error_response(StatusCode::NOT_FOUND, &format!("Layout '{}' not found", layout_name));
                }
                Err(e) => {
                    return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string());
                }
            }
        }
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };

    // Parse layout data
    let layout: serde_json::Value = match serde_json::from_str(&layout_data) {
        Ok(l) => l,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Invalid layout data: {}", e)),
    };

    // Extract overlays array
    let overlays = layout.get("overlays")
        .and_then(|o| o.as_array())
        .cloned()
        .unwrap_or_default();

    // Generate HTML with iframes for each overlay (using relative URLs)
    let mut overlay_html = String::new();
    for overlay in overlays {
        let overlay_type = overlay.get("type").and_then(|t| t.as_str()).unwrap_or("unknown");
        let x = overlay.get("x").and_then(|v| v.as_i64()).unwrap_or(0);
        let y = overlay.get("y").and_then(|v| v.as_i64()).unwrap_or(0);
        let width = overlay.get("width").and_then(|v| v.as_i64()).unwrap_or(400);
        let height = overlay.get("height").and_then(|v| v.as_i64()).unwrap_or(300);
        let z_index = overlay.get("zIndex").and_then(|v| v.as_i64()).unwrap_or(1);

        overlay_html.push_str(&format!(
            r#"<iframe src="/overlay/{}" style="position: absolute; left: {}px; top: {}px; width: {}px; height: {}px; z-index: {}; border: none; background: transparent;"></iframe>"#,
            overlay_type, x, y, width, height, z_index
        ));
    }

    let html = format!(
        r#"<!DOCTYPE html>
<html lang="en" style="background: transparent !important;">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>{}</title>
  <style>
    * {{
      box-sizing: border-box;
    }}
    html, body {{
      margin: 0 !important;
      padding: 0 !important;
      width: 1920px !important;
      height: 1080px !important;
      background-color: transparent !important;
      overflow: hidden !important;
      -webkit-font-smoothing: antialiased;
    }}
    body {{
      position: relative;
    }}
  </style>
</head>
<body>
  {}
</body>
</html>"#,
        layout_name, overlay_html
    );

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/html; charset=utf-8")
        .header("Access-Control-Allow-Origin", "*")
        .body(full_body(&html))
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

async fn handle_serve_overlay_file(path: String) -> Response<BoxBody<Bytes, Infallible>> {
    use std::path::Path;

    // Extract filename from path (e.g., "/ticker" or "/ticker.js")
    let filename = path.trim_start_matches('/');

    if filename.is_empty() {
        return error_response(StatusCode::BAD_REQUEST, "Missing file name");
    }

    // Determine file extension and construct path
    // Path is relative to the workspace root (one level up from bridge/)
    let (file_path, content_type) = if filename.ends_with(".js") {
        (format!("../dist/overlays/{}", filename), "application/javascript; charset=utf-8")
    } else if filename.ends_with(".css") {
        (format!("../dist/overlays/{}", filename), "text/css; charset=utf-8")
    } else {
        // Assume it's an HTML file request (no extension)
        (format!("../dist/overlays/{}.html", filename), "text/html; charset=utf-8")
    };

    // Check if file exists
    if !Path::new(&file_path).exists() {
        log::warn!("[Overlays] File not found: {}", file_path);
        return error_response(StatusCode::NOT_FOUND, &format!("Overlay '{}' not found", filename));
    }

    // Read and serve the file with no-cache headers for OBS
    match std::fs::read(&file_path) {
        Ok(content) => {
            use http_body_util::BodyExt;
            Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", content_type)
                .header("Access-Control-Allow-Origin", "*")
                .header("Cache-Control", "no-cache, no-store, must-revalidate")
                .header("Pragma", "no-cache")
                .header("Expires", "0")
                .body(BoxBody::new(Full::new(Bytes::from(content)).map_err(|err: Infallible| match err {})))
                .unwrap()
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to read file: {}", e)),
    }
}

fn full_body(s: &str) -> BoxBody<Bytes, Infallible> {
    use http_body_util::combinators::BoxBody;
    use http_body_util::BodyExt;
    BoxBody::new(Full::new(Bytes::from(s.to_string())).map_err(|err: Infallible| match err {}))
}
