use hyper::{Response, StatusCode};
use http_body_util::{Full, combinators::BoxBody};
use hyper::body::Bytes;
use std::convert::Infallible;
use std::path::PathBuf;
use std::fs;

/// Handle /api/plugins/list - list runtime plugins
pub fn handle_list_plugins() -> Response<BoxBody<Bytes, Infallible>> {
    let plugins_dir = dirs::data_local_dir()
        .expect("Failed to get local data directory")
        .join("WebArcade")
        .join("plugins");

    if !plugins_dir.exists() {
        let json = serde_json::json!({
            "plugins": []
        }).to_string();

        return Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/json")
            .header("Access-Control-Allow-Origin", "*")
            .body(full_body(&json))
            .unwrap();
    }

    let mut plugins = Vec::new();

    if let Ok(entries) = fs::read_dir(&plugins_dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_dir() {
                    let manifest_path = path.join("manifest.json");
                    if let Ok(manifest_content) = fs::read_to_string(&manifest_path) {
                        if let Ok(manifest) = serde_json::from_str::<serde_json::Value>(&manifest_content) {
                            plugins.push(manifest);
                        }
                    }
                }
            }
        }
    }

    let json = serde_json::json!({
        "plugins": plugins
    }).to_string();

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .header("Access-Control-Allow-Origin", "*")
        .body(full_body(&json))
        .unwrap()
}

/// Handle /api/plugins/{plugin_id}/{file} - serve plugin files
pub fn handle_serve_plugin_file(plugin_id: &str, file_path: &str) -> Response<BoxBody<Bytes, Infallible>> {
    let plugins_dir = dirs::data_local_dir()
        .expect("Failed to get local data directory")
        .join("WebArcade")
        .join("plugins");

    let plugin_dir = plugins_dir.join(plugin_id);
    let file = plugin_dir.join(file_path);

    // Security: Ensure the file is within the plugin directory
    if !file.starts_with(&plugin_dir) {
        return error_response(StatusCode::FORBIDDEN, "Access denied");
    }

    if !file.exists() {
        return error_response(StatusCode::NOT_FOUND, "File not found");
    }

    match fs::read(&file) {
        Ok(content) => {
            let content_type = if file_path.ends_with(".js") {
                "application/javascript"
            } else if file_path.ends_with(".json") {
                "application/json"
            } else {
                "application/octet-stream"
            };

            Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", content_type)
                .header("Access-Control-Allow-Origin", "*")
                .body(BoxBody::new(Full::new(Bytes::from(content))))
                .unwrap()
        }
        Err(_) => error_response(StatusCode::INTERNAL_SERVER_ERROR, "Failed to read file")
    }
}

fn full_body(s: &str) -> BoxBody<Bytes, Infallible> {
    use http_body_util::combinators::BoxBody;
    use http_body_util::BodyExt;
    BoxBody::new(Full::new(Bytes::from(s.to_string())).map_err(|err: Infallible| match err {}))
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
