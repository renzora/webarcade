use libloading::{Library, Symbol};
use std::path::{Path, PathBuf};
use std::fs;
use anyhow::{Result, anyhow};

pub struct DynamicPluginLoader {
    plugins_dir: PathBuf,
    loaded_libraries: Vec<(Library, PluginInfo)>,
}

impl DynamicPluginLoader {
    pub fn new(plugins_dir: PathBuf) -> Self {
        Self {
            plugins_dir,
            loaded_libraries: Vec::new(),
        }
    }

    /// Discover and load all plugins from the plugins directory
    pub fn load_all_plugins(&mut self) -> Result<Vec<PluginInfo>> {
        log::info!("🔍 Scanning for dynamic plugins in: {:?}", self.plugins_dir);

        if !self.plugins_dir.exists() {
            fs::create_dir_all(&self.plugins_dir)?;
            log::info!("📁 Created plugins directory");
            return Ok(Vec::new());
        }

        let entries = fs::read_dir(&self.plugins_dir)?;
        let mut plugins = Vec::new();

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            // Look for plugin directories
            if path.is_dir() {
                match self.load_plugin_from_dir(&path) {
                    Ok(plugin_info) => {
                        plugins.push(plugin_info);
                        log::info!("✅ Loaded dynamic plugin: {:?}", path.file_name().unwrap());
                    }
                    Err(e) => {
                        log::warn!("⚠️  Failed to load plugin from {:?}: {}", path, e);
                    }
                }
            }
        }

        log::info!("📦 Successfully loaded {} dynamic plugins", plugins.len());
        Ok(plugins)
    }

    fn load_plugin_from_dir(&mut self, plugin_dir: &Path) -> Result<PluginInfo> {
        // Read manifest.json
        let metadata_path = plugin_dir.join("manifest.json");
        if !metadata_path.exists() {
            return Err(anyhow!("No manifest.json found"));
        }

        let metadata_str = fs::read_to_string(metadata_path)?;
        let metadata: serde_json::Value = serde_json::from_str(&metadata_str)?;

        let plugin_id = metadata["id"].as_str()
            .ok_or_else(|| anyhow!("Missing plugin id in metadata"))?
            .to_string();

        let has_backend = metadata["has_backend"].as_bool().unwrap_or(false);

        // Read routes configuration if it exists
        let routes_path = plugin_dir.join("routes.json");
        let routes = if routes_path.exists() {
            let routes_str = fs::read_to_string(routes_path)?;
            let routes_config: serde_json::Value = serde_json::from_str(&routes_str)?;
            routes_config["routes"].as_array().cloned().unwrap_or_default()
        } else {
            Vec::new()
        };

        // Only load backend if it exists
        if has_backend {
            let dll_path = self.find_platform_binary(&plugin_dir, &plugin_id)?;

            log::info!("📚 Loading DLL: {:?}", dll_path);

            // Load the DLL
            // SAFETY: Loading a plugin DLL. Plugins are trusted code.
            let lib = unsafe { Library::new(&dll_path)? };

            let plugin_info = PluginInfo {
                id: plugin_id.clone(),
                dir: plugin_dir.to_path_buf(),
                has_backend,
                has_frontend: metadata["has_frontend"].as_bool().unwrap_or(false),
                routes: routes.clone(),
            };

            // Register the library for handler lookups
            let lib_arc = std::sync::Arc::new(lib);
            crate::bridge::core::plugin_exports::register_plugin_library(plugin_id.clone(), lib_arc);

            // Load a fresh copy for storage (libraries can be loaded multiple times)
            let lib_storage = unsafe { Library::new(&dll_path)? };
            self.loaded_libraries.push((lib_storage, plugin_info.clone()));

            log::info!("✅ DLL loaded successfully: {} ({} routes)", plugin_id, routes.len());

            Ok(plugin_info)
        } else {
            Ok(PluginInfo {
                id: plugin_id,
                dir: plugin_dir.to_path_buf(),
                has_backend,
                has_frontend: metadata["has_frontend"].as_bool().unwrap_or(false),
                routes,
            })
        }
    }

    fn find_platform_binary(&self, plugin_dir: &Path, plugin_id: &str) -> Result<PathBuf> {
        // Look for platform-specific binary in plugin root directory
        #[cfg(target_os = "windows")]
        let binary_path = plugin_dir.join(format!("{}.dll", plugin_id));

        #[cfg(target_os = "linux")]
        let binary_path = plugin_dir.join(format!("lib{}.so", plugin_id));

        #[cfg(target_os = "macos")]
        let binary_path = plugin_dir.join(format!("lib{}.dylib", plugin_id));

        if binary_path.exists() {
            Ok(binary_path)
        } else {
            Err(anyhow!("Platform binary not found: {:?}", binary_path))
        }
    }

    /// Get the list of loaded libraries
    pub fn loaded_count(&self) -> usize {
        self.loaded_libraries.len()
    }

    /// Get information about all loaded plugins
    pub fn get_loaded_plugins(&self) -> Vec<PluginInfo> {
        self.loaded_libraries.iter().map(|(_, info)| info.clone()).collect()
    }

    /// Initialize all dynamic plugins by calling their plugin_init function
    pub fn init_dynamic_plugins(&self, ctx: *const ()) -> Result<()> {
        for (lib, info) in &self.loaded_libraries {
            log::info!("🔧 Initializing dynamic plugin: {}", info.id);

            unsafe {
                let init_fn: Symbol<extern "C" fn(*const ()) -> i32> =
                    lib.get(b"plugin_init\0")
                        .map_err(|e| anyhow!("Failed to load plugin_init for {}: {}", info.id, e))?;

                let result = init_fn(ctx);
                if result != 0 {
                    return Err(anyhow!("Plugin init failed for {}: error code {}", info.id, result));
                }
            }

            log::info!("✅ Initialized dynamic plugin: {}", info.id);
        }
        Ok(())
    }

    /// Start all dynamic plugins by calling their plugin_start function
    pub fn start_dynamic_plugins(&self, ctx: *const ()) -> Result<()> {
        for (lib, info) in &self.loaded_libraries {
            log::info!("🚀 Starting dynamic plugin: {}", info.id);

            unsafe {
                let start_fn: Symbol<extern "C" fn(*const ()) -> i32> =
                    lib.get(b"plugin_start\0")
                        .map_err(|e| anyhow!("Failed to load plugin_start for {}: {}", info.id, e))?;

                let result = start_fn(ctx);
                if result != 0 {
                    return Err(anyhow!("Plugin start failed for {}: error code {}", info.id, result));
                }
            }

            log::info!("✅ Started dynamic plugin: {}", info.id);
        }
        Ok(())
    }

    /// Stop all dynamic plugins by calling their plugin_stop function
    pub fn stop_dynamic_plugins(&self) -> Result<()> {
        for (lib, info) in &self.loaded_libraries {
            log::info!("🛑 Stopping dynamic plugin: {}", info.id);

            unsafe {
                let stop_fn: Symbol<extern "C" fn() -> i32> =
                    lib.get(b"plugin_stop\0")
                        .map_err(|e| anyhow!("Failed to load plugin_stop for {}: {}", info.id, e))?;

                let result = stop_fn();
                if result != 0 {
                    log::warn!("⚠️  Plugin stop returned error for {}: code {}", info.id, result);
                }
            }

            log::info!("✅ Stopped dynamic plugin: {}", info.id);
        }
        Ok(())
    }
}

/// Information about a loaded plugin
#[derive(Debug, Clone)]
pub struct PluginInfo {
    pub id: String,
    pub dir: PathBuf,
    pub has_backend: bool,
    pub has_frontend: bool,
    pub routes: Vec<serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_loader_creation() {
        let temp_dir = env::temp_dir().join("webarcade_test_plugins");
        let loader = DynamicPluginLoader::new(temp_dir);
        assert_eq!(loader.loaded_count(), 0);
    }

    #[test]
    fn test_empty_directory() {
        let temp_dir = env::temp_dir().join("webarcade_test_plugins_empty");
        let _ = fs::remove_dir_all(&temp_dir);
        let mut loader = DynamicPluginLoader::new(temp_dir.clone());

        let result = loader.load_all_plugins();
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
