//! WebArcade CLI - Plugin Builder & App Packager
//!
//! A standalone CLI tool for building WebArcade plugins and packaging the app.
//!
//! Usage:
//!   webarcade new <plugin-id>       Create a new plugin project
//!   webarcade build <plugin-id>     Build a specific plugin
//!   webarcade build --all           Build all plugins
//!   webarcade list                  List available plugins
//!   webarcade package               Package the app (interactive)
//!   webarcade package --locked      Package with embedded plugins

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use dialoguer::{Input, Select, Confirm, theme::ColorfulTheme};
use console::style;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Parser)]
#[command(name = "webarcade")]
#[command(about = "WebArcade CLI - Build plugins and package apps")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new plugin project
    New {
        /// Plugin ID (e.g., my-plugin)
        plugin_id: String,

        /// Plugin display name
        #[arg(short, long)]
        name: Option<String>,

        /// Plugin author
        #[arg(short, long)]
        author: Option<String>,

        /// Create frontend-only plugin (no Rust backend)
        #[arg(long)]
        frontend_only: bool,
    },
    /// Build a plugin from source
    Build {
        /// Plugin ID to build (or --all to build all)
        plugin_id: Option<String>,

        /// Build all plugins
        #[arg(long)]
        all: bool,
    },
    /// List available plugins in projects/
    List,
    /// Package the app for distribution
    Package {
        /// Skip interactive prompts and use current config
        #[arg(long)]
        skip_prompts: bool,

        /// Use locked mode (embed plugins in binary)
        #[arg(long)]
        locked: bool,

        /// App name (skips prompt)
        #[arg(long)]
        name: Option<String>,

        /// App version (skips prompt)
        #[arg(long)]
        version: Option<String>,

        /// App description (skips prompt)
        #[arg(long)]
        description: Option<String>,

        /// App author (skips prompt)
        #[arg(long)]
        author: Option<String>,
    },
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Some(cmd) => run_command(cmd),
        None => interactive_menu(),
    };

    if let Err(e) = result {
        eprintln!("{} {}", style("Error:").red().bold(), e);
        std::process::exit(1);
    }
}

fn run_command(cmd: Commands) -> Result<()> {
    match cmd {
        Commands::New { plugin_id, name, author, frontend_only } => {
            create_plugin(&plugin_id, name, author, frontend_only)
        }
        Commands::Build { plugin_id, all } => {
            if all {
                build_all_plugins()
            } else if let Some(id) = plugin_id {
                build_plugin(&id)
            } else {
                anyhow::bail!("Please specify a plugin ID or use --all");
            }
        }
        Commands::List => list_plugins(),
        Commands::Package { skip_prompts, locked, name, version, description, author } => {
            package_app(skip_prompts, locked, name, version, description, author)
        }
    }
}

fn print_banner() {
    println!();
    println!("{}", style(r#"
    ╦ ╦┌─┐┌┐ ╔═╗┬─┐┌─┐┌─┐┌┬┐┌─┐
    ║║║├┤ ├┴┐╠═╣├┬┘│  ├─┤ ││├┤
    ╚╩╝└─┘└─┘╩ ╩┴└─└─┘┴ ┴─┴┘└─┘"#).cyan().bold());
    println!("    {}", style("Build amazing desktop apps with ease").dim());
    println!();
}

fn wait_for_enter() {
    println!();
    print!("{}", style("Press Enter to continue...").dim());
    std::io::stdout().flush().unwrap();
    let _ = std::io::stdin().read_line(&mut String::new());
}

fn clear_screen() {
    // Clear screen and move cursor to top
    print!("\x1B[2J\x1B[1;1H");
    std::io::stdout().flush().unwrap();
}

fn interactive_menu() -> Result<()> {
    let theme = ColorfulTheme::default();

    clear_screen();
    print_banner();

    loop {
        let menu_items = vec![
            "📦 Package App        - Build and create installer",
            "🔨 Build Plugin       - Compile a plugin",
            "✨ Create Plugin      - Create a new plugin project",
            "📋 List Plugins       - Show available plugins",
            "🚪 Exit",
        ];

        let selection = Select::with_theme(&theme)
            .with_prompt("What would you like to do?")
            .items(&menu_items)
            .default(0)
            .interact()?;

        println!();

        let result = match selection {
            0 => package_app(false, false, None, None, None, None),
            1 => interactive_build_plugin(),
            2 => interactive_create_plugin(),
            3 => list_plugins(),
            4 => {
                println!("{}", style("👋 Goodbye! Happy coding!").cyan());
                println!();
                return Ok(());
            }
            _ => Ok(()),
        };

        if let Err(e) = result {
            eprintln!("{} {}", style("Error:").red().bold(), e);
        }

        wait_for_enter();
        clear_screen();
        print_banner();
    }
}

fn interactive_build_plugin() -> Result<()> {
    let theme = ColorfulTheme::default();
    let plugins_dir = get_plugins_dir()?;

    // Get list of plugin directories
    let mut plugins: Vec<String> = Vec::new();
    if plugins_dir.exists() {
        for entry in fs::read_dir(&plugins_dir)? {
            let entry = entry?;
            if entry.path().is_dir() {
                plugins.push(entry.file_name().to_string_lossy().to_string());
            }
        }
    }

    if plugins.is_empty() {
        println!("{}", style("No plugins found. Create one first!").yellow());
        return Ok(());
    }

    // Add "Build All" option
    let mut options = vec!["🔨 Build All Plugins".to_string()];
    for plugin in &plugins {
        options.push(format!("   {}", plugin));
    }
    options.push("← Back".to_string());

    let selection = Select::with_theme(&theme)
        .with_prompt("Select a plugin to build")
        .items(&options)
        .default(0)
        .interact()?;

    println!();

    if selection == 0 {
        build_all_plugins()
    } else if selection == options.len() - 1 {
        Ok(()) // Back to menu
    } else {
        let plugin_id = &plugins[selection - 1];
        build_plugin(plugin_id)
    }
}

fn interactive_create_plugin() -> Result<()> {
    let theme = ColorfulTheme::default();

    let plugin_id: String = Input::with_theme(&theme)
        .with_prompt("Plugin ID (e.g., my-plugin)")
        .validate_with(|input: &String| {
            if input.is_empty() {
                Err("Plugin ID cannot be empty")
            } else if !input.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
                Err("Plugin ID can only contain letters, numbers, hyphens, and underscores")
            } else {
                Ok(())
            }
        })
        .interact_text()?;

    let display_name: String = Input::with_theme(&theme)
        .with_prompt("Display name")
        .default(plugin_id.split(|c| c == '-' || c == '_')
            .map(|s| {
                let mut chars = s.chars();
                match chars.next() {
                    Some(c) => c.to_uppercase().chain(chars).collect(),
                    None => String::new(),
                }
            })
            .collect::<Vec<String>>()
            .join(" "))
        .interact_text()?;

    let author: String = Input::with_theme(&theme)
        .with_prompt("Author")
        .default("WebArcade".to_string())
        .interact_text()?;

    let plugin_types = vec![
        "Full-stack (frontend + Rust backend)",
        "Frontend-only (just JavaScript)",
    ];
    let type_selection = Select::with_theme(&theme)
        .with_prompt("Plugin type")
        .items(&plugin_types)
        .default(0)
        .interact()?;

    let frontend_only = type_selection == 1;

    println!();

    create_plugin(&plugin_id, Some(display_name), Some(author), frontend_only)
}

/// Get the repo root directory (where cli, plugins, app folders are)
fn get_repo_root() -> Result<PathBuf> {
    let mut current = std::env::current_dir()?;

    // Check if we're already at repo root
    if current.join("plugins").exists() && current.join("app").exists() {
        return Ok(current);
    }

    // Check if we're in cli/ directory
    if current.ends_with("cli") {
        if let Some(parent) = current.parent() {
            if parent.join("plugins").exists() {
                return Ok(parent.to_path_buf());
            }
        }
    }

    // Walk up the directory tree
    loop {
        if current.join("plugins").exists() && current.join("app").exists() {
            return Ok(current);
        }
        if !current.pop() {
            break;
        }
    }

    anyhow::bail!("Could not find repo root (looking for plugins/ and app/ directories)")
}

fn get_plugins_dir() -> Result<PathBuf> {
    Ok(get_repo_root()?.join("plugins"))
}

fn get_build_dir() -> Result<PathBuf> {
    Ok(get_repo_root()?.join("build"))
}

fn get_dist_plugins_dir() -> Result<PathBuf> {
    Ok(get_repo_root()?.join("app").join("plugins"))
}

fn create_plugin(plugin_id: &str, name: Option<String>, author: Option<String>, frontend_only: bool) -> Result<()> {
    let plugins_dir = get_plugins_dir()?;
    let plugin_dir = plugins_dir.join(plugin_id);

    // Validate plugin ID
    if !plugin_id.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
        anyhow::bail!("Plugin ID must only contain alphanumeric characters, hyphens, and underscores");
    }

    if plugin_dir.exists() {
        anyhow::bail!("Plugin '{}' already exists at {}", plugin_id, plugin_dir.display());
    }

    // Create plugin directory
    fs::create_dir_all(&plugin_dir)?;

    let display_name = name.unwrap_or_else(|| {
        // Convert plugin-id to "Plugin Id"
        plugin_id
            .split(|c| c == '-' || c == '_')
            .map(|s| {
                let mut chars = s.chars();
                match chars.next() {
                    Some(c) => c.to_uppercase().chain(chars).collect(),
                    None => String::new(),
                }
            })
            .collect::<Vec<String>>()
            .join(" ")
    });

    let author_name = author.unwrap_or_else(|| "WebArcade".to_string());

    // Generate struct name from plugin_id (my-plugin -> MyPlugin)
    let struct_name = plugin_id
        .split(|c| c == '-' || c == '_')
        .map(|s| {
            let mut chars = s.chars();
            match chars.next() {
                Some(c) => c.to_uppercase().chain(chars).collect(),
                None => String::new(),
            }
        })
        .collect::<String>() + "Plugin";

    println!("Creating plugin: {}", plugin_id);
    println!("  Location: {}", plugin_dir.display());
    println!("  Name: {}", display_name);
    println!("  Author: {}", author_name);
    println!("  Type: {}", if frontend_only { "frontend-only" } else { "full-stack" });
    println!();

    // Create index.jsx (always required)
    let index_jsx = if frontend_only {
        format!(r#"import {{ plugin }} from '@/api/plugin';

export default plugin({{
    id: '{plugin_id}',
    name: '{display_name}',
    version: '1.0.0',
    description: '{display_name} plugin',
    author: '{author_name}',

    start(api) {{
        // Register the plugin tab (shows in main tab bar)
        api.add({{
            panel: 'tab',
            label: '{display_name}',
        }});

        // Register the main viewport
        api.add({{
            panel: 'viewport',
            id: 'main',
            label: '{display_name}',
            component: () => (
                <div class="flex items-center justify-center h-full">
                    <h1 class="text-4xl font-bold">{display_name}</h1>
                </div>
            ),
        }});
    }},

    active(api) {{
        console.log('[{display_name}] Activated');
    }},

    inactive(api) {{
        console.log('[{display_name}] Deactivated');
    }},

    stop(api) {{
        console.log('[{display_name}] Stopped');
    }}
}});
"#)
    } else {
        format!(r#"import {{ plugin }} from '@/api/plugin';
import Viewport from './viewport';

export default plugin({{
    id: '{plugin_id}',
    name: '{display_name}',
    version: '1.0.0',
    description: '{display_name} plugin',
    author: '{author_name}',

    start(api) {{
        // Register the plugin tab (shows in main tab bar)
        api.add({{
            panel: 'tab',
            label: '{display_name}',
        }});

        // Register the main viewport
        api.add({{
            panel: 'viewport',
            id: 'main',
            label: '{display_name}',
            component: Viewport,
        }});

        // Example: Register left panel tab
        // api.add({{
        //     panel: 'left',
        //     id: 'explorer',
        //     label: 'Explorer',
        //     component: ExplorerPanel,
        // }});

        // Example: Register bottom panel tab
        // api.add({{
        //     panel: 'bottom',
        //     id: 'console',
        //     label: 'Console',
        //     component: ConsolePanel,
        // }});
    }},

    active(api) {{
        console.log('[{display_name}] Activated');
    }},

    inactive(api) {{
        console.log('[{display_name}] Deactivated');
    }},

    stop(api) {{
        console.log('[{display_name}] Stopped');
    }}
}});
"#)
    };
    fs::write(plugin_dir.join("index.jsx"), index_jsx)?;
    println!("  Created index.jsx");

    if !frontend_only {
        // Create viewport.jsx
        let viewport_jsx = format!(r#"import {{ createSignal, onMount }} from 'solid-js';
import {{ api }} from '@/api/bridge';

export default function Viewport() {{
    const [message, setMessage] = createSignal('Loading...');

    onMount(async () => {{
        try {{
            const response = await api('{plugin_id}/hello');
            const data = await response.json();
            setMessage(data.message);
        }} catch (error) {{
            setMessage('Error: ' + error.message);
        }}
    }});

    return (
        <div class="p-4">
            <h1 class="text-xl font-bold mb-4">{display_name}</h1>
            <p class="text-base-content/70">{{message()}}</p>
        </div>
    );
}}
"#);
        fs::write(plugin_dir.join("viewport.jsx"), viewport_jsx)?;
        println!("  Created viewport.jsx");

        // Create Cargo.toml
        let cargo_toml = format!(r#"[package]
name = "{plugin_id}"
version = "1.0.0"
edition = "2021"

[routes]
"GET /hello" = "handle_hello"

[profile.release]
opt-level = "z"
lto = true
codegen-units = 1
strip = true
"#);
        fs::write(plugin_dir.join("Cargo.toml"), cargo_toml)?;
        println!("  Created Cargo.toml");

        // Create mod.rs
        let mod_rs = format!(r#"pub mod router;

use api::{{Plugin, PluginMetadata}};

pub struct {struct_name};

impl Plugin for {struct_name} {{
    fn metadata(&self) -> PluginMetadata {{
        PluginMetadata {{
            id: "{plugin_id}".into(),
            name: "{display_name}".into(),
            version: "1.0.0".into(),
            description: "{display_name} plugin".into(),
            author: "{author_name}".into(),
            dependencies: vec![],
        }}
    }}
}}
"#);
        fs::write(plugin_dir.join("mod.rs"), mod_rs)?;
        println!("  Created mod.rs");

        // Create router.rs
        let router_rs = format!(r#"use api::{{HttpRequest, HttpResponse, json, json_response}};

pub async fn handle_hello(_req: HttpRequest) -> HttpResponse {{
    json_response(&json!({{
        "message": "Hello from {display_name}!"
    }}))
}}
"#);
        fs::write(plugin_dir.join("router.rs"), router_rs)?;
        println!("  Created router.rs");
    }

    println!();
    println!("Plugin created successfully!");
    println!();
    println!("Next steps:");
    println!("  1. Edit the plugin files in: {}", plugin_dir.display());
    println!("  2. Build with: bun run plugin:build {}", plugin_id);
    println!("  3. Run the app: bun run dev");

    Ok(())
}

fn list_plugins() -> Result<()> {
    let plugins_dir = get_plugins_dir()?;

    if !plugins_dir.exists() {
        println!("No plugins directory found at: {}", plugins_dir.display());
        return Ok(());
    }

    println!("Plugins in {}:", plugins_dir.display());
    println!();

    let mut sources = Vec::new();
    let mut compiled = Vec::new();

    for entry in fs::read_dir(&plugins_dir)? {
        let entry = entry?;
        let path = entry.path();
        let name = entry.file_name();
        let name_str = name.to_string_lossy();

        if path.is_dir() {
            // Source directory
            let has_backend = path.join("mod.rs").exists() || path.join("Cargo.toml").exists();
            let has_frontend = path.join("index.jsx").exists() || path.join("index.js").exists();

            let type_str = match (has_backend, has_frontend) {
                (true, true) => "full-stack",
                (true, false) => "backend-only",
                (false, true) => "frontend-only",
                (false, false) => "empty",
            };

            sources.push((name_str.to_string(), type_str));
        } else if path.extension().map(|e| e == "dll" || e == "so" || e == "dylib").unwrap_or(false) {
            // Compiled plugin
            let stem = path.file_stem().unwrap_or_default().to_string_lossy();
            // Remove "lib" prefix on Linux/macOS
            let plugin_name = stem.strip_prefix("lib").unwrap_or(&stem).to_string();
            compiled.push(plugin_name);
        }
    }

    if !sources.is_empty() {
        println!("  Source (directories):");
        for (name, type_str) in &sources {
            let is_built = compiled.iter().any(|c| c == name);
            let status = if is_built { "built" } else { "not built" };
            println!("    {} ({}, {})", name, type_str, status);
        }
    }

    if !compiled.is_empty() {
        println!();
        println!("  Compiled (.dll files):");
        for name in &compiled {
            println!("    {}", name);
        }
    }

    if sources.is_empty() && compiled.is_empty() {
        println!("  (no plugins found)");
    }

    Ok(())
}

fn build_all_plugins() -> Result<()> {
    let plugins_dir = get_plugins_dir()?;

    if !plugins_dir.exists() {
        anyhow::bail!("Plugins directory not found: {}", plugins_dir.display());
    }

    let mut plugins = Vec::new();
    for entry in fs::read_dir(&plugins_dir)? {
        let entry = entry?;
        let path = entry.path();
        // Only build source directories, not .dll files
        if path.is_dir() {
            plugins.push(entry.file_name().to_string_lossy().to_string());
        }
    }

    if plugins.is_empty() {
        println!("No plugin source directories found in {}", plugins_dir.display());
        return Ok(());
    }

    println!("Building {} plugins...", plugins.len());
    println!();

    let mut success_count = 0;
    let mut fail_count = 0;

    for plugin_id in &plugins {
        print!("Building {}... ", plugin_id);
        std::io::stdout().flush()?;

        match build_plugin(plugin_id) {
            Ok(_) => {
                println!("OK");
                success_count += 1;
            }
            Err(e) => {
                println!("FAILED: {}", e);
                fail_count += 1;
            }
        }
    }

    println!();
    println!("Results: {} succeeded, {} failed", success_count, fail_count);

    if fail_count > 0 {
        anyhow::bail!("Some plugins failed to build");
    }

    Ok(())
}

fn build_plugin(plugin_id: &str) -> Result<()> {
    let builder = PluginBuilder::new(plugin_id)?;
    builder.build()
}

struct PluginBuilder {
    plugin_id: String,
    plugin_dir: PathBuf,
    build_dir: PathBuf,
    dist_plugins_dir: PathBuf,
    repo_root: PathBuf,
}

impl PluginBuilder {
    fn new(plugin_id: &str) -> Result<Self> {
        let repo_root = get_repo_root()?;
        let plugins_dir = get_plugins_dir()?;
        let plugin_dir = plugins_dir.join(plugin_id);

        if !plugin_dir.exists() {
            anyhow::bail!("Plugin source not found: {}", plugin_dir.display());
        }

        if !plugin_dir.is_dir() {
            anyhow::bail!("Plugin source must be a directory: {}", plugin_dir.display());
        }

        let build_dir = get_build_dir()?.join(plugin_id);
        fs::create_dir_all(&build_dir)?;

        let dist_plugins_dir = get_dist_plugins_dir()?;
        fs::create_dir_all(&dist_plugins_dir)?;

        Ok(Self {
            plugin_id: plugin_id.to_string(),
            plugin_dir,
            build_dir,
            dist_plugins_dir,
            repo_root,
        })
    }

    fn build(&self) -> Result<()> {
        let has_backend = self.plugin_dir.join("mod.rs").exists()
            && self.plugin_dir.join("Cargo.toml").exists();
        let has_frontend = self.plugin_dir.join("index.jsx").exists()
            || self.plugin_dir.join("index.js").exists();

        // Check if plugin has routes (needs bridge feature)
        let has_routes = self.has_routes();

        println!("Building plugin: {} (backend: {}, frontend: {}, routes: {})",
            self.plugin_id, has_backend, has_frontend, has_routes);

        // Clean build directory
        if self.build_dir.exists() {
            fs::remove_dir_all(&self.build_dir)?;
        }
        fs::create_dir_all(&self.build_dir)?;

        // Build frontend first
        if has_frontend {
            println!("  Bundling frontend...");
            self.bundle_frontend()?;
        }

        // Frontend-only plugins: output JS file to app/plugins
        if !has_backend {
            let js_name = format!("{}.js", self.plugin_id);
            println!("  Installing {} to app/plugins/...", js_name);
            let src_plugin_js = self.build_dir.join("plugin.js");
            let dest_plugin_js = self.dist_plugins_dir.join(&js_name);
            if src_plugin_js.exists() {
                fs::copy(&src_plugin_js, &dest_plugin_js)?;
            }

            // Clean up build directory
            println!("  Cleaning up build artifacts...");
            self.cleanup_build_dir()?;

            println!("  Done!");
            return Ok(());
        }

        // Backend plugins: build DLL with embedded frontend
        let frontend_js = if has_frontend {
            let plugin_js_path = self.build_dir.join("plugin.js");
            if plugin_js_path.exists() {
                fs::read_to_string(&plugin_js_path)?
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        // Create package.json / manifest
        let manifest = self.create_manifest()?;

        println!("  Setting up Rust backend...");
        self.setup_backend_build(&frontend_js, &manifest, has_routes)?;

        println!("  Compiling DLL...");
        self.compile_backend()?;

        // Copy final DLL to app/plugins
        println!("  Installing {}.dll to app/plugins/...", self.plugin_id);
        self.install_dll()?;

        // Clean up build directory
        println!("  Cleaning up build artifacts...");
        self.cleanup_build_dir()?;

        println!("  Done!");
        Ok(())
    }

    /// Clean up the build directory after successful build
    fn cleanup_build_dir(&self) -> Result<()> {
        if self.build_dir.exists() {
            fs::remove_dir_all(&self.build_dir)?;
        }

        // Also remove the parent build/ directory if it's empty
        if let Some(parent) = self.build_dir.parent() {
            if parent.exists() {
                if let Ok(entries) = fs::read_dir(parent) {
                    if entries.count() == 0 {
                        let _ = fs::remove_dir(parent);
                    }
                }
            }
        }

        Ok(())
    }

    /// Check if the plugin has routes defined in Cargo.toml
    fn has_routes(&self) -> bool {
        let cargo_toml_path = self.plugin_dir.join("Cargo.toml");
        if !cargo_toml_path.exists() {
            return false;
        }

        if let Ok(content) = fs::read_to_string(&cargo_toml_path) {
            if let Ok(cargo_toml) = content.parse::<toml::Value>() {
                if let Some(routes_table) = cargo_toml.get("routes").and_then(|r| r.as_table()) {
                    return !routes_table.is_empty();
                }
            }
        }
        false
    }

    fn setup_backend_build(&self, frontend_js: &str, manifest: &str, has_routes: bool) -> Result<()> {
        let rust_build_dir = self.build_dir.join("rust_build");
        fs::create_dir_all(&rust_build_dir)?;

        // Copy Rust source files
        self.copy_rust_files(&self.plugin_dir, &rust_build_dir)?;

        // Find API crate
        let api_path = self.repo_root.join("api");
        if !api_path.join("Cargo.toml").exists() {
            anyhow::bail!("API crate not found at: {}", api_path.display());
        }

        let api_path_str = api_path.to_string_lossy().replace("\\", "/");

        // Generate Cargo.toml
        // API dependency with optional bridge feature (only if plugin has routes)
        let api_dep = if has_routes {
            format!("api = {{ path = \"{}\", features = [\"bridge\"] }}", api_path_str)
        } else {
            format!("api = {{ path = \"{}\" }}", api_path_str)
        };

        let plugin_cargo_toml = self.plugin_dir.join("Cargo.toml");
        let cargo_toml = if plugin_cargo_toml.exists() {
            let mut content = fs::read_to_string(&plugin_cargo_toml)?;

            // Inject API dependency path with appropriate features
            let re = regex::Regex::new(r#"api\s*=\s*\{[^}]*\}"#)?;
            content = if re.is_match(&content) {
                re.replace(&content, &api_dep).to_string()
            } else {
                let deps_re = regex::Regex::new(r"(?m)^\[dependencies\]\s*$")?;
                if let Some(mat) = deps_re.find(&content) {
                    let insert_pos = mat.end();
                    let mut new_content = content.clone();
                    new_content.insert_str(insert_pos, &format!("\n{}", api_dep));
                    new_content
                } else {
                    format!("{}\n[dependencies]\n{}\n", content, api_dep)
                }
            };

            // Ensure [lib] section
            let lib_section_re = regex::Regex::new(r"(?m)\n?\[lib\][^\[]*")?;
            content = lib_section_re.replace(&content, "").to_string();

            let package_re = regex::Regex::new(r"(?m)(\[package\][^\[]+)")?;
            if let Some(mat) = package_re.find(&content) {
                let insert_pos = mat.end();
                content.insert_str(insert_pos, "\n[lib]\ncrate-type = [\"cdylib\"]\npath = \"lib.rs\"\n");
            }

            content
        } else {
            format!(
                r#"[package]
name = "{}"
version = "1.0.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]
path = "lib.rs"

[dependencies]
{}

[profile.release]
opt-level = "z"
lto = true
codegen-units = 1
strip = true
"#,
                self.plugin_id, api_dep
            )
        };

        fs::write(rust_build_dir.join("Cargo.toml"), cargo_toml)?;

        // Create .cargo/config.toml
        let cargo_config_dir = rust_build_dir.join(".cargo");
        fs::create_dir_all(&cargo_config_dir)?;
        let cargo_config = r#"[target.x86_64-pc-windows-msvc]
rustflags = ["-C", "link-args=/FORCE:UNRESOLVED"]

[target.x86_64-unknown-linux-gnu]
rustflags = ["-C", "link-args=-Wl,--allow-shlib-undefined"]

[target.x86_64-apple-darwin]
rustflags = ["-C", "link-args=-undefined dynamic_lookup"]

[target.aarch64-apple-darwin]
rustflags = ["-C", "link-args=-undefined dynamic_lookup"]
"#;
        fs::write(cargo_config_dir.join("config.toml"), cargo_config)?;

        // Generate lib.rs with embedded assets
        self.create_lib_rs(&rust_build_dir, frontend_js, manifest, has_routes)?;

        Ok(())
    }

    fn copy_rust_files(&self, src: &Path, dst: &Path) -> Result<()> {
        let plugin_mod_dir = dst.join("plugin_mod");
        fs::create_dir_all(&plugin_mod_dir)?;

        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let path = entry.path();
            let file_name = entry.file_name();
            let file_name_str = file_name.to_string_lossy();

            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext == "rs" {
                        let dest_path = plugin_mod_dir.join(&file_name);
                        let content = fs::read_to_string(&path)?;

                        let modified_content = if file_name_str == "mod.rs" {
                            if content.contains("pub mod router;") {
                                content
                            } else {
                                content.replace("mod router;", "pub mod router;")
                            }
                        } else if file_name_str == "router.rs" {
                            let re = regex::Regex::new(r"(?m)^async fn ([a-zA-Z_][a-zA-Z0-9_]*)\(([^)]*)\) -> HttpResponse")?;
                            re.replace_all(&content, "pub async fn $1($2) -> HttpResponse").to_string()
                        } else {
                            content
                        };

                        fs::write(&dest_path, modified_content)?;
                    }
                }
            }
        }

        Ok(())
    }

    fn create_lib_rs(&self, rust_build_dir: &Path, frontend_js: &str, manifest: &str, has_routes: bool) -> Result<()> {
        let plugin_struct = self.get_plugin_struct_name();

        // Escape the embedded strings for Rust
        let escaped_frontend = frontend_js.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n").replace('\r', "");
        let escaped_manifest = manifest.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n").replace('\r', "");

        // Only generate handler wrappers if plugin has routes
        let handler_wrappers = if !has_routes {
            String::new()
        } else {
            let handlers = self.extract_handlers()?;
            handlers.iter().map(|(handler_name, takes_request)| {
            let handler_call = if *takes_request {
                format!("plugin_mod::router::{}(http_request.clone()).await", handler_name)
            } else {
                format!("plugin_mod::router::{}().await", handler_name)
            };

            format!(r##"
#[no_mangle]
pub extern "C" fn {handler_name}(request_ptr: *const u8, request_len: usize, _runtime_ptr: *const ()) -> *const u8 {{
    use std::panic;
    use std::ffi::CString;
    use api::ffi_http::Response as FFIResponse;
    use api::http::HttpRequest;

    let result = panic::catch_unwind(|| {{
        let _http_request = match HttpRequest::from_ffi_json(request_ptr, request_len) {{
            Ok(r) => r,
            Err(e) => {{
                let error_response = FFIResponse::new(400)
                    .json(&api::serde_json::json!({{"error": e}}));
                return error_response.into_ffi_ptr();
            }}
        }};
        #[allow(unused_variables)]
        let http_request = _http_request;

        // Create a dedicated single-threaded runtime for this handler
        // This avoids deadlock when called from within an existing async context
        let rt = api::tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed to create handler runtime");
        rt.block_on(async {{
            let handler_result = {handler_call};
            let response = handler_result;

            let (parts, body) = response.into_parts();
            let status = parts.status.as_u16();

            let mut headers = std::collections::HashMap::new();
            for (key, value) in parts.headers.iter() {{
                if let Ok(v) = value.to_str() {{
                    headers.insert(key.to_string(), v.to_string());
                }}
            }}

            let body_bytes = body.to_vec();

            let mut ffi_response = FFIResponse::new(status);
            ffi_response.headers = headers.clone();

            let content_type = headers.get("content-type")
                .or_else(|| headers.get("Content-Type"))
                .cloned()
                .unwrap_or_default()
                .to_lowercase();

            let is_binary = content_type.starts_with("image/")
                || content_type.starts_with("application/octet-stream");

            if is_binary {{
                use api::base64::Engine;
                ffi_response.body_base64 = Some(
                    api::base64::engine::general_purpose::STANDARD.encode(&body_bytes)
                );
            }} else if let Ok(body_str) = String::from_utf8(body_bytes.clone()) {{
                if let Ok(json_value) = api::serde_json::from_str::<api::serde_json::Value>(&body_str) {{
                    ffi_response.body = Some(json_value);
                }} else {{
                    ffi_response.body = Some(api::serde_json::Value::String(body_str));
                }}
            }} else {{
                use api::base64::Engine;
                ffi_response.body_base64 = Some(
                    api::base64::engine::general_purpose::STANDARD.encode(&body_bytes)
                );
            }}

            ffi_response.into_ffi_ptr()
        }})
    }});

    match result {{
        Ok(ptr) => ptr,
        Err(_) => {{
            let error = CString::new(r#"{{"__ffi_response__":true,"status":500,"headers":{{"Content-Type":"application/json"}},"body":{{"error":"Handler panicked"}}}}"#).unwrap();
            Box::leak(Box::new(error)).as_ptr() as *const u8
        }}
    }}
}}
"##)
            }).collect::<Vec<_>>().join("\n")
        };

        // Generate lib.rs - use minimal version if no routes (no bridge dependencies)
        let lib_content = if has_routes {
            format!(r#"// Auto-generated plugin library (with bridge support)
pub mod plugin_mod;
pub use plugin_mod::*;
pub use api::ffi_http::free_string;

/// Embedded frontend JavaScript (plugin.js)
const EMBEDDED_FRONTEND: &str = "{escaped_frontend}";

/// Embedded manifest (package.json)
const EMBEDDED_MANIFEST: &str = "{escaped_manifest}";

#[no_mangle]
pub extern "C" fn plugin_init(_ffi_ctx: *const ()) -> i32 {{ 0 }}

#[no_mangle]
pub extern "C" fn plugin_start(_ffi_ctx: *const ()) -> i32 {{ 0 }}

#[no_mangle]
pub extern "C" fn plugin_stop() -> i32 {{ 0 }}

#[no_mangle]
pub extern "C" fn plugin_metadata() -> *const u8 {{
    use api::{{Plugin, serde_json}};
    let plugin = plugin_mod::{plugin_struct};
    let metadata = plugin.metadata();
    let json = serde_json::to_string(&metadata).unwrap_or_default();
    Box::leak(Box::new(json)).as_ptr() as *const u8
}}

/// Returns the embedded manifest (package.json) as a null-terminated string
#[no_mangle]
pub extern "C" fn get_plugin_manifest() -> *const u8 {{
    let manifest = EMBEDDED_MANIFEST.to_string();
    let leaked = Box::leak(Box::new(manifest));
    leaked.as_ptr()
}}

/// Returns the length of the embedded manifest
#[no_mangle]
pub extern "C" fn get_plugin_manifest_len() -> usize {{
    EMBEDDED_MANIFEST.len()
}}

/// Returns the embedded frontend (plugin.js) as a null-terminated string
#[no_mangle]
pub extern "C" fn get_plugin_frontend() -> *const u8 {{
    let frontend = EMBEDDED_FRONTEND.to_string();
    let leaked = Box::leak(Box::new(frontend));
    leaked.as_ptr()
}}

/// Returns the length of the embedded frontend
#[no_mangle]
pub extern "C" fn get_plugin_frontend_len() -> usize {{
    EMBEDDED_FRONTEND.len()
}}

/// Returns whether this plugin has a frontend
#[no_mangle]
pub extern "C" fn has_frontend() -> bool {{
    !EMBEDDED_FRONTEND.is_empty()
}}

/// Free a string allocated by this plugin
#[no_mangle]
pub extern "C" fn free_plugin_string(ptr: *mut u8) {{
    if !ptr.is_null() {{
        unsafe {{
            let _ = std::ffi::CString::from_raw(ptr as *mut i8);
        }}
    }}
}}

{handler_wrappers}
"#)
        } else {
            // Minimal version without bridge dependencies (no tokio, http, etc.)
            format!(r#"// Auto-generated plugin library (minimal - no bridge)
pub mod plugin_mod;
pub use plugin_mod::*;

/// Embedded frontend JavaScript (plugin.js)
const EMBEDDED_FRONTEND: &str = "{escaped_frontend}";

/// Embedded manifest (package.json)
const EMBEDDED_MANIFEST: &str = "{escaped_manifest}";

#[no_mangle]
pub extern "C" fn plugin_init(_ffi_ctx: *const ()) -> i32 {{ 0 }}

#[no_mangle]
pub extern "C" fn plugin_start(_ffi_ctx: *const ()) -> i32 {{ 0 }}

#[no_mangle]
pub extern "C" fn plugin_stop() -> i32 {{ 0 }}

#[no_mangle]
pub extern "C" fn plugin_metadata() -> *const u8 {{
    use api::{{Plugin, serde_json}};
    let plugin = plugin_mod::{plugin_struct};
    let metadata = plugin.metadata();
    let json = serde_json::to_string(&metadata).unwrap_or_default();
    Box::leak(Box::new(json)).as_ptr() as *const u8
}}

/// Returns the embedded manifest (package.json) as a null-terminated string
#[no_mangle]
pub extern "C" fn get_plugin_manifest() -> *const u8 {{
    let manifest = EMBEDDED_MANIFEST.to_string();
    let leaked = Box::leak(Box::new(manifest));
    leaked.as_ptr()
}}

/// Returns the length of the embedded manifest
#[no_mangle]
pub extern "C" fn get_plugin_manifest_len() -> usize {{
    EMBEDDED_MANIFEST.len()
}}

/// Returns the embedded frontend (plugin.js) as a null-terminated string
#[no_mangle]
pub extern "C" fn get_plugin_frontend() -> *const u8 {{
    let frontend = EMBEDDED_FRONTEND.to_string();
    let leaked = Box::leak(Box::new(frontend));
    leaked.as_ptr()
}}

/// Returns the length of the embedded frontend
#[no_mangle]
pub extern "C" fn get_plugin_frontend_len() -> usize {{
    EMBEDDED_FRONTEND.len()
}}

/// Returns whether this plugin has a frontend
#[no_mangle]
pub extern "C" fn has_frontend() -> bool {{
    !EMBEDDED_FRONTEND.is_empty()
}}

/// Free a string allocated by this plugin
#[no_mangle]
pub extern "C" fn free_plugin_string(ptr: *mut u8) {{
    if !ptr.is_null() {{
        unsafe {{
            let _ = std::ffi::CString::from_raw(ptr as *mut i8);
        }}
    }}
}}
"#)
        };

        fs::write(rust_build_dir.join("lib.rs"), lib_content)?;
        Ok(())
    }

    fn extract_handlers(&self) -> Result<Vec<(String, bool)>> {
        let mut handlers: Vec<(String, bool)> = Vec::new();

        let cargo_toml_path = self.plugin_dir.join("Cargo.toml");
        if cargo_toml_path.exists() {
            let cargo_content = fs::read_to_string(&cargo_toml_path)?;
            if let Ok(cargo_toml) = cargo_content.parse::<toml::Value>() {
                if let Some(routes_table) = cargo_toml.get("routes").and_then(|r| r.as_table()) {
                    for (_, value) in routes_table {
                        if let Some(handler) = value.as_str() {
                            if !handlers.iter().any(|(h, _)| h == handler) {
                                handlers.push((handler.to_string(), false));
                            }
                        }
                    }
                }
            }
        }

        let router_path = self.plugin_dir.join("router.rs");
        if router_path.exists() {
            let router_content = fs::read_to_string(&router_path)?;

            for (handler_name, takes_request) in handlers.iter_mut() {
                let pattern = format!(r"(?m)^pub\s+async\s+fn\s+{}\s*\(([^)]*)\)", regex::escape(handler_name));
                if let Ok(re) = regex::Regex::new(&pattern) {
                    if let Some(captures) = re.captures(&router_content) {
                        if let Some(params) = captures.get(1) {
                            let params_str = params.as_str().trim();
                            *takes_request = !params_str.is_empty() &&
                                (params_str.contains("HttpRequest") ||
                                 params_str.contains("Request") ||
                                 params_str.contains(":"));
                        }
                    }
                }
            }
        }

        Ok(handlers)
    }

    fn get_plugin_struct_name(&self) -> String {
        let parts: Vec<&str> = self.plugin_id.split(|c| c == '_' || c == '-').collect();
        let mut name = String::new();
        for part in parts {
            let mut chars = part.chars();
            if let Some(first) = chars.next() {
                name.push(first.to_uppercase().next().unwrap());
                name.push_str(chars.as_str());
            }
        }
        name.push_str("Plugin");
        name
    }

    fn compile_backend(&self) -> Result<()> {
        let rust_build_dir = self.build_dir.join("rust_build");

        let status = Command::new("cargo")
            .current_dir(&rust_build_dir)
            .args(&["build", "--release", "--lib"])
            .status()
            .context("Failed to run cargo build")?;

        if !status.success() {
            anyhow::bail!("Cargo build failed");
        }

        // Copy compiled binary
        self.copy_compiled_binary(&rust_build_dir)?;

        Ok(())
    }

    fn copy_compiled_binary(&self, rust_build_dir: &Path) -> Result<()> {
        let target_dir = rust_build_dir.join("target").join("release");

        let lib_name = if cfg!(target_os = "windows") {
            format!("{}.dll", self.plugin_id)
        } else if cfg!(target_os = "macos") {
            format!("lib{}.dylib", self.plugin_id)
        } else {
            format!("lib{}.so", self.plugin_id)
        };

        let src_path = target_dir.join(&lib_name);
        if src_path.exists() {
            let dest_path = self.build_dir.join(&lib_name);
            fs::copy(&src_path, &dest_path)?;
            Ok(())
        } else {
            anyhow::bail!("Compiled library not found: {}", src_path.display())
        }
    }

    fn bundle_frontend(&self) -> Result<()> {
        let has_frontend = self.plugin_dir.join("index.jsx").exists()
            || self.plugin_dir.join("index.js").exists();

        if !has_frontend {
            return Ok(());
        }

        // Install dependencies if needed
        self.install_npm_dependencies()?;

        // Find bundler script
        let bundler_script = self.repo_root.join("app").join("scripts").join("build.js");

        if !bundler_script.exists() {
            println!("    Warning: Frontend bundler not found at {}", bundler_script.display());
            return Ok(());
        }

        let plugin_dir_str = self.plugin_dir.to_string_lossy();
        let build_dir_str = self.build_dir.to_string_lossy();

        let output = if Command::new("bun").arg("--version").output().is_ok() {
            Command::new("bun")
                .arg("run")
                .arg(&bundler_script)
                .arg(&*plugin_dir_str)
                .arg(&*build_dir_str)
                .output()
                .context("Failed to run bundler with bun")?
        } else {
            Command::new("node")
                .arg(&bundler_script)
                .arg(&*plugin_dir_str)
                .arg(&*build_dir_str)
                .output()
                .context("Failed to run bundler with node")?
        };

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Frontend bundling failed: {}", stderr);
        }

        Ok(())
    }

    fn install_npm_dependencies(&self) -> Result<()> {
        let package_json_path = self.plugin_dir.join("package.json");
        if !package_json_path.exists() {
            return Ok(());
        }

        let content = fs::read_to_string(&package_json_path)?;
        let json: serde_json::Value = serde_json::from_str(&content)?;

        let has_deps = json.get("dependencies").and_then(|d| d.as_object()).map(|o| !o.is_empty()).unwrap_or(false);
        let has_dev_deps = json.get("devDependencies").and_then(|d| d.as_object()).map(|o| !o.is_empty()).unwrap_or(false);

        if !has_deps && !has_dev_deps {
            return Ok(());
        }

        let status = if Command::new("bun").arg("--version").output().is_ok() {
            Command::new("bun")
                .arg("install")
                .current_dir(&self.plugin_dir)
                .status()
        } else {
            Command::new("npm")
                .arg("install")
                .current_dir(&self.plugin_dir)
                .status()
        };

        if let Ok(s) = status {
            if !s.success() {
                println!("    Warning: Failed to install npm dependencies");
            }
        }

        Ok(())
    }

    fn install_dll(&self) -> Result<()> {
        // Find the compiled DLL in build directory
        let lib_name = if cfg!(target_os = "windows") {
            format!("{}.dll", self.plugin_id)
        } else if cfg!(target_os = "macos") {
            format!("lib{}.dylib", self.plugin_id)
        } else {
            format!("lib{}.so", self.plugin_id)
        };

        let src_path = self.build_dir.join(&lib_name);
        if !src_path.exists() {
            anyhow::bail!("Compiled library not found: {}", src_path.display());
        }

        // Copy to build/plugins directory
        let dest_path = self.dist_plugins_dir.join(&lib_name);
        fs::copy(&src_path, &dest_path)?;

        Ok(())
    }

    fn create_manifest(&self) -> Result<String> {
        let package_json_path = self.plugin_dir.join("package.json");

        let mut package_json = if package_json_path.exists() {
            let content = fs::read_to_string(&package_json_path)?;
            serde_json::from_str::<serde_json::Value>(&content)?
        } else {
            serde_json::json!({
                "name": self.plugin_id,
                "version": "1.0.0"
            })
        };

        let routes = self.extract_routes()?;

        package_json["webarcade"] = serde_json::json!({
            "id": self.plugin_id,
            "routes": routes
        });

        Ok(serde_json::to_string_pretty(&package_json)?)
    }

    fn extract_routes(&self) -> Result<Vec<serde_json::Value>> {
        let mut routes = Vec::new();

        let cargo_toml_path = self.plugin_dir.join("Cargo.toml");
        if cargo_toml_path.exists() {
            let cargo_content = fs::read_to_string(&cargo_toml_path)?;
            if let Ok(cargo_toml) = cargo_content.parse::<toml::Value>() {
                if let Some(routes_table) = cargo_toml.get("routes").and_then(|r| r.as_table()) {
                    for (key, value) in routes_table {
                        if let Some(handler) = value.as_str() {
                            let parts: Vec<&str> = key.splitn(2, ' ').collect();
                            if parts.len() == 2 {
                                routes.push(serde_json::json!({
                                    "method": parts[0],
                                    "path": parts[1],
                                    "handler": handler
                                }));
                            }
                        }
                    }
                }
            }
        }

        Ok(routes)
    }
}

// ============================================================================
// PACKAGE COMMAND - Interactive app packaging
// ============================================================================

#[derive(Debug, Clone)]
struct AppConfig {
    name: String,
    version: String,
    description: String,
    author: String,
    identifier: String,
    locked: bool,
}

impl AppConfig {
    fn from_cargo_toml(cargo_toml_path: &Path) -> Result<Self> {
        let content = fs::read_to_string(cargo_toml_path)?;
        let doc: toml::Value = content.parse()?;

        let package = doc.get("package").context("Missing [package] section")?;
        let packager = doc.get("package")
            .and_then(|p| p.get("metadata"))
            .and_then(|m| m.get("packager"));

        Ok(Self {
            name: package.get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("MyApp")
                .to_string(),
            version: package.get("version")
                .and_then(|v| v.as_str())
                .unwrap_or("0.1.0")
                .to_string(),
            description: package.get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            author: packager
                .and_then(|p| p.get("authors"))
                .and_then(|a| a.as_array())
                .and_then(|arr| arr.first())
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown")
                .to_string(),
            identifier: packager
                .and_then(|p| p.get("identifier"))
                .and_then(|v| v.as_str())
                .unwrap_or("com.app.myapp")
                .to_string(),
            locked: false,
        })
    }

    fn write_to_cargo_toml(&self, cargo_toml_path: &Path) -> Result<()> {
        let content = fs::read_to_string(cargo_toml_path)?;
        let mut doc: toml_edit::DocumentMut = content.parse()?;

        // Update [package] section
        doc["package"]["name"] = toml_edit::value(&self.name);
        doc["package"]["version"] = toml_edit::value(&self.version);
        doc["package"]["description"] = toml_edit::value(&self.description);

        // Update [package.metadata.packager] section
        if doc.get("package").is_none() {
            doc["package"] = toml_edit::table();
        }
        if doc["package"].get("metadata").is_none() {
            doc["package"]["metadata"] = toml_edit::table();
        }
        if doc["package"]["metadata"].get("packager").is_none() {
            doc["package"]["metadata"]["packager"] = toml_edit::table();
        }

        doc["package"]["metadata"]["packager"]["product-name"] = toml_edit::value(&self.name);
        doc["package"]["metadata"]["packager"]["identifier"] = toml_edit::value(&self.identifier);

        // Update authors array
        let mut authors = toml_edit::Array::new();
        authors.push(&self.author);
        doc["package"]["metadata"]["packager"]["authors"] = toml_edit::value(authors);

        // Update binaries path to match package name
        if let Some(binaries) = doc["package"]["metadata"]["packager"].get_mut("binaries") {
            if let Some(arr) = binaries.as_array_of_tables_mut() {
                if let Some(first) = arr.iter_mut().next() {
                    first["path"] = toml_edit::value(&self.name);
                }
            }
        }

        // Update appdata-paths for cleanup on uninstall
        let mut appdata = toml_edit::Array::new();
        appdata.push(format!("$LOCALAPPDATA\\{}", &self.name));
        doc["package"]["metadata"]["packager"]["nsis"]["appdata-paths"] = toml_edit::value(appdata);

        fs::write(cargo_toml_path, doc.to_string())?;
        Ok(())
    }
}

fn package_app(
    skip_prompts: bool,
    locked: bool,
    name: Option<String>,
    version: Option<String>,
    description: Option<String>,
    author: Option<String>,
) -> Result<()> {
    let repo_root = get_repo_root()?;
    let app_dir = repo_root.join("app");
    let cargo_toml_path = app_dir.join("Cargo.toml");

    if !cargo_toml_path.exists() {
        anyhow::bail!("app/Cargo.toml not found. Are you in the correct directory?");
    }

    println!();
    println!("{}", style("╔══════════════════════════════════════════╗").cyan());
    println!("{}", style("║       WebArcade App Packager             ║").cyan());
    println!("{}", style("╚══════════════════════════════════════════╝").cyan());
    println!();

    // Load existing config
    let mut config = AppConfig::from_cargo_toml(&cargo_toml_path)?;
    config.locked = locked;

    let theme = ColorfulTheme::default();

    if !skip_prompts {
        // Interactive prompts
        config.name = if let Some(n) = name {
            n
        } else {
            Input::with_theme(&theme)
                .with_prompt("App name")
                .default(config.name)
                .interact_text()?
        };

        config.version = if let Some(v) = version {
            v
        } else {
            Input::with_theme(&theme)
                .with_prompt("Version")
                .default(config.version)
                .interact_text()?
        };

        config.description = if let Some(d) = description {
            d
        } else {
            Input::with_theme(&theme)
                .with_prompt("Description")
                .default(config.description)
                .allow_empty(true)
                .interact_text()?
        };

        config.author = if let Some(a) = author {
            a
        } else {
            Input::with_theme(&theme)
                .with_prompt("Author")
                .default(config.author)
                .interact_text()?
        };

        // Generate identifier from name
        let default_identifier = format!(
            "com.{}.app",
            config.name.to_lowercase().replace(' ', "").replace('-', "")
        );
        config.identifier = Input::with_theme(&theme)
            .with_prompt("Identifier")
            .default(if config.identifier == "com.app.myapp" { default_identifier } else { config.identifier })
            .interact_text()?;

        // Plugin mode selection
        let plugin_modes = vec!["Unlocked (plugins loaded from disk)", "Locked (plugins embedded in binary)"];
        let mode_index = Select::with_theme(&theme)
            .with_prompt("Plugin mode")
            .items(&plugin_modes)
            .default(if config.locked { 1 } else { 0 })
            .interact()?;
        config.locked = mode_index == 1;

        println!();
        println!("{}", style("Configuration:").bold());
        println!("  Name:        {}", style(&config.name).green());
        println!("  Version:     {}", style(&config.version).green());
        println!("  Description: {}", style(&config.description).green());
        println!("  Author:      {}", style(&config.author).green());
        println!("  Identifier:  {}", style(&config.identifier).green());
        println!("  Plugin mode: {}", style(if config.locked { "Locked" } else { "Unlocked" }).green());
        println!();

        if !Confirm::with_theme(&theme)
            .with_prompt("Proceed with packaging?")
            .default(true)
            .interact()? {
            println!("Packaging cancelled.");
            return Ok(());
        }
    } else {
        // Use provided args or defaults
        if let Some(n) = name { config.name = n; }
        if let Some(v) = version { config.version = v; }
        if let Some(d) = description { config.description = d; }
        if let Some(a) = author { config.author = a; }
    }

    println!();
    println!("{} Updating configuration...", style("[1/4]").bold().dim());
    config.write_to_cargo_toml(&cargo_toml_path)?;
    println!("  {} Cargo.toml updated", style("✓").green());

    println!("{} Building frontend...", style("[2/4]").bold().dim());
    let frontend_status = Command::new("bun")
        .current_dir(&repo_root)
        .args(["run", "build:prod"])
        .status()
        .context("Failed to run bun")?;

    if !frontend_status.success() {
        anyhow::bail!("Frontend build failed");
    }
    println!("  {} Frontend built", style("✓").green());

    println!("{} Compiling Rust binary...", style("[3/4]").bold().dim());
    let mut cargo_args = vec!["build", "--release"];
    if config.locked {
        cargo_args.push("--features");
        cargo_args.push("locked-plugins");
    }

    let cargo_status = Command::new("cargo")
        .current_dir(&app_dir)
        .args(&cargo_args)
        .status()
        .context("Failed to run cargo build")?;

    if !cargo_status.success() {
        anyhow::bail!("Cargo build failed");
    }
    println!("  {} Binary compiled", style("✓").green());

    println!("{} Creating installer...", style("[4/4]").bold().dim());
    let packager_status = Command::new("cargo")
        .current_dir(&app_dir)
        .args(["packager", "--release"])
        .status()
        .context("Failed to run cargo packager")?;

    if !packager_status.success() {
        anyhow::bail!("Packaging failed");
    }
    println!("  {} Installer created", style("✓").green());

    // Find the output file
    let output_dir = app_dir.join("target").join("release");
    let installer_name = format!("{}_{}_x64-setup.exe", config.name, config.version);
    let installer_path = output_dir.join(&installer_name);

    println!();
    println!("{}", style("╔══════════════════════════════════════════╗").green());
    println!("{}", style("║           Packaging Complete!            ║").green());
    println!("{}", style("╚══════════════════════════════════════════╝").green());
    println!();
    println!("  {} {}", style("Binary:").bold(), output_dir.join(format!("{}.exe", config.name)).display());
    if installer_path.exists() {
        println!("  {} {}", style("Installer:").bold(), installer_path.display());
    } else {
        println!("  {} {}", style("Installer:").bold(), output_dir.display());
    }
    println!();

    Ok(())
}
