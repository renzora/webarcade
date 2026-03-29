use webarcade::{App, Request, Response};

fn main() {
    App::new("WebArcade Test", 1024, 768)
        .min_size(400, 300)
        .route("GET", "/api/hello", handle_hello)
        .route("GET", "/api/time", handle_time)
        .frontend("dist")
        .run();
}

fn handle_hello(req: Request) -> Response {
    let name = req.query("name").unwrap_or("WebArcade");
    Response::json(&serde_json::json!({
        "message": format!("Hello, {}!", name),
        "framework": "webarcade",
        "version": "1.0.0"
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
