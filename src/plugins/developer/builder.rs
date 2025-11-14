use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use zip::write::FileOptions;
use zip::ZipWriter;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildProgress {
    pub step: String,
    pub progress: f32,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildResult {
    pub success: bool,
    pub output_path: Option<String>,
    pub error: Option<String>,
}

pub struct PluginBuilder {
    plugin_dir: PathBuf,
    plugin_id: String,
    build_dir: PathBuf,
    project_root: PathBuf,
}

impl PluginBuilder {
    pub fn new(plugin_id: &str) -> Result<Self> {
        let cwd = std::env::current_dir()?;
        let project_root = if cwd.ends_with("src-tauri") {
            cwd.parent().unwrap().to_path_buf()
        } else {
            cwd.clone()
        };

        // Check for plugin in AppData/Local/WebArcade/projects first
        let appdata_dir = dirs::data_local_dir()
            .or_else(|| dirs::data_dir())
            .expect("Could not determine data directory");

        let plugin_dir_appdata = appdata_dir.join("WebArcade").join("projects").join(plugin_id);
        let plugin_dir_root = project_root.join("src").join("plugins").join(plugin_id);

        let plugin_dir = if plugin_dir_appdata.exists() {
            plugin_dir_appdata
        } else if plugin_dir_root.exists() {
            plugin_dir_root
        } else {
            anyhow::bail!("Plugin directory does not exist: {:?} or {:?}", plugin_dir_appdata, plugin_dir_root);
        };

        let build_dir = project_root.join("dist").join("plugins").join(plugin_id);
        fs::create_dir_all(&build_dir)?;

        Ok(Self {
            plugin_dir,
            plugin_id: plugin_id.to_string(),
            build_dir,
            project_root,
        })
    }

    pub fn build(&self, progress_callback: impl Fn(BuildProgress), log_callback: impl Fn(String)) -> Result<BuildResult> {
        // Check if plugin has Rust backend
        let has_backend = self.plugin_dir.join("mod.rs").exists();

        progress_callback(BuildProgress {
            step: "prepare".to_string(),
            progress: 0.1,
            message: "Preparing plugin build...".to_string(),
        });

        // Clean build directory
        if self.build_dir.exists() {
            fs::remove_dir_all(&self.build_dir)?;
        }
        fs::create_dir_all(&self.build_dir)?;

        // Copy backend and compile if present
        if has_backend {
            progress_callback(BuildProgress {
                step: "backend_setup".to_string(),
                progress: 0.2,
                message: "Setting up Rust backend...".to_string(),
            });

            self.setup_backend_build()?;

            progress_callback(BuildProgress {
                step: "backend_compile".to_string(),
                progress: 0.3,
                message: "Compiling Rust backend as dynamic library...".to_string(),
            });

            self.compile_backend(&progress_callback, &log_callback)?;
        }

        // Bundle frontend
        progress_callback(BuildProgress {
            step: "frontend_bundle".to_string(),
            progress: 0.7,
            message: "Bundling frontend code...".to_string(),
        });

        self.bundle_frontend()?;

        // Create package
        progress_callback(BuildProgress {
            step: "packaging".to_string(),
            progress: 0.9,
            message: "Creating distribution package...".to_string(),
        });

        let zip_path = self.create_package(has_backend)?;

        progress_callback(BuildProgress {
            step: "complete".to_string(),
            progress: 1.0,
            message: format!("Build complete: {}", zip_path),
        });

        Ok(BuildResult {
            success: true,
            output_path: Some(zip_path),
            error: None,
        })
    }

    fn read_cargo_dependencies_from_package_json(&self) -> Result<std::collections::HashMap<String, String>> {
        let package_json_path = self.plugin_dir.join("package.json");
        let mut deps = std::collections::HashMap::new();

        if package_json_path.exists() {
            let content = fs::read_to_string(&package_json_path)?;
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(cargo_deps) = json.get("cargo_dependencies").and_then(|v| v.as_object()) {
                    for (key, value) in cargo_deps {
                        if let Some(version) = value.as_str() {
                            deps.insert(key.clone(), version.to_string());
                        }
                    }
                }
            }
        }

        Ok(deps)
    }

    fn setup_backend_build(&self) -> Result<()> {
        // Create a temporary build directory for Rust compilation
        let rust_build_dir = self.build_dir.join("rust_build");
        fs::create_dir_all(&rust_build_dir)?;

        // Copy all plugin Rust source files
        self.copy_rust_files(&self.plugin_dir, &rust_build_dir)?;

        // Check if plugin has its own Cargo.toml
        let plugin_cargo_toml = self.plugin_dir.join("Cargo.toml");
        let api_path = self.project_root.join("src-tauri").join("api");
        let api_path_str = api_path.to_string_lossy().replace("\\", "/");

        // Read cargo_dependencies from package.json if it exists
        let additional_deps = self.read_cargo_dependencies_from_package_json()?;

        let cargo_toml = if plugin_cargo_toml.exists() {
            // Read existing Cargo.toml and update webarcade_api path to absolute
            let mut content = fs::read_to_string(&plugin_cargo_toml)?;

            // Check if webarcade_api is already present
            let re = regex::Regex::new(r#"webarcade_api\s*=\s*\{[^}]*path\s*=\s*"[^"]*"[^}]*\}"#).unwrap();

            content = if re.is_match(&content) {
                // Replace existing webarcade_api path
                re.replace(&content, format!("webarcade_api = {{ path = \"{}\" }}", api_path_str)).to_string()
            } else {
                // Inject webarcade_api if not present
                // Find [dependencies] section and add it there
                let deps_re = regex::Regex::new(r"(?m)^\[dependencies\]\s*$").unwrap();
                if let Some(mat) = deps_re.find(&content) {
                    let insert_pos = mat.end();
                    let mut new_content = content.clone();
                    new_content.insert_str(insert_pos, &format!("\nwebarcade_api = {{ path = \"{}\" }}", api_path_str));
                    new_content
                } else {
                    content
                }
            };

            // Also inject lazy_static if not present
            if !content.contains("lazy_static") {
                let deps_re = regex::Regex::new(r"(?m)^\[dependencies\]\s*$").unwrap();
                if let Some(mat) = deps_re.find(&content) {
                    let insert_pos = mat.end();
                    content.insert_str(insert_pos, "\nlazy_static = \"1.5\"");
                }
            }

            content
        } else {
            // Create default Cargo.toml
            let mut deps = format!("webarcade_api = {{ path = \"{}\" }}\n", api_path_str);
            deps.push_str("lazy_static = \"1.5\"\n");

            // Add additional dependencies from package.json
            for (dep_name, dep_version) in &additional_deps {
                deps.push_str(&format!("{} = \"{}\"\n", dep_name, dep_version));
            }

            format!(
                r#"[package]
name = "{}"
version = "1.0.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]
path = "lib.rs"

[dependencies]
{}
[profile.release]
opt-level = "z"
lto = true
codegen-units = 1
strip = true
"#,
                self.plugin_id, deps
            )
        };

        fs::write(rust_build_dir.join("Cargo.toml"), cargo_toml)?;

        // Create .cargo/config.toml to allow undefined symbols (they'll be resolved at runtime)
        let cargo_config_dir = rust_build_dir.join(".cargo");
        fs::create_dir_all(&cargo_config_dir)?;

        let cargo_config = r#"[target.x86_64-pc-windows-msvc]
rustflags = ["-C", "link-args=/FORCE:UNRESOLVED"]

[target.x86_64-unknown-linux-gnu]
rustflags = ["-C", "link-args=-Wl,--allow-shlib-undefined"]

[target.x86_64-apple-darwin]
rustflags = ["-C", "link-args=-undefined dynamic_lookup"]

[target.aarch64-apple-darwin]
rustflags = ["-C", "link-args=-undefined dynamic_lookup"]
"#;
        fs::write(cargo_config_dir.join("config.toml"), cargo_config)?;

        // Create lib.rs that includes core and the plugin
        self.create_lib_rs(&rust_build_dir)?;

        Ok(())
    }

    fn create_lib_rs(&self, rust_build_dir: &Path) -> Result<()> {
        // Get plugin struct name (e.g., "MyPlugin" from plugin dir name)
        let plugin_struct = self.get_plugin_struct_name();

        // Extract handler functions from router.rs
        let handlers = self.extract_handlers()?;

        // Generate handler wrapper functions
        let handler_wrappers = handlers.iter().map(|handler_name| {
            format!("
#[no_mangle]
pub extern \"C\" fn {}() -> *const u8 {{
    use std::panic;
    use std::ffi::CString;

    let result = panic::catch_unwind(|| {{
        // Use the shared runtime instead of creating a new one
        RUNTIME.block_on(async {{
            use webarcade_api::http_body_util::BodyExt;

            // Call the handler and get the HttpResponse
            let response = plugin_mod::router::{}().await;

            // Extract the body bytes from the response
            let (_parts, body) = response.into_parts();
            let body_bytes = body.collect().await.unwrap().to_bytes();
            let body_str = String::from_utf8(body_bytes.to_vec()).unwrap();

            body_str
        }})
    }});

    match result {{
        Ok(json_string) => {{
            // Convert to CString to ensure null-termination for C FFI
            let c_string = CString::new(json_string).unwrap();
            Box::leak(Box::new(c_string)).as_ptr() as *const u8
        }}
        Err(_) => {{
            let error = CString::new(\"{{\\\"error\\\": \\\"Handler panicked\\\"}}\").unwrap();
            Box::leak(Box::new(error)).as_ptr() as *const u8
        }}
    }}
}}
", handler_name, handler_name)
        }).collect::<Vec<_>>().join("\n");

        let lib_content = format!(r#"// Auto-generated plugin library
// This file uses the webarcade_api to provide a clean plugin interface

// Include plugin modules
pub mod plugin_mod;

// Re-export plugin
pub use plugin_mod::*;
use webarcade_api::tokio;
use std::sync::Arc;

// Shared runtime for all handler calls to avoid creating multiple runtimes
lazy_static::lazy_static! {{
    static ref RUNTIME: Arc<tokio::runtime::Runtime> = Arc::new(
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .worker_threads(2)
            .build()
            .expect("Failed to create tokio runtime")
    );
}}

// Export plugin lifecycle functions for dynamic loading
// Note: These are no-ops since we use manifest-based routing
#[no_mangle]
pub extern "C" fn plugin_init(_ffi_ctx: *const ()) -> i32 {{
    // Routes are registered from manifest, so init is a no-op
    0
}}

#[no_mangle]
pub extern "C" fn plugin_start(_ffi_ctx: *const ()) -> i32 {{
    // Routes are registered from manifest, so start is a no-op
    0
}}

#[no_mangle]
pub extern "C" fn plugin_stop() -> i32 {{
    // Stop is a no-op for manifest-based plugins
    0
}}

#[no_mangle]
pub extern "C" fn plugin_metadata() -> *const u8 {{
    use webarcade_api::{{Plugin, serde_json}};
    let plugin = plugin_mod::{};
    let metadata = plugin.metadata();
    let json = serde_json::to_string(&metadata).unwrap_or_default();
    Box::leak(Box::new(json)).as_ptr() as *const u8
}}

// Auto-generated handler wrappers
{}
"#, plugin_struct, handler_wrappers);

        fs::write(rust_build_dir.join("lib.rs"), lib_content)?;

        Ok(())
    }

    fn extract_handlers(&self) -> Result<Vec<String>> {
        let router_path = self.plugin_dir.join("router.rs");
        if !router_path.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&router_path)?;
        let mut handlers = Vec::new();

        // Look for route! macro usages to extract handler names
        // Pattern: route!(router, METHOD "/path" => handler_name);
        let re = regex::Regex::new(r"route!\s*\([^,]+,\s*\w+\s+[^=]+=>\s*([a-zA-Z_][a-zA-Z0-9_]*)\s*\)").unwrap();

        for cap in re.captures_iter(&content) {
            if let Some(handler_name) = cap.get(1) {
                handlers.push(handler_name.as_str().to_string());
            }
        }

        Ok(handlers)
    }

    fn get_plugin_struct_name(&self) -> String {
        // Convert plugin_id to PascalCase and add "Plugin" suffix
        let parts: Vec<&str> = self.plugin_id.split('_').collect();
        let mut name = String::new();
        for part in parts {
            let mut chars = part.chars();
            if let Some(first) = chars.next() {
                name.push(first.to_uppercase().next().unwrap());
                name.push_str(chars.as_str());
            }
        }
        name.push_str("Plugin");
        name
    }

    fn copy_rust_files(&self, src: &Path, dst: &Path) -> Result<()> {
        // Create plugin_mod directory to hold the plugin files
        let plugin_mod_dir = dst.join("plugin_mod");
        fs::create_dir_all(&plugin_mod_dir)?;

        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let path = entry.path();
            let file_name = entry.file_name();
            let file_name_str = file_name.to_string_lossy();

            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext == "rs" {
                        let dest_path = plugin_mod_dir.join(&file_name);

                        let content = fs::read_to_string(&path)?;
                        let modified_content = if file_name_str == "mod.rs" {
                            // Make the router module public
                            content.replace("mod router;", "pub mod router;")
                        } else if file_name_str == "router.rs" {
                            // Make handler functions public
                            // Pattern: async fn handler_name() -> HttpResponse
                            let re = regex::Regex::new(r"async fn ([a-zA-Z_][a-zA-Z0-9_]*)\(\) -> HttpResponse").unwrap();
                            re.replace_all(&content, "pub async fn $1() -> HttpResponse").to_string()
                        } else {
                            content
                        };

                        fs::write(&dest_path, modified_content)?;
                    }
                }
            }
        }

        Ok(())
    }

    fn compile_backend(&self, progress_callback: &impl Fn(BuildProgress), log_callback: &impl Fn(String)) -> Result<()> {
        let rust_build_dir = self.build_dir.join("rust_build");

        log::info!("[Developer] Compiling {} as cdylib...", self.plugin_id);
        log_callback(format!("Compiling {} as cdylib...\n", self.plugin_id));

        // Build for current platform
        let mut cmd = Command::new("cargo");
        cmd.current_dir(&rust_build_dir)
            .args(&["build", "--release", "--lib"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        log_callback("Running: cargo build --release --lib\n".to_string());

        let mut child = cmd.spawn().context("Failed to spawn cargo build")?;

        // Read both stdout and stderr concurrently using threads and channels
        use std::io::BufRead;
        use std::sync::mpsc;

        let stdout = child.stdout.take();
        let stderr = child.stderr.take();

        let (tx, rx) = mpsc::channel::<String>();

        let mut handles = vec![];

        // Spawn thread for stdout
        if let Some(stdout) = stdout {
            let tx = tx.clone();
            let handle = std::thread::spawn(move || {
                let reader = std::io::BufReader::new(stdout);
                for line in reader.lines() {
                    if let Ok(line) = line {
                        let _ = tx.send(format!("{}\n", line));
                    }
                }
            });
            handles.push(handle);
        }

        // Spawn thread for stderr
        if let Some(stderr) = stderr {
            let tx = tx.clone();
            let handle = std::thread::spawn(move || {
                let reader = std::io::BufReader::new(stderr);
                for line in reader.lines() {
                    if let Ok(line) = line {
                        let _ = tx.send(format!("{}\n", line));
                    }
                }
            });
            handles.push(handle);
        }

        // Drop the original sender so the channel closes when threads are done
        drop(tx);

        // Receive and log messages from both streams
        for msg in rx {
            log_callback(msg);
        }

        // Wait for both threads to finish
        for handle in handles {
            let _ = handle.join();
        }

        let status = child.wait().context("Failed to wait for cargo build")?;

        if !status.success() {
            log::error!("[Developer] Cargo build failed");
            log_callback("Cargo build failed!\n".to_string());
            anyhow::bail!("Cargo build failed");
        }

        log_callback("Rust compilation successful!\n".to_string());

        progress_callback(BuildProgress {
            step: "backend_compile".to_string(),
            progress: 0.6,
            message: "Rust compilation successful".to_string(),
        });

        // Copy compiled binaries to build output
        self.copy_compiled_binaries(&rust_build_dir)?;

        Ok(())
    }

    fn copy_compiled_binaries(&self, rust_build_dir: &Path) -> Result<()> {
        let target_dir = rust_build_dir.join("target").join("release");

        // Determine library name based on platform
        let lib_patterns = if cfg!(target_os = "windows") {
            vec![format!("{}.dll", self.plugin_id)]
        } else if cfg!(target_os = "macos") {
            vec![format!("lib{}.dylib", self.plugin_id)]
        } else {
            vec![format!("lib{}.so", self.plugin_id)]
        };

        // Find and copy the compiled library directly to build_dir root
        for pattern in lib_patterns {
            let src_path = target_dir.join(&pattern);
            if src_path.exists() {
                let dest_path = self.build_dir.join(&pattern);
                fs::copy(&src_path, &dest_path)?;

                log::info!("[Developer] Copied binary to root: {}", pattern);
                return Ok(());
            }
        }

        anyhow::bail!("Could not find compiled library in {:?}", target_dir);
    }

    fn bundle_frontend(&self) -> Result<()> {
        // Check if plugin has frontend files
        let has_frontend = self.plugin_dir.join("index.jsx").exists()
            || self.plugin_dir.join("index.js").exists();

        if !has_frontend {
            log::info!("[Developer] No frontend files found, skipping frontend bundling");
            return Ok(());
        }

        // Install npm dependencies if package.json has dependencies
        self.install_npm_dependencies()?;

        // Use Node.js bundler to bundle frontend code
        log::info!("[Developer] Bundling frontend with RSpack...");

        let bundler_script = self.project_root.join("scripts/build_plugin.js");
        let plugin_dir_str = self.plugin_dir.to_string_lossy();
        let build_dir_str = self.build_dir.to_string_lossy();

        // Run the Node.js bundler (try bun first, then node)
        let output = if Command::new("bun").arg("--version").output().is_ok() {
            Command::new("bun")
                .arg("run")
                .arg(bundler_script)
                .arg(&*plugin_dir_str)
                .arg(&*build_dir_str)
                .output()
                .context("Failed to execute frontend bundler with bun")?
        } else {
            Command::new("node")
                .arg(bundler_script)
                .arg(&*plugin_dir_str)
                .arg(&*build_dir_str)
                .output()
                .context("Failed to execute frontend bundler with node")?
        };

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            log::error!("[Developer] Frontend bundling failed:");
            log::error!("    STDOUT:\n{}", stdout);
            log::error!("    STDERR:\n{}", stderr);
            anyhow::bail!("Frontend bundling failed: {}", stderr);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        log::info!("[Developer] Frontend bundling output:\n{}", stdout);

        Ok(())
    }

    fn install_npm_dependencies(&self) -> Result<()> {
        let package_json_path = self.plugin_dir.join("package.json");

        // Check if package.json exists and has dependencies
        if !package_json_path.exists() {
            return Ok(());
        }

        let content = fs::read_to_string(&package_json_path)?;
        let json: serde_json::Value = serde_json::from_str(&content)?;

        // Check if there are any dependencies or devDependencies
        let has_deps = json.get("dependencies").and_then(|d| d.as_object()).map(|o| !o.is_empty()).unwrap_or(false);
        let has_dev_deps = json.get("devDependencies").and_then(|d| d.as_object()).map(|o| !o.is_empty()).unwrap_or(false);

        if !has_deps && !has_dev_deps {
            log::info!("[Developer] No npm dependencies to install");
            return Ok(());
        }

        log::info!("[Developer] Installing npm dependencies...");

        // Try bun first (faster), then npm
        let install_result = if Command::new("bun").arg("--version").output().is_ok() {
            log::info!("[Developer] Using bun to install dependencies");
            Command::new("bun")
                .arg("install")
                .current_dir(&self.plugin_dir)
                .output()
        } else {
            log::info!("[Developer] Using npm to install dependencies");
            Command::new("npm")
                .arg("install")
                .current_dir(&self.plugin_dir)
                .output()
        };

        match install_result {
            Ok(output) => {
                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    log::warn!("[Developer] Dependency installation had issues: {}", stderr);
                    // Don't fail the build, just warn - bundler might still work
                } else {
                    log::info!("[Developer] Dependencies installed successfully");
                }
            }
            Err(e) => {
                log::warn!("[Developer] Could not install dependencies: {}", e);
                // Don't fail the build - user might have dependencies pre-installed
            }
        }

        Ok(())
    }

    fn copy_dir_recursive(&self, src: &Path, dst: &Path) -> Result<()> {
        fs::create_dir_all(dst)?;

        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let path = entry.path();
            let file_name = entry.file_name();
            let dst_path = dst.join(&file_name);

            if path.is_dir() {
                self.copy_dir_recursive(&path, &dst_path)?;
            } else {
                fs::copy(&path, &dst_path)?;
            }
        }

        Ok(())
    }

    fn create_package(&self, has_backend: bool) -> Result<String> {
        let cwd = std::env::current_dir()?;
        let dist_root = if cwd.ends_with("src-tauri") {
            cwd.parent().unwrap().join("dist")
        } else {
            cwd.join("dist")
        };

        fs::create_dir_all(&dist_root)?;

        let zip_path = dist_root.join(format!("{}.zip", self.plugin_id));
        let zip_file = fs::File::create(&zip_path)?;
        let mut zip = ZipWriter::new(zip_file);
        let options = FileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);

        // Add all files from build_dir root (flat structure)
        for entry in fs::read_dir(&self.build_dir)? {
            let entry = entry?;
            let path = entry.path();
            let file_name = entry.file_name();
            let file_name_str = file_name.to_string_lossy();

            // Skip directories (like rust_build, src)
            if path.is_dir() {
                continue;
            }

            // Add file to zip root
            zip.start_file(file_name_str.as_ref(), options)?;
            let content = fs::read(&path)?;
            zip.write_all(&content)?;
        }

        // Add manifest
        let manifest = self.create_manifest(has_backend)?;
        zip.start_file("manifest.json", options)?;
        zip.write_all(manifest.as_bytes())?;

        // Add routes configuration (separate from manifest for easier parsing)
        let routes_config = self.create_routes_config()?;
        zip.start_file("routes.json", options)?;
        zip.write_all(routes_config.as_bytes())?;

        // Add README
        let readme = format!(
            r#"# {} Plugin

WebArcade Plugin Package

## Installation

Drag and drop this .zip file anywhere in the WebArcade window to install.

## Contents

- Frontend: {}
- Backend: {}
- Platform: {}

## Plugin Structure

All files are in the root directory for simplicity:
- manifest.json - Plugin metadata
- plugin.js - Frontend code (if has_frontend)
- routes.json - Backend routes (if has_backend)
- *.dll / lib*.so / lib*.dylib - Native binary (if has_backend)

## Note

This plugin was built on {} and includes platform-specific binaries.
For other platforms, you may need to rebuild from source.
"#,
            self.plugin_id,
            if self.build_dir.join("plugin.js").exists() { "Included" } else { "None" },
            if has_backend { "Included" } else { "None" },
            if cfg!(target_os = "windows") {
                "Windows"
            } else if cfg!(target_os = "macos") {
                "macOS"
            } else {
                "Linux"
            },
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        );

        zip.start_file("README.md", options)?;
        zip.write_all(readme.as_bytes())?;

        zip.finish()?;

        Ok(zip_path.to_string_lossy().to_string())
    }

    fn create_manifest(&self, has_backend: bool) -> Result<String> {
        let package_json_path = self.plugin_dir.join("package.json");
        let metadata = if package_json_path.exists() {
            let content = fs::read_to_string(package_json_path)?;
            serde_json::from_str::<serde_json::Value>(&content).ok()
        } else {
            None
        };

        // Extract routes from router.rs
        let routes = self.extract_routes()?;

        let manifest = serde_json::json!({
            "id": self.plugin_id,
            "name": metadata.as_ref()
                .and_then(|m| m.get("name"))
                .and_then(|n| n.as_str())
                .unwrap_or(&self.plugin_id),
            "version": metadata.as_ref()
                .and_then(|m| m.get("version"))
                .and_then(|v| v.as_str())
                .unwrap_or("1.0.0"),
            "description": metadata.as_ref()
                .and_then(|m| m.get("description"))
                .and_then(|d| d.as_str())
                .unwrap_or(""),
            "author": metadata.as_ref()
                .and_then(|m| m.get("author"))
                .and_then(|a| a.as_str())
                .unwrap_or("Unknown"),
            "has_backend": has_backend,
            "has_frontend": self.plugin_dir.join("index.jsx").exists(),
            "build_date": chrono::Utc::now().to_rfc3339(),
            "build_platform": std::env::consts::OS,
            "supported_platforms": if has_backend {
                vec![std::env::consts::OS]
            } else {
                vec!["windows", "macos", "linux"]
            },
            "routes": routes,
        });

        Ok(serde_json::to_string_pretty(&manifest)?)
    }

    fn extract_routes(&self) -> Result<Vec<serde_json::Value>> {
        let router_path = self.plugin_dir.join("router.rs");
        if !router_path.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&router_path)?;
        let mut routes = Vec::new();

        // Pattern: route!(router, METHOD "/path" => handler_name);
        let re = regex::Regex::new(r#"route!\s*\([^,]+,\s*(\w+)\s+"([^"]+)"\s*=>\s*([a-zA-Z_][a-zA-Z0-9_]*)\s*\)"#).unwrap();

        for cap in re.captures_iter(&content) {
            if let (Some(method), Some(path), Some(handler)) = (cap.get(1), cap.get(2), cap.get(3)) {
                routes.push(serde_json::json!({
                    "method": method.as_str(),
                    "path": path.as_str(),
                    "handler": handler.as_str(),
                }));
            }
        }

        Ok(routes)
    }

    fn create_routes_config(&self) -> Result<String> {
        let routes = self.extract_routes()?;

        let config = serde_json::json!({
            "plugin_id": self.plugin_id,
            "routes": routes,
        });

        Ok(serde_json::to_string_pretty(&config)?)
    }

}
