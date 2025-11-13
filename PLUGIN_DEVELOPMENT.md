# Plugin Development Guide

WebArcade is an **open-source platform** that lets you create native plugins using **SolidJS** (frontend) and **Rust** (backend). Plugins can add new features, widgets, and functionality to the WebArcade ecosystem.

## Two Ways to Develop Plugins

### 1. Using the Developer IDE (Recommended)

WebArcade includes a built-in Developer IDE for creating and building plugins:

1. Open WebArcade and navigate to the **Developer** plugin from the menu
2. Click **"New Plugin"** in the toolbar
3. Choose a template:
   - **Basic**: Simple frontend-only plugin with viewport
   - **Widget**: Plugin with dashboard widget component
   - **Backend**: Rust backend with API routes
   - **Full-stack**: Complete plugin with frontend, backend, and widget
4. Fill in plugin details (ID, name, description, author)
5. The IDE will generate the plugin structure in `src/plugins/developer/projects/`
6. Edit files in the built-in code editor
7. Click **"Build"** to compile your plugin into a distributable `.zip`

**Development Directory**: `src/plugins/developer/projects/your-plugin-name/`

**Note**: Plugins in the projects directory are excluded from the main build and are only for development.

### 2. Manual Plugin Development

You can also create plugins manually for more control or when building outside WebArcade.

## Installing Plugins

The easiest way to install a plugin is to **drag and drop** a plugin `.zip` file anywhere in the WebArcade application window. The app will automatically:
1. Extract the plugin to the runtime plugins directory
2. Validate its structure and manifest
3. Install the plugin files
4. Prompt you to restart to load the plugin

### Plugin Installation Locations

**Runtime Plugins (Drag & Drop):**
- Windows: `%LOCALAPPDATA%\WebArcade\plugins\`
- Linux: `~/.local/share/WebArcade/plugins/`
- macOS: `~/Library/Application Support/WebArcade/plugins/`

**Development Plugins (IDE):**
- Located in `src/plugins/developer/projects/` (not compiled into main app)

## Plugin Structure

### Minimal Frontend-Only Plugin

```
my-plugin/
├── manifest.json          # Required: Plugin metadata
├── index.jsx             # Optional: Main plugin entry point
└── Widget.jsx            # Optional: A widget component
```

### Full-Stack Plugin with DLL Backend

```
my-plugin/
├── manifest.json          # Required: Plugin metadata
├── plugin.js             # Frontend bundle (ES module)
├── routes.json           # Optional: Backend route definitions
├── my-plugin.dll         # Windows binary
├── libmy-plugin.so       # Linux binary
└── libmy-plugin.dylib    # macOS binary
```

**Note:** All files are in the root directory. Platform detection happens automatically based on filename extensions (`.dll`, `.so`, `.dylib`).

## manifest.json

Every plugin **must** include a `manifest.json` file:

```json
{
  "id": "my-plugin",
  "name": "My Awesome Plugin",
  "version": "1.0.0",
  "description": "A plugin that does amazing things",
  "author": "Your Name",
  "has_frontend": true,
  "has_backend": false
}
```

### Required Fields

- `id` (string): Unique identifier (alphanumeric, hyphens, underscores only)
- `name` (string): Display name
- `version` (string): Semantic version (e.g., "1.0.0")

### Optional Fields

- `description` (string): Brief description of the plugin
- `author` (string): Plugin author
- `has_frontend` (boolean): Whether the plugin has frontend components (default: true)
- `has_backend` (boolean): Whether the plugin has Rust backend code (default: false)

## Frontend Plugin (index.jsx)

Frontend plugins use the WebArcade Plugin API:

```jsx
import { createPlugin } from '@/api/plugin';
import { IconPlugin } from '@tabler/icons-solidjs';
import MyViewport from './MyViewport.jsx';
import MyWidget from './widgets/MyWidget.jsx';

export default createPlugin({
  id: 'my-plugin',
  name: 'My Plugin',
  version: '1.0.0',
  description: 'Does cool things',
  author: 'Your Name',

  async onStart(api) {
    console.log('[My Plugin] Starting...');

    // Register a viewport (main view)
    api.viewport('my-plugin-viewport', {
      label: 'My Plugin',
      component: MyViewport,
      icon: IconPlugin,
      description: 'Main plugin interface'
    });

    // Add menu item
    api.menu('my-plugin-menu', {
      label: 'My Plugin',
      icon: IconPlugin,
      onClick: () => {
        api.open('my-plugin-viewport', {
          label: 'My Plugin'
        });
      }
    });

    // Register a widget (dashboard component)
    api.widget('my-plugin-widget', {
      title: 'My Widget',
      component: MyWidget,
      icon: IconPlugin,
      description: 'A dashboard widget',
      defaultSize: { w: 2, h: 2 },
      minSize: { w: 1, h: 1 },
      maxSize: { w: 4, h: 4 }
    });
  },

  async onStop() {
    console.log('[My Plugin] Stopping...');
  }
});
```

## Widgets

Widgets are draggable components that can be placed on the dashboard. Create widget files in a `widgets/` directory:

```jsx
// widgets/MyWidget.jsx
import { createSignal } from 'solid-js';
import { IconPlugin } from '@tabler/icons-solidjs';

export default function MyWidget() {
  const [count, setCount] = createSignal(0);

  return (
    <div class="card bg-gradient-to-br from-primary/20 to-primary/5 bg-base-100 shadow-lg h-full flex flex-col p-4">
      {/* Header */}
      <div class="flex items-center justify-between mb-2">
        <div class="flex items-center gap-2">
          <IconPlugin size={20} class="text-primary opacity-80" />
          <span class="text-sm font-medium opacity-70">My Widget</span>
        </div>
      </div>

      {/* Content */}
      <div class="flex-1 flex flex-col items-center justify-center">
        <div class="text-4xl font-bold text-primary mb-4">
          {count()}
        </div>
        <button
          class="btn btn-primary btn-sm"
          onClick={() => setCount(count() + 1)}
        >
          Increment
        </button>
      </div>

      {/* Footer */}
      <div class="text-xs opacity-50 text-center mt-2">
        Click to increment
      </div>
    </div>
  );
}
```

**Important**: Widgets must be imported and registered in `index.jsx` using `api.widget()` to be included in the build.

## Backend Plugin (Rust)

Backend plugins use the **webarcade_api** wrapper, which provides a safe, sandboxed interface to the WebArcade core.

### Why Use webarcade_api?

- ✅ **Security**: Sandboxed API prevents malicious plugins from accessing internal systems
- ✅ **Safety**: Type-safe boundaries with automatic memory management
- ✅ **Simplicity**: Clean abstractions hide FFI complexity
- ✅ **Maintainability**: Easy-to-use async functions instead of raw C exports

### Cargo.toml

Every backend plugin needs a `Cargo.toml` file:

```toml
[package]
name = "my-plugin"
version = "1.0.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]  # Dynamic library for plugin loading
path = "mod.rs"

[dependencies]
# webarcade_api is automatically injected by the plugin builder
# Add any additional dependencies your plugin needs here

[profile.release]
opt-level = "z"      # Optimize for size
lto = true           # Link-time optimization
codegen-units = 1
strip = true         # Strip symbols
```

**Note**: The `webarcade_api` dependency is automatically added by the plugin builder with the correct absolute path, so you don't need to include it in your Cargo.toml. The builder will handle it for you!

### mod.rs

```rust
mod router;

use webarcade_api::prelude::*;
use std::sync::Arc;

pub struct MyPlugin;

#[async_trait]
impl Plugin for MyPlugin {
    plugin_metadata!(
        "my-plugin",
        "My Plugin",
        "1.0.0",
        "Does backend things",
        author: "Your Name"
    );

    async fn init(&self, ctx: &Context) -> Result<()> {
        log::info!("[My Plugin] Initializing...");

        // Database migrations (optional)
        ctx.migrate(&[
            r"
            CREATE TABLE IF NOT EXISTS my_plugin_data (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                value TEXT NOT NULL,
                created_at INTEGER NOT NULL
            )
            ",
        ])?;

        // Register API routes
        router::register_routes(ctx).await?;

        Ok(())
    }

    async fn start(&self, _ctx: Arc<Context>) -> Result<()> {
        log::info!("[My Plugin] Starting...");
        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        log::info!("[My Plugin] Stopping...");
        Ok(())
    }
}
```

### router.rs

```rust
use webarcade_api::prelude::*;

pub async fn register_routes(ctx: &Context) -> Result<()> {
    let mut router = Router::new();

    // Register route handlers
    route!(router, GET "/hello" => handle_hello);
    route!(router, GET "/data" => handle_data);

    // Register the router with your plugin ID
    ctx.register_router("my-plugin", router).await;

    Ok(())
}

async fn handle_hello() -> HttpResponse {
    let response = json!({
        "message": "Hello from my plugin!"
    });

    json_response(&response)
}

#[derive(Serialize)]
struct DataResponse {
    timestamp: u64,
    value: String,
}

async fn handle_data() -> HttpResponse {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let data = DataResponse {
        timestamp,
        value: String::from("Data from Rust backend!"),
    };

    json_response(&data)
}
```

### Database Access

Use the Context to interact with the plugin database:

```rust
async fn save_data(value: String) -> Result<()> {
    let ctx = Context::global();
    let db = ctx.db();

    let params = json!({
        "value": value,
        "created_at": timestamp()
    });

    db.execute(
        "INSERT INTO my_plugin_data (value, created_at) VALUES (?1, ?2)",
        &params
    ).await?;

    Ok(())
}

async fn get_all_data() -> Result<Vec<MyData>> {
    let ctx = Context::global();
    let db = ctx.db();

    let results = db.query(
        "SELECT * FROM my_plugin_data ORDER BY created_at DESC",
        &json!({})
    ).await?;

    Ok(results)
}
```

## Using Third-Party JavaScript Libraries

You can add any npm package to your plugin by adding it to `package.json`:

```json
{
  "name": "my-plugin",
  "version": "1.0.0",
  "description": "My awesome plugin",
  "author": "Your Name",
  "dependencies": {
    "canvas-confetti": "^1.9.2",
    "date-fns": "^3.0.0",
    "chart.js": "^4.4.0"
  }
}
```

Then use them in your plugin code:

```jsx
import confetti from 'canvas-confetti';
import { format } from 'date-fns';

export default function MyViewport() {
  const celebrate = () => {
    confetti({
      particleCount: 100,
      spread: 70,
      origin: { y: 0.6 }
    });
  };

  return (
    <div>
      <p>Today is {format(new Date(), 'MMMM do, yyyy')}</p>
      <button onClick={celebrate}>Celebrate!</button>
    </div>
  );
}
```

**How it works:**
1. The plugin builder automatically detects dependencies in `package.json`
2. Runs `bun install` or `npm install` before bundling
3. Bundles the dependencies into your `plugin.js`
4. Dependencies are included in the final plugin package

**Note:** External dependencies increase your plugin size. Use them wisely!

## Calling Backend from Frontend

Use the `bridge` API to call backend routes:

```jsx
import { bridge } from '@/api/bridge';

async function fetchData() {
  try {
    // Calls /my-plugin/hello endpoint
    const response = await bridge('/my-plugin/hello');
    const data = await response.json();
    console.log(data.message);
  } catch (err) {
    console.error('Failed to fetch:', err);
  }
}
```

## Building Your Plugin for Distribution

### Using the Developer IDE

1. Open the Developer IDE
2. Select your plugin from the dropdown
3. Click **"Build"** in the toolbar
4. The plugin will be compiled to `dist/plugins/your-plugin-name.zip`
5. Distribute the `.zip` file

### Manual Build

```bash
# Build Rust backend (if applicable)
cd src/plugins/developer/projects/my-plugin
cargo build --release

# Bundle frontend with rspack/webpack
npm run build:plugin

# Create zip
zip -r my-plugin.zip manifest.json plugin.js my-plugin.dll
```

## Security Considerations

Since WebArcade is **open-source**, developers have access to the codebase. To maintain security:

### For Plugin Developers

- ✅ **Use webarcade_api wrapper**: Provides safe, sandboxed access to plugin APIs
- ✅ **Declare permissions clearly**: Document what your plugin accesses (database, network, filesystem)
- ✅ **Follow security best practices**: Validate inputs, sanitize outputs, use HTTPS for network calls
- ❌ **Avoid direct FFI**: Don't create manual `#[no_mangle]` exports to bypass the API wrapper

### For Users

- ✅ **Install trusted plugins**: Only install plugins from known developers or the official marketplace
- ✅ **Review source code**: Check the plugin's source if available
- ✅ **Check permissions**: Understand what APIs the plugin uses before installing
- ⚠️ **Understand risks**: Plugins have access to WebArcade's runtime environment

### Future Enhancements

- 🔐 **Plugin signing**: Cryptographic signatures for verified plugins
- 📋 **Permission system**: Plugins declare required permissions (database, network, filesystem)
- 🛡️ **Sandboxing**: Process isolation for untrusted plugins
- 🏪 **Plugin marketplace**: Curated, reviewed plugins with reputation scores

## API Reference

### Plugin API Methods (Frontend)

- `api.viewport(id, config)` - Register a viewport tab
- `api.menu(id, config)` - Register a menu item
- `api.widget(id, config)` - Register a dashboard widget
- `api.open(viewportId, options)` - Open a viewport
- `api.showProps(visible)` - Show/hide properties panel
- `api.showMenu(visible)` - Show/hide menu
- `api.showFooter(visible)` - Show/hide footer
- `api.showTabs(visible)` - Show/hide tabs

### Context API Methods (Backend)

- `ctx.register_router(plugin_id, router)` - Register HTTP routes
- `ctx.migrate(migrations)` - Run database migrations
- `ctx.db()` - Access plugin database
- `ctx.emit(event_name, data)` - Emit events
- `Context::global()` - Get global context from async handlers

### Router Methods

- `route!(router, GET "/path" => handler)` - Register GET route
- `route!(router, POST "/path" => handler)` - Register POST route
- `route!(router, PUT "/path" => handler)` - Register PUT route
- `route!(router, DELETE "/path" => handler)` - Register DELETE route

### Response Helpers

- `json_response(data)` - Create JSON response
- `error_response(status, message)` - Create error response

## Tips

- **Use unique IDs**: Plugin IDs must be unique across all plugins
- **Follow naming conventions**: Use kebab-case for IDs (e.g., `my-awesome-plugin`)
- **Test thoroughly**: Test your plugin before distributing
- **Keep it small**: Only include necessary files in the zip
- **Version properly**: Use semantic versioning (major.minor.patch)
- **Import widgets**: Always import and register widgets in index.jsx
- **Use webarcade_api**: Stick to the safe API wrapper for backend code

## Example Plugins

Check the `src/plugins/` directory for built-in examples:
- `dashboard` - Dashboard with widget grid
- `developer` - Developer IDE with project management
- `system` - System widgets (CPU, Memory, Clock, etc.)

Check `src/plugins/developer/projects/demo` for a complete fullstack example.

## Getting Help

- Check existing plugins in `src/plugins/` and `src/plugins/developer/projects/`
- Review the Plugin API in `src/api/plugin/`
- Review the webarcade_api in `src-tauri/api/`
- Open an issue on GitHub for questions

Happy plugin development! 🚀
