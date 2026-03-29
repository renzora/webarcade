use serde::{Deserialize, Serialize};
use tao::window::Window;
use tao::dpi::LogicalSize;

#[derive(Debug, Serialize)]
pub struct IpcResponse {
    pub id: u64,
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl IpcResponse {
    pub fn ok(id: u64, data: impl Serialize) -> Self {
        Self {
            id,
            success: true,
            data: Some(serde_json::to_value(data).unwrap_or(serde_json::Value::Null)),
            error: None,
        }
    }

    pub fn ok_empty(id: u64) -> Self {
        Self { id, success: true, data: None, error: None }
    }

    pub fn err(id: u64, msg: impl Into<String>) -> Self {
        Self { id, success: false, data: None, error: Some(msg.into()) }
    }
}

#[derive(Debug, Deserialize)]
pub struct IpcRequest {
    pub id: u64,
    pub command: String,
    #[serde(default)]
    pub args: serde_json::Value,
}

pub fn handle_ipc_command(request: &IpcRequest, window: &Window) -> IpcResponse {
    let id = request.id;
    let args = &request.args;

    match request.command.as_str() {
        "ping" => IpcResponse::ok(id, "pong"),

        "minimize" => {
            window.set_minimized(true);
            IpcResponse::ok_empty(id)
        }

        "maximize" => {
            window.set_maximized(true);
            IpcResponse::ok_empty(id)
        }

        "unmaximize" => {
            window.set_maximized(false);
            IpcResponse::ok_empty(id)
        }

        "toggleMaximize" => {
            window.set_maximized(!window.is_maximized());
            IpcResponse::ok_empty(id)
        }

        "fullscreen" => {
            let enabled = args.get("enabled").and_then(|v| v.as_bool()).unwrap_or(true);
            if enabled {
                window.set_fullscreen(Some(tao::window::Fullscreen::Borderless(None)));
            } else {
                window.set_fullscreen(None);
            }
            IpcResponse::ok_empty(id)
        }

        "setSize" => {
            let width = args.get("width").and_then(|v| v.as_f64()).unwrap_or(800.0);
            let height = args.get("height").and_then(|v| v.as_f64()).unwrap_or(600.0);
            window.set_inner_size(LogicalSize::new(width, height));
            IpcResponse::ok_empty(id)
        }

        "getSize" => {
            let size = window.inner_size();
            IpcResponse::ok(id, serde_json::json!({
                "width": size.width,
                "height": size.height
            }))
        }

        "setPosition" => {
            let x = args.get("x").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let y = args.get("y").and_then(|v| v.as_f64()).unwrap_or(0.0);
            window.set_outer_position(tao::dpi::LogicalPosition::new(x, y));
            IpcResponse::ok_empty(id)
        }

        "getPosition" => {
            let pos = window.outer_position().unwrap_or(tao::dpi::PhysicalPosition::new(0, 0));
            IpcResponse::ok(id, serde_json::json!({ "x": pos.x, "y": pos.y }))
        }

        "setMinSize" => {
            let width = args.get("width").and_then(|v| v.as_f64()).unwrap_or(400.0);
            let height = args.get("height").and_then(|v| v.as_f64()).unwrap_or(300.0);
            window.set_min_inner_size(Some(LogicalSize::new(width, height)));
            IpcResponse::ok_empty(id)
        }

        "setMaxSize" => {
            let width = args.get("width").and_then(|v| v.as_f64());
            let height = args.get("height").and_then(|v| v.as_f64());
            match (width, height) {
                (Some(w), Some(h)) => window.set_max_inner_size(Some(LogicalSize::new(w, h))),
                _ => window.set_max_inner_size(None::<LogicalSize<f64>>),
            }
            IpcResponse::ok_empty(id)
        }

        "center" => {
            if let Some(monitor) = window.current_monitor() {
                let screen_size = monitor.size();
                let window_size = window.outer_size();
                let x = (screen_size.width as i32 - window_size.width as i32) / 2;
                let y = (screen_size.height as i32 - window_size.height as i32) / 2;
                window.set_outer_position(tao::dpi::PhysicalPosition::new(x, y));
            }
            IpcResponse::ok_empty(id)
        }

        "setTitle" => {
            let title = args.get("title").and_then(|v| v.as_str()).unwrap_or("WebArcade");
            window.set_title(title);
            IpcResponse::ok_empty(id)
        }

        "startDrag" => {
            let _ = window.drag_window();
            IpcResponse::ok_empty(id)
        }

        "isMaximized" => IpcResponse::ok(id, window.is_maximized()),

        _ => IpcResponse::err(id, format!("Unknown command: {}", request.command)),
    }
}
