# WebArcade

A lightweight desktop framework for building apps with a Rust backend and web frontend. Uses system webviews — no Electron, no bundled browser.

## How it works

Your Rust code defines API routes. Your frontend (HTML/JS — SolidJS, React, plain, whatever) calls those routes with `fetch()`. The framework handles the window, protocol routing, and IPC bridge between them.

```
Frontend (fetch("/api/data"))  →  app:// protocol  →  Rust handler  →  JSON response
```

## Quick start

Add `webarcade` to your project:

```toml
[dependencies]
webarcade = { git = "https://github.com/renzora/webarcade" }
serde_json = "1"
```

Write your app:

```rust
#![windows_subsystem = "windows"]

use webarcade::{App, Request, Response};
use webarcade::include_dir::include_dir;

static DIST: webarcade::include_dir::Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/dist");

fn main() {
    App::new("My App", 1024, 768)
        .route("GET", "/api/greet", |req: Request| {
            let name = req.query("name").unwrap_or("World");
            Response::json(&serde_json::json!({ "message": format!("Hello, {}!", name) }))
        })
        .frontend(&DIST)
        .run();
}
```

Create `dist/index.html`:

```html
<!DOCTYPE html>
<html>
<body>
    <div id="result"></div>
    <script>
        fetch("/api/greet?name=WebArcade")
            .then(r => r.json())
            .then(data => document.getElementById("result").textContent = data.message);
    </script>
</body>
</html>
```

Build and run:

```sh
cargo run
```

Single binary output. No config files, no plugins, no CLI tools.

## API

### App

```rust
App::new("Title", width, height)    // Create app with window title and size
    .min_size(400, 300)             // Set minimum window size (optional)
    .decorations(true)              // Use native titlebar (default: false)
    .route("GET", "/path", handler) // Register a route
    .frontend(&DIST)                // Set frontend directory
    .run();                         // Launch (blocks until closed)
```

### Request

```rust
fn handler(req: Request) -> Response {
    req.method                      // "GET", "POST", etc.
    req.path                        // "/api/greet"
    req.body                        // Raw bytes (Vec<u8>)
    req.query("key")                // Get a query parameter
    req.json::<MyStruct>()          // Parse body as JSON
    req.text()                      // Body as string
}
```

### Response

```rust
Response::json(&data)               // JSON with 200 status
Response::text("hello")             // Plain text with 200 status
Response::bytes(vec, "image/png")   // Raw bytes with content type
Response::error(404, "Not found")   // Error JSON with status code

// Chaining
Response::json(&data)
    .with_status(201)
    .with_header("X-Custom", "value")
```

### Frontend

Use `fetch()` with relative paths to call your Rust routes:

```js
// GET with query params
const resp = await fetch("/api/users?page=1");
const data = await resp.json();

// POST with body
const resp = await fetch("/api/users", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ name: "Alice" })
});
```

### Window controls (JS)

The framework injects `window.__WEBARCADE__` into the webview:

```js
const win = __WEBARCADE__.window;

await win.minimize();
await win.maximize();
await win.unmaximize();
await win.toggleMaximize();
await win.close();
await win.setFullscreen(true);
await win.setSize(1920, 1080);
await win.getSize();               // { width, height }
await win.setPosition(100, 100);
await win.getPosition();           // { x, y }
await win.setMinSize(400, 300);
await win.setMaxSize(1920, 1080);
await win.center();
await win.setTitle("New Title");
await win.isMaximized();           // true/false

// Check if running in desktop app
__WEBARCADE__.isNative              // true in app, false in browser
```

### Custom titlebar

Disable decorations (default) and add `data-drag-region` to your titlebar element:

```html
<div data-drag-region>
    <span>My App</span>
    <button onclick="__WEBARCADE__.window.close()">X</button>
</div>
```

The drag region automatically handles:
- Click and drag to move the window
- Double-click to toggle maximize
- Buttons/inputs inside the region remain clickable

## Frontend framework

WebArcade doesn't prescribe a frontend framework. Put whatever you want in `dist/` — SolidJS, React, Svelte, plain HTML. The framework just serves it and routes API calls to Rust.

For SPA routing, paths without file extensions automatically fall back to `index.html`.

## Building for release

```sh
cargo build --release
```

The frontend is embedded in the binary via `include_dir!`, so the output is a single executable.

## Platform support

| Platform | Webview | Status |
|----------|---------|--------|
| Windows  | WebView2 | Supported |
| macOS    | WebKit   | Supported |
| Linux    | WebKitGTK | Supported |

## License

MIT
