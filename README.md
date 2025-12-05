# WebArcade

A lightweight plugin platform for building native desktop applications with **SolidJS** (frontend) and **Rust** (backend).

## Table of Contents

1. [Getting Started](#getting-started)
2. [Plugin Modes](#plugin-modes)
3. [App Configuration](#app-configuration)
4. [Project Structure](#project-structure)
5. [Plugin Development](#plugin-development)
6. [Plugin API Reference](#plugin-api-reference)
7. [Bridge API Reference](#bridge-api-reference)
8. [CLI Reference](#cli-reference)
9. [Troubleshooting](#troubleshooting)

---

## Getting Started

### Prerequisites

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
# Build frontend only
bun run build

# Build production app with installer
bun run app

# Build locked app (plugins embedded in binary)
bun run app:locked

# Run the built app
bun run app:run
```

### Available Scripts

| Script | Description |
|--------|-------------|
| `bun run build` | Build frontend to `app/dist/` |
| `bun run build:prod` | Build frontend for production |
| `bun run app` | Build app + installer |
| `bun run app:locked` | Build app with embedded plugins |
| `bun run app:run` | Run the built executable |
| `bun run plugin:new <id>` | Create a new plugin |
| `bun run plugin:build` | Build all plugins |
| `bun run plugin:list` | List available plugins |

---

## Plugin Modes

### Unlocked Mode (Default)

```bash
bun run app
```

- Plugins loaded from `plugins/` folder
- Users can add/remove plugins after installation

### Locked Mode

```bash
bun run app:locked
```

- All plugins embedded in binary
- Single executable, no external files

---

## App Configuration

### Customizing the App

Edit `app/Cargo.toml`:

```toml
[package]
name = "MyApp"              # Executable filename
version = "1.0.0"
description = "My App"

[package.metadata.packager]
product-name = "My App"
identifier = "com.myapp"
icons = ["icon.ico", "icon.png"]
```

### App Icon

Place `icon.png` in `app/` directory - the build script converts it to `icon.ico` automatically.

---

## Project Structure

```
webarcade/
├── src/                    # Frontend (SolidJS)
│   ├── api/               # Plugin and Bridge APIs
│   ├── panels/            # Unified panel system
│   └── layout/            # Main layout
├── app/                    # Desktop runtime (Rust)
│   ├── src/               # Runtime source
│   ├── dist/              # Built frontend
│   └── plugins/           # Production plugins
├── plugins/               # Plugin source code
├── build/plugins/         # Built plugins (development)
└── cli/                   # Plugin build CLI
```

---

## Plugin Development

### Quick Start

```bash
# Create a frontend-only plugin
bun run plugin:new my-plugin --frontend-only

# Create a full-stack plugin (with Rust backend)
bun run plugin:new my-plugin

# Build the plugin
bun run plugin:build my-plugin
```

### Plugin Structure

**Frontend-only plugin:**
```
plugins/my-plugin/
├── index.jsx           # Plugin entry (required)
└── viewport.jsx        # Components (optional)
```
→ Builds to: `build/plugins/my-plugin.js`

**Full-stack plugin:**
```
plugins/my-plugin/
├── index.jsx           # Frontend entry
├── viewport.jsx        # Components
├── Cargo.toml          # Routes & dependencies
├── mod.rs              # Plugin metadata
└── router.rs           # HTTP handlers
```
→ Builds to: `build/plugins/my-plugin.dll`

### Basic Plugin Example

```jsx
import { plugin } from '@/api/plugin';

export default plugin({
    id: 'my-plugin',
    name: 'My Plugin',
    version: '1.0.0',

    start(api) {
        // Register plugin tab (shows in main tab bar)
        api.add({
            panel: 'tab',
            label: 'My Plugin',
            icon: MyIcon,
        });

        // Register main viewport
        api.add({
            panel: 'viewport',
            id: 'main',
            component: MainView,
        });

        // Register left panel tab
        api.add({
            panel: 'left',
            id: 'explorer',
            label: 'Explorer',
            component: Explorer,
        });

        // Register right panel tab
        api.add({
            panel: 'right',
            id: 'properties',
            label: 'Properties',
            component: Properties,
        });

        // Register bottom panel tab
        api.add({
            panel: 'bottom',
            id: 'console',
            label: 'Console',
            component: Console,
        });
    },

    active(api) {
        // Called when plugin becomes active
    },

    inactive(api) {
        // Called when plugin becomes inactive
    },

    stop(api) {
        // Called when plugin is stopped
    }
});
```

---

## Plugin API Reference

### The `api.add()` Method

Register components to different panel locations:

```jsx
api.add({
    panel: 'tab' | 'viewport' | 'left' | 'right' | 'bottom',
    id: 'unique-id',           // Required for viewport/panels
    label: 'Display Label',    // Tab label
    icon: IconComponent,       // Optional icon
    component: MyComponent,    // SolidJS component
    visible: true,             // Initial visibility (default: true)
    shared: false,             // Allow other plugins to use (default: false)
    order: 0,                  // Sort order
    closable: true,            // Can user close tab (default: true)
    start: (api) => {},        // First time mounted
    active: (api) => {},       // Plugin became active
    inactive: (api) => {},     // Plugin became inactive
});
```

### Panel Types

| Panel | Description |
|-------|-------------|
| `tab` | Main plugin tab bar (one per plugin) |
| `viewport` | Main content area (supports multiple tabs) |
| `left` | Left sidebar (supports multiple tabs) |
| `right` | Right sidebar (supports multiple tabs) |
| `bottom` | Bottom panel (supports multiple tabs) |

### Panel Visibility

```jsx
api.showLeft(true);     // Show left panel
api.showRight(true);    // Show right panel
api.showBottom(true);   // Show bottom panel

api.hideLeft();         // Hide left panel
api.hideRight();        // Hide right panel
api.hideBottom();       // Hide bottom panel

api.toggleLeft();       // Toggle left panel
api.toggleRight();      // Toggle right panel
api.toggleBottom();     // Toggle bottom panel
```

### Shared Panels

Panels can be shared between plugins:

```jsx
// Plugin A - registers a shared panel
api.add({
    panel: 'left',
    id: 'file-explorer',
    label: 'Files',
    component: FileExplorer,
    shared: true,  // Other plugins can use this
});

// Plugin B - uses Plugin A's panel
const sharedPanel = api.useShared('plugin-a:file-explorer');
```

### Remove Components

```jsx
api.remove('component-id');  // Remove by ID
```

### Window Controls

```jsx
await api.setWindowSize(1280, 720);
await api.setWindowPosition(100, 100);
await api.maximizeWindow();
await api.minimizeWindow();
await api.fullscreen(true);
await api.setWindowTitle('My App');
await api.exit();
```

---

## Bridge API Reference

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
```

### Rust Backend

**Cargo.toml:**
```toml
[package]
name = "my-plugin"
version = "1.0.0"
edition = "2021"

[routes]
"GET /hello" = "handle_hello"
"POST /items" = "handle_create"
"GET /items/:id" = "handle_get_item"
```

**router.rs:**
```rust
use api::{HttpRequest, HttpResponse, json, json_response, error_response};

pub async fn handle_hello(_req: HttpRequest) -> HttpResponse {
    json_response(&json!({
        "message": "Hello from Rust!"
    }))
}

pub async fn handle_get_item(req: HttpRequest) -> HttpResponse {
    let id = req.path_params.get("id").cloned().unwrap_or_default();
    json_response(&json!({ "id": id }))
}
```

### HttpRequest Methods

| Method | Returns | Description |
|--------|---------|-------------|
| `query("name")` | `Option<String>` | Get query parameter |
| `path_params.get("name")` | `Option<&String>` | Get path parameter |
| `header("name")` | `Option<&String>` | Get header |
| `body_json<T>()` | `Result<T>` | Parse body as JSON |

---

## CLI Reference

```bash
# Create plugins
bun run plugin:new my-plugin
bun run plugin:new my-plugin --frontend-only
bun run plugin:new my-plugin --name "My Plugin" --author "You"

# Build plugins
bun run plugin:build my-plugin    # Build specific plugin
bun run plugin:build --all        # Build all plugins

# List plugins
bun run plugin:list
```

### Build Output

| Plugin Type | Output |
|-------------|--------|
| Frontend-only | `build/plugins/foo.js` |
| Full-stack | `build/plugins/foo.dll` |

---

## Troubleshooting

### Common Errors

**"Plugin not detected"**
- Ensure `index.jsx` exists in the plugin directory

**"Handler not found"**
- Check route names in `Cargo.toml` match function names
- Ensure handlers are `pub`

**"Build failed"**
- Check handler signature: `pub async fn name(req: HttpRequest) -> HttpResponse`

**"DLL won't reload"**
- Restart the app - DLLs are locked while loaded

### Development Tips

1. Set `RUST_LOG=debug` for detailed logs
2. Check browser DevTools for frontend errors
3. Plugin changes require rebuild: `bun run plugin:build <name>`
4. Frontend-only plugins build in ~1s, full-stack ~10-30s

---

## License

MIT License
