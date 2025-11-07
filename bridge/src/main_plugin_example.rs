// Example of how main.rs would look with the plugin system
// This is a demonstration - you can gradually migrate to this approach

use std::env;
use std::collections::HashMap;
use std::sync::Arc;
use log::{info, error};
use anyhow::Result;

use webarcade_bridge::core::{EventBus, ServiceRegistry, PluginManager, RouterRegistry};
use webarcade_bridge::plugins;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logger
    env_logger::Builder::from_default_env()
        .format_timestamp_secs()
        .init();

    info!("🎮 WebArcade Bridge - Plugin System");

    // Get configuration
    let port = env::var("BRIDGE_PORT").unwrap_or_else(|_| "3001".to_string());
    let ws_port = env::var("WS_PORT").unwrap_or_else(|_| "3002".to_string());

    // Initialize core systems
    info!("📦 Initializing core systems...");

    let event_bus = Arc::new(EventBus::new());
    let service_registry = Arc::new(ServiceRegistry::new());
    let router_registry = RouterRegistry::new();

    // Get database path
    let db_path = webarcade_bridge::core::database::get_database_path();
    webarcade_bridge::core::database::ensure_database_dir()?;

    info!("💾 Database: {}", db_path.display());

    // Load plugin configuration
    let plugin_config = load_plugin_config()?;

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
    plugins::register_all_plugins(&mut plugin_manager);

    // Initialize all plugins
    info!("🔧 Initializing plugins...");
    plugin_manager.init_all().await?;

    // Start all plugins
    info!("🚀 Starting plugins...");
    plugin_manager.start_all().await?;

    // List loaded plugins
    let loaded_plugins = plugin_manager.list_plugins();
    info!("✅ Loaded {} plugins:", loaded_plugins.len());
    for plugin in loaded_plugins {
        info!("   - {} v{}: {}", plugin.name, plugin.version, plugin.description);
    }

    // Start WebSocket server (for real-time events)
    let event_bus_clone = event_bus.clone();
    tokio::spawn(async move {
        if let Err(e) = start_websocket_server(ws_port, event_bus_clone).await {
            error!("WebSocket server error: {}", e);
        }
    });

    // Start HTTP server
    let addr = format!("127.0.0.1:{}", port);
    info!("🌐 HTTP server listening on http://{}", addr);

    // In a real implementation, you would:
    // 1. Create a router that collects routes from all plugins
    // 2. Start the HTTP server with that router
    // For now, this is a demonstration

    // Keep server running
    tokio::signal::ctrl_c().await?;

    // Shutdown
    info!("🛑 Shutting down...");
    plugin_manager.stop_all().await?;

    info!("👋 Goodbye!");
    Ok(())
}

// Load plugin configuration from JSON file
fn load_plugin_config() -> Result<HashMap<String, serde_json::Value>> {
    let config_path = std::path::PathBuf::from("bridge/plugins.json");

    if config_path.exists() {
        let content = std::fs::read_to_string(config_path)?;
        let config: serde_json::Value = serde_json::from_str(&content)?;

        let mut plugin_configs = HashMap::new();

        if let Some(plugins) = config["plugins"].as_array() {
            for plugin in plugins {
                if let (Some(id), Some(enabled)) = (
                    plugin["id"].as_str(),
                    plugin["enabled"].as_bool(),
                ) {
                    if enabled {
                        if let Some(plugin_config) = plugin["config"].as_object() {
                            plugin_configs.insert(
                                id.to_string(),
                                serde_json::Value::Object(plugin_config.clone()),
                            );
                        } else {
                            plugin_configs.insert(id.to_string(), serde_json::json!({}));
                        }
                    }
                }
            }
        }

        Ok(plugin_configs)
    } else {
        info!("No plugins.json found, using default configuration");
        Ok(HashMap::new())
    }
}

// WebSocket server that broadcasts events from the event bus
async fn start_websocket_server(
    port: String,
    event_bus: Arc<EventBus>,
) -> Result<()> {
    use tokio::net::TcpListener;
    use tokio_tungstenite::accept_async;
    use futures_util::StreamExt;

    let addr = format!("127.0.0.1:{}", port);
    let listener = TcpListener::bind(&addr).await?;
    info!("🔌 WebSocket server listening on ws://{}", addr);

    while let Ok((stream, _)) = listener.accept().await {
        let event_bus = event_bus.clone();

        tokio::spawn(async move {
            if let Ok(ws_stream) = accept_async(stream).await {
                info!("WebSocket client connected");

                let (_write, _read) = ws_stream.split();

                // Subscribe to events
                let mut events = event_bus.subscribe();

                while let Ok(event) = events.recv().await {
                    // In a real implementation, send event to WebSocket client
                    info!("Broadcasting event: {}.{}", event.source_plugin, event.event_type);
                }
            }
        });
    }

    Ok(())
}
