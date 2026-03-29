#![windows_subsystem = "windows"]

use webarcade::{App, Request, Response};
use include_dir::{include_dir, Dir};

static DIST: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/dist");

fn main() {
    env_logger::init();

    App::new("WebArcade Test", 1024, 768)
        .min_size(400, 300)
        .route("GET", "/api/hello", handle_hello)
        .route("GET", "/api/time", handle_time)
        .frontend(&DIST)
        .run();
}

fn handle_hello(req: Request) -> Response {
    let name = req.query("name").unwrap_or("WebArcade");
    Response::json(&serde_json::json!({
        "message": format!("Hello, {}!", name),
        "framework": "webarcade",
        "version": "0.1.0"
    }))
}

fn handle_time(_req: Request) -> Response {
    Response::json(&serde_json::json!({
        "timestamp": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }))
}
