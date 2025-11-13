// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::{Arc, Mutex};
use tauri::{Emitter, Listener, Manager};
use std::collections::VecDeque;

mod bridge;
mod plugin_installer;

// Re-export bridge modules at crate root for plugin compatibility
pub use bridge::core;
pub use bridge::modules;
pub use bridge::plugins;

// Global log buffer for capturing bridge logs
static LOG_BUFFER: Mutex<Option<VecDeque<String>>> = Mutex::new(None);
static LOG_CAPACITY: usize = 500; // Keep last 500 log lines

fn init_log_buffer() {
    let mut buffer = LOG_BUFFER.lock().unwrap();
    *buffer = Some(VecDeque::with_capacity(LOG_CAPACITY));
}

pub fn add_log(message: String) {
    if let Ok(mut buffer) = LOG_BUFFER.lock() {
        if let Some(ref mut buf) = *buffer {
            if buf.len() >= LOG_CAPACITY {
                buf.pop_front();
            }
            buf.push_back(message);
        }
    }
}

#[tauri::command]
fn get_bridge_logs() -> Vec<String> {
    if let Ok(buffer) = LOG_BUFFER.lock() {
        if let Some(ref buf) = *buffer {
            return buf.iter().cloned().collect();
        }
    }
    vec![]
}

#[tauri::command]
async fn check_bridge_health() -> Result<String, String> {
    match reqwest::get("http://127.0.0.1:3001/health").await {
        Ok(response) => {
            if response.status().is_success() {
                match response.text().await {
                    Ok(body) => Ok(body),
                    Err(e) => Err(format!("Failed to read response: {}", e))
                }
            } else {
                Err(format!("Bridge returned status: {}", response.status()))
            }
        }
        Err(e) => Err(format!("Bridge not responding: {}", e))
    }
}

#[tauri::command]
async fn install_plugin_from_zip(
    zip_data: Vec<u8>,
    file_name: String,
) -> Result<plugin_installer::InstallResult, String> {
    log::info!("Received plugin installation request: {}", file_name);

    // Get the runtime plugins directory (AppData/WebArcade/plugins)
    let plugins_dir = dirs::data_local_dir()
        .ok_or_else(|| "Failed to get local data directory".to_string())?
        .join("WebArcade")
        .join("plugins");

    log::info!("Installing plugin to: {:?}", plugins_dir);

    // Create installer
    let installer = plugin_installer::PluginInstaller::new(plugins_dir);

    // Install the plugin
    match installer.install_from_zip(&zip_data, &file_name) {
        Ok(result) => {
            log::info!("Plugin installed successfully: {}", result.plugin_id);

            // TODO: Notify bridge to reload plugins
            // For now, plugins will be loaded on next app restart

            Ok(result)
        }
        Err(e) => {
            log::error!("Plugin installation failed: {}", e);
            Err(format!("Installation failed: {}", e))
        }
    }
}


fn start_bridge_server() {
    let start_msg = "Starting integrated bridge server...";
    println!("{}", start_msg);
    add_log(format!("[BRIDGE] {}", start_msg));

    // Start the bridge server in a separate tokio runtime
    std::thread::spawn(|| {
        let runtime = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
        runtime.block_on(async {
            add_log("[BRIDGE] Tokio runtime created".to_string());
            match bridge::run_server().await {
                Ok(_) => {
                    add_log("[BRIDGE] Server started successfully".to_string());
                }
                Err(e) => {
                    let error_msg = format!("Bridge server error: {}", e);
                    eprintln!("{}", error_msg);
                    add_log(format!("[BRIDGE ERROR] {}", error_msg));
                }
            }
        });
    });
}


fn main() {
    // Initialize log buffer
    init_log_buffer();
    add_log("[MAIN] WebArcade starting...".to_string());

    // Shared state to track if close is approved
    let close_approved = Arc::new(Mutex::new(false));
    let close_approved_clone = close_approved.clone();

    tauri::Builder::default()
        .setup(move |app| {
            let handle = app.handle().clone();
            let close_approved = close_approved.clone();

            add_log("[SETUP] Tauri app setup starting...".to_string());

            // Start integrated bridge server
            start_bridge_server();

            add_log("[SETUP] Bridge server started".to_string());
            
            // Listen for graceful close approval from frontend
            app.listen("proceed-with-close", move |_event| {
                println!("Graceful close approved by frontend");
                {
                    let mut approved = close_approved.lock().unwrap();
                    *approved = true;
                }
                
                // Now trigger a close event which will be allowed through
                if let Some(window) = handle.get_webview_window("main") {
                    println!("Requesting window close after approval");
                    let _ = window.close();
                }
            });
            
            Ok(())
        })
        .on_window_event(move |window, event| {
            match event {
                tauri::WindowEvent::CloseRequested { api, .. } => {
                    // Check if close was already approved
                    let approved = {
                        let approved = close_approved_clone.lock().unwrap();
                        *approved
                    };
                    
                    if approved {
                        println!("Window close approved - allowing close");
                        // Allow the close to proceed
                        return;
                    }
                    
                    println!("Window close requested - intercepted for save prompt");
                    // Prevent default close behavior
                    api.prevent_close();
                    
                    // Emit event to frontend to handle save prompt
                    match window.emit("window-close-requested", ()) {
                        Ok(_) => println!("Successfully emitted window-close-requested event"),
                        Err(e) => println!("Failed to emit event: {}", e),
                    }
                }
                _ => {}
            }
        })
        .invoke_handler(tauri::generate_handler![
            get_bridge_logs,
            check_bridge_health,
            install_plugin_from_zip
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}