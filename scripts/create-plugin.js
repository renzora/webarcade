#!/usr/bin/env node

const fs = require('fs');
const path = require('path');

// Get plugin name from command line arguments
const pluginName = process.argv[2];

if (!pluginName) {
  console.error('Usage: bun run scripts/create-plugin.js <plugin-name>');
  console.error('Example: bun run scripts/create-plugin.js my-plugin');
  process.exit(1);
}

// Validate plugin name (kebab-case)
if (!/^[a-z][a-z0-9-]*$/.test(pluginName)) {
  console.error('Error: Plugin name must be in kebab-case (lowercase letters, numbers, and hyphens)');
  console.error('Example: my-plugin, stats-counter, twitch-integration');
  process.exit(1);
}

const pluginDir = path.join(__dirname, '..', 'plugins', pluginName);

// Check if plugin already exists
if (fs.existsSync(pluginDir)) {
  console.error(`Error: Plugin "${pluginName}" already exists at ${pluginDir}`);
  process.exit(1);
}

// Convert kebab-case to PascalCase for class names
const toPascalCase = (str) => {
  return str
    .split('-')
    .map(word => word.charAt(0).toUpperCase() + word.slice(1))
    .join('');
};

// Convert kebab-case to Title Case for display names
const toTitleCase = (str) => {
  return str
    .split('-')
    .map(word => word.charAt(0).toUpperCase() + word.slice(1))
    .join(' ');
};

const pascalName = toPascalCase(pluginName);
const titleName = toTitleCase(pluginName);

// Create plugin directory
console.log(`Creating plugin: ${pluginName}`);
fs.mkdirSync(pluginDir, { recursive: true });

// Frontend template (index.jsx)
const frontendTemplate = `import { createPlugin } from '@/api/plugin';
import { IconPlugConnected } from '@tabler/icons-solidjs';
import ${pascalName}Viewport from './viewport.jsx';

export default createPlugin({
  id: '${pluginName}',
  name: '${titleName}',
  version: '1.0.0',
  description: '${titleName} plugin for WebArcade',
  author: 'WebArcade Team',

  async onInit(api) {
    console.log('[${titleName} Plugin] Initializing...');
    // Initialize plugin state, load data, etc.
  },

  async onStart(api) {
    console.log('[${titleName} Plugin] Starting...');

    // Register viewport
    api.viewport('${pluginName}-viewport', {
      label: '${titleName}',
      component: ${pascalName}Viewport,
      icon: IconPlugConnected,
      description: '${titleName} viewport'
    });

    // Optionally register a property tab
    // api.tab('${pluginName}-tab', {
    //   title: '${titleName}',
    //   component: ${pascalName}Panel,
    //   icon: IconPlugConnected,
    //   order: 100
    // });

    console.log('[${titleName} Plugin] Started successfully');
  },

  onUpdate() {
    // Called every frame (~60 FPS)
    // Use for animations, polling, etc.
  },

  async onStop() {
    console.log('[${titleName} Plugin] Stopping...');
  },

  async onDispose() {
    console.log('[${titleName} Plugin] Disposing...');
  }
});
`;

// Viewport template
const viewportTemplate = `import { createSignal } from 'solid-js';

export default function ${pascalName}Viewport() {
  const [count, setCount] = createSignal(0);

  return (
    <div style={{
      display: 'flex',
      'flex-direction': 'column',
      'align-items': 'center',
      'justify-content': 'center',
      height: '100%',
      padding: '20px',
      gap: '20px'
    }}>
      <h1>${titleName}</h1>
      <p>Welcome to your new plugin!</p>

      <div style={{ 'text-align': 'center' }}>
        <p>Counter: {count()}</p>
        <button
          onClick={() => setCount(count() + 1)}
          style={{
            padding: '10px 20px',
            'font-size': '16px',
            cursor: 'pointer',
            'background-color': '#007bff',
            color: 'white',
            border: 'none',
            'border-radius': '4px'
          }}
        >
          Increment
        </button>
      </div>

      <div style={{ 'margin-top': '20px', 'font-size': '14px', color: '#666' }}>
        <p>Edit <code>plugins/${pluginName}/viewport.jsx</code> to customize this view</p>
        <p>Edit <code>plugins/${pluginName}/index.jsx</code> to add more functionality</p>
        <p>Edit <code>plugins/${pluginName}/mod.rs</code> to add backend endpoints</p>
      </div>
    </div>
  );
}
`;

// Backend template (mod.rs)
const backendModTemplate = `mod router;

use crate::core::plugin::{Plugin, PluginMetadata};
use crate::core::plugin_context::PluginContext;
use crate::plugin_metadata;
use async_trait::async_trait;
use std::sync::Arc;
use anyhow::Result;

pub struct ${pascalName}Plugin;

#[async_trait]
impl Plugin for ${pascalName}Plugin {
    plugin_metadata!("${pluginName}", "${titleName}", "1.0.0", "${titleName} plugin for WebArcade");

    async fn init(&self, ctx: &PluginContext) -> Result<()> {
        log::info!("[${titleName}] Initializing plugin...");

        // Run database migrations if needed
        // ctx.migrate(&[
        //     r#"
        //     CREATE TABLE IF NOT EXISTS ${pluginName.replace(/-/g, '_')}_data (
        //         id INTEGER PRIMARY KEY AUTOINCREMENT,
        //         data TEXT NOT NULL,
        //         created_at INTEGER NOT NULL
        //     )
        //     "#,
        // ])?;

        // Register HTTP routes
        router::register_routes(ctx).await?;

        log::info!("[${titleName}] Plugin initialized successfully");
        Ok(())
    }

    async fn start(&self, _ctx: Arc<PluginContext>) -> Result<()> {
        log::info!("[${titleName}] Starting plugin...");

        // Start background tasks if needed
        // tokio::spawn(async move {
        //     // Background work
        // });

        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        log::info!("[${titleName}] Stopping plugin...");
        Ok(())
    }
}
`;

// Backend router template
const backendRouterTemplate = `use crate::core::plugin_context::PluginContext;
use crate::core::plugin_router::PluginRouter;
use crate::core::router_utils::*;
use crate::route;
use anyhow::Result;
use hyper::{Request, Response, StatusCode, body::Incoming};
use hyper::body::Bytes;
use http_body_util::combinators::BoxBody;
use std::convert::Infallible;

pub async fn register_routes(ctx: &PluginContext) -> Result<()> {
    let mut router = PluginRouter::new();

    // Example GET route
    route!(router, GET "/hello" => handle_hello);

    // Example POST route
    route!(router, POST "/data" => handle_post_data);

    ctx.register_router("${pluginName}", router).await;
    Ok(())
}

async fn handle_hello() -> Response<BoxBody<Bytes, Infallible>> {
    json_response(&serde_json::json!({
        "message": "Hello from ${titleName} plugin!",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

async fn handle_post_data(req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    match read_json_body(req).await {
        Ok(body) => {
            log::info!("[${titleName}] Received data: {:?}", body);

            // Process the data here
            // For example, save to database:
            // ctx.execute("INSERT INTO ${pluginName.replace(/-/g, '_')}_data (data, created_at) VALUES (?1, ?2)",
            //             rusqlite::params![body.to_string(), chrono::Utc::now().timestamp()])?;

            json_response(&serde_json::json!({
                "success": true,
                "message": "Data received successfully"
            }))
        }
        Err(e) => error_response(StatusCode::BAD_REQUEST, &e)
    }
}
`;

// README template
const readmeTemplate = `# ${titleName} Plugin

${titleName} plugin for WebArcade.

## Features

- Example viewport with counter
- Backend HTTP endpoints
- Database integration (commented out)

## File Structure

- \`index.jsx\` - Frontend plugin entry point
- \`viewport.jsx\` - Main viewport component
- \`mod.rs\` - Backend plugin module
- \`router.rs\` - Backend HTTP routes

## Development

### Frontend
The frontend uses SolidJS. Edit \`viewport.jsx\` to customize the UI.

### Backend
The backend uses Rust. Edit \`router.rs\` to add HTTP endpoints.

### API Endpoints

- \`GET /api/${pluginName}/hello\` - Returns a hello message
- \`POST /api/${pluginName}/data\` - Accepts JSON data

## Usage

The plugin is automatically discovered and loaded by WebArcade.

To open the viewport, use the viewport selector or call:
\`\`\`javascript
api.open('${pluginName}-viewport');
\`\`\`
`;

// Write files
console.log('Creating files...');
fs.writeFileSync(path.join(pluginDir, 'index.jsx'), frontendTemplate);
fs.writeFileSync(path.join(pluginDir, 'viewport.jsx'), viewportTemplate);
fs.writeFileSync(path.join(pluginDir, 'mod.rs'), backendModTemplate);
fs.writeFileSync(path.join(pluginDir, 'router.rs'), backendRouterTemplate);
fs.writeFileSync(path.join(pluginDir, 'README.md'), readmeTemplate);

console.log('✅ Plugin scaffolding created successfully!');
console.log('');
console.log('Plugin created at:', pluginDir);
console.log('');
console.log('Next steps:');
console.log('  1. Run plugin discovery scripts:');
console.log('     bun run scripts/discover-plugins.js');
console.log('     bun run scripts/discover-backend-plugins.js');
console.log('');
console.log('  2. Restart the development server to load the plugin');
console.log('');
console.log('  3. Customize your plugin:');
console.log('     - Edit plugins/' + pluginName + '/index.jsx for frontend logic');
console.log('     - Edit plugins/' + pluginName + '/viewport.jsx for UI');
console.log('     - Edit plugins/' + pluginName + '/mod.rs for backend logic');
console.log('     - Edit plugins/' + pluginName + '/router.rs for HTTP endpoints');
console.log('');
console.log('Happy coding! 🚀');
