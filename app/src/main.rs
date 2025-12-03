#![windows_subsystem = "windows"]
#![allow(dead_code)]
#![allow(unused_imports)]

use std::sync::{Arc, Mutex};
use tao::{
    dpi::{LogicalPosition, LogicalSize},
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopBuilder},
    window::{Window, WindowBuilder},
};
use wry::{WebViewBuilder, WebContext};
use serde::{Deserialize, Serialize};

mod ipc;
mod bridge;
mod plugin_installer;

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
        Self {
            id,
            success: true,
            data: None,
            error: None,
        }
    }

    pub fn err(id: u64, msg: impl Into<String>) -> Self {
        Self {
            id,
            success: false,
            data: None,
            error: Some(msg.into()),
        }
    }
}

#[derive(Debug, Deserialize)]
struct IpcRequest {
    id: u64,
    command: String,
    #[serde(default)]
    args: serde_json::Value,
}

// Custom event for IPC responses
#[derive(Debug)]
enum UserEvent {
    IpcResponse(String),
}

fn main() {
    // Initialize logger
    let _ = env_logger::Builder::from_default_env()
        .format_timestamp_secs()
        .try_init();

    log::info!("WebArcade starting...");

    // Start the bridge server in background
    start_bridge_server();

    let event_loop: EventLoop<UserEvent> = EventLoopBuilder::with_user_event().build();
    let proxy = event_loop.create_proxy();

    // Create borderless window
    let window = WindowBuilder::new()
        .with_title("WebArcade")
        .with_inner_size(LogicalSize::new(1280.0, 720.0))
        .with_min_inner_size(LogicalSize::new(800.0, 600.0))
        .with_decorations(false)
        .build(&event_loop)
        .expect("Failed to create window");

    let window = Arc::new(window);
    let window_for_ipc = window.clone();

    // Get WebView2 data directory in AppData (avoids Program Files permission issues)
    let data_dir = dirs::data_local_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join(env!("CARGO_PKG_NAME"));

    // Create web context with custom data directory
    let mut web_context = WebContext::new(Some(data_dir));

    // Create webview with IPC handler
    let webview = WebViewBuilder::with_web_context(&mut web_context)
        .with_ipc_handler(move |message| {
            let message_str = message.body();
            log::debug!("IPC message received: {}", message_str);

            // Parse the IPC request
            match serde_json::from_str::<IpcRequest>(message_str) {
                Ok(request) => {
                    let response = handle_ipc_command(&request, &window_for_ipc);
                    let response_json = serde_json::to_string(&response).unwrap_or_default();
                    let _ = proxy.send_event(UserEvent::IpcResponse(response_json));
                }
                Err(e) => {
                    log::error!("Failed to parse IPC request: {}", e);
                    let response = IpcResponse::err(0, format!("Invalid request: {}", e));
                    let response_json = serde_json::to_string(&response).unwrap_or_default();
                    let _ = proxy.send_event(UserEvent::IpcResponse(response_json));
                }
            }
        })
        .with_url(get_webview_url())
        .with_devtools(true)
        .with_initialization_script(include_str!("ipc_bridge.js"))
        .build(&window)
        .expect("Failed to create webview");

    // Store webview for sending responses
    let webview = Arc::new(Mutex::new(webview));
    let webview_for_events = webview.clone();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::UserEvent(UserEvent::IpcResponse(response_json)) => {
                // Send response back to JavaScript
                if let Ok(wv) = webview_for_events.lock() {
                    let script = format!("window.__WEBARCADE_IPC_CALLBACK__({})", response_json);
                    let _ = wv.evaluate_script(&script);
                }
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            _ => {}
        }
    });
}

fn get_webview_url() -> &'static str {
    // Always load from bridge server which serves embedded dist/
    "http://127.0.0.1:3001"
}

fn handle_ipc_command(request: &IpcRequest, window: &Window) -> IpcResponse {
    let id = request.id;
    let args = &request.args;

    match request.command.as_str() {
        "ping" => IpcResponse::ok(id, "pong"),

        "close" => {
            std::process::exit(0);
        }

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
            window.set_outer_position(LogicalPosition::new(x, y));
            IpcResponse::ok_empty(id)
        }

        "getPosition" => {
            let pos = window.outer_position().unwrap_or(tao::dpi::PhysicalPosition::new(0, 0));
            IpcResponse::ok(id, serde_json::json!({
                "x": pos.x,
                "y": pos.y
            }))
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

        "isMaximized" => {
            IpcResponse::ok(id, window.is_maximized())
        }

        _ => IpcResponse::err(id, format!("Unknown command: {}", request.command))
    }
}

/// Start the bridge server in a background thread
fn start_bridge_server() {
    log::info!("[BRIDGE] Starting integrated bridge server...");

    std::thread::spawn(|| {
        let runtime = match tokio::runtime::Builder::new_multi_thread()
            .worker_threads(4)
            .enable_all()
            .thread_name("bridge-worker")
            .build()
        {
            Ok(rt) => rt,
            Err(e) => {
                log::error!("Failed to create tokio runtime: {}", e);
                return;
            }
        };

        runtime.block_on(async {
            log::info!("[BRIDGE] Tokio runtime created");

            match bridge::run_server().await {
                Ok(_) => log::info!("[BRIDGE] Server stopped"),
                Err(e) => log::error!("[BRIDGE ERROR] {}", e),
            }
        });
    });
}
