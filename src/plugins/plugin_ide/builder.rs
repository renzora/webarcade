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

        // Check for plugin in plugin_ide subfolder first (for dev plugins)
        let plugin_dir_ide = project_root.join("plugins").join("plugin_ide").join(plugin_id);
        let plugin_dir_root = project_root.join("plugins").join(plugin_id);

        let plugin_dir = if plugin_dir_ide.exists() {
            plugin_dir_ide
        } else if plugin_dir_root.exists() {
            plugin_dir_root
        } else {
            anyhow::bail!("Plugin directory does not exist: {:?} or {:?}", plugin_dir_ide, plugin_dir_root);
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

    fn setup_backend_build(&self) -> Result<()> {
        // Create a temporary build directory for Rust compilation
        let rust_build_dir = self.build_dir.join("rust_build");
        fs::create_dir_all(&rust_build_dir)?;

        // Copy all plugin Rust source files
        self.copy_rust_files(&self.plugin_dir, &rust_build_dir)?;

        // Check if plugin has its own Cargo.toml
        let plugin_cargo_toml = self.plugin_dir.join("Cargo.toml");
        let api_path = self.project_root.join("webarcade_api");
        let api_path_str = api_path.to_string_lossy().replace("\\", "/");

        let cargo_toml = if plugin_cargo_toml.exists() {
            // Read existing Cargo.toml and inject webarcade_api path
            let mut content = fs::read_to_string(&plugin_cargo_toml)?;

            // Ensure webarcade_api is in dependencies
            if !content.contains("webarcade_api") {
                // Add webarcade_api to dependencies section
                if let Some(deps_pos) = content.find("[dependencies]") {
                    let insert_pos = deps_pos + "[dependencies]".len();
                    content.insert_str(insert_pos, &format!("\nwebarcade_api = {{ path = \"{}\" }}", api_path_str));
                } else {
                    // No dependencies section, add it
                    content.push_str(&format!("\n[dependencies]\nwebarcade_api = {{ path = \"{}\" }}\n", api_path_str));
                }
            } else {
                // Replace webarcade_api path
                let re = regex::Regex::new(r#"webarcade_api\s*=\s*\{[^}]*\}"#).unwrap();
                content = re.replace(&content, format!("webarcade_api = {{ path = \"{}\" }}", api_path_str)).to_string();
            }

            // Ensure crate-type is cdylib
            if !content.contains("crate-type") {
                if let Some(lib_pos) = content.find("[lib]") {
                    let insert_pos = lib_pos + "[lib]".len();
                    content.insert_str(insert_pos, "\ncrate-type = [\"cdylib\"]");
                } else {
                    // Add [lib] section before [dependencies]
                    if let Some(deps_pos) = content.find("[dependencies]") {
                        content.insert_str(deps_pos, "[lib]\ncrate-type = [\"cdylib\"]\npath = \"lib.rs\"\n\n");
                    } else {
                        content.push_str("\n[lib]\ncrate-type = [\"cdylib\"]\npath = \"lib.rs\"\n");
                    }
                }
            }

            // Ensure path is set to lib.rs
            if !content.contains("path") || !content.contains("lib.rs") {
                let re = regex::Regex::new(r#"\[lib\][^\[]*"#).unwrap();
                if let Some(mat) = re.find(&content) {
                    let lib_section = mat.as_str();
                    if !lib_section.contains("path") {
                        let insert_pos = mat.end();
                        content.insert_str(insert_pos - 1, "path = \"lib.rs\"\n");
                    }
                }
            }

            content
        } else {
            // Create default Cargo.toml
            format!(
                r#"[package]
name = "{}"
version = "1.0.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]
path = "lib.rs"

[dependencies]
webarcade_api = {{ path = "{}" }}

[profile.release]
opt-level = "z"
lto = true
codegen-units = 1
strip = true
"#,
                self.plugin_id, api_path_str
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

    let result = panic::catch_unwind(|| {{
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        rt.block_on(async {{
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
        Ok(json_string) => Box::leak(Box::new(json_string)).as_ptr(),
        Err(_) => {{
            let error = String::from(\"{{\\\"error\\\": \\\"Handler panicked\\\"}}\");
            Box::leak(Box::new(error)).as_ptr()
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

        log::info!("[plugin_ide] Compiling {} as cdylib...", self.plugin_id);
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
            log::error!("[plugin_ide] Cargo build failed");
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
        let binary_dir = self.build_dir.join("binaries");
        fs::create_dir_all(&binary_dir)?;

        // Determine library name based on platform
        let lib_patterns = if cfg!(target_os = "windows") {
            vec![format!("{}.dll", self.plugin_id)]
        } else if cfg!(target_os = "macos") {
            vec![format!("lib{}.dylib", self.plugin_id)]
        } else {
            vec![format!("lib{}.so", self.plugin_id)]
        };

        // Find and copy the compiled library
        for pattern in lib_patterns {
            let src_path = target_dir.join(&pattern);
            if src_path.exists() {
                let platform = if cfg!(target_os = "windows") {
                    "windows"
                } else if cfg!(target_os = "macos") {
                    "macos"
                } else {
                    "linux"
                };

                let dest_dir = binary_dir.join(platform);
                fs::create_dir_all(&dest_dir)?;

                let dest_path = dest_dir.join(&pattern);
                fs::copy(&src_path, &dest_path)?;

                log::info!("[plugin_ide] Copied {} binary: {}", platform, pattern);
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
            log::info!("[plugin_ide] No frontend files found, skipping frontend bundling");
            return Ok(());
        }

        let frontend_dir = self.build_dir.join("frontend");
        fs::create_dir_all(&frontend_dir)?;

        // Use Node.js bundler to bundle frontend code
        log::info!("[plugin_ide] Bundling frontend with RSpack...");

        let bundler_script = self.project_root.join("scripts/bundle-plugin-frontend.js");
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
            log::error!("[plugin_ide] Frontend bundling failed:");
            log::error!("    STDOUT:\n{}", stdout);
            log::error!("    STDERR:\n{}", stderr);
            anyhow::bail!("Frontend bundling failed: {}", stderr);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        log::info!("[plugin_ide] Frontend bundling output:\n{}", stdout);

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

        // Add manifest
        let manifest = self.create_manifest(has_backend)?;
        zip.start_file("manifest.json", options)?;
        zip.write_all(manifest.as_bytes())?;

        // Add routes configuration (separate from manifest for easier parsing)
        let routes_config = self.create_routes_config()?;
        zip.start_file("routes.json", options)?;
        zip.write_all(routes_config.as_bytes())?;

        // Add frontend files
        let frontend_dir = self.build_dir.join("frontend");
        if frontend_dir.exists() {
            self.add_dir_to_zip(&mut zip, &frontend_dir, "frontend", &options)?;
        }

        // Add binaries if present
        let binary_dir = self.build_dir.join("binaries");
        if binary_dir.exists() {
            self.add_dir_to_zip(&mut zip, &binary_dir, "binaries", &options)?;
        }

        // Add README
        let readme = format!(
            r#"# {} Plugin

WebArcade Plugin Package

## Installation

1. Open WebArcade
2. Navigate to Plugins → Add Plugin
3. Upload this zip file
4. The plugin will be extracted and loaded automatically

## Contents

- Frontend: {}
- Backend: {}
- Platform: {}

## Note

This plugin was built on {} and includes platform-specific binaries.
For other platforms, you may need to rebuild from source in development mode.
"#,
            self.plugin_id,
            if frontend_dir.exists() { "Included" } else { "None" },
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

    fn add_dir_to_zip(
        &self,
        zip: &mut ZipWriter<fs::File>,
        src: &Path,
        prefix: &str,
        options: &FileOptions,
    ) -> Result<()> {
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let path = entry.path();
            let name = path.file_name().unwrap();
            let zip_path = format!("{}/{}", prefix, name.to_string_lossy());

            if path.is_dir() {
                zip.add_directory(&zip_path, *options)?;
                self.add_dir_to_zip_recursive(zip, &path, &zip_path, options)?;
            } else {
                zip.start_file(&zip_path, *options)?;
                let content = fs::read(&path)?;
                zip.write_all(&content)?;
            }
        }

        Ok(())
    }

    fn add_dir_to_zip_recursive(
        &self,
        zip: &mut ZipWriter<fs::File>,
        src: &Path,
        prefix: &str,
        options: &FileOptions,
    ) -> Result<()> {
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let path = entry.path();
            let name = path.file_name().unwrap();
            let zip_path = format!("{}/{}", prefix, name.to_string_lossy());

            if path.is_dir() {
                zip.add_directory(&zip_path, *options)?;
                self.add_dir_to_zip_recursive(zip, &path, &zip_path, options)?;
            } else {
                zip.start_file(&zip_path, *options)?;
                let content = fs::read(&path)?;
                zip.write_all(&content)?;
            }
        }

        Ok(())
    }
}
