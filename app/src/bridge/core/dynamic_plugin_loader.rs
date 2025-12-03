use libloading::Library;
use std::path::{Path, PathBuf};
use std::fs;
use std::sync::Arc;
use anyhow::{Result, anyhow};

// Include embedded plugins when feature is enabled
#[cfg(feature = "locked-plugins")]
mod embedded {
    include!(concat!(env!("OUT_DIR"), "/embedded_plugins.rs"));
}

pub struct DynamicPluginLoader {
    plugins_dir: PathBuf,
}

impl DynamicPluginLoader {
    pub fn new(plugins_dir: PathBuf) -> Self {
        Self { plugins_dir }
    }

    /// Discover and load all plugins
    ///
    /// When `locked-plugins` feature is enabled: loads from embedded binary data
    /// Otherwise: loads from plugins directory on disk
    pub fn load_all_plugins(&mut self) -> Result<Vec<PluginInfo>> {
        #[cfg(feature = "locked-plugins")]
        {
            self.load_embedded_plugins()
        }

        #[cfg(not(feature = "locked-plugins"))]
        {
            self.load_external_plugins()
        }
    }

    /// Load plugins embedded in the binary (locked mode)
    #[cfg(feature = "locked-plugins")]
    fn load_embedded_plugins(&mut self) -> Result<Vec<PluginInfo>> {
        log::info!("🔒 Loading embedded plugins (locked mode)");

        let mut plugins = Vec::new();

        for plugin in embedded::EMBEDDED_PLUGINS {
            log::info!("📦 Loading embedded plugin: {}", plugin.id);

            if plugin.is_dll {
                // For DLLs, we need to write to a temp file and load it
                // (libloading requires a file path)
                match self.load_embedded_dll(plugin.id, plugin.data) {
                    Ok(plugin_info) => plugins.push(plugin_info),
                    Err(e) => log::warn!("⚠️  Failed to load embedded plugin {}: {}", plugin.id, e),
                }
            } else {
                // JS plugin - store the data directly
                let js_content = String::from_utf8_lossy(plugin.data).to_string();
                crate::bridge::core::plugin_exports::register_embedded_js(plugin.id.to_string(), js_content);

                plugins.push(PluginInfo {
                    id: plugin.id.to_string(),
                    dll_path: PathBuf::new(),
                    has_backend: false,
                    has_frontend: true,
                    routes: vec![],
                    frontend_path: None,
                    embedded_js: Some(plugin.id.to_string()),
                });
                log::info!("✅ Loaded embedded JS plugin: {}", plugin.id);
            }
        }

        log::info!("📦 Successfully loaded {} embedded plugins", plugins.len());
        Ok(plugins)
    }

    /// Load a DLL from embedded bytes
    #[cfg(feature = "locked-plugins")]
    fn load_embedded_dll(&mut self, plugin_id: &str, data: &[u8]) -> Result<PluginInfo> {
        // Write DLL to temp directory
        let temp_dir = std::env::temp_dir().join("emils_plugins");
        fs::create_dir_all(&temp_dir)?;

        #[cfg(target_os = "windows")]
        let dll_name = format!("{}.dll", plugin_id);
        #[cfg(target_os = "linux")]
        let dll_name = format!("lib{}.so", plugin_id);
        #[cfg(target_os = "macos")]
        let dll_name = format!("lib{}.dylib", plugin_id);

        let dll_path = temp_dir.join(&dll_name);
        fs::write(&dll_path, data)?;

        self.load_plugin_from_dll(&dll_path)
    }

    /// Load plugins from external directory (unlocked mode)
    #[cfg(not(feature = "locked-plugins"))]
    fn load_external_plugins(&mut self) -> Result<Vec<PluginInfo>> {
        log::info!("🔓 Scanning for external plugins in: {:?}", self.plugins_dir);

        if !self.plugins_dir.exists() {
            fs::create_dir_all(&self.plugins_dir)?;
            return Ok(Vec::new());
        }

        let entries = fs::read_dir(&self.plugins_dir)?;
        let mut plugins = Vec::new();
        let mut loaded_ids = std::collections::HashSet::new();

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                if let Some(ext) = path.extension() {
                    let ext_str = ext.to_string_lossy().to_lowercase();

                    // Get plugin ID from filename
                    let stem = path.file_stem()
                        .map(|s| s.to_string_lossy().to_string())
                        .unwrap_or_default();

                    // Remove "lib" prefix on Linux/macOS
                    let plugin_id = stem.strip_prefix("lib").unwrap_or(&stem).to_string();

                    // Skip if we already loaded this plugin (DLL takes precedence over JS)
                    if loaded_ids.contains(&plugin_id) {
                        continue;
                    }

                    if ext_str == "dll" || ext_str == "so" || ext_str == "dylib" {
                        // DLL plugin (has backend)
                        match self.load_plugin_from_dll(&path) {
                            Ok(plugin_info) => {
                                loaded_ids.insert(plugin_info.id.clone());
                                plugins.push(plugin_info);
                            }
                            Err(e) => {
                                log::warn!("⚠️  Failed to load plugin from {:?}: {}", path, e);
                            }
                        }
                    } else if ext_str == "js" {
                        // Frontend-only JS plugin
                        log::info!("📦 Loading frontend-only plugin: {} from {:?}", plugin_id, path);
                        loaded_ids.insert(plugin_id.clone());
                        plugins.push(PluginInfo {
                            id: plugin_id.clone(),
                            dll_path: PathBuf::new(), // No DLL
                            has_backend: false,
                            has_frontend: true,
                            routes: vec![],
                            frontend_path: Some(path.clone()),
                            #[cfg(feature = "locked-plugins")]
                            embedded_js: None,
                        });
                        log::info!("✅ Loaded frontend-only plugin: {}", plugin_id);
                    }
                }
            }
        }

        log::info!("📦 Successfully loaded {} plugins", plugins.len());
        Ok(plugins)
    }

    fn find_dll_in_dir(&self, dir: &Path, plugin_id: &str) -> Option<PathBuf> {
        // Look for platform-specific library
        #[cfg(target_os = "windows")]
        let lib_name = format!("{}.dll", plugin_id);

        #[cfg(target_os = "linux")]
        let lib_name = format!("lib{}.so", plugin_id);

        #[cfg(target_os = "macos")]
        let lib_name = format!("lib{}.dylib", plugin_id);

        let dll_path = dir.join(&lib_name);
        if dll_path.exists() {
            return Some(dll_path);
        }

        // Fallback: search for any DLL in the directory
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(ext) = path.extension() {
                    if ext == "dll" || ext == "so" || ext == "dylib" {
                        return Some(path);
                    }
                }
            }
        }

        None
    }

    fn load_plugin_from_dll(&mut self, dll_path: &Path) -> Result<PluginInfo> {
        // Extract plugin ID from filename
        let stem = dll_path.file_stem()
            .ok_or_else(|| anyhow!("Invalid DLL filename"))?
            .to_string_lossy();

        // Remove "lib" prefix on Linux/macOS
        let plugin_id = stem.strip_prefix("lib").unwrap_or(&stem).to_string();

        log::info!("📦 Loading plugin DLL: {} from {:?}", plugin_id, dll_path);

        // Load the library
        let lib = unsafe { Library::new(dll_path)? };
        let lib_arc = Arc::new(lib);

        // Get manifest from the DLL
        let manifest = self.get_manifest_from_dll(&lib_arc)?;
        let webarcade_config = manifest.get("webarcade")
            .ok_or_else(|| anyhow!("Manifest missing 'webarcade' section"))?;

        let routes = webarcade_config.get("routes")
            .and_then(|r| r.as_array())
            .cloned()
            .unwrap_or_default();

        // Check if plugin has frontend
        let has_frontend = self.check_has_frontend(&lib_arc);

        // Plugin has backend if it has any routes
        let has_backend = !routes.is_empty();

        // Register the plugin library for FFI calls (needed for both backend handlers and frontend extraction)
        crate::bridge::core::plugin_exports::register_plugin_library(plugin_id.clone(), lib_arc);

        let plugin_type = if has_backend { "full-stack" } else { "frontend-only" };
        log::info!("✅ Loaded {} plugin: {} ({} routes, frontend: {})", plugin_type, plugin_id, routes.len(), has_frontend);

        Ok(PluginInfo {
            id: plugin_id,
            dll_path: dll_path.to_path_buf(),
            has_backend,
            has_frontend,
            routes,
            frontend_path: None, // Frontend is embedded in DLL
            #[cfg(feature = "locked-plugins")]
            embedded_js: None,
        })
    }

    fn get_manifest_from_dll(&self, lib: &Arc<Library>) -> Result<serde_json::Value> {
        type GetManifestFn = unsafe extern "C" fn() -> *const u8;
        type GetManifestLenFn = unsafe extern "C" fn() -> usize;

        unsafe {
            let get_manifest: libloading::Symbol<GetManifestFn> = lib.get(b"get_plugin_manifest")?;
            let get_manifest_len: libloading::Symbol<GetManifestLenFn> = lib.get(b"get_plugin_manifest_len")?;

            let ptr = get_manifest();
            let len = get_manifest_len();

            if ptr.is_null() || len == 0 {
                return Err(anyhow!("Plugin returned null/empty manifest"));
            }

            let slice = std::slice::from_raw_parts(ptr, len);
            let manifest_str = std::str::from_utf8(slice)?;
            let manifest: serde_json::Value = serde_json::from_str(manifest_str)?;

            Ok(manifest)
        }
    }

    fn check_has_frontend(&self, lib: &Arc<Library>) -> bool {
        type HasFrontendFn = unsafe extern "C" fn() -> bool;

        unsafe {
            if let Ok(has_frontend) = lib.get::<HasFrontendFn>(b"has_frontend") {
                has_frontend()
            } else {
                false
            }
        }
    }

    /// Get frontend JavaScript from a loaded plugin
    pub fn get_frontend_js(plugin_id: &str) -> Result<String> {
        type GetFrontendFn = unsafe extern "C" fn() -> *const u8;
        type GetFrontendLenFn = unsafe extern "C" fn() -> usize;

        let lib = crate::bridge::core::plugin_exports::get_plugin_library(plugin_id)
            .ok_or_else(|| anyhow!("Plugin not loaded: {}", plugin_id))?;

        unsafe {
            let get_frontend: libloading::Symbol<GetFrontendFn> = lib.get(b"get_plugin_frontend")?;
            let get_frontend_len: libloading::Symbol<GetFrontendLenFn> = lib.get(b"get_plugin_frontend_len")?;

            let ptr = get_frontend();
            let len = get_frontend_len();

            if ptr.is_null() || len == 0 {
                return Err(anyhow!("Plugin has no frontend"));
            }

            let slice = std::slice::from_raw_parts(ptr, len);
            let frontend_str = std::str::from_utf8(slice)?;

            Ok(frontend_str.to_string())
        }
    }
}

/// Information about a loaded plugin
#[derive(Debug, Clone)]
pub struct PluginInfo {
    pub id: String,
    pub dll_path: PathBuf,
    pub has_backend: bool,
    pub has_frontend: bool,
    pub routes: Vec<serde_json::Value>,
    /// Path to plugin.js for frontend-only plugins (no DLL)
    pub frontend_path: Option<PathBuf>,
    /// Key for embedded JS content (locked-plugins mode)
    #[cfg(feature = "locked-plugins")]
    pub embedded_js: Option<String>,
}
