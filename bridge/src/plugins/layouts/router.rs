use crate::core::plugin_context::PluginContext;
use crate::core::plugin_router::PluginRouter;
use anyhow::Result;
use hyper::{Method, Request, Response, StatusCode, body::Incoming};
use hyper::body::Bytes;
use http_body_util::{Full, combinators::BoxBody};
use std::convert::Infallible;

pub async fn register_routes(ctx: &PluginContext) -> Result<()> {
    // API routes at /layouts/*
    let mut router = PluginRouter::new();

    // Clone context for use in POST handler
    let ctx_for_save = ctx.clone();

    // GET /layouts - Get all layouts
    router.route(Method::GET, "/", |_path, _query, _req| {
        Box::pin(async move {
            handle_get_layouts().await
        })
    });

    // GET /layouts/:name - Get specific layout
    router.route(Method::GET, "/*", |path, _query, _req| {
        Box::pin(async move {
            handle_get_layout(path).await
        })
    });

    // POST /layouts/:name - Save/update layout
    router.route(Method::POST, "/*", move |path, _query, req| {
        let ctx = ctx_for_save.clone();
        Box::pin(async move {
            handle_save_layout(path, req, ctx).await
        })
    });

    // DELETE /layouts/:name - Delete layout
    router.route(Method::DELETE, "/*", |path, _query, _req| {
        Box::pin(async move {
            handle_delete_layout(path).await
        })
    });

    // OPTIONS for CORS preflight
    router.route(Method::OPTIONS, "/*", |_path, _query, _req| {
        Box::pin(async move { cors_preflight_response() })
    });

    ctx.register_router("layouts", router).await;

    // Overlay HTML routes at /overlay/layout/*
    let mut overlay_router = PluginRouter::new();

    overlay_router.route(Method::GET, "/*", |path, _query, _req| {
        Box::pin(async move {
            handle_serve_layout_html(path).await
        })
    });

    overlay_router.route(Method::OPTIONS, "/*", |_path, _query, _req| {
        Box::pin(async move { cors_preflight_response() })
    });

    ctx.register_router("overlay/layout", overlay_router).await;

    Ok(())
}

async fn handle_get_layouts() -> Response<BoxBody<Bytes, Infallible>> {
    let mut layout_names = Vec::new();

    // Read layouts from database only
    let db_path = crate::core::database::get_database_path();
    if let Ok(conn) = rusqlite::Connection::open(&db_path) {
        if let Ok(mut stmt) = conn.prepare("SELECT name FROM layouts ORDER BY updated_at DESC") {
            if let Ok(rows) = stmt.query_map([], |row| row.get::<_, String>(0)) {
                for name in rows.flatten() {
                    layout_names.push(name);
                }
            }
        }
    }

    json_response(&layout_names)
}

async fn handle_get_layout(path: String) -> Response<BoxBody<Bytes, Infallible>> {
    let name = if let Some(n) = path.strip_prefix("/") {
        urlencoding::decode(n).unwrap_or_default().to_string()
    } else {
        return error_response(StatusCode::BAD_REQUEST, "Invalid path");
    };

    // Read from database only
    let db_path = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&db_path) {
        Ok(conn) => {
            match conn.query_row(
                "SELECT name, layout_data, created_at, updated_at FROM layouts WHERE name = ?1",
                rusqlite::params![name],
                |row| {
                    let layout_data_str: String = row.get(1)?;
                    let layout_data: serde_json::Value = serde_json::from_str(&layout_data_str)
                        .unwrap_or(serde_json::json!({"overlays": []}));

                    // Extract overlays from layout_data
                    let overlays = layout_data.get("overlays")
                        .cloned()
                        .unwrap_or(serde_json::json!([]));

                    Ok(serde_json::json!({
                        "name": row.get::<_, String>(0)?,
                        "overlays": overlays,
                        "created_at": row.get::<_, i64>(2)?,
                        "updated_at": row.get::<_, i64>(3)?,
                    }))
                },
            ) {
                Ok(layout) => json_response(&layout),
                Err(rusqlite::Error::QueryReturnedNoRows) => {
                    error_response(StatusCode::NOT_FOUND, "Layout not found")
                }
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn handle_save_layout(path: String, req: Request<Incoming>, ctx: PluginContext) -> Response<BoxBody<Bytes, Infallible>> {
    let name = if let Some(n) = path.strip_prefix("/") {
        urlencoding::decode(n).unwrap_or_default().to_string()
    } else {
        return error_response(StatusCode::BAD_REQUEST, "Invalid path");
    };

    match read_json_body(req).await {
        Ok(body) => {
            // Frontend sends {name, overlays}, we need to store {overlays} as layout_data
            let overlays = match body.get("overlays") {
                Some(data) => data.clone(),
                None => return error_response(StatusCode::BAD_REQUEST, "Missing overlays data"),
            };

            let layout_data = serde_json::to_string(&serde_json::json!({
                "overlays": overlays
            })).unwrap_or_else(|_| r#"{"overlays":[]}"#.to_string());

            let db_path = crate::core::database::get_database_path();
            match rusqlite::Connection::open(&db_path) {
                Ok(conn) => {
                    let now = current_timestamp();

                    match conn.execute(
                        "INSERT INTO layouts (name, layout_data, created_at, updated_at)
                         VALUES (?1, ?2, ?3, ?3)
                         ON CONFLICT(name) DO UPDATE SET
                           layout_data = ?2,
                           updated_at = ?3",
                        rusqlite::params![name, layout_data, now],
                    ) {
                        Ok(_) => {
                            // Broadcast layout update event via WebSocket
                            let event_payload = serde_json::json!({
                                "layout_name": name,
                                "layout": {
                                    "overlays": overlays
                                }
                            });
                            ctx.emit("layout_update", &event_payload);
                            log::info!("[Layouts] Broadcasted layout_update event for layout: {}", name);

                            json_response(&serde_json::json!({ "success": true }))
                        },
                        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
                    }
                }
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
        }
        Err(e) => error_response(StatusCode::BAD_REQUEST, &e),
    }
}

async fn handle_delete_layout(path: String) -> Response<BoxBody<Bytes, Infallible>> {
    let name = if let Some(n) = path.strip_prefix("/") {
        urlencoding::decode(n).unwrap_or_default().to_string()
    } else {
        return error_response(StatusCode::BAD_REQUEST, "Invalid path");
    };

    let db_path = crate::core::database::get_database_path();
    match rusqlite::Connection::open(&db_path) {
        Ok(conn) => {
            match conn.execute(
                "DELETE FROM layouts WHERE name = ?1",
                rusqlite::params![name],
            ) {
                Ok(rows) if rows > 0 => json_response(&serde_json::json!({ "success": true })),
                Ok(_) => error_response(StatusCode::NOT_FOUND, "Layout not found"),
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
            }
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

fn cors_preflight_response() -> Response<BoxBody<Bytes, Infallible>> {
    Response::builder()
        .status(StatusCode::OK)
        .header("Access-Control-Allow-Origin", "*")
        .header("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE, OPTIONS")
        .header("Access-Control-Allow-Headers", "Content-Type")
        .body(full_body(""))
        .unwrap()
}

fn current_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

async fn handle_serve_layout_html(path: String) -> Response<BoxBody<Bytes, Infallible>> {
    

    // Extract layout name from path (e.g., "/Main")
    let layout_name = path.trim_start_matches('/');

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

    // Determine frontend URL (where overlays are served)
    let frontend_url = std::env::var("FRONTEND_URL")
        .unwrap_or_else(|_| "http://localhost:3000".to_string());

    // Generate HTML with iframes for each overlay
    let mut overlay_html = String::new();
    for overlay in overlays {
        let overlay_type = overlay.get("type").and_then(|t| t.as_str()).unwrap_or("unknown");
        let x = overlay.get("x").and_then(|v| v.as_i64()).unwrap_or(0);
        let y = overlay.get("y").and_then(|v| v.as_i64()).unwrap_or(0);
        let width = overlay.get("width").and_then(|v| v.as_i64()).unwrap_or(400);
        let height = overlay.get("height").and_then(|v| v.as_i64()).unwrap_or(300);
        let z_index = overlay.get("zIndex").and_then(|v| v.as_i64()).unwrap_or(1);

        overlay_html.push_str(&format!(
            r#"<iframe src="{}/overlay/{}" style="position: absolute; left: {}px; top: {}px; width: {}px; height: {}px; z-index: {}; border: none; background: transparent;"></iframe>"#,
            frontend_url, overlay_type, x, y, width, height, z_index
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
