use crate::routing::Router;
use crate::request::{Request, Response};
use crate::window::{self, IpcRequest, IpcResponse};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tao::dpi::LogicalSize;
use tao::event::{Event, WindowEvent};
use tao::event_loop::{ControlFlow, EventLoop, EventLoopBuilder};
use tao::window::WindowBuilder;
use wry::{WebViewBuilder, WebContext};

const IPC_BRIDGE_JS: &str = include_str!("ipc_bridge.js");

#[derive(Debug)]
enum UserEvent {
    IpcResponse(String),
    CloseRequested,
}

pub struct App {
    title: String,
    width: f64,
    height: f64,
    min_width: f64,
    min_height: f64,
    decorations: bool,
    router: Router,
    frontend_path: Option<String>,
}

impl App {
    pub fn new(title: impl Into<String>, width: u32, height: u32) -> Self {
        Self {
            title: title.into(),
            width: width as f64,
            height: height as f64,
            min_width: 400.0,
            min_height: 300.0,
            decorations: false,
            router: Router::new(),
            frontend_path: None,
        }
    }

    pub fn min_size(mut self, width: u32, height: u32) -> Self {
        self.min_width = width as f64;
        self.min_height = height as f64;
        self
    }

    pub fn decorations(mut self, enabled: bool) -> Self {
        self.decorations = enabled;
        self
    }

    pub fn route<F>(mut self, method: &str, path: &str, handler: F) -> Self
    where
        F: Fn(Request) -> Response + Send + Sync + 'static,
    {
        self.router.add(method, path, handler);
        self
    }

    /// Set the frontend directory (relative to the executable)
    pub fn frontend(mut self, path: impl Into<String>) -> Self {
        self.frontend_path = Some(path.into());
        self
    }

    pub fn run(self) {
        // Hide console window on Windows
        #[cfg(target_os = "windows")]
        {
            use windows::Win32::System::Console::GetConsoleWindow;
            use windows::Win32::UI::WindowsAndMessaging::{ShowWindow, SW_HIDE};
            unsafe {
                let console = GetConsoleWindow();
                if !console.is_invalid() {
                    let _ = ShowWindow(console, SW_HIDE);
                }
            }
        }

        #[cfg(target_os = "linux")]
        if std::env::var("GDK_BACKEND").is_err() {
            std::env::set_var("GDK_BACKEND", "x11");
        }

        // Resolve frontend directory relative to executable
        let frontend = self.frontend_path.map(|rel| {
            let exe_dir = std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(|p| p.to_path_buf()))
                .unwrap_or_else(|| PathBuf::from("."));

            // Try relative to exe first, then relative to cwd (for dev)
            let from_exe = exe_dir.join(&rel);
            if from_exe.exists() {
                from_exe
            } else {
                PathBuf::from(&rel)
            }
        });

        let router = Arc::new(self.router);
        let frontend = Arc::new(frontend);

        let event_loop: EventLoop<UserEvent> = EventLoopBuilder::with_user_event().build();
        let proxy = event_loop.create_proxy();
        let close_proxy = event_loop.create_proxy();

        let window = WindowBuilder::new()
            .with_title(&self.title)
            .with_inner_size(LogicalSize::new(self.width, self.height))
            .with_min_inner_size(LogicalSize::new(self.min_width, self.min_height))
            .with_decorations(self.decorations)
            .build(&event_loop)
            .expect("Failed to create window");

        let window = Arc::new(window);
        let window_for_ipc = window.clone();

        let data_dir = dirs::data_local_dir()
            .unwrap_or_else(std::env::temp_dir)
            .join(&self.title.replace(' ', "_").to_lowercase());

        let mut web_context = WebContext::new(Some(data_dir));

        let router_for_protocol = router.clone();
        let frontend_for_protocol = frontend.clone();

        let webview = WebViewBuilder::with_web_context(&mut web_context)
            .with_custom_protocol("app".into(), move |_webview, request| {
                let method = request.method().as_str();
                let path = request.uri().path();
                let query = request.uri().query().unwrap_or("");
                let body = request.body();

                let (status, content_type, response_body, extra_headers) =
                    crate::protocol::handle_request(
                        &router_for_protocol,
                        frontend_for_protocol.as_ref().as_ref(),
                        method,
                        path,
                        query,
                        body,
                    );

                let mut builder = wry::http::Response::builder()
                    .status(status)
                    .header("Content-Type", &content_type)
                    .header("Access-Control-Allow-Origin", "app://localhost");

                for (k, v) in &extra_headers {
                    builder = builder.header(k.as_str(), v.as_str());
                }

                builder.body(response_body.into()).unwrap()
            })
            .with_ipc_handler(move |message| {
                let message_str = message.body();

                match serde_json::from_str::<IpcRequest>(message_str) {
                    Ok(request) => {
                        if request.command == "close" {
                            let _ = close_proxy.send_event(UserEvent::CloseRequested);
                            return;
                        }
                        let response = window::handle_ipc_command(&request, &window_for_ipc);
                        let response_json = serde_json::to_string(&response).unwrap_or_default();
                        let _ = proxy.send_event(UserEvent::IpcResponse(response_json));
                    }
                    Err(e) => {
                        let response = IpcResponse::err(0, format!("Invalid request: {}", e));
                        let response_json = serde_json::to_string(&response).unwrap_or_default();
                        let _ = proxy.send_event(UserEvent::IpcResponse(response_json));
                    }
                }
            })
            .with_url("app://localhost/")
            .with_devtools(cfg!(debug_assertions))
            .with_initialization_script(&format!(
                "document.addEventListener('DOMContentLoaded', () => {{ \
                    const meta = document.createElement('meta'); \
                    meta.httpEquiv = 'Content-Security-Policy'; \
                    meta.content = \"default-src 'self' app: ; script-src 'self' app: 'unsafe-inline'; style-src 'self' app: 'unsafe-inline'; img-src 'self' app: data: blob: https:; font-src 'self' app: data:; connect-src 'self' app:\"; \
                    document.head.prepend(meta); \
                }});"
            ))
            .with_initialization_script(IPC_BRIDGE_JS)
            .build(&window)
            .expect("Failed to create webview");

        let webview = Arc::new(Mutex::new(webview));
        let webview_for_events = webview.clone();

        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Wait;

            match event {
                Event::UserEvent(UserEvent::IpcResponse(response_json)) => {
                    if let Ok(wv) = webview_for_events.lock() {
                        let script = format!("window.__WEBARCADE_IPC_CALLBACK__({})", response_json);
                        let _ = wv.evaluate_script(&script);
                    }
                }
                Event::UserEvent(UserEvent::CloseRequested) => {
                    *control_flow = ControlFlow::Exit;
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
}
