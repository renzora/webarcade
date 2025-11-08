use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use anyhow::{Result, anyhow};
use crate::core::plugin::{Plugin, PluginMetadata};
use crate::core::plugin_context::PluginContext;
use crate::core::events::EventBus;
use crate::core::services::ServiceRegistry;
use crate::core::plugin_router::RouterRegistry;

pub struct PluginManager {
    plugins: HashMap<String, Box<dyn Plugin>>,
    contexts: HashMap<String, Arc<PluginContext>>,
    event_bus: Arc<EventBus>,
    service_registry: Arc<ServiceRegistry>,
    router_registry: RouterRegistry,
    config: HashMap<String, serde_json::Value>,
    db_path: String,
}

impl PluginManager {
    pub fn new(
        event_bus: Arc<EventBus>,
        service_registry: Arc<ServiceRegistry>,
        router_registry: RouterRegistry,
        config: HashMap<String, serde_json::Value>,
        db_path: String,
    ) -> Self {
        Self {
            plugins: HashMap::new(),
            contexts: HashMap::new(),
            event_bus,
            service_registry,
            router_registry,
            config,
            db_path,
        }
    }

    /// Register a plugin
    pub fn register<P: Plugin + 'static>(&mut self, plugin: P) {
        let metadata = plugin.metadata();
        let plugin_id = metadata.id.clone();

        log::info!("[PluginManager] Registering plugin: {} v{}", metadata.name, metadata.version);

        // Create plugin context
        let ctx = Arc::new(PluginContext::new(
            plugin_id.clone(),
            self.event_bus.clone(),
            self.service_registry.clone(),
            self.router_registry.clone_registry(),
            self.config.get(&plugin_id).cloned().unwrap_or(serde_json::json!({})),
            self.db_path.clone(),
        ));

        self.contexts.insert(plugin_id.clone(), ctx);
        self.plugins.insert(plugin_id, Box::new(plugin));
    }

    /// Initialize all plugins (respecting dependencies)
    pub async fn init_all(&self) -> Result<()> {
        let load_order = self.resolve_dependencies()?;

        for plugin_id in &load_order {
            if let (Some(plugin), Some(ctx)) = (
                self.plugins.get(plugin_id),
                self.contexts.get(plugin_id)
            ) {
                let metadata = plugin.metadata();
                log::info!("[PluginManager] Initializing: {} v{}", metadata.name, metadata.version);

                plugin.init(ctx).await.map_err(|e| {
                    anyhow!("Failed to initialize plugin '{}': {}", plugin_id, e)
                })?;
            }
        }

        Ok(())
    }

    /// Start all plugins
    pub async fn start_all(&self) -> Result<()> {
        let load_order = self.resolve_dependencies()?;

        for plugin_id in &load_order {
            if let (Some(plugin), Some(ctx)) = (
                self.plugins.get(plugin_id),
                self.contexts.get(plugin_id)
            ) {
                let metadata = plugin.metadata();
                log::info!("[PluginManager] Starting: {}", metadata.name);

                plugin.start(ctx.clone()).await.map_err(|e| {
                    anyhow!("Failed to start plugin '{}': {}", plugin_id, e)
                })?;
            }
        }

        log::info!("[PluginManager] All plugins started successfully");
        Ok(())
    }

    /// Stop all plugins
    pub async fn stop_all(&self) -> Result<()> {
        // Stop in reverse order
        let load_order = self.resolve_dependencies()?;

        for plugin_id in load_order.iter().rev() {
            if let Some(plugin) = self.plugins.get(plugin_id) {
                let metadata = plugin.metadata();
                log::info!("[PluginManager] Stopping: {}", metadata.name);

                if let Err(e) = plugin.stop().await {
                    log::error!("Failed to stop plugin '{}': {}", plugin_id, e);
                }
            }
        }

        Ok(())
    }

    /// Get plugin context by ID
    pub fn get_context(&self, plugin_id: &str) -> Option<Arc<PluginContext>> {
        self.contexts.get(plugin_id).cloned()
    }

    /// List all registered plugins
    pub fn list_plugins(&self) -> Vec<PluginMetadata> {
        self.plugins.values()
            .map(|p| p.metadata())
            .collect()
    }

    /// Resolve plugin dependencies and return load order
    fn resolve_dependencies(&self) -> Result<Vec<String>> {
        let mut order = Vec::new();
        let mut visited = HashSet::new();
        let mut visiting = HashSet::new();

        for plugin_id in self.plugins.keys() {
            if !visited.contains(plugin_id) {
                self.visit_plugin(plugin_id, &mut order, &mut visited, &mut visiting)?;
            }
        }

        Ok(order)
    }

    /// Depth-first search for topological sort
    fn visit_plugin(
        &self,
        plugin_id: &str,
        order: &mut Vec<String>,
        visited: &mut HashSet<String>,
        visiting: &mut HashSet<String>,
    ) -> Result<()> {
        if visited.contains(plugin_id) {
            return Ok(());
        }

        if visiting.contains(plugin_id) {
            return Err(anyhow!("Circular dependency detected involving plugin: {}", plugin_id));
        }

        visiting.insert(plugin_id.to_string());

        if let Some(plugin) = self.plugins.get(plugin_id) {
            let metadata = plugin.metadata();

            // Visit dependencies first
            for dep in &metadata.dependencies {
                if !self.plugins.contains_key(dep) {
                    return Err(anyhow!(
                        "Plugin '{}' depends on '{}', but it is not registered",
                        plugin_id, dep
                    ));
                }
                self.visit_plugin(dep, order, visited, visiting)?;
            }
        }

        visiting.remove(plugin_id);
        visited.insert(plugin_id.to_string());
        order.push(plugin_id.to_string());

        Ok(())
    }
}
