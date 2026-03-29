use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    match args.get(1).map(|s| s.as_str()) {
        Some("new") => {
            let name = match args.get(2) {
                Some(n) => n,
                None => {
                    eprintln!("Usage: webarcade new <project-name>");
                    std::process::exit(1);
                }
            };
            create_project(name);
        }
        Some("--version" | "-V") => {
            println!("webarcade {}", env!("CARGO_PKG_VERSION"));
        }
        _ => {
            println!("WebArcade - SolidJS + Rust desktop framework");
            println!();
            println!("Usage:");
            println!("  webarcade new <project-name>    Create a new project");
            println!("  webarcade --version             Show version");
        }
    }
}

fn create_project(name: &str) {
    let root = Path::new(name);

    if root.exists() {
        eprintln!("Error: directory '{}' already exists", name);
        std::process::exit(1);
    }

    println!("Creating project '{}'...", name);

    let dirs = ["src", "src/components"];
    for dir in &dirs {
        fs::create_dir_all(root.join(dir)).expect("Failed to create directory");
    }

    // Cargo.toml
    write(root.join("Cargo.toml"), &format!(
r#"[package]
name = "{name}"
version = "0.1.0"
edition = "2021"

[dependencies]
webarcade = "1"
serde_json = "1"
"#));

    // src/main.rs
    write(root.join("src/main.rs"), &format!(
r#"use webarcade::{{App, Request, Response}};

fn main() {{
    App::new("{name}", 1280, 720)
        .min_size(800, 600)
        .route("GET", "/api/greet", handle_greet)
        .frontend("dist")
        .run();
}}

fn handle_greet(req: Request) -> Response {{
    let name = req.query("name").unwrap_or("World");
    Response::json(&serde_json::json!({{
        "message": format!("Hello, {{}}!", name)
    }}))
}}
"#));

    // package.json
    write(root.join("package.json"), &format!(
r#"{{
  "name": "{name}",
  "private": true,
  "type": "module",
  "scripts": {{
    "dev": "vite",
    "build": "vite build"
  }},
  "dependencies": {{
    "solid-js": "^1.9.0"
  }},
  "devDependencies": {{
    "vite": "^6.0.0",
    "vite-plugin-solid": "^2.11.0",
    "tailwindcss": "^4.0.0",
    "@tailwindcss/vite": "^4.0.0",
    "daisyui": "^5.0.0"
  }}
}}
"#));

    // vite.config.js
    write(root.join("vite.config.js"),
r#"import { defineConfig } from "vite";
import solid from "vite-plugin-solid";
import tailwindcss from "@tailwindcss/vite";

export default defineConfig({
  plugins: [solid(), tailwindcss()],
  build: {
    outDir: "dist",
    emptyOutDir: true,
  },
});
"#);

    // index.html
    write(root.join("index.html"), &format!(
r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{name}</title>
</head>
<body>
    <div id="app"></div>
    <script type="module" src="/src/index.jsx"></script>
</body>
</html>
"#));

    // src/index.css
    write(root.join("src/index.css"),
r#"@import "tailwindcss";
@plugin "daisyui";
"#);

    // src/index.jsx
    write(root.join("src/index.jsx"),
r#"import { render } from "solid-js/web";
import App from "./App";
import "./index.css";

render(() => <App />, document.getElementById("app"));
"#);

    // src/App.jsx
    write(root.join("src/App.jsx"),
r#"import { createSignal, createResource, Show } from "solid-js";
import Titlebar from "./components/Titlebar";

const fetchGreet = async (name) => {
  const resp = await fetch(`/api/greet?name=${name}`);
  return resp.json();
};

export default function App() {
  const [name, setName] = createSignal("World");
  const [data] = createResource(name, fetchGreet);

  return (
    <div class="flex flex-col h-screen bg-base-300 text-base-content">
      <Titlebar />
      <div class="flex-1 flex flex-col items-center justify-center gap-6 p-8">
        <h1 class="text-4xl font-bold">Welcome to WebArcade</h1>
        <input
          type="text"
          class="input input-bordered w-64"
          placeholder="Enter a name"
          value={name()}
          onInput={(e) => setName(e.target.value)}
        />
        <Show when={!data.loading} fallback={<span class="loading loading-spinner" />}>
          <pre class="bg-base-200 p-4 rounded-lg text-sm">
            {JSON.stringify(data(), null, 2)}
          </pre>
        </Show>
      </div>
    </div>
  );
}
"#);

    // src/components/Titlebar.jsx
    write(root.join("src/components/Titlebar.jsx"),
r#"export default function Titlebar() {
  const win = window.__WEBARCADE__?.window;

  return (
    <div class="h-9 bg-base-200 flex items-center justify-between px-3 select-none" data-drag-region>
      <span class="text-sm opacity-70">WebArcade</span>
      <div class="flex gap-1">
        <button class="btn btn-ghost btn-xs" onClick={() => win?.minimize()}>&#x2014;</button>
        <button class="btn btn-ghost btn-xs" onClick={() => win?.toggleMaximize()}>&#x25A1;</button>
        <button class="btn btn-ghost btn-xs hover:btn-error" onClick={() => win?.close()}>&#x2715;</button>
      </div>
    </div>
  );
}
"#);

    // .gitignore
    write(root.join(".gitignore"),
r#"target/
dist/
node_modules/
"#);

    // Install npm deps and build
    println!("Installing dependencies...");
    if run_cmd("npm", &["install"], root) {
        println!("Building frontend...");
        run_cmd("npm", &["run", "build"], root);
    } else {
        println!("Warning: npm install failed. Run 'npm install' manually.");
    }

    println!();
    println!("Done! To get started:");
    println!();
    println!("  cd {name}");
    println!("  cargo run");
    println!();
    println!("To rebuild frontend after changes:");
    println!();
    println!("  npm run build && cargo run");
}

fn write(path: impl AsRef<Path>, content: &str) {
    fs::write(&path, content).unwrap_or_else(|e| {
        eprintln!("Failed to write {}: {}", path.as_ref().display(), e);
        std::process::exit(1);
    });
}

fn run_cmd(cmd: &str, args: &[&str], dir: &Path) -> bool {
    Command::new(cmd)
        .args(args)
        .current_dir(dir)
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}
