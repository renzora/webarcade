#![allow(dead_code)]
#![allow(unused_variables)]

pub mod modules;
pub mod core;
pub mod plugins;

use std::env;
use std::net::SocketAddr;
use std::sync::Arc;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;
use log::{info, error};
use anyhow::Result;
use hyper::{Request, Response, StatusCode, body::Incoming};
use hyper::body::Bytes;
use http_body_util::{Full, combinators::BoxBody};
use std::convert::Infallible;

use crate::bridge::core::{EventBus, ServiceRegistry, PluginManager, WebSocketBridge, RouterRegistry, DynamicPluginLoader};

/// Start the WebArcade bridge server
pub async fn run_server() -> Result<()> {
    // Initialize logger with timestamp
    env_logger::Builder::from_default_env()
        .format_timestamp_secs()
        .init();

    info!("🎮 WebArcade Bridge - Plugin System v2.0");
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    // Get configuration
    let port = env::var("BRIDGE_PORT").unwrap_or_else(|_| "3001".to_string());
    let ws_port = env::var("WS_PORT").unwrap_or_else(|_| "3002".to_string());

    // Initialize core systems
    info!("📦 Initializing core systems...");

    let event_bus = Arc::new(EventBus::new());
    let service_registry = Arc::new(ServiceRegistry::new());

    // Get database path
    let db_path = crate::bridge::core::database::get_database_path();
    crate::bridge::core::database::ensure_database_dir()?;

    info!("💾 Database: {}", db_path.display());

    // Create router registry
    let router_registry = RouterRegistry::new();

    // Create plugin manager
    let mut plugin_manager = PluginManager::new(
        event_bus.clone(),
        service_registry.clone(),
        router_registry.clone_registry(),
        db_path.to_string_lossy().to_string(),
    );

    // Register static (compiled) plugins
    info!("📦 Registering static plugins...");
    crate::bridge::plugins::register_all_plugins(&mut plugin_manager);

    // Initialize static plugins
    info!("🔧 Initializing static plugins...");
    plugin_manager.init_all().await?;

    // Start static plugins
    info!("🚀 Starting static plugins...");
    plugin_manager.start_all().await?;

    // Load dynamic (runtime) plugins
    info!("📦 Loading dynamic plugins...");
    let plugins_dir = dirs::data_local_dir()
        .expect("Failed to get local data directory")
        .join("WebArcade")
        .join("plugins");

    let mut dynamic_loader = DynamicPluginLoader::new(plugins_dir.clone());

    match dynamic_loader.load_all_plugins() {
        Ok(dynamic_plugins) => {
            info!("✅ Discovered {} dynamic plugins", dynamic_plugins.len());

            // Register dynamic plugin routes
            for plugin_info in &dynamic_plugins {
                info!("   - {} (backend: {}, frontend: {})",
                    plugin_info.id,
                    plugin_info.has_backend,
                    plugin_info.has_frontend
                );

                // Register routes from routes.json
                if !plugin_info.routes.is_empty() {
                    info!("     └─ Registering {} routes", plugin_info.routes.len());

                    // Create a router for this plugin
                    let mut plugin_router = crate::bridge::core::PluginRouter::new();

                    for route in &plugin_info.routes {
                        if let (Some(method_str), Some(path), Some(handler_name)) = (
                            route.get("method").and_then(|v| v.as_str()),
                            route.get("path").and_then(|v| v.as_str()),
                            route.get("handler").and_then(|v| v.as_str()),
                        ) {
                            // Parse HTTP method
                            let method = match method_str {
                                "GET" => hyper::Method::GET,
                                "POST" => hyper::Method::POST,
                                "PUT" => hyper::Method::PUT,
                                "DELETE" => hyper::Method::DELETE,
                                "PATCH" => hyper::Method::PATCH,
                                _ => {
                                    error!("Unknown HTTP method: {}", method_str);
                                    continue;
                                }
                            };

                            let plugin_id = plugin_info.id.clone();
                            let handler_name_owned = handler_name.to_string();

                            // Create a handler that will call the DLL function
                            plugin_router.route(method, path, move |_path_arg, _query, _req| {
                                let plugin_id = plugin_id.clone();
                                let handler_name = handler_name_owned.clone();

                                Box::pin(async move {
                                    use http_body_util::Full;
                                    use hyper::body::Bytes;
                                    use http_body_util::combinators::BoxBody;
                                    use libloading::Symbol;

                                    // Look up the plugin library
                                    let libs = crate::bridge::core::plugin_exports::PLUGIN_LIBRARIES.lock().unwrap();

                                    if let Some(lib) = libs.get(&plugin_id) {
                                        // Look up the handler function in the DLL
                                        // Handler signature: extern "C" fn() -> *const u8
                                        let result: Result<Symbol<extern "C" fn() -> *const u8>, _> = unsafe {
                                            lib.get(handler_name.as_bytes())
                                        };

                                        match result {
                                            Ok(handler_fn) => {
                                                // Call the handler
                                                let json_ptr = handler_fn();

                                                if json_ptr.is_null() {
                                                    let error_json = serde_json::json!({
                                                        "error": "Handler returned null"
                                                    }).to_string();

                                                    return hyper::Response::builder()
                                                        .status(500)
                                                        .header("Content-Type", "application/json")
                                                        .header("Access-Control-Allow-Origin", "*")
                                                        .body(BoxBody::new(Full::new(Bytes::from(error_json))))
                                                        .unwrap();
                                                }

                                                // Read the JSON string from the pointer
                                                let json_str = unsafe {
                                                    let c_str = std::ffi::CStr::from_ptr(json_ptr as *const i8);
                                                    c_str.to_string_lossy().into_owned()
                                                };

                                                // Free the string (if the plugin exports free_string)
                                                let free_result: Result<Symbol<extern "C" fn(*mut u8)>, _> = unsafe {
                                                    lib.get(b"free_string")
                                                };
                                                if let Ok(free_fn) = free_result {
                                                    free_fn(json_ptr as *mut u8);
                                                }

                                                hyper::Response::builder()
                                                    .status(200)
                                                    .header("Content-Type", "application/json")
                                                    .header("Access-Control-Allow-Origin", "*")
                                                    .body(BoxBody::new(Full::new(Bytes::from(json_str))))
                                                    .unwrap()
                                            }
                                            Err(e) => {
                                                let error_json = serde_json::json!({
                                                    "error": format!("Handler function '{}' not found in plugin: {}", handler_name, e)
                                                }).to_string();

                                                hyper::Response::builder()
                                                    .status(500)
                                                    .header("Content-Type", "application/json")
                                                    .header("Access-Control-Allow-Origin", "*")
                                                    .body(BoxBody::new(Full::new(Bytes::from(error_json))))
                                                    .unwrap()
                                            }
                                        }
                                    } else {
                                        let error_json = serde_json::json!({
                                            "error": format!("Plugin library not found: {}", plugin_id)
                                        }).to_string();

                                        hyper::Response::builder()
                                            .status(500)
                                            .header("Content-Type", "application/json")
                                            .header("Access-Control-Allow-Origin", "*")
                                            .body(BoxBody::new(Full::new(Bytes::from(error_json))))
                                            .unwrap()
                                    }
                                })
                            });

                            info!("       {} {} -> {}", method_str, path, handler_name);
                        }
                    }

                    // Register the router
                    let plugin_id = plugin_info.id.clone();
                    let registry_clone = router_registry.clone_registry();
                    tokio::spawn(async move {
                        registry_clone.register(plugin_id, plugin_router).await;
                    });
                }
            }
        }
        Err(e) => {
            info!("⚠️  No dynamic plugins loaded: {}", e);
        }
    }

    // List loaded static plugins
    let loaded_plugins = plugin_manager.list_plugins();
    info!("✅ Loaded {} static plugins:", loaded_plugins.len());
    for plugin in &loaded_plugins {
        info!("   - {} v{}: {}", plugin.name, plugin.version, plugin.description);
    }

    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    // Start WebSocket server for real-time events
    let event_bus_ws = event_bus.clone();
    let ws_port_clone = ws_port.clone();
    tokio::spawn(async move {
        let ws_bridge = WebSocketBridge::new(event_bus_ws);
        if let Err(e) = ws_bridge.start(ws_port_clone).await {
            error!("WebSocket server error: {}", e);
        }
    });

    // Start HTTP server
    let addr: SocketAddr = format!("127.0.0.1:{}", port).parse()?;
    let listener = TcpListener::bind(addr).await?;

    info!("🌐 HTTP server listening on http://{}", addr);
    info!("📡 WebSocket server listening on ws://127.0.0.1:{}", ws_port);
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    info!("✨ WebArcade Bridge is ready!");
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("\n🟢 SERVER READY - You can now use the application!\n");

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);
        let router_registry = router_registry.clone_registry();

        tokio::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(io, service_fn(move |req| {
                    let router = router_registry.clone_registry();
                    async move {
                        Ok::<_, std::convert::Infallible>(handle_request(req, router).await)
                    }
                }))
                .await
            {
                error!("Error serving connection: {:?}", err);
            }
        });
    }
}

async fn handle_request(req: Request<Incoming>, router_registry: RouterRegistry) -> Response<BoxBody<Bytes, Infallible>> {
    let method = req.method().clone();
    let path = req.uri().path().to_string();
    let query = req.uri().query().unwrap_or("").to_string();

    // Health check endpoint
    if path == "/health" || path == "/api/health" {
        return health_response();
    }

    // Runtime plugin API endpoints
    if path == "/api/plugins/list" {
        return modules::system_api::handle_list_plugins();
    }
    if path.starts_with("/api/plugins/") && path.len() > 13 {
        let parts: Vec<&str> = path[13..].split('/').collect();
        if parts.len() >= 2 {
            let plugin_id = parts[0];
            let file_path = parts[1..].join("/");
            return modules::system_api::handle_serve_plugin_file(plugin_id, &file_path);
        }
    }

    // Parse plugin name from path
    // Expected format: /{plugin_name}/{route}
    if path.len() > 1 {
        let path_parts: Vec<&str> = path.trim_start_matches('/').split('/').collect();
        if !path_parts.is_empty() {
            let plugin_name = path_parts[0];
            let plugin_path = if path_parts.len() > 1 {
                format!("/{}", path_parts[1..].join("/"))
            } else {
                "/".to_string()
            };

            // Try to route to plugin
            if let Some(response) = router_registry.route(
                plugin_name,
                &method,
                &plugin_path,
                &query,
                req,
            ).await {
                return response;
            } else {
                return error_response(StatusCode::NOT_FOUND, &format!("Route not found: {}", path));
            }
        }
    }

    error_response(StatusCode::NOT_FOUND, "Invalid path")
}

fn health_response() -> Response<BoxBody<Bytes, Infallible>> {
    let json = serde_json::json!({"status": "ok", "message": "WebArcade Bridge is ready"}).to_string();
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