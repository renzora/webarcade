use crate::core::plugin_context::PluginContext;
use crate::core::plugin_router::PluginRouter;
use crate::core::router_utils::*;
use crate::route;
use anyhow::Result;
use http_body_util::{BodyExt, combinators::BoxBody};
use hyper::{body::Bytes, body::Incoming, Request, Response, StatusCode};
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::fs;
use std::path::{Path, PathBuf};

pub async fn register_routes(ctx: &PluginContext) -> Result<()> {
    let mut router = PluginRouter::new();

    route!(router, GET "/plugins" => handle_list_plugins);
    route!(router, GET "/debug" => handle_debug);
    route!(router, GET "/tree/:plugin", path => handle_get_tree);
    route!(router, GET "/file/*", path => handle_get_file);
    route!(router, PUT "/file/*", path => handle_save_file);
    route!(router, OPTIONS "/file/*" => handle_cors_preflight);
    route!(router, POST "/file/:plugin", path => handle_create_file);
    route!(router, DELETE "/file/*", path => handle_delete_file);
    route!(router, POST "/build/:plugin", path => handle_build_plugin);
    route!(router, OPTIONS "/build/:plugin" => handle_cors_preflight);
    route!(router, OPTIONS "/create" => handle_cors_preflight);
    route!(router, POST "/create" => handle_create_plugin);

    ctx.register_router("plugin_ide", router).await;

    Ok(())
}

#[derive(Serialize, Deserialize)]
struct PluginInfo {
    id: String,
    name: String,
    path: String,
    has_backend: bool,
    has_frontend: bool,
}

#[derive(Serialize, Deserialize)]
struct FileNode {
    name: String,
    path: String,
    #[serde(rename = "type")]
    node_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    children: Option<Vec<FileNode>>,
}

fn get_plugins_dirs() -> Vec<PathBuf> {
    let cwd = std::env::current_dir().unwrap();
    // Go up one level from src-tauri to project root, then into plugins/plugin_ide
    let project_root = cwd.parent().unwrap_or(&cwd);
    vec![
        project_root.join("plugins").join("plugin_ide"),
    ]
}

async fn handle_cors_preflight() -> Response<BoxBody<Bytes, Infallible>> {
    Response::builder()
        .status(StatusCode::OK)
        .header("Access-Control-Allow-Origin", "*")
        .header("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE, OPTIONS")
        .header("Access-Control-Allow-Headers", "Content-Type")
        .body(full_body(""))
        .unwrap()
}

async fn handle_debug() -> Response<BoxBody<Bytes, Infallible>> {
    let cwd = std::env::current_dir().unwrap();
    let project_root = cwd.parent().unwrap_or(&cwd);
    let plugins_dirs = get_plugins_dirs();

    let debug_info = serde_json::json!({
        "cwd": cwd.to_string_lossy(),
        "project_root": project_root.to_string_lossy(),
        "plugins_dirs": plugins_dirs.iter().map(|p| p.to_string_lossy().to_string()).collect::<Vec<_>>(),
        "test_path_exists": project_root.join("plugins").join("plugin_ide").join("test").exists(),
        "test_has_index": project_root.join("plugins").join("plugin_ide").join("test").join("index.jsx").exists(),
    });

    json_response(&debug_info)
}

async fn handle_list_plugins() -> Response<BoxBody<Bytes, Infallible>> {
    let plugins_dirs = get_plugins_dirs();
    let mut plugins = Vec::new();

    for plugins_dir in plugins_dirs {
        log::info!("[plugin_ide] Scanning directory: {:?}", plugins_dir);
        if let Ok(entries) = fs::read_dir(&plugins_dir) {
            for entry in entries.flatten() {
                if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                    let plugin_path = entry.path();
                    let plugin_id = entry.file_name().to_string_lossy().to_string();

                    let has_backend = plugin_path.join("mod.rs").exists();
                    let has_frontend = plugin_path.join("index.jsx").exists()
                        || plugin_path.join("index.js").exists();

                    // Only include directories that have plugin files
                    if has_backend || has_frontend {
                        log::info!("[plugin_ide] Found plugin: {} (backend: {}, frontend: {})", plugin_id, has_backend, has_frontend);
                        plugins.push(PluginInfo {
                            id: plugin_id.clone(),
                            name: plugin_id,
                            path: plugin_path.to_string_lossy().to_string(),
                            has_backend,
                            has_frontend,
                        });
                    }
                }
            }
        }
    }

    log::info!("[plugin_ide] Total plugins found: {}", plugins.len());
    json_response(&plugins)
}

fn find_plugin_path(plugin_id: &str) -> Option<PathBuf> {
    for plugins_dir in get_plugins_dirs() {
        let plugin_path = plugins_dir.join(plugin_id);
        if plugin_path.exists() {
            return Some(plugin_path);
        }
    }
    None
}

async fn handle_get_tree(path: String) -> Response<BoxBody<Bytes, Infallible>> {
    let parts: Vec<&str> = path.trim_start_matches("/tree/").split('/').collect();
    let plugin_id = match parts.get(0) {
        Some(id) => id,
        None => return error_response(StatusCode::BAD_REQUEST, "Missing plugin ID"),
    };

    let plugin_path = match find_plugin_path(plugin_id) {
        Some(path) => path,
        None => return error_response(StatusCode::NOT_FOUND, "Plugin not found"),
    };

    match build_file_tree(&plugin_path, plugin_id) {
        Ok(tree) => json_response(&tree),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to build tree: {}", e)),
    }
}

fn build_file_tree(path: &Path, name: &str) -> Result<FileNode> {
    let mut node = FileNode {
        name: name.to_string(),
        path: path.to_string_lossy().to_string(),
        node_type: if path.is_dir() { "folder".to_string() } else { "file".to_string() },
        children: None,
    };

    if path.is_dir() {
        let mut children = Vec::new();

        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                let entry_path = entry.path();
                let entry_name = entry.file_name().to_string_lossy().to_string();

                // Skip hidden files and node_modules
                if entry_name.starts_with('.') || entry_name == "node_modules" || entry_name == "target" {
                    continue;
                }

                if let Ok(child) = build_file_tree(&entry_path, &entry_name) {
                    children.push(child);
                }
            }
        }

        // Sort: folders first, then files, alphabetically
        children.sort_by(|a, b| {
            match (&a.node_type[..], &b.node_type[..]) {
                ("folder", "file") => std::cmp::Ordering::Less,
                ("file", "folder") => std::cmp::Ordering::Greater,
                _ => a.name.cmp(&b.name),
            }
        });

        node.children = Some(children);
    }

    Ok(node)
}

async fn handle_get_file(path: String) -> Response<BoxBody<Bytes, Infallible>> {
    let path = path.trim_start_matches("/file/");
    let parts: Vec<&str> = path.splitn(2, '/').collect();

    if parts.len() < 2 {
        return error_response(StatusCode::BAD_REQUEST, "Invalid path");
    }

    let plugin_id = parts[0];
    let file_path = parts[1];

    let plugin_path = match find_plugin_path(plugin_id) {
        Some(path) => path,
        None => return error_response(StatusCode::NOT_FOUND, "Plugin not found"),
    };

    let full_path = plugin_path.join(file_path);

    log::info!("[plugin_ide] Current dir: {:?}", std::env::current_dir().unwrap());
    log::info!("[plugin_ide] Plugin path: {:?}", plugin_path);
    log::info!("[plugin_ide] Looking for file: {:?}", full_path);
    log::info!("[plugin_ide] File exists: {}", full_path.exists());

    if !full_path.exists() {
        return error_response(StatusCode::NOT_FOUND, &format!("File not found: {:?}", full_path));
    }

    match fs::read_to_string(&full_path) {
        Ok(content) => Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "text/plain; charset=utf-8")
            .header("Access-Control-Allow-Origin", "*")
            .body(full_body(&content))
            .unwrap(),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to read file: {}", e)),
    }
}

async fn handle_save_file(path: String, req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    let path = path.trim_start_matches("/file/");
    let parts: Vec<&str> = path.splitn(2, '/').collect();

    if parts.len() < 2 {
        return error_response(StatusCode::BAD_REQUEST, "Invalid path");
    }

    let plugin_id = parts[0];
    let file_path = parts[1];

    let plugin_path = match find_plugin_path(plugin_id) {
        Some(path) => path,
        None => return error_response(StatusCode::NOT_FOUND, "Plugin not found"),
    };

    let full_path = plugin_path.join(file_path);

    // Read request body
    let body = match req.collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(e) => return error_response(StatusCode::BAD_REQUEST, &format!("Failed to read body: {}", e)),
    };

    let content = match String::from_utf8(body.to_vec()) {
        Ok(s) => s,
        Err(e) => return error_response(StatusCode::BAD_REQUEST, &format!("Invalid UTF-8: {}", e)),
    };

    // Ensure parent directory exists
    if let Some(parent) = full_path.parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            return error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to create directory: {}", e));
        }
    }

    match fs::write(&full_path, content) {
        Ok(_) => Response::builder()
            .status(StatusCode::OK)
            .header("Access-Control-Allow-Origin", "*")
            .body(full_body("File saved"))
            .unwrap(),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to write file: {}", e)),
    }
}

#[derive(Deserialize)]
struct CreateFileRequest {
    path: String,
    #[serde(rename = "type")]
    file_type: String,
    name: String,
}

async fn handle_create_file(path: String, req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    let plugin_id = path.trim_start_matches("/file/");

    let body = match req.collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(e) => return error_response(StatusCode::BAD_REQUEST, &format!("Failed to read body: {}", e)),
    };

    let create_req: CreateFileRequest = match serde_json::from_slice(&body) {
        Ok(req) => req,
        Err(e) => return error_response(StatusCode::BAD_REQUEST, &format!("Invalid JSON: {}", e)),
    };

    let base_path = match find_plugin_path(plugin_id) {
        Some(path) => path,
        None => return error_response(StatusCode::NOT_FOUND, "Plugin not found"),
    };

    let target_path = if create_req.path.is_empty() {
        base_path.join(&create_req.name)
    } else {
        base_path.join(&create_req.path).join(&create_req.name)
    };

    let result = if create_req.file_type == "folder" {
        fs::create_dir_all(&target_path)
    } else {
        if let Some(parent) = target_path.parent() {
            if let Err(e) = fs::create_dir_all(parent) {
                return error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to create directory: {}", e));
            }
        }
        fs::write(&target_path, "")
    };

    match result {
        Ok(_) => Response::builder()
            .status(StatusCode::OK)
            .header("Access-Control-Allow-Origin", "*")
            .body(full_body("Created"))
            .unwrap(),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to create: {}", e)),
    }
}

async fn handle_delete_file(path: String) -> Response<BoxBody<Bytes, Infallible>> {
    let path = path.trim_start_matches("/file/");
    let parts: Vec<&str> = path.splitn(2, '/').collect();

    if parts.len() < 2 {
        return error_response(StatusCode::BAD_REQUEST, "Invalid path");
    }

    let plugin_id = parts[0];
    let file_path = parts[1];

    let plugin_path = match find_plugin_path(plugin_id) {
        Some(path) => path,
        None => return error_response(StatusCode::NOT_FOUND, "Plugin not found"),
    };

    let full_path = plugin_path.join(file_path);

    if !full_path.exists() {
        return error_response(StatusCode::NOT_FOUND, "File not found");
    }

    let result = if full_path.is_dir() {
        fs::remove_dir_all(&full_path)
    } else {
        fs::remove_file(&full_path)
    };

    match result {
        Ok(_) => Response::builder()
            .status(StatusCode::OK)
            .header("Access-Control-Allow-Origin", "*")
            .body(full_body("Deleted"))
            .unwrap(),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to delete: {}", e)),
    }
}

#[derive(Deserialize)]
struct CreatePluginRequest {
    id: String,
    name: String,
    description: String,
    author: String,
    template: String,
    #[serde(default)]
    location: Option<String>,
}

async fn handle_build_plugin(path: String) -> Response<BoxBody<Bytes, Infallible>> {
    let plugin_id = path.trim_start_matches("/build/");

    let plugin_path = match find_plugin_path(plugin_id) {
        Some(path) => path,
        None => return error_response(StatusCode::NOT_FOUND, "Plugin not found"),
    };

    log::info!("[plugin_ide] Building plugin: {}", plugin_id);

    // Create a zip file of the plugin
    let cwd = std::env::current_dir().unwrap();
    let project_root = cwd.parent().unwrap_or(&cwd);
    let output_dir = project_root.join("dist").join("plugins");

    if let Err(e) = fs::create_dir_all(&output_dir) {
        return error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to create output directory: {}", e));
    }

    let zip_path = output_dir.join(format!("{}.zip", plugin_id));

    // Create zip file
    match create_plugin_zip(&plugin_path, &zip_path) {
        Ok(_) => {
            let response = serde_json::json!({
                "success": true,
                "plugin": plugin_id,
                "output": zip_path.to_string_lossy(),
            });
            json_response(&response)
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to create zip: {}", e)),
    }
}

fn create_plugin_zip(source_dir: &Path, zip_path: &Path) -> Result<()> {
    use std::io::Write;

    let file = fs::File::create(zip_path)?;
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::FileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o755);

    let walkdir = |dir: &Path| -> Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            let name = path.strip_prefix(source_dir)?;

            // Skip hidden files, node_modules, and target directories
            let file_name = entry.file_name().to_string_lossy().to_string();
            if file_name.starts_with('.') || file_name == "node_modules" || file_name == "target" {
                continue;
            }

            if path.is_file() {
                zip.start_file(name.to_string_lossy().to_string(), options)?;
                let mut f = fs::File::open(&path)?;
                std::io::copy(&mut f, &mut zip)?;
            } else if path.is_dir() {
                zip.add_directory(name.to_string_lossy().to_string(), options)?;
                walkdir(&path)?;
            }
        }
        Ok(())
    };

    walkdir(source_dir)?;
    zip.finish()?;
    Ok(())
}

async fn handle_create_plugin(req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    let body = match req.collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(e) => return error_response(StatusCode::BAD_REQUEST, &format!("Failed to read body: {}", e)),
    };

    let create_req: CreatePluginRequest = match serde_json::from_slice(&body) {
        Ok(req) => req,
        Err(e) => return error_response(StatusCode::BAD_REQUEST, &format!("Invalid JSON: {}", e)),
    };

    // All plugins are created in the plugin_ide directory
    let cwd = std::env::current_dir().unwrap();
    let project_root = cwd.parent().unwrap_or(&cwd);
    let base_dir = project_root.join("plugins").join("plugin_ide");

    let plugin_path = base_dir.join(&create_req.id);

    if plugin_path.exists() {
        return error_response(StatusCode::CONFLICT, "Plugin already exists");
    }

    if let Err(e) = fs::create_dir_all(&plugin_path) {
        return error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to create directory: {}", e));
    }

    // Create files based on template
    let result = match create_req.template.as_str() {
        "basic" => create_basic_template(&plugin_path, &create_req),
        "widget" => create_widget_template(&plugin_path, &create_req),
        "backend" => create_backend_template(&plugin_path, &create_req),
        "fullstack" => create_fullstack_template(&plugin_path, &create_req),
        _ => create_basic_template(&plugin_path, &create_req),
    };

    match result {
        Ok(_) => {
            let response = serde_json::json!({
                "plugin": {
                    "id": create_req.id,
                    "name": create_req.name,
                    "path": plugin_path.to_string_lossy(),
                }
            });
            json_response(&response)
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to create plugin: {}", e)),
    }
}

fn create_basic_template(path: &Path, req: &CreatePluginRequest) -> Result<()> {
    // Create index.jsx with createPlugin
    let index_content = format!(
        r#"import {{ createPlugin }} from '@/api/plugin';
import {{ IconCode }} from '@tabler/icons-solidjs';
import MyViewport from './viewport.jsx';

export default createPlugin({{
  id: '{}',
  name: '{}',
  version: '1.0.0',
  description: '{}',
  author: '{}',

  async onInit() {{
    console.log('[{}] Initializing...');
  }},

  async onStart(api) {{
    console.log('[{}] Starting...');

    // Register viewport
    api.viewport('{}-viewport', {{
      label: '{}',
      component: MyViewport,
      icon: IconCode,
      description: '{}'
    }});

    // Add menu item
    api.menu('{}-menu', {{
      label: '{}',
      icon: IconCode,
      onClick: () => {{
        api.open('{}-viewport', {{
          label: '{}'
        }});
      }}
    }});

    api.showProps(true);
    api.showMenu(true);
    api.showFooter(true);
    api.showTabs(true);
  }},

  async onStop() {{
    console.log('[{}] Stopping...');
  }}
}});
"#,
        req.id,
        req.name,
        req.description,
        req.author,
        req.name,
        req.name,
        req.id,
        req.name,
        req.description,
        req.id,
        req.name,
        req.id,
        req.name,
        req.name
    );

    fs::write(path.join("index.jsx"), index_content)?;

    // Create viewport.jsx
    let viewport_content = format!(
        r#"export default function MyViewport() {{
  return (
    <div class="h-full w-full flex flex-col bg-base-200 p-4">
      <h1 class="text-2xl font-bold mb-4">{}</h1>
      <p class="text-base-content/70">{}</p>
    </div>
  );
}}
"#,
        req.name,
        req.description
    );

    fs::write(path.join("viewport.jsx"), viewport_content)?;

    let package_json = serde_json::json!({
        "name": req.id,
        "version": "1.0.0",
        "description": req.description,
        "author": req.author,
    });

    fs::write(
        path.join("package.json"),
        serde_json::to_string_pretty(&package_json)?,
    )?;

    Ok(())
}

fn create_widget_template(path: &Path, req: &CreatePluginRequest) -> Result<()> {
    // Create index.jsx with createPlugin
    let index_content = format!(
        r#"import {{ createPlugin }} from '@/api/plugin';
import {{ IconChartBar }} from '@tabler/icons-solidjs';
import MyViewport from './viewport.jsx';

export default createPlugin({{
  id: '{}',
  name: '{}',
  version: '1.0.0',
  description: '{}',
  author: '{}',

  async onStart(api) {{
    console.log('[{}] Starting...');

    api.viewport('{}-viewport', {{
      label: '{}',
      component: MyViewport,
      icon: IconChartBar,
      description: '{}'
    }});

    api.menu('{}-menu', {{
      label: '{}',
      icon: IconChartBar,
      onClick: () => {{
        api.open('{}-viewport', {{ label: '{}' }});
      }}
    }});
  }}
}});
"#,
        req.id, req.name, req.description, req.author,
        req.name, req.id, req.name, req.description,
        req.id, req.name, req.id, req.name
    );

    fs::write(path.join("index.jsx"), index_content)?;

    // Create viewport.jsx
    let viewport_content = format!(
        r#"export default function MyViewport() {{
  return (
    <div class="h-full w-full flex flex-col bg-base-200 p-4">
      <h1 class="text-2xl font-bold mb-4">{}</h1>
      <p class="text-base-content/70">{}</p>
    </div>
  );
}}
"#,
        req.name, req.description
    );

    fs::write(path.join("viewport.jsx"), viewport_content)?;

    // Create widgets directory structure
    let widgets_dir = path.join("widgets").join("main");
    fs::create_dir_all(&widgets_dir)?;

    let widget_content = format!(
        r#"export default function MainWidget(props) {{
  return (
    <div class="card bg-base-100 shadow-xl">
      <div class="card-body">
        <h2 class="card-title">{}</h2>
        <p>Widget content here</p>
      </div>
    </div>
  );
}}
"#,
        req.name
    );

    fs::write(widgets_dir.join("Viewport.jsx"), widget_content)?;

    let metadata = serde_json::json!({
        "id": "main",
        "name": format!("{} Widget", req.name),
        "description": format!("Main widget for {}", req.name),
        "defaultConfig": {},
    });

    fs::write(
        widgets_dir.join("metadata.json"),
        serde_json::to_string_pretty(&metadata)?,
    )?;

    let package_json = serde_json::json!({
        "name": req.id,
        "version": "1.0.0",
        "description": req.description,
        "author": req.author,
    });

    fs::write(
        path.join("package.json"),
        serde_json::to_string_pretty(&package_json)?,
    )?;

    Ok(())
}

fn create_backend_template(path: &Path, req: &CreatePluginRequest) -> Result<()> {
    create_basic_template(path, req)?;

    let plugin_struct_name = capitalize_plugin_name(&req.id);
    let table_name = req.id.replace("-", "_");

    let mod_content = format!(
        r#"mod router;

use crate::core::plugin::Plugin;
use crate::core::plugin_context::PluginContext;
use crate::plugin_metadata;
use async_trait::async_trait;
use std::sync::Arc;
use anyhow::Result;

pub struct {}Plugin;

#[async_trait]
impl Plugin for {}Plugin {{
    plugin_metadata!("{}", "{}", "1.0.0", "{}", author: "{}");

    async fn init(&self, ctx: &PluginContext) -> Result<()> {{
        log::info!("[{}] Initializing");

        // Database migrations
        ctx.migrate(&[
            r"
            CREATE TABLE IF NOT EXISTS {}_data (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                created_at INTEGER NOT NULL
            )
            ",
        ])?;

        // Register routes
        router::register_routes(ctx).await?;

        log::info!("[{}] Initialized successfully");
        Ok(())
    }}

    async fn start(&self, _ctx: Arc<PluginContext>) -> Result<()> {{
        log::info!("[{}] Started");
        Ok(())
    }}

    async fn stop(&self) -> Result<()> {{
        log::info!("[{}] Stopped");
        Ok(())
    }}
}}
"#,
        plugin_struct_name,
        plugin_struct_name,
        req.id,
        req.name,
        req.description,
        req.author,
        req.id,
        table_name,
        req.id,
        req.id,
        req.id
    );

    fs::write(path.join("mod.rs"), mod_content)?;

    let router_content = format!(
        r#"use crate::core::plugin_context::PluginContext;
use crate::core::plugin_router::PluginRouter;
use crate::core::router_utils::*;
use crate::route;
use anyhow::Result;
use http_body_util::combinators::BoxBody;
use hyper::{{body::Bytes, body::Incoming, Request, Response, StatusCode}};
use std::convert::Infallible;

pub async fn register_routes(ctx: &PluginContext) -> Result<()> {{
    let mut router = PluginRouter::new();

    route!(router, GET "/hello" => handle_hello);

    ctx.register_router("{}", router).await;

    Ok(())
}}

async fn handle_hello() -> Response<BoxBody<Bytes, Infallible>> {{
    let response = serde_json::json!({{
        "message": "Hello from {}!"
    }});

    json_response(&response)
}}
"#,
        req.id, req.name
    );

    fs::write(path.join("router.rs"), router_content)?;

    Ok(())
}

fn create_fullstack_template(path: &Path, req: &CreatePluginRequest) -> Result<()> {
    create_backend_template(path, req)?;
    create_widget_template(path, req)?;

    let api_content = format!(
        r#"export async function fetchHello() {{
  const response = await fetch('/{}/hello');
  return response.json();
}}
"#,
        req.id
    );

    fs::write(path.join("api.js"), api_content)?;

    Ok(())
}

fn capitalize_plugin_name(id: &str) -> String {
    let snake = id.replace("-", "_");
    snake
        .split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect()
}
