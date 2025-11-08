use std::sync::Arc;
use std::future::Future;
use rusqlite::Connection;
use anyhow::Result;
use serde::Serialize;
use serde_json::Value;
use tokio::sync::broadcast;
use crate::core::events::{Event, EventBus};
use crate::core::services::ServiceRegistry;
use crate::core::plugin_router::{PluginRouter, RouterRegistry};

/// Plugin context - API provided to plugins
#[derive(Clone)]
pub struct PluginContext {
    plugin_id: String,
    event_bus: Arc<EventBus>,
    service_registry: Arc<ServiceRegistry>,
    router_registry: RouterRegistry,
    config: Value,
    db_path: String,
}

impl PluginContext {
    pub fn new(
        plugin_id: String,
        event_bus: Arc<EventBus>,
        service_registry: Arc<ServiceRegistry>,
        router_registry: RouterRegistry,
        config: Value,
        db_path: String,
    ) -> Self {
        Self {
            plugin_id,
            event_bus,
            service_registry,
            router_registry,
            config,
            db_path,
        }
    }

    /// Get plugin ID
    pub fn plugin_id(&self) -> &str {
        &self.plugin_id
    }

    /// Get plugin configuration
    pub fn config(&self) -> &Value {
        &self.config
    }

    // ==================== Database ====================

    /// Get database connection
    pub fn db(&self) -> Result<Connection> {
        Ok(Connection::open(&self.db_path)?)
    }

    /// Run database migrations for this plugin
    pub fn migrate(&self, migrations: &[&str]) -> Result<()> {
        let conn = self.db()?;

        // Create migrations table if it doesn't exist
        conn.execute(
            "CREATE TABLE IF NOT EXISTS plugin_migrations (
                plugin_id TEXT NOT NULL,
                version INTEGER NOT NULL,
                executed_at INTEGER NOT NULL,
                PRIMARY KEY (plugin_id, version)
            )",
            [],
        )?;

        // Get current version
        let current_version: i32 = conn.query_row(
            "SELECT COALESCE(MAX(version), 0) FROM plugin_migrations WHERE plugin_id = ?1",
            [&self.plugin_id],
            |row| row.get(0),
        ).unwrap_or(0);

        // Execute pending migrations
        for (i, sql) in migrations.iter().enumerate() {
            let version = (i + 1) as i32;
            if version > current_version {
                log::info!("[{}] Running migration {}", self.plugin_id, version);
                conn.execute_batch(sql)?;
                conn.execute(
                    "INSERT INTO plugin_migrations (plugin_id, version, executed_at) VALUES (?1, ?2, ?3)",
                    rusqlite::params![&self.plugin_id, version, current_timestamp()],
                )?;
            }
        }

        Ok(())
    }

    // ==================== Events ====================

    /// Publish event
    pub fn emit<T: Serialize>(&self, event_type: &str, payload: &T) {
        self.event_bus.publish_typed(&self.plugin_id, event_type, payload);
    }

    /// Subscribe to specific event type
    pub async fn subscribe_to(&self, event_type: &str) -> broadcast::Receiver<Event> {
        self.event_bus.subscribe_to(event_type).await
    }

    /// Subscribe to all events
    pub fn subscribe_all(&self) -> broadcast::Receiver<Event> {
        self.event_bus.subscribe()
    }

    // ==================== Services ====================

    /// Register a service method that other plugins can call
    pub async fn provide_service<F, Fut>(&self, method_name: &str, handler: F)
    where
        F: Fn(Value) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<Value>> + Send + 'static,
    {
        let service_id = format!("{}.{}", self.plugin_id, method_name);
        self.service_registry.register(&service_id, handler).await;
    }

    /// Call another plugin's service
    pub async fn call_service(&self, plugin_id: &str, method: &str, input: Value) -> Result<Value> {
        let service_id = format!("{}.{}", plugin_id, method);
        self.service_registry.call(&service_id, input).await
    }

    /// Check if a service exists
    pub async fn has_service(&self, plugin_id: &str, method: &str) -> bool {
        let service_id = format!("{}.{}", plugin_id, method);
        self.service_registry.has_service(&service_id).await
    }

    /// List all available services
    pub async fn list_services(&self) -> Vec<String> {
        self.service_registry.list_services().await
    }

    // ==================== Routing ====================

    /// Register HTTP routes for this plugin
    pub async fn register_router(&self, plugin_name: &str, router: PluginRouter) {
        self.router_registry.register(plugin_name.to_string(), router).await;
    }

    /// Get the router registry
    pub fn router_registry(&self) -> &RouterRegistry {
        &self.router_registry
    }
}

fn current_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}
