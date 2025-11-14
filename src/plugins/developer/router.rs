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
    route!(router, POST "/file/:plugin", path => handle_create_file);
    route!(router, DELETE "/file/*", path => handle_delete_file);
    route!(router, POST "/build/:plugin", path => handle_build_plugin);
    route!(router, POST "/install/:plugin", path => handle_install_plugin);
    route!(router, POST "/create" => handle_create_plugin);

    ctx.register_router("developer", router).await;

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
    // Use AppData/Local/WebArcade/projects directory
    let appdata_dir = dirs::data_local_dir()
        .or_else(|| dirs::data_dir())
        .expect("Could not determine data directory");

    vec![
        appdata_dir.join("WebArcade").join("projects"),
    ]
}

async fn handle_debug() -> Response<BoxBody<Bytes, Infallible>> {
    let cwd = std::env::current_dir().unwrap();
    let plugins_dirs = get_plugins_dirs();

    let debug_info = serde_json::json!({
        "cwd": cwd.to_string_lossy(),
        "plugins_dirs": plugins_dirs.iter().map(|p| p.to_string_lossy().to_string()).collect::<Vec<_>>(),
        "appdata_projects_exists": plugins_dirs.first().map(|p| p.exists()).unwrap_or(false),
    });

    json_response(&debug_info)
}

async fn handle_list_plugins() -> Response<BoxBody<Bytes, Infallible>> {
    let plugins_dirs = get_plugins_dirs();
    let mut plugins = Vec::new();

    for plugins_dir in plugins_dirs {
        log::info!("[Developer] Scanning directory: {:?}", plugins_dir);
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
                        log::info!("[Developer] Found plugin: {} (backend: {}, frontend: {})", plugin_id, has_backend, has_frontend);
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

    log::info!("[Developer] Total plugins found: {}", plugins.len());
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

    log::info!("[Developer] Current dir: {:?}", std::env::current_dir().unwrap());
    log::info!("[Developer] Plugin path: {:?}", plugin_path);
    log::info!("[Developer] Looking for file: {:?}", full_path);
    log::info!("[Developer] File exists: {}", full_path.exists());

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

async fn handle_build_plugin(path: String, _req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    use tokio_stream::wrappers::UnboundedReceiverStream;
    use tokio_stream::StreamExt;

    let plugin_id = path.trim_start_matches("/build/");

    let plugin_path = match find_plugin_path(plugin_id) {
        Some(path) => path,
        None => return error_response(StatusCode::NOT_FOUND, "Plugin not found"),
    };

    log::info!("[Developer] Building plugin: {}", plugin_id);

    // Use the real PluginBuilder
    let builder = match crate::plugins::developer::builder::PluginBuilder::new(plugin_id) {
        Ok(b) => b,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to initialize builder: {}", e)),
    };

    // Create a tokio channel for streaming build output (unbounded to avoid blocking)
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<String>();

    // Run the build in a separate thread
    std::thread::spawn(move || {
        let result = builder.build(
            |progress| {
                log::info!("[Developer] Build progress: {} - {}", progress.step, progress.message);

                // Send progress as JSON event
                let event = serde_json::json!({
                    "type": "progress",
                    "step": progress.step,
                    "progress": progress.progress,
                    "message": progress.message
                });

                if let Ok(json) = serde_json::to_string(&event) {
                    let _ = tx.send(format!("{}\n", json));
                }
            },
            |log_msg| {
                log::info!("[Developer] {}", log_msg);

                // Determine log type based on content
                let log_type = if log_msg.contains("error") || log_msg.contains("failed") {
                    "error"
                } else if log_msg.contains("warning") {
                    "warning"
                } else {
                    "info"
                };

                // Send log as JSON event
                let event = serde_json::json!({
                    "type": log_type,
                    "message": log_msg.trim()
                });

                if let Ok(json) = serde_json::to_string(&event) {
                    let _ = tx.send(format!("{}\n", json));
                }
            },
        );

        // Send final result
        match result {
            Ok(build_result) => {
                if build_result.success {
                    let event = serde_json::json!({
                        "type": "success",
                        "step": "complete",
                        "progress": 1.0,
                        "message": format!("Build successful: {}", build_result.output_path.unwrap_or_default())
                    });

                    if let Ok(json) = serde_json::to_string(&event) {
                        let _ = tx.send(format!("{}\n", json));
                    }
                } else {
                    let event = serde_json::json!({
                        "type": "error",
                        "step": "error",
                        "message": build_result.error.unwrap_or_else(|| "Build failed".to_string())
                    });

                    if let Ok(json) = serde_json::to_string(&event) {
                        let _ = tx.send(format!("{}\n", json));
                    }
                }
            }
            Err(e) => {
                let event = serde_json::json!({
                    "type": "error",
                    "step": "error",
                    "message": format!("Build failed: {}", e)
                });

                if let Ok(json) = serde_json::to_string(&event) {
                    let _ = tx.send(format!("{}\n", json));
                }
            }
        }
    });

    // Convert the receiver into a stream
    let stream = UnboundedReceiverStream::new(rx)
        .map(|msg| Ok::<_, Infallible>(hyper::body::Frame::data(Bytes::from(msg))));

    let body = http_body_util::StreamBody::new(stream);

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/x-ndjson")
        .header("Cache-Control", "no-cache")
        .header("Access-Control-Allow-Origin", "*")
        .body(BoxBody::new(body))
        .unwrap()
}

fn create_plugin_zip(source_dir: &Path, zip_path: &Path) -> Result<()> {
    let file = fs::File::create(zip_path)?;
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::FileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o755);

    fn walk_directory(
        dir: &Path,
        source_dir: &Path,
        zip: &mut zip::ZipWriter<fs::File>,
        options: zip::write::FileOptions,
    ) -> Result<()> {
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
                std::io::copy(&mut f, zip)?;
            } else if path.is_dir() {
                zip.add_directory(name.to_string_lossy().to_string(), options)?;
                walk_directory(&path, source_dir, zip, options)?;
            }
        }
        Ok(())
    }

    walk_directory(source_dir, source_dir, &mut zip, options)?;
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

    // All plugins are created in AppData/Local/WebArcade/projects
    let appdata_dir = match dirs::data_local_dir() {
        Some(dir) => dir,
        None => return error_response(StatusCode::INTERNAL_SERVER_ERROR, "Could not find AppData directory"),
    };

    let base_dir = appdata_dir.join("WebArcade").join("projects");

    // Ensure projects directory exists
    if let Err(e) = fs::create_dir_all(&base_dir) {
        return error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to create projects directory: {}", e));
    }

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
        "dependencies": {}
    });

    fs::write(
        path.join("package.json"),
        serde_json::to_string_pretty(&package_json)?,
    )?;

    Ok(())
}

fn create_widget_template(path: &Path, req: &CreatePluginRequest) -> Result<()> {
    // Create index.jsx with createPlugin and widget registration
    let index_content = format!(
        r#"import {{ createPlugin }} from '@/api/plugin';
import {{ IconChartBar }} from '@tabler/icons-solidjs';
import MyViewport from './viewport.jsx';
import MainWidget from './widgets/MainWidget.jsx';

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

    // Register the widget
    api.widget('{}-widget', {{
      title: '{}',
      component: MainWidget,
      icon: IconChartBar,
      description: '{}',
      defaultSize: {{ w: 2, h: 2 }},
      minSize: {{ w: 1, h: 1 }},
      maxSize: {{ w: 4, h: 4 }}
    }});

    console.log('[{}] Started successfully with widget');
  }}
}});
"#,
        req.id, req.name, req.description, req.author,
        req.name, req.id, req.name, req.description,
        req.id, req.name, req.id, req.name,
        req.id, req.name, req.description,
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
        req.name, req.description
    );

    fs::write(path.join("viewport.jsx"), viewport_content)?;

    // Create widgets directory
    let widgets_dir = path.join("widgets");
    fs::create_dir_all(&widgets_dir)?;

    let widget_content = format!(
        r#"import {{ createSignal }} from 'solid-js';
import {{ IconChartBar }} from '@tabler/icons-solidjs';

export default function MainWidget() {{
  const [count, setCount] = createSignal(0);

  return (
    <div class="card bg-gradient-to-br from-primary/20 to-primary/5 bg-base-100 shadow-lg h-full flex flex-col p-4">
      {{/* Header */}}
      <div class="flex items-center justify-between mb-2">
        <div class="flex items-center gap-2">
          <IconChartBar size={{20}} class="text-primary opacity-80" />
          <span class="text-sm font-medium opacity-70">{}</span>
        </div>
      </div>

      {{/* Content */}}
      <div class="flex-1 flex flex-col items-center justify-center">
        <div class="text-4xl font-bold text-primary mb-4">
          {{count()}}
        </div>
        <button
          class="btn btn-primary btn-sm"
          onClick={{() => setCount(count() + 1)}}
        >
          Increment
        </button>
      </div>

      {{/* Footer */}}
      <div class="text-xs opacity-50 text-center mt-2">
        Click to increment
      </div>
    </div>
  );
}}
"#,
        req.name
    );

    fs::write(widgets_dir.join("MainWidget.jsx"), widget_content)?;

    let package_json = serde_json::json!({
        "name": req.id,
        "version": "1.0.0",
        "description": req.description,
        "author": req.author,
        "dependencies": {}
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

use webarcade_api::prelude::*;
use std::sync::Arc;

pub struct {}Plugin;

#[async_trait]
impl Plugin for {}Plugin {{
    plugin_metadata!("{}", "{}", "1.0.0", "{}", author: "{}");

    async fn init(&self, ctx: &Context) -> Result<()> {{
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

    async fn start(&self, _ctx: Arc<Context>) -> Result<()> {{
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
        r#"use webarcade_api::prelude::*;

pub async fn register_routes(ctx: &Context) -> Result<()> {{
    let mut router = Router::new();

    route!(router, GET "/hello" => handle_hello);

    ctx.register_router("{}", router).await;

    Ok(())
}}

async fn handle_hello() -> HttpResponse {{
    let response = json!({{
        "message": "Hello from {}!"
    }});

    json_response(&response)
}}
"#,
        req.id, req.name
    );

    fs::write(path.join("router.rs"), router_content)?;

    // Create Cargo.toml
    // Note: webarcade_api dependency is automatically added by the build system
    let cargo_toml = format!(
        r#"[package]
name = "{}"
version = "1.0.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]
path = "mod.rs"

[dependencies]
# webarcade_api is automatically injected by the plugin builder

[profile.release]
opt-level = "z"
lto = true
codegen-units = 1
strip = true
"#,
        req.id
    );

    fs::write(path.join("Cargo.toml"), cargo_toml)?;

    Ok(())
}

fn create_fullstack_template(path: &Path, req: &CreatePluginRequest) -> Result<()> {
    // Create Rust backend files
    let plugin_struct_name = capitalize_plugin_name(&req.id);
    let table_name = req.id.replace("-", "_");

    let mod_content = format!(
        r#"mod router;

use webarcade_api::prelude::*;
use std::sync::Arc;

pub struct {}Plugin;

#[async_trait]
impl Plugin for {}Plugin {{
    plugin_metadata!("{}", "{}", "1.0.0", "{}", author: "{}");

    async fn init(&self, ctx: &Context) -> Result<()> {{
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

    async fn start(&self, _ctx: Arc<Context>) -> Result<()> {{
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
        r#"use webarcade_api::prelude::*;

pub async fn register_routes(ctx: &Context) -> Result<()> {{
    let mut router = Router::new();

    route!(router, GET "/hello" => handle_hello);

    ctx.register_router("{}", router).await;

    Ok(())
}}

async fn handle_hello() -> HttpResponse {{
    let response = json!({{
        "message": "Hello from {}!"
    }});

    json_response(&response)
}}
"#,
        req.id, req.name
    );

    fs::write(path.join("router.rs"), router_content)?;

    // Create frontend files with widget
    let index_content = format!(
        r#"import {{ createPlugin }} from '@/api/plugin';
import {{ IconRocket }} from '@tabler/icons-solidjs';
import MyViewport from './viewport.jsx';
import MainWidget from './widgets/MainWidget.jsx';

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
      icon: IconRocket,
      description: '{}'
    }});

    api.menu('{}-menu', {{
      label: '{}',
      icon: IconRocket,
      onClick: () => {{
        api.open('{}-viewport', {{ label: '{}' }});
      }}
    }});

    // Register the widget
    api.widget('{}-widget', {{
      title: '{}',
      component: MainWidget,
      icon: IconRocket,
      description: '{}',
      defaultSize: {{ w: 2, h: 2 }},
      minSize: {{ w: 1, h: 1 }},
      maxSize: {{ w: 4, h: 4 }}
    }});

    console.log('[{}] Started successfully with backend and widget');
  }}
}});
"#,
        req.id, req.name, req.description, req.author,
        req.name, req.id, req.name, req.description,
        req.id, req.name, req.id, req.name,
        req.id, req.name, req.description,
        req.name
    );

    fs::write(path.join("index.jsx"), index_content)?;

    // Create viewport.jsx
    let viewport_content = format!(
        r#"import {{ createSignal, onMount }} from 'solid-js';
import {{ bridge }} from '@/api/bridge';

export default function MyViewport() {{
  const [message, setMessage] = createSignal('Loading...');

  onMount(async () => {{
    try {{
      const response = await bridge('/{}/hello');
      const data = await response.json();
      setMessage(data.message);
    }} catch (err) {{
      setMessage('Error loading data');
    }}
  }});

  return (
    <div class="h-full w-full flex flex-col bg-base-200 p-4">
      <h1 class="text-2xl font-bold mb-4">{}</h1>
      <p class="text-base-content/70 mb-4">{}</p>
      <div class="card bg-base-100 shadow-xl p-4">
        <p class="font-mono">{{message()}}</p>
      </div>
    </div>
  );
}}
"#,
        req.id, req.name, req.description
    );

    fs::write(path.join("viewport.jsx"), viewport_content)?;

    // Create widgets directory
    let widgets_dir = path.join("widgets");
    fs::create_dir_all(&widgets_dir)?;

    let widget_content = format!(
        r#"import {{ createSignal, onMount }} from 'solid-js';
import {{ IconRocket }} from '@tabler/icons-solidjs';
import {{ bridge }} from '@/api/bridge';

export default function MainWidget() {{
  const [message, setMessage] = createSignal('Loading...');
  const [loading, setLoading] = createSignal(true);

  const fetchData = async () => {{
    try {{
      setLoading(true);
      const response = await bridge('/{}/hello');
      const data = await response.json();
      setMessage(data.message);
    }} catch (err) {{
      setMessage('Error');
    }} finally {{
      setLoading(false);
    }}
  }};

  onMount(fetchData);

  return (
    <div class="card bg-gradient-to-br from-accent/20 to-accent/5 bg-base-100 shadow-lg h-full flex flex-col p-4">
      {{/* Header */}}
      <div class="flex items-center justify-between mb-2">
        <div class="flex items-center gap-2">
          <IconRocket size={{20}} class="text-accent opacity-80" />
          <span class="text-sm font-medium opacity-70">{}</span>
        </div>
      </div>

      {{/* Content */}}
      <div class="flex-1 flex flex-col items-center justify-center">
        {{loading() ? (
          <span class="loading loading-spinner loading-md text-accent"></span>
        ) : (
          <div class="text-center">
            <div class="text-lg font-medium mb-2">{{message()}}</div>
            <button
              class="btn btn-accent btn-sm"
              onClick={{fetchData}}
            >
              Refresh
            </button>
          </div>
        )}}
      </div>

      {{/* Footer */}}
      <div class="text-xs opacity-50 text-center mt-2">
        Backend API connected
      </div>
    </div>
  );
}}
"#,
        req.id, req.name
    );

    fs::write(widgets_dir.join("MainWidget.jsx"), widget_content)?;

    let package_json = serde_json::json!({
        "name": req.id,
        "version": "1.0.0",
        "description": req.description,
        "author": req.author,
        "dependencies": {}
    });

    fs::write(
        path.join("package.json"),
        serde_json::to_string_pretty(&package_json)?,
    )?;

    // Create Cargo.toml
    // Note: webarcade_api dependency is automatically added by the build system
    let cargo_toml = format!(
        r#"[package]
name = "{}"
version = "1.0.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]
path = "mod.rs"

[dependencies]
# webarcade_api is automatically injected by the plugin builder

[profile.release]
opt-level = "z"
lto = true
codegen-units = 1
strip = true
"#,
        req.id
    );

    fs::write(path.join("Cargo.toml"), cargo_toml)?;

    Ok(())
}

async fn handle_install_plugin(path: String, _req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    let plugin_id = path.trim_start_matches("/install/");

    let cwd = std::env::current_dir().unwrap();
    let project_root = cwd.parent().unwrap_or(&cwd);
    let dist_root = project_root.join("dist");
    let zip_path = dist_root.join(format!("{}.zip", plugin_id));

    if !zip_path.exists() {
        return error_response(StatusCode::NOT_FOUND, "Plugin zip file not found. Please build first.");
    }

    // Get AppData directory
    let appdata_dir = match dirs::data_local_dir() {
        Some(dir) => dir,
        None => return error_response(StatusCode::INTERNAL_SERVER_ERROR, "Could not find AppData directory"),
    };

    let plugins_dir = appdata_dir.join("WebArcade").join("plugins");
    let install_dir = plugins_dir.join(plugin_id);

    // Create plugins directory if it doesn't exist
    if let Err(e) = fs::create_dir_all(&plugins_dir) {
        return error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to create plugins directory: {}", e));
    }

    // Remove existing installation if present
    if install_dir.exists() {
        if let Err(e) = fs::remove_dir_all(&install_dir) {
            return error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to remove old installation: {}", e));
        }
    }

    // Create install directory
    if let Err(e) = fs::create_dir_all(&install_dir) {
        return error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to create install directory: {}", e));
    }

    // Extract zip file
    let zip_file = match fs::File::open(&zip_path) {
        Ok(file) => file,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to open zip file: {}", e)),
    };

    let mut archive = match zip::ZipArchive::new(zip_file) {
        Ok(archive) => archive,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to read zip archive: {}", e)),
    };

    for i in 0..archive.len() {
        let mut file = match archive.by_index(i) {
            Ok(file) => file,
            Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to read file from archive: {}", e)),
        };

        let outpath = match file.enclosed_name() {
            Some(path) => install_dir.join(path),
            None => continue,
        };

        if file.name().ends_with('/') {
            if let Err(e) = fs::create_dir_all(&outpath) {
                return error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to create directory: {}", e));
            }
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    if let Err(e) = fs::create_dir_all(p) {
                        return error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to create parent directory: {}", e));
                    }
                }
            }
            let mut outfile = match fs::File::create(&outpath) {
                Ok(f) => f,
                Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to create file: {}", e)),
            };
            if let Err(e) = std::io::copy(&mut file, &mut outfile) {
                return error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to extract file: {}", e));
            }
        }
    }

    log::info!("[Developer] Plugin {} installed to {:?}", plugin_id, install_dir);

    let response = serde_json::json!({
        "success": true,
        "plugin_id": plugin_id,
        "install_path": install_dir.to_string_lossy()
    });

    json_response(&response)
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
