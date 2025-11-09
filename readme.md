# WebArcade Plugin Development Guide

> Comprehensive documentation for building plugins in the WebArcade system

## Table of Contents

1. [Quick Start](#quick-start)
2. [Introduction](#introduction)
3. [Plugin Architecture Overview](#plugin-architecture-overview)
4. [File Structure](#file-structure)
5. [Frontend Plugin Development](#frontend-plugin-development)
6. [Backend Plugin Development](#backend-plugin-development)
7. [Simplified Plugin Utilities](#simplified-plugin-utilities)
8. [Database Integration](#database-integration)
9. [Plugin Communication](#plugin-communication)
10. [Bridge & WebSocket System](#bridge--websocket-system)
11. [Icons and UI Components](#icons-and-ui-components)
12. [Plugin Discovery & Registration](#plugin-discovery--registration)
13. [Complete Example](#complete-example)
14. [Best Practices](#best-practices)
15. [Troubleshooting](#troubleshooting)

---

## Quick Start

### Create Your First Plugin

The fastest way to create a new plugin is using the scaffolding tool:

```bash
# Create a new plugin
npm run create-plugin my-awesome-plugin

# Discover and register the plugin
npm run discover

# Restart your dev server
npm run web
```

This creates a complete plugin with:
- ✅ Frontend UI (index.jsx, viewport.jsx)
- ✅ Backend API (mod.rs, router.rs)
- ✅ Example routes and endpoints
- ✅ Documentation (README.md)

The generated plugin is ready to use immediately and follows all best practices!

### Plugin File Structure

```
plugins/my-awesome-plugin/
├── index.jsx       # Frontend plugin entry point
├── viewport.jsx    # Main UI component
├── mod.rs          # Backend plugin module
├── router.rs       # HTTP route handlers
└── README.md       # Plugin documentation
```

---

## Introduction

WebArcade uses a **modular plugin architecture** that allows developers to extend the application with custom functionality. Plugins can be:

- **Frontend-only**: UI components, visualizations, dashboards
- **Backend-only**: Background services, data processing
- **Full-stack**: Complete features with UI and backend logic

The system is built on:
- **Frontend**: Solid.js (reactive UI framework)
- **Backend**: Rust with Tokio (async runtime)
- **Communication**: HTTP REST APIs + WebSockets
- **Database**: SQLite (shared database with plugin isolation)

---

## Plugin Architecture Overview

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    WebArcade Application                     │
├─────────────────────────────────────────────────────────────┤
│  Frontend (Solid.js)                                         │
│  ├── Plugin Manager                                          │
│  ├── Plugin API (viewport, tabs, menus, etc.)               │
│  └── Reactive Stores (signals, createStore)                 │
├─────────────────────────────────────────────────────────────┤
│  Bridge Layer                                                │
│  ├── HTTP Server (port 3001) - REST API                     │
│  └── WebSocket Server (port 3002) - Event streaming         │
├─────────────────────────────────────────────────────────────┤
│  Backend (Rust + Tokio)                                      │
│  ├── Plugin Manager                                          │
│  ├── Plugin Context (database, events, services, routes)    │
│  ├── Event Bus (broadcast channels)                         │
│  ├── Service Registry (inter-plugin RPC)                    │
│  └── Router Registry (HTTP routing)                         │
├─────────────────────────────────────────────────────────────┤
│  Database (SQLite)                                           │
│  └── Shared database with plugin migrations                 │
└─────────────────────────────────────────────────────────────┘
```

### Plugin Lifecycle

**Frontend:**
```
DISCOVERED → LOADING → LOADED → INITIALIZING → INITIALIZED → STARTING → RUNNING
                                                                            ↓
                                                                        onUpdate()
                                                                        (60 FPS loop)
```

**Backend:**
```
register() → init() → start()
```

---

## File Structure

### Minimal Frontend-Only Plugin

```
plugins/
└── my-plugin/
    └── index.jsx          # Plugin entry point (required)
```

### Full-Stack Plugin

```
plugins/
└── my-plugin/
    ├── index.jsx          # Frontend plugin definition (required)
    ├── viewport.jsx       # Main viewport component
    ├── panel.jsx          # Right sidebar panel component
    ├── store.jsx          # Shared state management
    ├── mod.rs            # Backend plugin implementation
    └── router.rs         # Backend HTTP routes
```

### Key Files Explained

| File | Purpose | Required |
|------|---------|----------|
| `index.jsx` | Plugin entry point, exports plugin factory | ✅ Yes |
| `viewport.jsx` | Main UI component rendered in viewport | ❌ No |
| `panel.jsx` | Right sidebar component | ❌ No |
| `store.jsx` | Reactive state management (signals/stores) | ❌ No |
| `mod.rs` | Backend Rust plugin implementation | ❌ No |
| `router.rs` | HTTP route handlers | ❌ No |

---

## Frontend Plugin Development

### 1. Plugin Entry Point (`index.jsx`)

Every plugin must export a plugin factory using `createPlugin()`:

```javascript
import { createPlugin } from '@/api/plugin';
import { IconDatabase } from '@tabler/icons-solidjs';
import MyViewport from './viewport.jsx';
import MyPanel from './panel.jsx';

export default createPlugin({
  id: 'my-plugin-unique-id',
  name: 'My Plugin',
  version: '1.0.0',
  description: 'Description of what this plugin does',
  author: 'Your Name',

  // Optional: Called once during initialization
  async onInit() {
    console.log('[My Plugin] Initializing...');
  },

  // Required: Called when plugin starts
  async onStart(api) {
    console.log('[My Plugin] Starting...');

    // Register a viewport type
    api.viewport('my-viewport-id', {
      label: 'My Viewport',
      component: MyViewport,
      icon: IconDatabase,
      description: 'Description shown in menus'
    });

    // Register a right panel tab
    api.tab('my-tab-id', {
      title: 'My Tab',
      component: MyPanel,
      icon: IconDatabase,
      order: 100,
      viewport: 'my-viewport-id'  // Associated viewport
    });

    // Control UI visibility
    api.showProps(true);
    api.showMenu(true);
    api.showFooter(true);
    api.showTabs(true);
  },

  // Optional: Called every frame (60 FPS)
  onUpdate() {
    // Use sparingly - called 60 times per second!
  },

  // Optional: Called when plugin stops
  async onStop() {
    console.log('[My Plugin] Stopping...');
  },

  // Optional: Called when plugin is disposed
  async onDispose() {
    console.log('[My Plugin] Disposing...');
  }
});
```

### 2. Plugin API Methods

The `api` object provided to `onStart()` has these methods:

#### Viewport Management
```javascript
// Register a viewport type
api.viewport(typeId, {
  label: 'Display Name',
  component: MyComponent,
  icon: IconComponent,
  description: 'Description text'
});

// Open a new viewport tab
api.open(typeId, {
  label: 'Tab Label',
  data: { /* custom data */ }
});

// Alias for open()
api.createViewportTab(typeId, options);
```

#### UI Component Registration
```javascript
// Right panel tab
api.tab(tabId, {
  title: 'Tab Title',
  component: MyComponent,
  icon: IconComponent,
  order: 100,  // Sort order
  viewport: 'viewport-id'  // Associated viewport
});

// Top menu item
api.menu(menuId, {
  label: 'Menu Item',
  icon: IconComponent,
  onClick: () => { /* handler */ }
});

// Left panel menu item
api.leftMenuItem(itemId, {
  label: 'Item Label',
  icon: IconComponent,
  onClick: () => { /* handler */ }
});

// Footer button
api.footer(buttonId, {
  label: 'Button',
  icon: IconComponent,
  onClick: () => { /* handler */ }
});

// Toolbar button
api.button(buttonId, {
  label: 'Button',
  icon: IconComponent,
  onClick: () => { /* handler */ }
});

// Bottom panel
api.panel(panelId, {
  title: 'Panel Title',
  component: MyComponent,
  icon: IconComponent
});
```

#### UI Visibility Control
```javascript
api.showProps(true);        // Show right properties panel
api.hideProps();

api.showMenu(true);         // Show top menu
api.hideMenu();

api.showFooter(true);       // Show footer
api.hideFooter();

api.showTabs(true);         // Show tabs
api.hideTabs();

api.showLeftPanel(true);    // Show left panel
api.hideLeftPanel();
```

#### Event System
```javascript
// Emit event (scoped to this plugin)
api.emit('my-event', { data: 'value' });

// Listen to any plugin's events
api.on('plugin-id:event-type', (data) => {
  console.log('Received:', data);
});

// Listen to own events only
api.onSelf('my-event', (data) => {
  console.log('My event:', data);
});
```

### 3. State Management (`store.jsx`)

Use Solid.js signals for reactive state:

```javascript
import { createSignal } from 'solid-js';
import { bridgeFetch } from '@/api/bridge';

// Create reactive signals
const [data, setData] = createSignal([]);
const [selectedItem, setSelectedItem] = createSignal(null);
const [loading, setLoading] = createSignal(false);
const [error, setError] = createSignal('');

// Async data loading
const loadData = async () => {
  setLoading(true);
  setError('');

  try {
    const response = await bridgeFetch('/my-plugin/data');
    const result = await response.json();
    setData(result);
  } catch (e) {
    console.error('Failed to load data:', e);
    setError(e.message);
  } finally {
    setLoading(false);
  }
};

// Export store API
export const myStore = {
  // Getters (signals)
  data,
  selectedItem,
  loading,
  error,

  // Setters
  setSelectedItem,

  // Actions
  loadData,

  // Computed values
  get itemCount() {
    return data().length;
  }
};
```

For complex state, use `createStore`:

```javascript
import { createStore } from 'solid-js/store';

const [store, setStore] = createStore({
  items: [],
  filters: {
    search: '',
    category: 'all'
  },
  ui: {
    selectedId: null,
    isEditing: false
  }
});

export const actions = {
  addItem: (item) => {
    setStore('items', (items) => [...items, item]);
  },

  setFilter: (key, value) => {
    setStore('filters', key, value);
  },

  selectItem: (id) => {
    setStore('ui', 'selectedId', id);
  }
};
```

### 4. Viewport Component (`viewport.jsx`)

Solid.js component with reactive rendering:

```javascript
import { For, Show, createSignal, createEffect, onCleanup } from 'solid-js';
import { myStore } from './store';

export default function MyViewport() {
  const [localState, setLocalState] = createSignal('');

  // Effect runs when dependencies change
  createEffect(() => {
    console.log('Data changed:', myStore.data());

    // Load data on mount
    myStore.loadData();

    // Cleanup
    onCleanup(() => {
      console.log('Component unmounting');
    });
  });

  const handleClick = (item) => {
    myStore.setSelectedItem(item);
  };

  return (
    <div class="h-full w-full flex flex-col bg-base-200 p-4">
      <h1 class="text-2xl font-bold mb-4">My Plugin</h1>

      {/* Conditional rendering */}
      <Show
        when={!myStore.loading()}
        fallback={<div class="loading loading-spinner"></div>}
      >
        {/* Error handling */}
        <Show when={myStore.error()}>
          <div class="alert alert-error">{myStore.error()}</div>
        </Show>

        {/* List rendering */}
        <For each={myStore.data()}>
          {(item, index) => (
            <div
              class="card bg-base-100 shadow-md mb-2 p-4 cursor-pointer hover:bg-base-300"
              onClick={() => handleClick(item)}
            >
              <h3 class="font-bold">{item.name}</h3>
              <p>{item.description}</p>
            </div>
          )}
        </For>
      </Show>
    </div>
  );
}
```

### 5. Panel Component (`panel.jsx`)

Right sidebar component:

```javascript
import { Show } from 'solid-js';
import { myStore } from './store';

export default function MyPanel() {
  return (
    <div class="p-4">
      <h2 class="text-lg font-bold mb-4">Properties</h2>

      <Show
        when={myStore.selectedItem()}
        fallback={<p class="text-sm text-base-content/60">No item selected</p>}
      >
        <div class="space-y-2">
          <div>
            <label class="label">
              <span class="label-text">Name</span>
            </label>
            <input
              type="text"
              class="input input-bordered w-full"
              value={myStore.selectedItem().name}
            />
          </div>

          <button class="btn btn-primary w-full">
            Save Changes
          </button>
        </div>
      </Show>
    </div>
  );
}
```

---

## Backend Plugin Development

### 1. Plugin Implementation (`mod.rs`)

```rust
mod router;  // Import router module

use crate::core::plugin::{Plugin, PluginMetadata};
use crate::core::plugin_context::PluginContext;
use async_trait::async_trait;
use std::sync::Arc;
use anyhow::Result;

pub struct MyPlugin;

#[async_trait]
impl Plugin for MyPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            id: "my-plugin".to_string(),
            name: "My Plugin".to_string(),
            version: "1.0.0".to_string(),
            description: "Plugin description".to_string(),
            author: "Your Name".to_string(),
            dependencies: vec![],  // Other plugin IDs this depends on
        }
    }

    async fn init(&self, ctx: &PluginContext) -> Result<()> {
        log::info!("[My Plugin] Initializing...");

        // Run database migrations
        ctx.migrate(&[
            r#"
            CREATE TABLE IF NOT EXISTS my_table (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                value TEXT,
                created_at INTEGER NOT NULL
            )
            "#,
            // Second migration (runs only if not already applied)
            r#"
            ALTER TABLE my_table ADD COLUMN updated_at INTEGER;
            "#,
        ])?;

        // Register HTTP routes
        router::register_routes(ctx).await?;

        // Register services for other plugins to call
        ctx.provide_service("calculate", |input| async move {
            let num = input.get("number")
                .and_then(|v| v.as_i64())
                .unwrap_or(0);
            Ok(serde_json::json!({ "result": num * 2 }))
        }).await;

        log::info!("[My Plugin] Initialized successfully");
        Ok(())
    }

    async fn start(&self, ctx: Arc<PluginContext>) -> Result<()> {
        log::info!("[My Plugin] Starting...");

        // Start background tasks if needed
        tokio::spawn(async move {
            // Background worker
        });

        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        log::info!("[My Plugin] Stopping...");
        Ok(())
    }
}
```

### 2. Plugin Context API

The `PluginContext` provides these APIs:

#### Database
```rust
// Get SQLite connection
let conn = ctx.db()?;

// Run migrations (tracked per plugin)
ctx.migrate(&[
    "CREATE TABLE IF NOT EXISTS my_table (...)",
    "ALTER TABLE my_table ADD COLUMN ...",
])?;
```

#### Events
```rust
// Publish event
ctx.emit("data-changed", &serde_json::json!({
    "id": 123,
    "action": "updated"
}));

// Subscribe to specific event type
let mut rx = ctx.subscribe_to("other-plugin:event-type").await;
tokio::spawn(async move {
    while let Ok(event) = rx.recv().await {
        log::info!("Received event: {:?}", event);
    }
});

// Subscribe to all events
let mut rx = ctx.subscribe_all();
```

#### Services (Inter-Plugin RPC)
```rust
// Register service
ctx.provide_service("method_name", |input| async move {
    let result = do_something(input)?;
    Ok(serde_json::json!(result))
}).await;

// Call another plugin's service
let result = ctx.call_service("other-plugin", "method_name",
    serde_json::json!({"param": "value"})
).await?;

// Check if service exists
if ctx.has_service("other-plugin", "method_name").await {
    // Call it
}

// List all services
let services = ctx.list_services().await;
```

#### Routing
```rust
// Register HTTP router
ctx.register_router("my-plugin", router).await;

// Access router registry
let registry = ctx.router_registry();
```

#### Configuration
```rust
// Get plugin config from plugins.json
let config = ctx.config();
let my_setting = config.get("my_setting")
    .and_then(|v| v.as_str())
    .unwrap_or("default");

// Get plugin ID
let id = ctx.plugin_id();
```

### 3. HTTP Routes (`router.rs`)

```rust
use crate::core::plugin_context::PluginContext;
use crate::core::plugin_router::PluginRouter;
use hyper::{Method, Request, Response, StatusCode, body::Incoming};
use http_body_util::{Full, BodyExt};
use hyper::body::{Bytes, Body};
use bytes::Buf;
use anyhow::Result;

pub async fn register_routes(ctx: &PluginContext) -> Result<()> {
    let mut router = PluginRouter::new();

    // GET /my-plugin/data
    router.route(Method::GET, "/data", |_path, _query, _req| {
        Box::pin(async move {
            handle_get_data().await
        })
    });

    // GET /my-plugin/item?id=123
    router.route(Method::GET, "/item", |_path, query, _req| {
        Box::pin(async move {
            handle_get_item(query).await
        })
    });

    // POST /my-plugin/create
    router.route(Method::POST, "/create", |_path, _query, req| {
        Box::pin(async move {
            handle_create(req).await
        })
    });

    ctx.register_router("my-plugin", router).await;
    Ok(())
}

// Helper type aliases for responses
type BoxBody<T, E> = http_body_util::combinators::BoxBody<T, E>;
use std::convert::Infallible;

fn full_body(s: &str) -> BoxBody<Bytes, Infallible> {
    http_body_util::Full::new(Bytes::from(s.to_string()))
        .map_err(|never| match never {})
        .boxed()
}

// GET handler
async fn handle_get_data() -> Response<BoxBody<Bytes, Infallible>> {
    let db_path = crate::core::database::get_database_path();

    match rusqlite::Connection::open(&db_path) {
        Ok(conn) => {
            let mut stmt = conn.prepare(
                "SELECT id, name, value FROM my_table ORDER BY created_at DESC"
            ).unwrap();

            let items: Vec<serde_json::Value> = stmt.query_map([], |row| {
                Ok(serde_json::json!({
                    "id": row.get::<_, i64>(0)?,
                    "name": row.get::<_, String>(1)?,
                    "value": row.get::<_, Option<String>>(2)?
                }))
            }).unwrap()
              .collect::<Result<Vec<_>, _>>()
              .unwrap();

            json_response(&items)
        }
        Err(e) => {
            error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
        }
    }
}

// Query parameter handler
async fn handle_get_item(query: &str) -> Response<BoxBody<Bytes, Infallible>> {
    // Parse query string
    let params: std::collections::HashMap<String, String> =
        url::form_urlencoded::parse(query.as_bytes())
            .into_owned()
            .collect();

    let id = params.get("id")
        .and_then(|s| s.parse::<i64>().ok());

    if id.is_none() {
        return error_response(StatusCode::BAD_REQUEST, "Missing 'id' parameter");
    }

    // Query database
    let db_path = crate::core::database::get_database_path();
    let conn = rusqlite::Connection::open(&db_path).unwrap();

    match conn.query_row(
        "SELECT id, name, value FROM my_table WHERE id = ?1",
        [id.unwrap()],
        |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, i64>(0)?,
                "name": row.get::<_, String>(1)?,
                "value": row.get::<_, Option<String>>(2)?
            }))
        }
    ) {
        Ok(item) => json_response(&item),
        Err(_) => error_response(StatusCode::NOT_FOUND, "Item not found")
    }
}

// POST handler with body
async fn handle_create(req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    // Read request body
    let whole_body = match req.collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(e) => {
            return error_response(
                StatusCode::BAD_REQUEST,
                &format!("Failed to read body: {}", e)
            );
        }
    };

    // Parse JSON
    let body: serde_json::Value = match serde_json::from_reader(whole_body.reader()) {
        Ok(v) => v,
        Err(e) => {
            return error_response(
                StatusCode::BAD_REQUEST,
                &format!("Invalid JSON: {}", e)
            );
        }
    };

    let name = body.get("name").and_then(|v| v.as_str()).unwrap_or("");
    let value = body.get("value").and_then(|v| v.as_str());

    // Insert into database
    let db_path = crate::core::database::get_database_path();
    let conn = rusqlite::Connection::open(&db_path).unwrap();

    match conn.execute(
        "INSERT INTO my_table (name, value, created_at) VALUES (?1, ?2, ?3)",
        rusqlite::params![name, value, current_timestamp()],
    ) {
        Ok(_) => {
            let id = conn.last_insert_rowid();
            json_response(&serde_json::json!({
                "success": true,
                "id": id
            }))
        }
        Err(e) => {
            error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
        }
    }
}

// Response helpers
fn json_response<T: serde::Serialize>(data: &T) -> Response<BoxBody<Bytes, Infallible>> {
    let json = serde_json::to_string(data).unwrap_or_else(|_| "{}".to_string());
    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .header("Access-Control-Allow-Origin", "*")
        .body(full_body(&json))
        .unwrap()
}

fn error_response(status: StatusCode, message: &str) -> Response<BoxBody<Bytes, Infallible>> {
    let json = serde_json::json!({"error": message}).to_string();
    Response::builder()
        .status(status)
        .header("Content-Type", "application/json")
        .header("Access-Control-Allow-Origin", "*")
        .body(full_body(&json))
        .unwrap()
}

fn current_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}
```

---

## Simplified Plugin Utilities

WebArcade provides simplified utilities to reduce boilerplate code when writing plugins.

### Router Utilities

Instead of writing duplicate helper functions in every plugin, import shared utilities:

```rust
use crate::core::router_utils::*;
```

#### Available Utilities

**Request Parsing:**
```rust
// Read JSON body from POST/PUT requests
let body = read_json_body(req).await?;
let name = body.get("name").and_then(|v| v.as_str());

// Parse query parameters
let id = parse_query_param(&query, "id");
```

**Response Helpers:**
```rust
// JSON response with CORS headers
json_response(&serde_json::json!({"data": "value"}))

// Error response with status code
error_response(StatusCode::BAD_REQUEST, "Invalid input")

// Convert string to HTTP body
full_body("Hello World")

// Convert bytes to HTTP body
bytes_body(vec![1, 2, 3])

// CORS preflight response
cors_preflight()
```

### Plugin Metadata Macro

Simplify plugin metadata definition with the `plugin_metadata!` macro:

**Before:**
```rust
fn metadata(&self) -> PluginMetadata {
    PluginMetadata {
        id: "my-plugin".to_string(),
        name: "My Plugin".to_string(),
        version: "1.0.0".to_string(),
        description: "Plugin description".to_string(),
        author: "WebArcade Team".to_string(),
        dependencies: vec![],
    }
}
```

**After:**
```rust
use crate::plugin_metadata;

impl Plugin for MyPlugin {
    plugin_metadata!("my-plugin", "My Plugin", "1.0.0", "Plugin description");

    // ... rest of plugin implementation
}
```

**With dependencies:**
```rust
plugin_metadata!("my-plugin", "My Plugin", "1.0.0", "Description",
                 deps: ["other-plugin", "another-plugin"]);
```

**With custom author:**
```rust
plugin_metadata!("my-plugin", "My Plugin", "1.0.0", "Description",
                 author: "Your Name");
```

### Route Registration Macro

Simplify route registration with the `route!` macro:

```rust
use crate::route;
```

**Before:**
```rust
router.route(Method::GET, "/data", |_path, _query, _req| {
    Box::pin(async move {
        handle_get_data().await
    })
});

router.route(Method::POST, "/create", |_path, _query, req| {
    Box::pin(async move {
        handle_create(req).await
    })
});
```

**After:**
```rust
route!(router, GET "/data" => handle_get_data);
route!(router, POST "/create" => handle_create);
route!(router, DELETE "/item/:id", path => handle_delete);
route!(router, GET "/search", query => handle_search);
```

**Supported Methods:**
- `GET`, `POST`, `PUT`, `DELETE`, `OPTIONS`

**Variants:**
```rust
// Simple GET/DELETE (no parameters)
route!(router, GET "/list" => handle_list);

// POST/PUT with request body
route!(router, POST "/create" => handle_create);

// With query string
route!(router, GET "/search", query => handle_search);

// With path parameters
route!(router, DELETE "/item/:id", path => handle_delete);

// With both path and query
route!(router, GET "/item/:id", path, query => handle_get_with_query);
```

### Database Helper Methods

The `PluginContext` provides convenient database methods:

```rust
// Execute a query and get multiple rows
let items: Vec<String> = ctx.query(
    "SELECT name FROM my_table WHERE active = ?1",
    [true],
    |row| row.get(0)
)?;

// Execute a query and get a single row
let count: i64 = ctx.query_row(
    "SELECT COUNT(*) FROM my_table",
    [],
    |row| row.get(0)
)?;

// Execute INSERT/UPDATE/DELETE
let rows_affected = ctx.execute(
    "UPDATE my_table SET value = ?1 WHERE id = ?2",
    rusqlite::params!["new_value", 123]
)?;

// Execute multiple SQL statements
ctx.execute_batch(r#"
    CREATE INDEX IF NOT EXISTS idx_name ON my_table(name);
    CREATE INDEX IF NOT EXISTS idx_created ON my_table(created_at);
"#)?;
```

### Complete Simplified Example

Here's a complete router using all the new utilities:

```rust
use crate::core::plugin_context::PluginContext;
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

    // Register routes using the macro
    route!(router, GET "/items" => handle_list);
    route!(router, GET "/item", query => handle_get_one);
    route!(router, POST "/item" => handle_create);
    route!(router, DELETE "/item/:id", path => handle_delete);
    route!(router, OPTIONS "/item" => cors_preflight);

    ctx.register_router("my-plugin", router).await;
    Ok(())
}

async fn handle_list() -> Response<BoxBody<Bytes, Infallible>> {
    let db_path = crate::core::database::get_database_path();
    let conn = match rusqlite::Connection::open(&db_path) {
        Ok(c) => c,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
    };

    let mut stmt = match conn.prepare("SELECT id, name FROM items") {
        Ok(s) => s,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
    };

    let items = stmt.query_map([], |row| {
        Ok(serde_json::json!({
            "id": row.get::<_, i64>(0)?,
            "name": row.get::<_, String>(1)?
        }))
    }).unwrap().collect::<Result<Vec<_>, _>>().unwrap();

    json_response(&items)
}

async fn handle_get_one(query: String) -> Response<BoxBody<Bytes, Infallible>> {
    let id = match parse_query_param(&query, "id") {
        Some(id_str) => match id_str.parse::<i64>() {
            Ok(id) => id,
            Err(_) => return error_response(StatusCode::BAD_REQUEST, "Invalid ID")
        },
        None => return error_response(StatusCode::BAD_REQUEST, "Missing ID parameter")
    };

    // Use database
    json_response(&serde_json::json!({"id": id, "name": "Example"}))
}

async fn handle_create(req: Request<Incoming>) -> Response<BoxBody<Bytes, Infallible>> {
    let body = match read_json_body(req).await {
        Ok(b) => b,
        Err(e) => return error_response(StatusCode::BAD_REQUEST, &e)
    };

    let name = body.get("name").and_then(|v| v.as_str()).unwrap_or("");

    json_response(&serde_json::json!({
        "success": true,
        "name": name
    }))
}

async fn handle_delete(path: String) -> Response<BoxBody<Bytes, Infallible>> {
    // path will be like "/item/123", extract the ID
    let id = path.trim_start_matches("/item/");

    json_response(&serde_json::json!({
        "success": true,
        "deleted": id
    }))
}
```

### Benefits

Using these utilities provides:

- **40-50% less code** in plugin routers
- **Consistent error handling** across all plugins
- **Better maintainability** - single source of truth for utilities
- **Type safety** - macros catch errors at compile time
- **CORS support** - automatic CORS headers on all responses

---

## Database Integration

### Database Path

All plugins share a single SQLite database located at:
```
<project-root>/data/counters.db
```

### Using the Database in Rust

```rust
// Get database connection
let conn = ctx.db()?;

// Create tables in init()
ctx.migrate(&[
    r#"
    CREATE TABLE IF NOT EXISTS my_items (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        name TEXT NOT NULL,
        description TEXT,
        count INTEGER DEFAULT 0,
        created_at INTEGER NOT NULL,
        updated_at INTEGER
    )
    "#,
])?;

// Query data
let mut stmt = conn.prepare(
    "SELECT id, name, count FROM my_items WHERE count > ?1"
)?;

let items = stmt.query_map([10], |row| {
    Ok((
        row.get::<_, i64>(0)?,
        row.get::<_, String>(1)?,
        row.get::<_, i64>(2)?,
    ))
})?;

for item in items {
    let (id, name, count) = item?;
    log::info!("Item {}: {} (count: {})", id, name, count);
}

// Insert data
conn.execute(
    "INSERT INTO my_items (name, description, count, created_at) VALUES (?1, ?2, ?3, ?4)",
    rusqlite::params!["Item Name", "Description", 5, current_timestamp()],
)?;

// Update data
conn.execute(
    "UPDATE my_items SET count = count + 1, updated_at = ?1 WHERE id = ?2",
    rusqlite::params![current_timestamp(), item_id],
)?;

// Delete data
conn.execute("DELETE FROM my_items WHERE id = ?1", [item_id])?;
```

### Migration System

Migrations are tracked per-plugin in the `plugin_migrations` table:

```rust
// In init()
ctx.migrate(&[
    // Migration 1
    r#"
    CREATE TABLE IF NOT EXISTS users (
        id INTEGER PRIMARY KEY,
        username TEXT NOT NULL UNIQUE
    )
    "#,

    // Migration 2 - only runs if migration 1 already applied
    r#"
    ALTER TABLE users ADD COLUMN email TEXT;
    "#,

    // Migration 3
    r#"
    CREATE INDEX idx_users_username ON users(username);
    "#,
])?;
```

Migrations are:
- **Versioned**: Each migration gets a version number (1, 2, 3, ...)
- **Tracked**: System records which migrations have been applied
- **Idempotent**: Safe to run multiple times (only pending migrations execute)
- **Plugin-scoped**: Each plugin tracks its own migrations

### Accessing Database from Frontend

```javascript
import { bridgeFetch } from '@/api/bridge';

// GET request
const response = await bridgeFetch('/my-plugin/data');
const items = await response.json();

// POST request
const response = await bridgeFetch('/my-plugin/create', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    name: 'New Item',
    value: 'Some value'
  })
});

const result = await response.json();
```

---

## Plugin Communication

### 1. Frontend Event System

Plugins communicate via CustomEvents:

```javascript
// In plugin A - emit event
api.emit('data-updated', {
  id: 123,
  action: 'created'
});

// In plugin B - listen for plugin A's events
api.on('plugin-a-id:data-updated', (data) => {
  console.log('Plugin A updated data:', data);
});

// Listen to own events only
api.onSelf('my-event', (data) => {
  console.log('My event fired:', data);
});
```

### 2. Backend Event System

Plugins use the EventBus for communication:

```rust
// Publish event
ctx.emit("data-changed", &serde_json::json!({
    "entity_id": 123,
    "type": "user",
    "action": "created"
}));

// Subscribe to specific event type
let mut rx = ctx.subscribe_to("other-plugin:data-changed").await;

tokio::spawn(async move {
    while let Ok(event) = rx.recv().await {
        log::info!("Received event from {}: {}",
            event.source_plugin,
            event.event_type
        );

        // Handle event
        if let Some(payload) = event.payload {
            // Process payload
        }
    }
});

// Subscribe to ALL events (use sparingly)
let mut rx = ctx.subscribe_all();
```

### 3. Service Registry (RPC)

Plugins can expose services that other plugins can call:

```rust
// Plugin A - provide service
ctx.provide_service("get_user", |input| async move {
    let user_id = input.get("id")
        .and_then(|v| v.as_i64())
        .ok_or_else(|| anyhow::anyhow!("Missing user_id"))?;

    let db_path = crate::core::database::get_database_path();
    let conn = rusqlite::Connection::open(&db_path)?;

    let username: String = conn.query_row(
        "SELECT username FROM users WHERE id = ?1",
        [user_id],
        |row| row.get(0)
    )?;

    Ok(serde_json::json!({
        "id": user_id,
        "username": username
    }))
}).await;

// Plugin B - call service
let user = ctx.call_service("plugin-a", "get_user",
    serde_json::json!({"id": 123})
).await?;

log::info!("Got user: {:?}", user);
```

### 4. Shared Stores (Frontend)

Access other plugin stores:

```javascript
// In plugin A's store.jsx
export const pluginAStore = {
  data,
  setData,
  loadData
};

// Make it globally accessible if needed
if (typeof window !== 'undefined') {
  window.pluginAStore = pluginAStore;
}

// In plugin B
import { pluginAStore } from '../plugin-a/store.jsx';

// Use plugin A's data
const items = pluginAStore.data();
```

---

## Bridge & WebSocket System

### Bridge Architecture

The bridge is a Rust backend server that:
- Runs on **HTTP port 3001** (REST API)
- Runs on **WebSocket port 3002** (Event streaming)
- Forwards plugin events to WebSocket clients
- Routes HTTP requests to plugin handlers

### Connecting to Bridge (Frontend)

```javascript
import { bridgeFetch, WEBARCADE_WS, BRIDGE_API } from '@/api/bridge';

// HTTP requests (auto-proxied in development)
const response = await bridgeFetch('/plugin-name/endpoint');

// Direct URLs for overlays
console.log(BRIDGE_API);      // 'http://localhost:3001'
console.log(WEBARCADE_WS);    // 'ws://localhost:3002'
```

### WebSocket Connection

```javascript
const ws = new WebSocket('ws://localhost:3002');

ws.addEventListener('open', () => {
  console.log('Connected to WebArcade bridge');
});

ws.addEventListener('message', (event) => {
  const data = JSON.parse(event.data);

  if (data.type === 'connected') {
    console.log('Welcome message:', data.message);
    console.log('Channel:', data.channel);
  } else {
    // Plugin event
    console.log('Event from plugin:', data.source_plugin);
    console.log('Event type:', data.event_type);
    console.log('Payload:', data.payload);
  }
});

ws.addEventListener('error', (error) => {
  console.error('WebSocket error:', error);
});

ws.addEventListener('close', () => {
  console.log('Disconnected from bridge');
  // Implement reconnection logic
});
```

### Event Format

Events forwarded to WebSocket clients:

```json
{
  "source_plugin": "my-plugin",
  "event_type": "data-changed",
  "payload": {
    "id": 123,
    "action": "updated"
  },
  "timestamp": 1699564800
}
```

### Welcome Message

On connection, clients receive:

```json
{
  "type": "connected",
  "system": "plugin-based",
  "message": "WebArcade Bridge - Plugin Event Stream",
  "channel": "your-twitch-username"
}
```

---

## Icons and UI Components

### Using Tabler Icons

WebArcade uses [@tabler/icons-solidjs](https://tabler-icons.io/):

```javascript
import {
  IconDatabase,
  IconSettings,
  IconUser,
  IconHome,
  IconPlus,
  IconTrash,
  IconEdit,
  IconCheck,
  IconX
} from '@tabler/icons-solidjs';

// Use in components
<IconDatabase class="w-6 h-6" />
<IconSettings class="w-5 h-5 text-primary" />
```

Browse all icons: https://tabler-icons.io/

### DaisyUI Components

WebArcade uses [DaisyUI](https://daisyui.com/) for UI components:

```javascript
// Buttons
<button class="btn">Default</button>
<button class="btn btn-primary">Primary</button>
<button class="btn btn-secondary">Secondary</button>
<button class="btn btn-accent">Accent</button>
<button class="btn btn-ghost">Ghost</button>
<button class="btn btn-link">Link</button>

// Loading states
<button class="btn btn-primary loading">Loading</button>
<span class="loading loading-spinner"></span>

// Inputs
<input type="text" class="input input-bordered w-full" />
<textarea class="textarea textarea-bordered w-full"></textarea>

// Cards
<div class="card bg-base-100 shadow-xl">
  <div class="card-body">
    <h2 class="card-title">Card Title</h2>
    <p>Card content</p>
    <div class="card-actions justify-end">
      <button class="btn btn-primary">Action</button>
    </div>
  </div>
</div>

// Alerts
<div class="alert alert-info">Info message</div>
<div class="alert alert-success">Success message</div>
<div class="alert alert-warning">Warning message</div>
<div class="alert alert-error">Error message</div>

// Badges
<span class="badge">Default</span>
<span class="badge badge-primary">Primary</span>
<span class="badge badge-secondary">Secondary</span>

// Modals
<dialog class="modal">
  <div class="modal-box">
    <h3 class="font-bold text-lg">Modal Title</h3>
    <p class="py-4">Modal content</p>
    <div class="modal-action">
      <button class="btn">Close</button>
    </div>
  </div>
</dialog>
```

---

## Plugin Discovery & Registration

### How Plugins are Discovered

1. **Frontend Discovery** (`npm run discover`):
   - Scans `plugins/` directory
   - Finds all `index.jsx` or `index.js` files
   - Generates `src/api/plugin/plugins.json`

2. **Backend Discovery** (`npm run discover`):
   - Scans `plugins/` for `mod.rs` files
   - Generates `src-tauri/src/bridge/plugins/generated.rs`
   - Auto-registers plugins with PluginManager

### Running Discovery

```bash
# Discover all plugins
npm run discover

# Or run individually
node scripts/discover-plugins.js
node scripts/discover-backend-plugins.js
```

### Frontend Plugin Registry

Generated at `src/api/plugin/plugins.json`:

```json
{
  "plugins": [
    {
      "id": "my-plugin",
      "path": "/plugins/my-plugin",
      "main": "index.jsx",
      "enabled": true,
      "priority": 1
    }
  ],
  "generatedAt": "2025-11-09T12:00:00.000Z",
  "generator": "discover-plugins.js"
}
```

### Backend Plugin Registry

Generated at `src-tauri/src/bridge/plugins/generated.rs`:

```rust
#[path = "../../../../plugins/my-plugin/mod.rs"]
pub mod my_plugin;

use crate::core::plugin_manager::PluginManager;

pub fn register_all_plugins(manager: &mut PluginManager) {
    log::info!("📦 Registering auto-discovered plugins...");

    manager.register(my_plugin::MyPlugin);

    log::info!("✅ Plugin registration complete");
}
```

### Plugin Configuration

Root `plugins.json` for user configuration:

```json
{
  "plugins": [
    {
      "id": "my-plugin",
      "enabled": true,
      "config": {
        "setting1": "value1",
        "setting2": 42
      }
    }
  ]
}
```

Access in backend:

```rust
let config = ctx.config();
let my_setting = config.get("setting1")
    .and_then(|v| v.as_str())
    .unwrap_or("default");
```

---

## Complete Example

Let's build a **Counter Plugin** that tracks click counts:

### File Structure

```
plugins/
└── counter/
    ├── index.jsx      # Plugin definition
    ├── viewport.jsx   # Main UI
    ├── panel.jsx      # Right panel
    ├── store.jsx      # State management
    ├── mod.rs        # Backend plugin
    └── router.rs     # HTTP routes
```

### `index.jsx`

```javascript
import { createPlugin } from '@/api/plugin';
import { IconNumber } from '@tabler/icons-solidjs';
import CounterViewport from './viewport.jsx';
import CounterPanel from './panel.jsx';

export default createPlugin({
  id: 'counter-plugin',
  name: 'Counter',
  version: '1.0.0',
  description: 'Track click counts',
  author: 'WebArcade',

  async onStart(api) {
    api.viewport('counter-viewport', {
      label: 'Counter',
      component: CounterViewport,
      icon: IconNumber,
      description: 'Click counter with persistence'
    });

    api.tab('counter-panel', {
      title: 'Counter',
      component: CounterPanel,
      icon: IconNumber,
      order: 10,
      viewport: 'counter-viewport'
    });

    console.log('[Counter] Plugin started');
  }
});
```

### `store.jsx`

```javascript
import { createSignal } from 'solid-js';
import { bridgeFetch } from '@/api/bridge';

const [count, setCount] = createSignal(0);
const [history, setHistory] = createSignal([]);
const [loading, setLoading] = createSignal(false);

const loadCount = async () => {
  setLoading(true);
  try {
    const response = await bridgeFetch('/counter/current');
    const data = await response.json();
    setCount(data.count || 0);
  } catch (e) {
    console.error('Failed to load count:', e);
  } finally {
    setLoading(false);
  }
};

const increment = async () => {
  try {
    const response = await bridgeFetch('/counter/increment', {
      method: 'POST'
    });
    const data = await response.json();
    setCount(data.count);
    loadHistory();
  } catch (e) {
    console.error('Failed to increment:', e);
  }
};

const reset = async () => {
  try {
    await bridgeFetch('/counter/reset', { method: 'POST' });
    setCount(0);
    loadHistory();
  } catch (e) {
    console.error('Failed to reset:', e);
  }
};

const loadHistory = async () => {
  try {
    const response = await bridgeFetch('/counter/history');
    const data = await response.json();
    setHistory(data);
  } catch (e) {
    console.error('Failed to load history:', e);
  }
};

export const counterStore = {
  count,
  history,
  loading,
  loadCount,
  increment,
  reset,
  loadHistory
};
```

### `viewport.jsx`

```javascript
import { Show, For, createEffect } from 'solid-js';
import { IconPlus, IconRefresh } from '@tabler/icons-solidjs';
import { counterStore } from './store';

export default function CounterViewport() {
  createEffect(() => {
    counterStore.loadCount();
    counterStore.loadHistory();
  });

  return (
    <div class="h-full w-full flex flex-col items-center justify-center bg-base-200 p-8">
      <div class="card bg-base-100 shadow-2xl w-full max-w-md">
        <div class="card-body items-center text-center">
          <h2 class="card-title text-4xl mb-4">Counter</h2>

          <Show
            when={!counterStore.loading()}
            fallback={<span class="loading loading-spinner loading-lg"></span>}
          >
            <div class="text-6xl font-bold text-primary my-8">
              {counterStore.count()}
            </div>

            <div class="card-actions flex gap-4">
              <button
                class="btn btn-primary btn-lg"
                onClick={counterStore.increment}
              >
                <IconPlus class="w-6 h-6" />
                Increment
              </button>

              <button
                class="btn btn-secondary btn-lg"
                onClick={counterStore.reset}
              >
                <IconRefresh class="w-6 h-6" />
                Reset
              </button>
            </div>
          </Show>
        </div>
      </div>
    </div>
  );
}
```

### `panel.jsx`

```javascript
import { For, Show } from 'solid-js';
import { counterStore } from './store';

export default function CounterPanel() {
  return (
    <div class="p-4">
      <h2 class="text-lg font-bold mb-4">History</h2>

      <Show
        when={counterStore.history().length > 0}
        fallback={<p class="text-sm text-base-content/60">No history yet</p>}
      >
        <div class="space-y-2">
          <For each={counterStore.history()}>
            {(entry) => (
              <div class="card bg-base-200 p-2">
                <div class="text-sm">
                  Count: <strong>{entry.count}</strong>
                </div>
                <div class="text-xs text-base-content/60">
                  {new Date(entry.timestamp * 1000).toLocaleString()}
                </div>
              </div>
            )}
          </For>
        </div>
      </Show>
    </div>
  );
}
```

### `mod.rs`

```rust
mod router;

use crate::core::plugin::{Plugin, PluginMetadata};
use crate::core::plugin_context::PluginContext;
use async_trait::async_trait;
use std::sync::Arc;
use anyhow::Result;

pub struct CounterPlugin;

#[async_trait]
impl Plugin for CounterPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            id: "counter".to_string(),
            name: "Counter".to_string(),
            version: "1.0.0".to_string(),
            description: "Click counter with persistence".to_string(),
            author: "WebArcade".to_string(),
            dependencies: vec![],
        }
    }

    async fn init(&self, ctx: &PluginContext) -> Result<()> {
        log::info!("[Counter] Initializing...");

        ctx.migrate(&[
            r#"
            CREATE TABLE IF NOT EXISTS counter_state (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                count INTEGER NOT NULL DEFAULT 0,
                updated_at INTEGER NOT NULL
            )
            "#,
            r#"
            CREATE TABLE IF NOT EXISTS counter_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                count INTEGER NOT NULL,
                action TEXT NOT NULL,
                timestamp INTEGER NOT NULL
            )
            "#,
        ])?;

        // Initialize counter if not exists
        let conn = ctx.db()?;
        conn.execute(
            "INSERT OR IGNORE INTO counter_state (id, count, updated_at) VALUES (1, 0, ?1)",
            [current_timestamp()],
        )?;

        router::register_routes(ctx).await?;

        log::info!("[Counter] Initialized");
        Ok(())
    }

    async fn start(&self, _ctx: Arc<PluginContext>) -> Result<()> {
        log::info!("[Counter] Started");
        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        log::info!("[Counter] Stopped");
        Ok(())
    }
}

fn current_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}
```

### `router.rs`

```rust
use crate::core::plugin_context::PluginContext;
use crate::core::plugin_router::PluginRouter;
use hyper::{Method, Response, StatusCode};
use http_body_util::Full;
use hyper::body::Bytes;
use anyhow::Result;

type BoxBody<T, E> = http_body_util::combinators::BoxBody<T, E>;
use std::convert::Infallible;

fn full_body(s: &str) -> BoxBody<Bytes, Infallible> {
    use http_body_util::BodyExt;
    Full::new(Bytes::from(s.to_string()))
        .map_err(|never| match never {})
        .boxed()
}

pub async fn register_routes(ctx: &PluginContext) -> Result<()> {
    let mut router = PluginRouter::new();

    router.route(Method::GET, "/current", |_path, _query, _req| {
        Box::pin(async move { get_current().await })
    });

    router.route(Method::POST, "/increment", |_path, _query, _req| {
        Box::pin(async move { increment().await })
    });

    router.route(Method::POST, "/reset", |_path, _query, _req| {
        Box::pin(async move { reset().await })
    });

    router.route(Method::GET, "/history", |_path, _query, _req| {
        Box::pin(async move { get_history().await })
    });

    ctx.register_router("counter", router).await;
    Ok(())
}

async fn get_current() -> Response<BoxBody<Bytes, Infallible>> {
    let db_path = crate::core::database::get_database_path();
    let conn = rusqlite::Connection::open(&db_path).unwrap();

    let count: i64 = conn.query_row(
        "SELECT count FROM counter_state WHERE id = 1",
        [],
        |row| row.get(0)
    ).unwrap_or(0);

    json_response(&serde_json::json!({ "count": count }))
}

async fn increment() -> Response<BoxBody<Bytes, Infallible>> {
    let db_path = crate::core::database::get_database_path();
    let conn = rusqlite::Connection::open(&db_path).unwrap();

    conn.execute(
        "UPDATE counter_state SET count = count + 1, updated_at = ?1 WHERE id = 1",
        [current_timestamp()],
    ).unwrap();

    let count: i64 = conn.query_row(
        "SELECT count FROM counter_state WHERE id = 1",
        [],
        |row| row.get(0)
    ).unwrap();

    conn.execute(
        "INSERT INTO counter_history (count, action, timestamp) VALUES (?1, 'increment', ?2)",
        rusqlite::params![count, current_timestamp()],
    ).unwrap();

    json_response(&serde_json::json!({ "count": count }))
}

async fn reset() -> Response<BoxBody<Bytes, Infallible>> {
    let db_path = crate::core::database::get_database_path();
    let conn = rusqlite::Connection::open(&db_path).unwrap();

    conn.execute(
        "UPDATE counter_state SET count = 0, updated_at = ?1 WHERE id = 1",
        [current_timestamp()],
    ).unwrap();

    conn.execute(
        "INSERT INTO counter_history (count, action, timestamp) VALUES (0, 'reset', ?1)",
        [current_timestamp()],
    ).unwrap();

    json_response(&serde_json::json!({ "success": true }))
}

async fn get_history() -> Response<BoxBody<Bytes, Infallible>> {
    let db_path = crate::core::database::get_database_path();
    let conn = rusqlite::Connection::open(&db_path).unwrap();

    let mut stmt = conn.prepare(
        "SELECT count, action, timestamp FROM counter_history ORDER BY timestamp DESC LIMIT 10"
    ).unwrap();

    let history: Vec<serde_json::Value> = stmt.query_map([], |row| {
        Ok(serde_json::json!({
            "count": row.get::<_, i64>(0)?,
            "action": row.get::<_, String>(1)?,
            "timestamp": row.get::<_, i64>(2)?
        }))
    }).unwrap()
      .collect::<Result<Vec<_>, _>>()
      .unwrap();

    json_response(&history)
}

fn json_response<T: serde::Serialize>(data: &T) -> Response<BoxBody<Bytes, Infallible>> {
    let json = serde_json::to_string(data).unwrap();
    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .header("Access-Control-Allow-Origin", "*")
        .body(full_body(&json))
        .unwrap()
}

fn current_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}
```

### Register the Plugin

```bash
# Run discovery
npm run discover

# This generates:
# - Frontend: src/api/plugin/plugins.json
# - Backend: src-tauri/src/bridge/plugins/generated.rs
```

---

## Best Practices

### 1. Plugin Naming

- **Plugin IDs**: Use kebab-case (`my-plugin`, `user-manager`)
- **File names**: Use kebab-case (`my-component.jsx`, `user-store.jsx`)
- **Component names**: Use PascalCase (`MyComponent`, `UserList`)
- **Function names**: Use camelCase (`loadData`, `handleClick`)

### 2. State Management

- Use **signals** for simple state
- Use **createStore** for complex/nested state
- Keep stores in separate `store.jsx` files
- Export a clean API from stores

### 3. Database Design

- **Prefix table names** with plugin name to avoid conflicts
- Use **migrations** for schema changes
- Always use **timestamps** for audit trails
- Index frequently queried columns

### 4. Error Handling

```javascript
// Frontend
try {
  const response = await bridgeFetch('/endpoint');
  if (!response.ok) {
    throw new Error(`HTTP ${response.status}: ${response.statusText}`);
  }
  const data = await response.json();
} catch (e) {
  console.error('Operation failed:', e);
  setError(e.message);
}
```

```rust
// Backend
match conn.execute(sql, params) {
    Ok(_) => {
        json_response(&serde_json::json!({"success": true}))
    }
    Err(e) => {
        log::error!("Database error: {}", e);
        error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
    }
}
```

### 5. Performance

- **Avoid onUpdate()** unless necessary (runs 60 times/sec)
- Use **createEffect** with proper dependencies
- Debounce expensive operations
- Clean up with **onCleanup**

### 6. Logging

```javascript
// Frontend
console.log('[My Plugin] Info message');
console.warn('[My Plugin] Warning');
console.error('[My Plugin] Error');
```

```rust
// Backend
log::info!("[My Plugin] Info message");
log::warn!("[My Plugin] Warning");
log::error!("[My Plugin] Error: {}", e);
```

### 7. Cleanup

```javascript
createEffect(() => {
  const interval = setInterval(() => {
    // Periodic task
  }, 1000);

  onCleanup(() => {
    clearInterval(interval);
  });
});
```

```rust
async fn start(&self, ctx: Arc<PluginContext>) -> Result<()> {
    let handle = tokio::spawn(async move {
        // Background task
    });

    // Store handle for cleanup in stop()
    Ok(())
}
```

---

## Troubleshooting

### Plugin Not Loading

1. **Check discovery ran**: `npm run discover`
2. **Verify file structure**: Must have `index.jsx` or `index.js`
3. **Check console**: Look for errors in browser console
4. **Verify export**: Plugin must export default function

### Backend Plugin Not Found

1. **Run discovery**: `npm run discover`
2. **Check generated.rs**: Should contain your plugin
3. **Rebuild backend**: `cargo build`
4. **Check logs**: Look for registration messages

### Database Errors

1. **Check migrations**: Run in correct order
2. **Verify database path**: Should be `data/counters.db`
3. **Check permissions**: Ensure write access to `data/` directory
4. **Inspect database**: Use `sqlite3 data/counters.db` to debug

### CORS Errors

Ensure backend responses include:
```rust
.header("Access-Control-Allow-Origin", "*")
```

### WebSocket Connection Failed

1. **Check bridge is running**: Should see "WebSocket server listening" log
2. **Verify port**: Default is 3002
3. **Check firewall**: Allow connections to localhost:3002

---

## Quick Reference

### File Locations

| Component | Path |
|-----------|------|
| Frontend Plugin API | `src/api/plugin/index.jsx` |
| Bridge Client | `src/api/bridge.js` |
| Frontend Discovery Script | `scripts/discover-plugins.js` |
| Backend Discovery Script | `scripts/discover-backend-plugins.js` |
| Generated Frontend Registry | `src/api/plugin/plugins.json` |
| Generated Backend Registry | `src-tauri/src/bridge/plugins/generated.rs` |
| Backend Plugin Trait | `src-tauri/src/bridge/core/plugin.rs` |
| Plugin Context | `src-tauri/src/bridge/core/plugin_context.rs` |
| Database Utilities | `src-tauri/src/bridge/core/database.rs` |
| WebSocket Bridge | `src-tauri/src/bridge/core/websocket_bridge.rs` |

### Commands

```bash
# Plugin Development
npm run create-plugin <name>  # Create a new plugin with scaffolding
npm run discover              # Discover all plugins

# Development
npm run web                   # Start dev server with bridge
npm run app                   # Run as Tauri app

# Building
npm run build                 # Build frontend
cargo build --release         # Build backend

# Backend only
cd src-tauri
cargo run                     # Run backend
cargo check                   # Check for errors
```

### Ports

- **HTTP API**: 3001 (`http://localhost:3001`)
- **WebSocket**: 3002 (`ws://localhost:3002`)
- **Dev Server**: 8080 (`http://localhost:8080`)

---

## Additional Resources

- **Solid.js Docs**: https://www.solidjs.com/docs/latest
- **Tabler Icons**: https://tabler-icons.io/
- **DaisyUI Components**: https://daisyui.com/components/
- **Rust Async Book**: https://rust-lang.github.io/async-book/
- **Tokio Docs**: https://tokio.rs/tokio/tutorial
- **rusqlite Docs**: https://docs.rs/rusqlite/

---

**Happy Plugin Development!** 🚀
