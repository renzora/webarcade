use screenshots::Screen;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tauri::State;
use image::ImageEncoder;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayInfo {
    pub id: usize,
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub x: i32,
    pub y: i32,
}

#[derive(Clone)]
pub struct CaptureState {
    pub active_captures: Arc<Mutex<Vec<ActiveCapture>>>,
}

#[derive(Clone)]
pub struct ActiveCapture {
    pub id: String,
    #[allow(dead_code)]
    pub screen_id: usize,
    pub latest_frame: Arc<Mutex<Option<Vec<u8>>>>,
    pub running: Arc<Mutex<bool>>,
}

impl CaptureState {
    pub fn new() -> Self {
        CaptureState {
            active_captures: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

#[tauri::command]
pub fn list_displays() -> Result<Vec<DisplayInfo>, String> {
    let screens = Screen::all().map_err(|e| format!("Failed to get displays: {}", e))?;

    let displays = screens
        .iter()
        .enumerate()
        .map(|(i, screen)| {
            let display_info = screen.display_info;
            DisplayInfo {
                id: i,
                name: format!("Display {}", i + 1),
                width: display_info.width,
                height: display_info.height,
                x: display_info.x,
                y: display_info.y,
            }
        })
        .collect();

    Ok(displays)
}

#[tauri::command]
pub fn start_display_capture(
    capture_id: String,
    screen_id: usize,
    state: State<CaptureState>,
) -> Result<(), String> {
    let screens = Screen::all().map_err(|e| format!("Failed to get displays: {}", e))?;

    if screen_id >= screens.len() {
        return Err("Invalid screen ID".to_string());
    }

    let screen = screens[screen_id].clone();
    let latest_frame = Arc::new(Mutex::new(None));
    let running = Arc::new(Mutex::new(true));

    let active_capture = ActiveCapture {
        id: capture_id.clone(),
        screen_id,
        latest_frame: latest_frame.clone(),
        running: running.clone(),
    };

    // Add to active captures
    {
        let mut captures = state.active_captures.lock().unwrap();
        captures.push(active_capture);
    }

    // Start capture thread
    thread::spawn(move || {
        // Target 30 FPS
        let frame_duration = Duration::from_millis(33);

        loop {
            {
                let is_running = *running.lock().unwrap();
                if !is_running {
                    break;
                }
            }

            // Capture frame
            if let Ok(image) = screen.capture() {
                // Convert to JPEG for efficient transfer
                let mut jpeg_data = Vec::new();
                let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut jpeg_data, 85);

                if encoder.write_image(
                    image.as_raw(),
                    image.width(),
                    image.height(),
                    image::ColorType::Rgba8.into()
                ).is_ok() {
                    let mut frame = latest_frame.lock().unwrap();
                    *frame = Some(jpeg_data);
                }
            }

            thread::sleep(frame_duration);
        }

        println!("[ScreenCapture] Capture thread stopped for {}", capture_id);
    });

    Ok(())
}

#[tauri::command]
pub fn get_display_frame(
    capture_id: String,
    state: State<CaptureState>,
) -> Result<Option<String>, String> {
    let captures = state.active_captures.lock().unwrap();

    if let Some(capture) = captures.iter().find(|c| c.id == capture_id) {
        let frame = capture.latest_frame.lock().unwrap();
        if let Some(jpeg_data) = frame.as_ref() {
            // Return as base64-encoded JPEG
            Ok(Some(base64::Engine::encode(&base64::engine::general_purpose::STANDARD, jpeg_data)))
        } else {
            Ok(None)
        }
    } else {
        Err("Capture not found".to_string())
    }
}

#[tauri::command]
pub fn stop_display_capture(
    capture_id: String,
    state: State<CaptureState>,
) -> Result<(), String> {
    let mut captures = state.active_captures.lock().unwrap();

    if let Some(pos) = captures.iter().position(|c| c.id == capture_id) {
        let capture = &captures[pos];
        *capture.running.lock().unwrap() = false;
        captures.remove(pos);
        Ok(())
    } else {
        Err("Capture not found".to_string())
    }
}
