// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use tauri::{Emitter, Listener, Manager};

fn start_bridge_server(app: &tauri::AppHandle) {
    if cfg!(debug_assertions) {
        // Development mode - bridge is already started by beforeDevCommand
        println!("Dev mode: bridge server should already be running");
        return;
    }

    // Production mode - locate and start the sidecar binary
    let resource_path = app
        .path()
        .resource_dir()
        .expect("failed to resolve resource directory");

    #[cfg(windows)]
    let bridge_path = resource_path.join("webarcade-bridge-x86_64-pc-windows-msvc.exe");
    #[cfg(not(windows))]
    let bridge_path = resource_path.join("webarcade-bridge");

    println!("Starting bridge server from: {:?}", bridge_path);

    std::thread::spawn(move || {
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x08000000;

            match Command::new(&bridge_path)
                .creation_flags(CREATE_NO_WINDOW)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .stdin(Stdio::null())
                .spawn()
            {
                Ok(_) => println!("Bridge server started successfully"),
                Err(e) => eprintln!("Failed to start bridge server: {}", e),
            }
        }

        #[cfg(not(windows))]
        {
            match Command::new(&bridge_path)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .stdin(Stdio::null())
                .spawn()
            {
                Ok(_) => println!("Bridge server started successfully"),
                Err(e) => eprintln!("Failed to start bridge server: {}", e),
            }
        }
    });
}


fn main() {
    // Shared state to track if close is approved
    let close_approved = Arc::new(Mutex::new(false));
    let close_approved_clone = close_approved.clone();
    
    tauri::Builder::default()
        .setup(move |app| {
            let handle = app.handle().clone();
            let close_approved = close_approved.clone();

            // Start bridge server as sidecar
            start_bridge_server(&handle);
            
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
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}