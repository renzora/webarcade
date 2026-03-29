use crate::mime;
use crate::routing::Router;
use crate::request::Request;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub fn handle_request(
    router: &Router,
    frontend: Option<&PathBuf>,
    frontend_embedded: Option<&'static include_dir::Dir<'static>>,
    method: &str,
    path: &str,
    query: &str,
    headers: HashMap<String, String>,
    body: &[u8],
) -> (u16, String, Vec<u8>, Vec<(String, String)>) {
    if let Some(handler) = router.match_route(method, path) {
        let req = Request::from_raw(method, path, query, headers, body);
        let response = handler(req);
        return (response.status, response.content_type, response.body, response.headers);
    }

    // Try embedded frontend first
    if let Some(dir) = frontend_embedded {
        if let Some((content, content_type)) = serve_embedded(dir, path) {
            return (200, content_type.to_string(), content, vec![]);
        }
    }

    // Fall back to disk
    if let Some(dir) = frontend {
        if let Some((content, content_type)) = serve_static(dir, path) {
            return (200, content_type.to_string(), content, vec![]);
        }
    }

    (404, "text/plain".to_string(), b"Not found".to_vec(), vec![])
}

fn serve_embedded(dir: &include_dir::Dir<'static>, path: &str) -> Option<(Vec<u8>, &'static str)> {
    let file_path = if path == "/" || path.is_empty() {
        "index.html"
    } else {
        path.trim_start_matches('/')
    };

    let mime_type = mime::from_path(file_path);

    if let Some(file) = dir.get_file(file_path) {
        return Some((file.contents().to_vec(), mime_type));
    }

    // SPA fallback
    if !file_path.contains('.') {
        if let Some(file) = dir.get_file("index.html") {
            return Some((file.contents().to_vec(), "text/html; charset=utf-8"));
        }
    }

    None
}

fn serve_static(dir: &Path, path: &str) -> Option<(Vec<u8>, &'static str)> {
    let file_path = if path == "/" || path.is_empty() {
        "index.html"
    } else {
        path.trim_start_matches('/')
    };

    let mime_type = mime::from_path(file_path);
    let full_path = dir.join(file_path);

    if let (Ok(canonical_dir), Ok(canonical_file)) = (dir.canonicalize(), full_path.canonicalize()) {
        if canonical_file.starts_with(&canonical_dir) {
            if let Ok(contents) = std::fs::read(&canonical_file) {
                return Some((contents, mime_type));
            }
        }
    }

    if !file_path.contains('.') {
        let index = dir.join("index.html");
        if let Ok(contents) = std::fs::read(&index) {
            return Some((contents, "text/html; charset=utf-8"));
        }
    }

    None
}
