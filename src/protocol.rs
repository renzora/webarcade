use crate::mime;
use crate::routing::Router;
use crate::request::Request;
use include_dir::Dir;

pub fn handle_request(
    router: &Router,
    frontend: Option<&Dir<'static>>,
    method: &str,
    path: &str,
    query: &str,
    body: &[u8],
) -> (u16, String, Vec<u8>, Vec<(String, String)>) {
    if let Some(handler) = router.match_route(method, path) {
        let req = Request::from_raw(method, path, query, body);
        let response = handler(req);
        return (response.status, response.content_type, response.body, response.headers);
    }

    if let Some(dir) = frontend {
        if let Some((content, content_type)) = serve_static(dir, path) {
            return (200, content_type.to_string(), content, vec![]);
        }
    }

    (404, "text/plain".to_string(), b"Not found".to_vec(), vec![])
}

fn serve_static(dir: &Dir<'static>, path: &str) -> Option<(Vec<u8>, &'static str)> {
    let file_path = if path == "/" || path.is_empty() {
        "index.html"
    } else {
        path.trim_start_matches('/')
    };

    let mime_type = mime::from_path(file_path);

    if let Some(file) = dir.get_file(file_path) {
        return Some((file.contents().to_vec(), mime_type));
    }

    if !file_path.contains('.') {
        if let Some(file) = dir.get_file("index.html") {
            return Some((file.contents().to_vec(), "text/html; charset=utf-8"));
        }
    }

    None
}
