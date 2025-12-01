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

After building, you'll find:
- **Installer:** `src-tauri/target/release/bundle/nsis/` (Windows `.exe` installer)
- **Binary:** `src-tauri/target/release/Webarcade.exe` (standalone executable)
- **Plugins:** `src-tauri/target/release/plugins/` (bundled DLLs)

### Available Scripts

| Script | Description |
|--------|-------------|
| `bun run dev` | Start development server with hot reload |
| `bun run build` | Build production installer |
| `bun run plugin:new <id>` | Create a new plugin project |
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
├── plugins/               # Plugin SOURCE code only
│   └── my-plugin/         # Source directory (index.jsx, mod.rs, etc.)
├── build/
│   └── plugins/           # Built plugins (development)
│       ├── my-plugin.js   # Frontend-only plugin
│       └── other.dll      # Full-stack plugin
├── cli/                   # Plugin build CLI tool
└── scripts/               # Build scripts
```

### Plugin Layout

Source code and built output are now **separated**:

**Source (plugins/):**
```
plugins/
├── my-plugin/              # Source directory
│   ├── index.jsx           # Frontend entry (required)
│   ├── mod.rs              # Backend entry (optional)
│   ├── router.rs           # Route handlers (optional)
│   └── Cargo.toml          # Routes & deps (optional)
└── other-plugin/
    └── index.jsx           # Frontend-only plugin
```

**Built Output (build/plugins/):**
```
build/plugins/
├── my-plugin.dll           # Full-stack plugin (has Rust backend)
├── other-plugin.js         # Frontend-only plugin (just JavaScript)
└── ...
```

**Production (bundled app):**
```
{app}/plugins/
├── my-plugin.dll           # Copied from build/plugins/
├── other-plugin.js
└── ...
```

**Flow:** Edit `plugins/my-plugin/` → `bun run plugin:build my-plugin` → Creates `build/plugins/my-plugin.js` or `.dll`

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

### Quick Start

Create a new plugin with the CLI:

```bash
# Create a full-stack plugin (frontend + Rust backend)
bun run plugin:new my-plugin

# Create with custom name and author
bun run plugin:new my-plugin --name "My Plugin" --author "Your Name"

# Create frontend-only plugin (no Rust backend)
bun run plugin:new my-plugin --frontend-only
```

This generates all boilerplate files in `plugins/my-plugin/`.

### Plugin Structure

**Frontend-only plugin** (no Rust backend):
```
plugins/my-plugin/
├── index.jsx               # Frontend entry (required)
└── viewport.jsx            # UI components (optional)
```
→ Builds to: `build/plugins/my-plugin.js` (~5-15 KB)

**Full-stack plugin** (with Rust backend):
```
plugins/my-plugin/
├── index.jsx               # Frontend entry (required)
├── viewport.jsx            # UI components (optional)
├── Cargo.toml              # Routes and dependencies
├── mod.rs                  # Plugin entry point
└── router.rs               # HTTP handlers
```
→ Builds to: `build/plugins/my-plugin.dll` (~200+ KB)

> **Note:** `index.jsx` is required - it identifies the directory as a plugin. If `mod.rs` + `Cargo.toml` exist, it's a full-stack plugin; otherwise it's frontend-only.

### Manual Setup

If you prefer to create files manually:

#### Step 1: Create Cargo.toml

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

#### Step 2: Create mod.rs

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

#### Step 3: Create router.rs

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

#### Step 4: Create index.jsx

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

#### Step 5: Build and Test

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

// App fullscreen mode (hides UI elements)
api.showFullscreen(true);   // Enter app fullscreen
api.hideFullscreen();       // Exit app fullscreen
api.toggleFullscreen();     // Toggle app fullscreen
api.getFullscreen();        // Get current state

// Bulk visibility controls
api.showAll();              // Show all panels
api.hideAll();              // Hide all panels
```

> **Note:** When switching viewports, all panels are hidden by default. Use `onActivate` in your viewport registration to show the panels your plugin needs.

#### Window Controls

```jsx
// Window size
await api.setWindowSize(1280, 720);          // Set window dimensions
const size = await api.getWindowSize();       // Returns { width, height }

// Window position
await api.setWindowPosition(100, 100);        // Set window position
const pos = await api.getWindowPosition();    // Returns { x, y }

// Size constraints
await api.setWindowMinSize(800, 600);         // Set minimum size
await api.setWindowMaxSize(1920, 1080);       // Set maximum size

// Window state
await api.maximizeWindow();                   // Maximize window
await api.minimizeWindow();                   // Minimize window
await api.unmaximizeWindow();                 // Restore from maximized
await api.centerWindow();                     // Center on screen
await api.fullscreen(true);                   // Enter fullscreen
await api.fullscreen(false);                  // Exit fullscreen

// Window title
await api.setWindowTitle('My App - Untitled');

// Exit application
await api.exit();                             // Close the Tauri app
```

> **Note:** Window control methods are async and interact with Tauri's window API. They only work when running in the Tauri desktop environment.

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

### Plugin Types

**Frontend-only plugins** compile to a single `.js` file:
- Just bundled JavaScript (~5-15 KB)
- No Rust compilation needed
- Fast builds (~1 second)

**Full-stack plugins** compile to a single `.dll` file containing:
- Compiled Rust backend code
- Bundled frontend JavaScript (embedded)
- Plugin manifest (embedded)

### Build Process

1. **Source** (`plugins/my-plugin/`)
   - Write `index.jsx` (required)
   - Optionally add `mod.rs`, `router.rs`, `Cargo.toml` for backend

2. **CLI Build** (`bun run plugin:build my-plugin`)
   - Bundles frontend with RSpack
   - **Frontend-only**: Outputs `build/plugins/my-plugin.js`
   - **Full-stack**: Embeds JS into DLL, outputs `build/plugins/my-plugin.dll`

3. **Output Location**
   - Development: `build/plugins/` (app loads from here)
   - Production: `{app}/plugins/` (Tauri bundles from build/plugins)

### Runtime Loading

The loader scans `build/plugins/` (dev) or `plugins/` (prod) for:

| File Type | Plugin Type | Loading Method |
|-----------|-------------|----------------|
| `*.js` | Frontend-only | Direct file read |
| `*.dll` | Full-stack | FFI extraction from DLL |

1. **Backend**: Bridge scans plugins directory
   - Finds `.js` files → registers as frontend-only
   - Finds `.dll` files → loads via FFI, extracts manifest & routes

2. **Frontend**: Plugin loader fetches plugin list
   - Frontend-only: Serves JS file directly
   - Full-stack: Extracts JS from DLL and serves it
   - Calls `onStart(api)` for initialization

### FFI Functions (DLL plugins only)

| Function | Returns | Description |
|----------|---------|-------------|
| `get_plugin_manifest()` | `*const u8` | Embedded package.json |
| `get_plugin_frontend()` | `*const u8` | Embedded plugin.js |
| `has_frontend()` | `bool` | Whether plugin has frontend |
| `{handler_name}()` | Response | Route handlers |

### Request Flow (Full-stack plugins)

```
Frontend fetch()
    ↓
HTTP Bridge (port 3001)
    ↓
Route matching (from embedded manifest)
    ↓
FFI call to DLL handler
    ↓
Your handler function
    ↓
Response back to frontend
```

---

## CLI Reference

### Commands

```bash
# Create a new plugin
bun run plugin:new my-plugin
bun run plugin:new my-plugin --name "My Plugin" --author "You"
bun run plugin:new my-plugin --frontend-only

# Build plugins (outputs to dist/plugins/)
bun run plugin:build my-plugin    # Build specific plugin
bun run plugin:build --all        # Build all plugins

# List available plugins
bun run plugin:list
```

### Build Output

| Plugin Type | Input | Output |
|-------------|-------|--------|
| Frontend-only | `plugins/foo/index.jsx` | `build/plugins/foo.js` |
| Full-stack | `plugins/foo/` (with mod.rs + Cargo.toml) | `build/plugins/foo.dll` |

### Direct CLI Usage

```bash
cd cli && cargo run --release -- new my-plugin
cd cli && cargo run --release -- build my-plugin
cd cli && cargo run --release -- build --all
cd cli && cargo run --release -- list
```

---

## Troubleshooting

### Common Errors

**"Plugin not detected"**
- Ensure `index.jsx` exists in the plugin source directory

**"Handler not found"**
- Check route names in `Cargo.toml` match function names exactly
- Ensure handlers are `pub`

**"Build failed"**
- Check Rust syntax errors
- Ensure handler signature is correct: `pub async fn name(req: HttpRequest) -> HttpResponse`

**"DLL won't reload"**
- Restart the app - DLLs are locked while loaded

**"Routes not working"**
- Verify `Cargo.toml` routes format: `"METHOD /path" = "handler_name"`
- Rebuild the plugin after changes

### Development Tips

1. Use `bun run dev:verbose` for detailed logs
2. Check browser DevTools for frontend errors
3. Plugin changes require rebuild: `bun run plugin:build <plugin-name>`
4. Source code stays in `plugins/`, built output goes to `build/plugins/`
5. App loads plugins from `build/plugins/` in dev mode
6. Frontend-only plugins build instantly (~1s), full-stack takes longer (~10-30s)

---

## License

MIT License
