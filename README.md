# WebArcade

Build desktop apps with Rust and any web frontend. No Electron, no bundled browser — uses your system's native webview.

Your frontend talks to Rust with `fetch()`. That's it.

## Install

```sh
cargo install webarcade
```

## Create a project

```sh
webarcade new my-app
cd my-app
npm install && npm run build
cargo run
```

## What you get

**Rust** (`src/main.rs`):

```rust
use webarcade::{App, Request, Response};

fn main() {
    App::new("My App", 1280, 720)
        .route("GET", "/api/greet", |req: Request| {
            let name = req.query("name").unwrap_or("World");
            Response::json(&serde_json::json!({ "message": format!("Hello, {}!", name) }))
        })
        .frontend("dist")
        .run();
}
```

**Frontend** (any framework — SolidJS, React, Svelte, plain HTML):

```js
const data = await fetch("/api/greet?name=WebArcade").then(r => r.json());
// { message: "Hello, WebArcade!" }
```

The frontend calls `fetch()`, Rust handles it, returns data. No special bindings, no codegen.

## Routes

Register routes in Rust, call them from the frontend:

```rust
App::new("My App", 1280, 720)
    .route("GET", "/api/users", get_users)
    .route("POST", "/api/users", create_user)
    .frontend("dist")
    .run();

fn get_users(_req: Request) -> Response {
    Response::json(&serde_json::json!([{ "name": "Alice" }]))
}

fn create_user(req: Request) -> Response {
    let body: serde_json::Value = req.json().unwrap();
    Response::json(&serde_json::json!({ "created": body["name"] }))
}
```

### Request

```rust
req.query("key")           // query parameter
req.json::<T>()            // parse body as JSON
req.text()                 // body as string
req.body                   // raw bytes
req.method                 // "GET", "POST", etc.
req.path                   // "/api/users"
```

### Response

```rust
Response::json(&data)                       // JSON
Response::text("hello")                     // plain text
Response::bytes(vec, "image/png")           // raw bytes
Response::error(404, "Not found")           // error
Response::json(&data).with_status(201)      // custom status
```

## Window controls

The framework injects `window.__WEBARCADE__` for native window control:

```js
const win = __WEBARCADE__.window;

win.minimize();
win.maximize();
win.toggleMaximize();
win.close();
win.setSize(1920, 1080);
win.center();
win.setTitle("New Title");
```

For a custom titlebar, add `data-drag-region` to any element:

```html
<div data-drag-region>
    <span>My App</span>
    <button onclick="__WEBARCADE__.window.close()">X</button>
</div>
```

Drag to move, double-click to maximize. Buttons inside still work.

## Platform support

| Platform | Webview |
|----------|---------|
| Windows  | WebView2 |
| macOS    | WebKit |
| Linux    | WebKitGTK |

## License

MIT
