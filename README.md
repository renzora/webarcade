# WebArcade

A full-stack plugin platform for building native desktop applications with **SolidJS** (frontend) and **Rust** (backend).

## Table of Contents

1. [Getting Started](#getting-started)
2. [Project Structure](#project-structure)
3. [Core Development](#core-development)
4. [Plugin Development](#plugin-development)
5. [Plugin API Reference](#plugin-api-reference)
6. [Bridge API Reference](#bridge-api-reference)
7. [How Plugins Work](#how-plugins-work)
8. [CLI Reference](#cli-reference)
9. [Troubleshooting](#troubleshooting)

---

## Getting Started

### Prerequisites

Install the following before starting:

1. **Rust** - https://rustup.rs/
2. **Bun** - https://bun.sh/

### Installation

```bash
git clone https://github.com/renzora/webarcade.git
cd webarcade
bun install
```

### Running the App

```bash
# Development mode (hot reload)
bun run dev

# Production build
bun run build
```

### Available Scripts

| Script | Description |
|--------|-------------|
| `bun run dev` | Start development server with hot reload |
| `bun run build` | Build production installer |
| `bun run plugin:build` | Build all plugins |
| `bun run plugin:list` | List available plugins |

---

## Project Structure

```
webarcade/
├── src/                    # Frontend (SolidJS)
│   ├── api/               # Bridge and plugin APIs
│   ├── components/        # UI components
│   └── panels/            # Panel components
├── src-tauri/             # Backend (Rust/Tauri)
│   ├── src/bridge/        # HTTP bridge and plugin loader
│   └── api/               # Plugin API crate
├── projects/              # Plugin source code (development)
├── plugins/               # Compiled plugins (runtime)
├── cli/                   # Plugin build CLI tool
└── scripts/               # Build scripts
```

### Projects vs Plugins

| | Projects | Plugins |
|--|----------|---------|
| **Location** | `projects/` | `plugins/` |
| **Contents** | Source files (.rs, .jsx) | Compiled files (.dll, .js) |
| **Purpose** | Development | Runtime |

**Flow:** Edit in `projects/` → Build with CLI → Output to `plugins/` → Loaded at runtime

---

## Core Development

### Frontend Stack

- **SolidJS** - Reactive UI framework
- **Tailwind CSS** - Styling
- **DaisyUI** - Component library
- **RSpack** - Bundler

### Backend Stack

- **Tauri** - Desktop framework
- **Rust** - System programming
- **Tokio** - Async runtime

### Bridge Architecture

The bridge connects frontend and backend via HTTP:

```
Frontend (SolidJS)
    ↓ fetch()
HTTP Bridge (localhost:3001)
    ↓ route matching
Plugin DLL (FFI call)
    ↓ response
Frontend
```

---

## Plugin Development

### Plugin Structure

```
my-plugin/
├── Cargo.toml      # Routes and metadata
├── mod.rs          # Plugin entry point
├── router.rs       # HTTP handlers
├── index.jsx       # Frontend entry (required)
└── viewport.jsx    # UI component (optional)
```

> **Note:** `index.jsx` is required - it identifies the directory as a plugin.

### Step 1: Create Cargo.toml

Define your plugin metadata and routes:

```toml
[package]
name = "my-plugin"
version = "1.0.0"
edition = "2021"

[routes]
"GET /hello" = "handle_hello"
"GET /items/:id" = "handle_get_item"
"POST /items" = "handle_create_item"

[profile.release]
opt-level = "z"
lto = true
```

### Step 2: Create mod.rs

```rust
pub mod router;

use api::{Plugin, PluginMetadata};

pub struct MyPlugin;

impl Plugin for MyPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            id: "my-plugin".into(),
            name: "My Plugin".into(),
            version: "1.0.0".into(),
            description: "Plugin description".into(),
            author: "Your Name".into(),
            dependencies: vec![],
        }
    }
}
```

### Step 3: Create router.rs

```rust
use api::{HttpRequest, HttpResponse, json, json_response, error_response};

pub async fn handle_hello(req: HttpRequest) -> HttpResponse {
    json_response(&json!({
        "message": "Hello from my plugin!"
    }))
}

pub async fn handle_get_item(req: HttpRequest) -> HttpResponse {
    let id = req.path_params.get("id").cloned().unwrap_or_default();

    json_response(&json!({
        "id": id,
        "name": "Example Item"
    }))
}
```

### Step 4: Create index.jsx

```jsx
import { createPlugin } from '@/api/plugin';
import Viewport from './viewport';

export default createPlugin({
    id: 'my-plugin',
    name: 'My Plugin',
    version: '1.0.0',

    async onStart(api) {
        api.viewport('my-viewport', {
            label: 'My Plugin',
            component: Viewport
        });

        api.menu('my-menu', {
            label: 'My Plugin',
            onClick: () => api.open('my-viewport')
        });
    }
});
```

### Step 5: Build and Test

```bash
# Build the plugin
bun run plugin:build my-plugin

# Or build all plugins
bun run plugin:build
```

---

## Plugin API Reference

### Frontend Plugin API

#### Registration Methods

```jsx
// Register a viewport (main view)
api.viewport('viewport-id', {
    label: 'Tab Label',
    component: MyComponent,
    icon: IconComponent,
    description: 'Description'
});

// Add menu item
api.menu('menu-id', {
    label: 'Menu Label',
    icon: IconComponent,
    onClick: () => api.open('viewport-id')
});

// Register panel content
api.leftPanel({ component: LeftPanelComponent });
api.rightPanel({ component: RightPanelComponent });

// Add bottom panel tab
api.bottomTab('tab-id', {
    title: 'Tab Title',
    component: TabComponent,
    icon: IconComponent
});

// Add toolbar button
api.toolbar('tool-id', {
    icon: IconTool,
    label: 'Tool',
    tooltip: 'Tool description',
    onClick: () => {},
    group: 'tools',
    order: 10
});
```

#### UI Visibility Controls

```jsx
api.showProps(true);        // Right panel
api.showLeftPanel(true);    // Left panel
api.showMenu(true);         // Top menu
api.showFooter(true);       // Footer bar
api.showTabs(true);         // Viewport tabs
api.showBottomPanel(true);  // Bottom panel
api.showToolbar(true);      // Toolbar
```

### Calling Backend from Frontend

```jsx
import { api } from '@/api/bridge';

// GET request
const response = await api('my-plugin/hello');
const data = await response.json();

// POST request
const response = await api('my-plugin/items', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ name: 'New Item' })
});

// With path parameters
const response = await api('my-plugin/items/123');

// With query parameters
const response = await api('my-plugin/search?q=test&limit=10');
```

---

## Bridge API Reference

### Rust Imports

```rust
use api::{
    HttpRequest,           // Request type
    HttpResponse,          // Response type
    json,                  // json!() macro
    json_response,         // Create JSON 200 response
    error_response,        // Create error response
    Serialize,             // Serde trait
    Deserialize,           // Serde trait
    Bytes,                 // For binary responses
};
```

### HttpRequest Methods

| Method | Returns | Description |
|--------|---------|-------------|
| `query("name")` | `Option<String>` | Get query parameter |
| `path_params.get("name")` | `Option<&String>` | Get path parameter |
| `header("name")` | `Option<&String>` | Get header |
| `body_bytes()` | `&[u8]` | Raw body bytes |
| `body_string()` | `Result<String>` | Body as UTF-8 string |
| `body_json<T>()` | `Result<T>` | Parse body as JSON |
| `is_multipart()` | `bool` | Check if multipart form |
| `parse_multipart()` | `Result<Vec<MultipartField>>` | Parse file uploads |

### Creating Responses

```rust
// JSON success (200)
json_response(&json!({"status": "ok"}))

// JSON with struct
#[derive(Serialize)]
struct User { id: u32, name: String }
json_response(&User { id: 1, name: "Alice".into() })

// Error responses
error_response(400, "Bad request")
error_response(404, "Not found")
error_response(500, "Server error")
```

### Handler Pattern

All handlers must follow this signature:

```rust
pub async fn handler_name(req: HttpRequest) -> HttpResponse {
    // Your code here
}
```

### Route Patterns

```toml
[routes]
"GET /path" = "handler"           # Exact match
"GET /items/:id" = "get_item"     # Path parameter
"POST /upload" = "upload"         # POST method
"DELETE /items/:id" = "delete"    # DELETE method
```

---

## How Plugins Work

### Build Process

1. **Source** (`projects/my-plugin/`)
   - You write `mod.rs`, `router.rs`, `index.jsx`
   - Define routes in `Cargo.toml`

2. **CLI Build** (`bun run plugin:build my-plugin`)
   - Generates FFI wrapper code (`lib.rs`)
   - Compiles Rust to platform binary (.dll/.so/.dylib)
   - Bundles frontend with RSpack

3. **Output** (`plugins/my-plugin/`)
   - `my-plugin.dll` - Compiled backend
   - `plugin.js` - Bundled frontend
   - `package.json` - Plugin manifest

### Runtime Loading

1. **Backend**: Bridge scans `plugins/` directory
   - Loads DLL via `libloading`
   - Registers routes from `package.json`
   - FFI calls invoke handler functions

2. **Frontend**: Plugin loader fetches plugin list
   - Dynamic imports `plugin.js`
   - Calls `onStart(api)` for initialization

### FFI Bridge

The CLI generates FFI wrappers that:
- Convert JSON requests to `HttpRequest`
- Call your async handler functions
- Convert `HttpResponse` back to JSON
- Handle errors and panics gracefully

```
Frontend fetch()
    ↓
HTTP Bridge (port 3001)
    ↓
Route matching
    ↓
FFI call to DLL
    ↓
Your handler function
    ↓
Response back to frontend
```

---

## CLI Reference

### Build Commands

```bash
# Build specific plugin
cd cli && cargo run --release -- build my-plugin

# Build all plugins
cd cli && cargo run --release -- build --all

# List available plugins
cd cli && cargo run --release -- list
```

### NPM Scripts

```bash
bun run plugin:build          # Build all plugins
bun run plugin:list           # List plugins
bun run cli:build             # Build CLI binary
```

---

## Troubleshooting

### Common Errors

**"Plugin not detected"**
- Ensure `index.jsx` exists in the plugin directory

**"Handler not found"**
- Check route names in `Cargo.toml` match function names exactly
- Ensure handlers are `pub`

**"Build failed"**
- Check Rust syntax errors
- Ensure handler signature is correct: `pub async fn name(req: HttpRequest) -> HttpResponse`

**"DLL won't reload"**
- Restart the app if the file is locked

**"Routes not working"**
- Verify `Cargo.toml` routes format: `"METHOD /path" = "handler_name"`

### Development Tips

1. Use `bun run dev:verbose` for detailed logs
2. Check browser DevTools for frontend errors
3. Plugin changes require rebuild: `bun run plugin:build`

---

## License

MIT License
