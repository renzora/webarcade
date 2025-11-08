#![allow(dead_code)]
#![allow(unused_variables)]

pub mod modules;
pub mod core;
pub mod plugins;

use std::env;
use std::fs;
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

use crate::bridge::core::{EventBus, ServiceRegistry, PluginManager, WebSocketBridge, RouterRegistry};

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

    // Load plugin configuration
    let plugin_config = load_plugin_config()?;

    // Create router registry
    let router_registry = RouterRegistry::new();

    // Create plugin manager
    let mut plugin_manager = PluginManager::new(
        event_bus.clone(),
        service_registry.clone(),
        router_registry.clone_registry(),
        plugin_config,
        db_path.to_string_lossy().to_string(),
    );

    // Register all plugins
    info!("📦 Registering plugins...");
    crate::bridge::plugins::register_all_plugins(&mut plugin_manager);

    // Initialize all plugins
    info!("🔧 Initializing plugins...");
    plugin_manager.init_all().await?;

    // Start all plugins
    info!("🚀 Starting plugins...");
    plugin_manager.start_all().await?;

    // List loaded plugins
    let loaded_plugins = plugin_manager.list_plugins();
    info!("✅ Loaded {} plugins:", loaded_plugins.len());
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

pub fn load_plugin_config() -> Result<std::collections::HashMap<String, serde_json::Value>> {
    let config_path = std::path::Path::new("plugins.json");

    if !config_path.exists() {
        // Create default config with all plugins enabled
        let default_config = serde_json::json!({
            "plugins": [
                {"id": "currency", "enabled": true, "config": {"starting_balance": 1000}},
                {"id": "notes", "enabled": true, "config": {}},
                {"id": "goals", "enabled": true, "config": {}},
                {"id": "todos", "enabled": true, "config": {}},
                {"id": "auction", "enabled": true, "config": {}},
                {"id": "roulette", "enabled": true, "config": {}},
                {"id": "levels", "enabled": true, "config": {"xp_per_message": 5}},
                {"id": "wheel", "enabled": true, "config": {}},
                {"id": "packs", "enabled": true, "config": {}},
                {"id": "files", "enabled": true, "config": {}},
                {"id": "system", "enabled": true, "config": {}},
                {"id": "ticker", "enabled": true, "config": {}},
                {"id": "text_commands", "enabled": true, "config": {}},
                {"id": "user_profiles", "enabled": true, "config": {}},
                {"id": "tts", "enabled": true, "config": {}},
                {"id": "confessions", "enabled": true, "config": {}},
                {"id": "household", "enabled": true, "config": {}},
                {"id": "twitch", "enabled": true, "config": {}},
                {"id": "hue", "enabled": false, "config": {}},
                {"id": "withings", "enabled": false, "config": {}}
            ]
        });

        fs::write(config_path, serde_json::to_string_pretty(&default_config)?)?;
        info!("📝 Created default plugins.json");
    }

    let config_content = fs::read_to_string(config_path)?;
    let config: serde_json::Value = serde_json::from_str(&config_content)?;

    let mut plugin_configs = std::collections::HashMap::new();

    if let Some(plugins) = config["plugins"].as_array() {
        for plugin in plugins {
            if let (Some(id), Some(enabled)) = (
                plugin["id"].as_str(),
                plugin["enabled"].as_bool(),
            ) {
                if enabled {
                    let plugin_config = plugin["config"].clone();
                    plugin_configs.insert(id.to_string(), plugin_config);
                }
            }
        }
    }

    Ok(plugin_configs)
}