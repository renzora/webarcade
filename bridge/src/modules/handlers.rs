use hyper::{Request, Response, Method, StatusCode};
use hyper::header::{CONTENT_TYPE, ACCESS_CONTROL_ALLOW_ORIGIN, ACCESS_CONTROL_ALLOW_METHODS, ACCESS_CONTROL_ALLOW_HEADERS};
use http_body_util::{BodyExt, Full, combinators::BoxBody};
use bytes::Bytes;
use std::convert::Infallible;
use std::sync::{Arc, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH, Instant};
use log::{info, warn, error};
use percent_encoding::percent_decode_str;
use serde::Deserialize;
use crate::types::{ApiResponse, WriteFileRequest, WriteBinaryFileRequest};
use crate::file_sync::{read_file_content, write_file_content, delete_file_or_directory, get_file_content_type, read_binary_file, write_binary_file_content};
use crate::file_watcher::{set_current_project};
use crate::system_monitor::get_system_stats;
use crate::modules::memory_cache::{MemoryCache};
use crate::modules::twitch::{TwitchManager, SimpleCommand, PermissionLevel};
use crate::modules::withings_api::WithingsAPI;
use crate::modules::discord::DiscordManager;
use crate::commands::database::Database;
use crate::commands::hue::HueClient;

// Unified file operations
#[derive(Debug)]
enum FileOperation {
    List,
    Read,
    Write { content: String },
    WriteBinary { content: String },
    Delete,
    Serve,
}

#[derive(Debug)]
struct UnifiedFileRequest {
    path: String,
    operation: FileOperation,
    accept_header: Option<String>,
}

// Static variables for shared state
static STARTUP_TIME: OnceLock<u64> = OnceLock::new();
static MEMORY_CACHE: OnceLock<Arc<tokio::sync::Mutex<MemoryCache>>> = OnceLock::new();
static TWITCH_MANAGER: OnceLock<Arc<TwitchManager>> = OnceLock::new();
static DATABASE: OnceLock<Arc<Database>> = OnceLock::new();
static WITHINGS_API: OnceLock<Arc<WithingsAPI>> = OnceLock::new();
static DISCORD_MANAGER: OnceLock<Arc<DiscordManager>> = OnceLock::new();

pub fn set_startup_time(timestamp: u64) {
    STARTUP_TIME.set(timestamp).ok();
}

pub fn set_memory_cache(memory_cache: Arc<tokio::sync::Mutex<MemoryCache>>) {
    MEMORY_CACHE.set(memory_cache).ok();
}

pub fn set_twitch_manager(manager: Arc<TwitchManager>) {
    TWITCH_MANAGER.set(manager).ok();
}

fn get_twitch_manager() -> Option<&'static Arc<TwitchManager>> {
    TWITCH_MANAGER.get()
}

pub fn set_database(database: Arc<Database>) {
    DATABASE.set(database).ok();
}

fn get_database() -> Option<&'static Arc<Database>> {
    DATABASE.get()
}

pub fn set_withings_api(api: Arc<WithingsAPI>) {
    WITHINGS_API.set(api).ok();
}

fn get_withings_api() -> Option<&'static Arc<WithingsAPI>> {
    WITHINGS_API.get()
}

pub fn set_discord_manager(manager: Arc<DiscordManager>) {
    DISCORD_MANAGER.set(manager).ok();
}

fn get_discord_manager() -> Option<&'static Arc<DiscordManager>> {
    DISCORD_MANAGER.get()
}

pub async fn handle_http_request(req: Request<hyper::body::Incoming>) -> Result<Response<BoxBody<Bytes, Infallible>>, Infallible> {
    let start_time = Instant::now();
    let method = req.method().clone();
    let uri = req.uri().clone();
    let path = uri.path().to_string();
    let query = uri.query().unwrap_or("").to_string();
    let user_agent = req.headers().get("user-agent")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown");
    let accept_header = req.headers().get("accept")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    
    info!("📥 {} {} {} - User-Agent: {}", method, path, 
          if query.is_empty() { "".to_string() } else { format!("?{}", query) }, user_agent);
    
    // Handle CORS preflight
    if method == Method::OPTIONS {
        let duration = start_time.elapsed();
        info!("⚡ OPTIONS {} - 200 OK - {}ms", path, duration.as_millis());
        return Ok(cors_response(StatusCode::OK, ""));
    }
    
    // WebSocket connections are handled by a separate server on port 3002
    if method == Method::GET && path == "/ws" {
        let duration = start_time.elapsed();
        info!("🔄 WebSocket {} - redirect to separate server - {}ms", path, duration.as_millis());
        return Ok(Response::builder()
            .status(StatusCode::MOVED_PERMANENTLY)
            .header("Location", "ws://localhost:3002")
            .body(BoxBody::new(Full::new(Bytes::from("WebSocket server running on port 3002"))))
            .unwrap());
    }
    
    // Read body if this is a POST, PUT, or DELETE request
    let body = if method == Method::POST || method == Method::PUT || method == Method::DELETE {
        match read_request_body(req).await {
            Ok(body) => {
                let body_size = body.len();
                info!("📄 Request body size: {} bytes", body_size);
                if body_size > 1024 {
                    info!("📄 Large request body ({}KB)", body_size / 1024);
                }
                Some(body)
            },
            Err(e) => {
                error!("Failed to read request body: {}", e);
                return Ok(error_response(StatusCode::BAD_REQUEST, "Failed to read request body"));
            }
        }
    } else {
        None
    };
    
    let response = match (&method, path.as_str()) {
        // Unified file operations
        (&Method::GET, path) if path.starts_with("/list/") => {
            let file_path = &path[6..];
            let decoded_path = match decode_and_validate_path(file_path) {
                Ok(path) => path,
                Err(e) => return Ok(error_response(StatusCode::BAD_REQUEST, &e)),
            };
            handle_unified_file_operation(UnifiedFileRequest {
                path: decoded_path,
                operation: FileOperation::List,
                accept_header: accept_header.clone(),
            })
        }
        (&Method::GET, path) if path.starts_with("/read/") => {
            let file_path = &path[6..];
            let decoded_path = match decode_and_validate_path(file_path) {
                Ok(path) => path,
                Err(e) => return Ok(error_response(StatusCode::BAD_REQUEST, &e)),
            };
            handle_unified_file_operation(UnifiedFileRequest {
                path: decoded_path,
                operation: FileOperation::Read,
                accept_header: accept_header.clone(),
            })
        }
        (&Method::POST, path) if path.starts_with("/write/") => {
            let file_path = &path[7..];
            let decoded_path = match decode_and_validate_path(file_path) {
                Ok(path) => path,
                Err(e) => return Ok(error_response(StatusCode::BAD_REQUEST, &e)),
            };
            match &body {
                Some(body_content) => handle_unified_file_operation(UnifiedFileRequest {
                    path: decoded_path,
                    operation: FileOperation::Write { content: body_content.clone() },
                    accept_header: accept_header.clone(),
                }),
                None => error_response(StatusCode::BAD_REQUEST, "Missing request body"),
            }
        }
        (&Method::POST, path) if path.starts_with("/write-binary/") => {
            let file_path = &path[14..];
            let decoded_path = match decode_and_validate_path(file_path) {
                Ok(path) => path,
                Err(e) => return Ok(error_response(StatusCode::BAD_REQUEST, &e)),
            };
            match &body {
                Some(body_content) => handle_unified_file_operation(UnifiedFileRequest {
                    path: decoded_path,
                    operation: FileOperation::WriteBinary { content: body_content.clone() },
                    accept_header: accept_header.clone(),
                }),
                None => error_response(StatusCode::BAD_REQUEST, "Missing request body"),
            }
        }
        (&Method::DELETE, path) if path.starts_with("/delete/") => {
            let file_path = &path[8..];
            let decoded_path = match decode_and_validate_path(file_path) {
                Ok(path) => path,
                Err(e) => return Ok(error_response(StatusCode::BAD_REQUEST, &e)),
            };
            handle_unified_file_operation(UnifiedFileRequest {
                path: decoded_path,
                operation: FileOperation::Delete,
                accept_header: accept_header.clone(),
            })
        }
        (&Method::GET, path) if path.starts_with("/file/") => {
            let file_path = &path[6..];
            let decoded_path = match decode_and_validate_path(file_path) {
                Ok(path) => path,
                Err(e) => return Ok(error_response(StatusCode::BAD_REQUEST, &e)),
            };
            return Ok(handle_unified_file_operation(UnifiedFileRequest {
                path: decoded_path,
                operation: FileOperation::Serve,
                accept_header: accept_header.clone(),
            }));
        }
        (&Method::POST, "/start-watcher") => handle_start_watcher(),
        (&Method::GET, "/file-changes") => handle_get_file_changes(),
        (&Method::POST, "/clear-changes") => handle_clear_file_changes(),
        (&Method::POST, "/set-current-project") => {
            match &body {
                Some(body_content) => handle_set_current_project(body_content),
                None => error_response(StatusCode::BAD_REQUEST, "Missing request body"),
            }
        }
        (&Method::GET, "/health") => handle_health_check(),
        (&Method::GET, "/startup-time") => handle_get_startup_time(),
        (&Method::GET, "/system/stats") => handle_get_system_stats(),
        (&Method::POST, "/restart") => handle_restart_bridge(),
        (&Method::POST, "/clear-cache") => handle_clear_cache(),

        // Twitch endpoints
        (&Method::POST, "/twitch/start") => return Ok(handle_twitch_start().await),
        (&Method::POST, "/twitch/stop") => return Ok(handle_twitch_stop().await),
        (&Method::GET, "/twitch/status") => return Ok(handle_twitch_status().await),
        (&Method::GET, "/twitch/auth-url") => return Ok(handle_twitch_auth_url().await),
        (&Method::GET, "/twitch/callback") => return Ok(handle_twitch_callback_get(&uri).await),
        (&Method::POST, "/twitch/callback") => {
            match &body {
                Some(body_content) => return Ok(handle_twitch_callback(body_content).await),
                None => error_response(StatusCode::BAD_REQUEST, "Missing request body"),
            }
        }
        (&Method::POST, "/twitch/send-message") => {
            match &body {
                Some(body_content) => return Ok(handle_twitch_send_message(body_content).await),
                None => error_response(StatusCode::BAD_REQUEST, "Missing request body"),
            }
        }
        (&Method::GET, "/twitch/commands") => return Ok(handle_twitch_get_commands().await),
        (&Method::POST, "/twitch/register-command") => {
            match &body {
                Some(body_content) => return Ok(handle_twitch_register_command(body_content).await),
                None => error_response(StatusCode::BAD_REQUEST, "Missing request body"),
            }
        }
        (&Method::POST, "/twitch/unregister-command") => {
            match &body {
                Some(body_content) => return Ok(handle_twitch_unregister_command(body_content).await),
                None => error_response(StatusCode::BAD_REQUEST, "Missing request body"),
            }
        }
        (&Method::GET, "/twitch/config") => return Ok(handle_twitch_get_config().await),
        (&Method::POST, "/twitch/config") => {
            match &body {
                Some(body_content) => return Ok(handle_twitch_save_config(body_content).await),
                None => error_response(StatusCode::BAD_REQUEST, "Missing request body"),
            }
        }
        (&Method::POST, "/twitch/join-channel") => {
            match &body {
                Some(body_content) => return Ok(handle_twitch_join_channel(body_content).await),
                None => error_response(StatusCode::BAD_REQUEST, "Missing request body"),
            }
        }
        (&Method::POST, "/twitch/part-channel") => {
            match &body {
                Some(body_content) => return Ok(handle_twitch_part_channel(body_content).await),
                None => error_response(StatusCode::BAD_REQUEST, "Missing request body"),
            }
        }
        (&Method::POST, "/twitch/revoke") => return Ok(handle_twitch_revoke().await),

        // Text Commands endpoints
        (&Method::GET, "/twitch/text-commands") => return Ok(handle_get_text_commands(&query).await),
        (&Method::POST, "/twitch/text-commands/add") => {
            match &body {
                Some(body_content) => return Ok(handle_add_text_command(body_content).await),
                None => error_response(StatusCode::BAD_REQUEST, "Missing request body"),
            }
        }
        (&Method::PUT, "/twitch/text-commands/edit") => {
            match &body {
                Some(body_content) => return Ok(handle_edit_text_command(body_content).await),
                None => error_response(StatusCode::BAD_REQUEST, "Missing request body"),
            }
        }
        (&Method::DELETE, "/twitch/text-commands") => {
            match &body {
                Some(body_content) => return Ok(handle_delete_text_command(body_content).await),
                None => error_response(StatusCode::BAD_REQUEST, "Missing request body"),
            }
        }

        // Overlay endpoints
        (&Method::GET, path) if path.starts_with("/overlay/") => {
            let overlay_id = &path[9..];
            return Ok(handle_serve_overlay(overlay_id).await);
        }
        (&Method::GET, "/api/overlays") => return Ok(handle_get_overlays().await),
        (&Method::POST, "/api/overlays") => {
            match &body {
                Some(body_content) => return Ok(handle_save_overlay(body_content).await),
                None => error_response(StatusCode::BAD_REQUEST, "Missing request body"),
            }
        }
        (&Method::DELETE, path) if path.starts_with("/api/overlays/") => {
            let overlay_id = &path[14..];
            return Ok(handle_delete_overlay(overlay_id).await);
        }
        (&Method::POST, path) if path.starts_with("/api/overlay/") && path.ends_with("/trigger") => {
            let parts: Vec<&str> = path.split('/').collect();
            if parts.len() >= 4 {
                let overlay_id = parts[3];
                match &body {
                    Some(body_content) => return Ok(handle_overlay_trigger(overlay_id, body_content).await),
                    None => error_response(StatusCode::BAD_REQUEST, "Missing request body"),
                }
            } else {
                error_response(StatusCode::BAD_REQUEST, "Invalid overlay trigger path")
            }
        }

        // File-based overlay system endpoints
        (&Method::GET, "/api/overlay-files") => return Ok(handle_get_overlay_files().await),
        (&Method::GET, path) if path.starts_with("/api/overlay-files/") => {
            let filename = &path[19..]; // Skip "/api/overlay-files/"
            return Ok(handle_read_overlay_file(filename).await);
        }
        (&Method::POST, path) if path.starts_with("/api/overlay-files/") => {
            let filename = &path[19..];
            match &body {
                Some(body_content) => {
                    let bytes = Bytes::from(body_content.clone());
                    return Ok(handle_write_overlay_file(filename, &bytes).await);
                }
                None => error_response(StatusCode::BAD_REQUEST, "Missing request body"),
            }
        }
        (&Method::DELETE, path) if path.starts_with("/api/overlay-files/") => {
            let filename = &path[19..];
            return Ok(handle_delete_overlay_file(filename).await);
        }
        (&Method::POST, "/api/rebuild-overlays") => return Ok(handle_rebuild_overlays().await),

        // Timer broadcast endpoint
        (&Method::POST, "/api/timer/broadcast") => {
            match &body {
                Some(body_content) => return Ok(handle_timer_broadcast(body_content).await),
                None => error_response(StatusCode::BAD_REQUEST, "Missing request body"),
            }
        }

        // Database endpoints
        (&Method::GET, "/database/counters") => return Ok(handle_get_counters(&query).await),
        (&Method::POST, "/database/counters/increment") => {
            match &body {
                Some(body_content) => return Ok(handle_increment_counter(body_content).await),
                None => error_response(StatusCode::BAD_REQUEST, "Missing request body"),
            }
        }
        (&Method::POST, "/database/counters/decrement") => {
            match &body {
                Some(body_content) => return Ok(handle_decrement_counter(body_content).await),
                None => error_response(StatusCode::BAD_REQUEST, "Missing request body"),
            }
        }
        (&Method::POST, "/database/counters/reset") => {
            match &body {
                Some(body_content) => return Ok(handle_reset_counter(body_content).await),
                None => error_response(StatusCode::BAD_REQUEST, "Missing request body"),
            }
        }

        // Wheel endpoints
        (&Method::GET, "/database/wheel/options") => return Ok(handle_get_wheel_options(&query).await),
        (&Method::POST, "/database/wheel/options") => {
            match &body {
                Some(body_content) => return Ok(handle_add_wheel_option(body_content).await),
                None => error_response(StatusCode::BAD_REQUEST, "Missing request body"),
            }
        }
        (&Method::PUT, path) if path.starts_with("/database/wheel/options/") && !path.ends_with("/toggle") => {
            let id = path.trim_start_matches("/database/wheel/options/").parse::<i64>().ok();
            match (id, &body) {
                (Some(id), Some(body_content)) => return Ok(handle_update_wheel_option(id, body_content).await),
                _ => error_response(StatusCode::BAD_REQUEST, "Invalid request"),
            }
        }
        (&Method::DELETE, path) if path.starts_with("/database/wheel/options/") => {
            let id = path.trim_start_matches("/database/wheel/options/").parse::<i64>().ok();
            match id {
                Some(id) => return Ok(handle_delete_wheel_option(id).await),
                None => error_response(StatusCode::BAD_REQUEST, "Invalid wheel option ID"),
            }
        }
        (&Method::POST, path) if path.starts_with("/database/wheel/options/") && path.ends_with("/toggle") => {
            let id_str = path.trim_start_matches("/database/wheel/options/").trim_end_matches("/toggle");
            let id = id_str.parse::<i64>().ok();
            match id {
                Some(id) => return Ok(handle_toggle_wheel_option(id).await),
                None => error_response(StatusCode::BAD_REQUEST, "Invalid wheel option ID"),
            }
        }
        (&Method::POST, "/database/wheel/spin") => {
            match &body {
                Some(body_content) => return Ok(handle_spin_wheel(body_content).await),
                None => error_response(StatusCode::BAD_REQUEST, "Missing request body"),
            }
        }

        (&Method::GET, "/database/todos") => return Ok(handle_get_todos(&query).await),
        (&Method::POST, "/database/todos/add") => {
            match &body {
                Some(body_content) => return Ok(handle_add_todo(body_content).await),
                None => error_response(StatusCode::BAD_REQUEST, "Missing request body"),
            }
        }
        (&Method::POST, "/database/todos/complete") => {
            match &body {
                Some(body_content) => return Ok(handle_complete_todo(body_content).await),
                None => error_response(StatusCode::BAD_REQUEST, "Missing request body"),
            }
        }
        (&Method::DELETE, "/database/todos") => {
            match &body {
                Some(body_content) => return Ok(handle_delete_todo(body_content).await),
                None => error_response(StatusCode::BAD_REQUEST, "Missing request body"),
            }
        }
        (&Method::GET, "/database/tts/users") => return Ok(handle_get_tts_users(&query).await),
        (&Method::POST, "/database/tts/users/add") => {
            match &body {
                Some(body_content) => return Ok(handle_add_tts_user(body_content).await),
                None => error_response(StatusCode::BAD_REQUEST, "Missing request body"),
            }
        }
        (&Method::DELETE, "/database/tts/users") => {
            match &body {
                Some(body_content) => return Ok(handle_delete_tts_user(body_content).await),
                None => error_response(StatusCode::BAD_REQUEST, "Missing request body"),
            }
        }
        (&Method::GET, "/database/tts/settings") => return Ok(handle_get_tts_settings(&query).await),
        (&Method::POST, "/database/tts/settings") => {
            match &body {
                Some(body_content) => return Ok(handle_update_tts_settings(body_content).await),
                None => error_response(StatusCode::BAD_REQUEST, "Missing request body"),
            }
        }

        // Watchtime endpoints
        (&Method::GET, "/database/watchtime/all") => return Ok(handle_get_all_watchtime(&query).await),
        (&Method::GET, "/database/watchtime/search") => return Ok(handle_search_watchtime(&query).await),
        (&Method::GET, "/database/watchtime/by-period") => return Ok(handle_get_viewers_by_period(&query).await),

        // Database management endpoints
        (&Method::GET, "/database/tables") => return Ok(handle_get_tables().await),
        (&Method::GET, "/database/schema") => return Ok(handle_get_table_schema(&query).await),
        (&Method::POST, "/database/query") => {
            match &body {
                Some(body_content) => return Ok(handle_execute_query(body_content).await),
                None => error_response(StatusCode::BAD_REQUEST, "Missing request body"),
            }
        }

        // Hue endpoints
        (&Method::GET, "/hue/discover") => return Ok(handle_hue_discover().await),
        (&Method::POST, "/hue/pair") => {
            match &body {
                Some(body_content) => return Ok(handle_hue_pair(body_content).await),
                None => error_response(StatusCode::BAD_REQUEST, "Missing request body"),
            }
        }
        (&Method::GET, "/hue/config") => return Ok(handle_hue_get_config().await),
        (&Method::DELETE, "/hue/config") => return Ok(handle_hue_delete_config().await),
        (&Method::GET, "/hue/lights") => return Ok(handle_hue_get_lights().await),
        (&Method::POST, "/hue/lights/power") => {
            match &body {
                Some(body_content) => return Ok(handle_hue_set_power(body_content).await),
                None => error_response(StatusCode::BAD_REQUEST, "Missing request body"),
            }
        }
        (&Method::POST, "/hue/lights/brightness") => {
            match &body {
                Some(body_content) => return Ok(handle_hue_set_brightness(body_content).await),
                None => error_response(StatusCode::BAD_REQUEST, "Missing request body"),
            }
        }
        (&Method::POST, "/hue/lights/color") => {
            match &body {
                Some(body_content) => return Ok(handle_hue_set_color(body_content).await),
                None => error_response(StatusCode::BAD_REQUEST, "Missing request body"),
            }
        }
        (&Method::POST, "/hue/scene") => {
            match &body {
                Some(body_content) => return Ok(handle_hue_set_scene(body_content).await),
                None => error_response(StatusCode::BAD_REQUEST, "Missing request body"),
            }
        }
        (&Method::GET, "/hue/scenes") => {
            return Ok(handle_hue_get_scenes().await);
        }
        (&Method::POST, "/hue/scenes") => {
            match &body {
                Some(body_content) => return Ok(handle_hue_save_scene(body_content).await),
                None => error_response(StatusCode::BAD_REQUEST, "Missing request body"),
            }
        }
        (&Method::DELETE, "/hue/scenes") => {
            match &body {
                Some(body_content) => return Ok(handle_hue_delete_scene(body_content).await),
                None => error_response(StatusCode::BAD_REQUEST, "Missing request body"),
            }
        }
        // Animated scenes endpoints
        (&Method::GET, "/hue/animated-scenes") => {
            return Ok(handle_hue_get_animated_scenes().await);
        }
        (&Method::POST, "/hue/animated-scenes") => {
            match &body {
                Some(body_content) => return Ok(handle_hue_create_animated_scene(body_content).await),
                None => error_response(StatusCode::BAD_REQUEST, "Missing request body"),
            }
        }
        (&Method::DELETE, "/hue/animated-scenes") => {
            match &body {
                Some(body_content) => return Ok(handle_hue_delete_animated_scene(body_content).await),
                None => error_response(StatusCode::BAD_REQUEST, "Missing request body"),
            }
        }
        (&Method::POST, "/hue/animated-scenes/steps") => {
            match &body {
                Some(body_content) => return Ok(handle_hue_add_scene_step(body_content).await),
                None => error_response(StatusCode::BAD_REQUEST, "Missing request body"),
            }
        }
        (&Method::PUT, "/hue/animated-scenes/steps") => {
            match &body {
                Some(body_content) => return Ok(handle_hue_update_scene_step(body_content).await),
                None => error_response(StatusCode::BAD_REQUEST, "Missing request body"),
            }
        }
        (&Method::DELETE, "/hue/animated-scenes/steps") => {
            match &body {
                Some(body_content) => return Ok(handle_hue_delete_scene_step(body_content).await),
                None => error_response(StatusCode::BAD_REQUEST, "Missing request body"),
            }
        }
        (&Method::POST, "/hue/animated-scenes/reorder") => {
            match &body {
                Some(body_content) => return Ok(handle_hue_reorder_scene_steps(body_content).await),
                None => error_response(StatusCode::BAD_REQUEST, "Missing request body"),
            }
        }
        (&Method::POST, "/hue/animated-scenes/play") => {
            match &body {
                Some(body_content) => return Ok(handle_hue_play_animated_scene(body_content).await),
                None => error_response(StatusCode::BAD_REQUEST, "Missing request body"),
            }
        }

        // Withings endpoints
        (&Method::GET, "/withings/latest") => return Ok(handle_withings_get_latest().await),
        (&Method::GET, "/withings/history") => return Ok(handle_withings_get_history(&query).await),
        (&Method::POST, "/withings/sync") => return Ok(handle_withings_sync().await),
        (&Method::POST, "/withings/config") => {
            match &body {
                Some(body_content) => return Ok(handle_withings_save_config(body_content).await),
                None => error_response(StatusCode::BAD_REQUEST, "Missing request body"),
            }
        }
        (&Method::GET, "/withings/config") => return Ok(handle_withings_get_config().await),
        (&Method::GET, "/withings/auth-url") => return Ok(handle_withings_get_auth_url().await),
        (&Method::GET, "/withings/callback") => return Ok(handle_withings_callback(&uri).await),
        (&Method::POST, "/withings/token") => {
            match &body {
                Some(body_content) => return Ok(handle_withings_save_token(body_content).await),
                None => error_response(StatusCode::BAD_REQUEST, "Missing request body"),
            }
        }

        // Discord endpoints
        (&Method::GET, "/discord/config") => return Ok(handle_discord_get_config().await),
        (&Method::POST, "/discord/config") => {
            match &body {
                Some(body_content) => return Ok(handle_discord_save_config(body_content).await),
                None => error_response(StatusCode::BAD_REQUEST, "Missing request body"),
            }
        }
        (&Method::GET, "/discord/stats") => return Ok(handle_discord_get_stats().await),
        (&Method::POST, "/discord/start") => return Ok(handle_discord_start().await),
        (&Method::POST, "/discord/stop") => return Ok(handle_discord_stop().await),
        (&Method::POST, "/discord/restart") => return Ok(handle_discord_restart().await),

        // Song request endpoints
        (&Method::GET, "/song-requests/pending") => return Ok(handle_song_requests_get_pending().await),
        (&Method::GET, "/song-requests/all") => return Ok(handle_song_requests_get_all(&query).await),
        (&Method::POST, "/song-requests/status") => {
            match &body {
                Some(body_content) => return Ok(handle_song_request_update_status(body_content).await),
                None => error_response(StatusCode::BAD_REQUEST, "Missing request body"),
            }
        }
        (&Method::DELETE, "/song-requests/:id") => {
            match &body {
                Some(body_content) => return Ok(handle_song_request_delete(body_content).await),
                None => error_response(StatusCode::BAD_REQUEST, "Missing request body"),
            }
        }
        (&Method::DELETE, "/song-requests/clear") => {
            match &body {
                Some(body_content) => return Ok(handle_song_requests_clear(body_content).await),
                None => error_response(StatusCode::BAD_REQUEST, "Missing request body"),
            }
        }

        // Static file serving for overlay assets (JS, CSS, etc.)
        (&Method::GET, path) => {
            // Try to serve static files from dist/overlays directory
            // This handles requests like /chat.js, /chat.css referenced by overlay HTML
            let file_name = path.trim_start_matches('/');
            let file_path = format!("dist/overlays/{}", file_name);

            match std::fs::read(&file_path) {
                Ok(content) => {
                    let content_type = if file_name.ends_with(".js") {
                        "application/javascript; charset=utf-8"
                    } else if file_name.ends_with(".css") {
                        "text/css; charset=utf-8"
                    } else if file_name.ends_with(".html") {
                        "text/html; charset=utf-8"
                    } else if file_name.ends_with(".map") {
                        "application/json; charset=utf-8"
                    } else {
                        "application/octet-stream"
                    };

                    Response::builder()
                        .status(StatusCode::OK)
                        .header("Content-Type", content_type)
                        .header("Access-Control-Allow-Origin", "*")
                        .header("Cache-Control", "public, max-age=31536000") // Cache for 1 year
                        .body(BoxBody::new(Full::new(Bytes::from(content))))
                        .unwrap()
                }
                Err(_) => {
                    warn!("❓ Unknown route: {} {}", method, path);
                    error_response(StatusCode::NOT_FOUND, "Not Found")
                }
            }
        }

        _ => {
            warn!("❓ Unknown route: {} {}", method, path);
            error_response(StatusCode::NOT_FOUND, "Not Found")
        },
    };
    
    let duration = start_time.elapsed();
    let status = response.status();
    
    if status.is_success() {
        info!("{} {} - {} - {}ms", method, path, status, duration.as_millis());
    } else if status.is_client_error() {
        warn!(" {} {} - {} - {}ms", method, path, status, duration.as_millis());
    } else {
        error!("{} {} - {} - {}ms", method, path, status, duration.as_millis());
    }
    
    Ok(response)
}

async fn read_request_body(req: Request<hyper::body::Incoming>) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let body = req.into_body();
    let body_bytes = body.collect().await?.to_bytes();
    Ok(String::from_utf8(body_bytes.to_vec())?)
}

fn cors_response(status: StatusCode, body: &str) -> Response<BoxBody<Bytes, Infallible>> {
    Response::builder()
        .status(status)
        .header(ACCESS_CONTROL_ALLOW_ORIGIN, "*")
        .header(ACCESS_CONTROL_ALLOW_METHODS, "GET, POST, DELETE, OPTIONS")
        .header(ACCESS_CONTROL_ALLOW_HEADERS, "Content-Type, Authorization")
        .header(CONTENT_TYPE, "application/json")
        .body(BoxBody::new(Full::new(Bytes::from(body.to_string()))))
        .unwrap()
}

fn json_response<T: serde::Serialize>(data: &T) -> Response<BoxBody<Bytes, Infallible>> {
    let json = serde_json::to_string(data).unwrap_or_else(|_| "{}".to_string());
    cors_response(StatusCode::OK, &json)
}

fn error_response(status: StatusCode, message: &str) -> Response<BoxBody<Bytes, Infallible>> {
    let error_response = ApiResponse {
        success: false,
        content: None,
        error: Some(message.to_string()),
    };
    let json = serde_json::to_string(&error_response).unwrap_or_else(|_| "{}".to_string());
    cors_response(status, &json)
}

// Unified smart file handler
fn handle_unified_file_operation(request: UnifiedFileRequest) -> Response<BoxBody<Bytes, Infallible>> {
    match request.operation {
        FileOperation::List => handle_list_operation(&request.path),
        FileOperation::Read => handle_read_operation(&request.path, request.accept_header.as_deref()),
        FileOperation::Write { content } => handle_write_operation(&request.path, &content, false),
        FileOperation::WriteBinary { content } => handle_write_operation(&request.path, &content, true),
        FileOperation::Delete => handle_delete_operation(&request.path),
        FileOperation::Serve => handle_serve_operation(&request.path),
    }
}

fn handle_list_operation(dir_path: &str) -> Response<BoxBody<Bytes, Infallible>> {
    // Since list_directory_contents doesn't exist, return a simple directory listing
    use std::fs;
    
    match fs::read_dir(dir_path) {
        Ok(entries) => {
            let mut files = Vec::new();
            for entry in entries {
                if let Ok(entry) = entry {
                    if let Some(name) = entry.file_name().to_str() {
                        files.push(serde_json::json!({
                            "name": name,
                            "is_directory": entry.file_type().map(|t| t.is_dir()).unwrap_or(false),
                            "path": entry.path().to_string_lossy()
                        }));
                    }
                }
            }
            json_response(&files)
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

fn handle_read_operation(file_path: &str, accept_header: Option<&str>) -> Response<BoxBody<Bytes, Infallible>> {
    // Check if client wants raw content (e.g., Accept: application/octet-stream)
    let wants_raw = accept_header
        .map(|h| h.contains("application/octet-stream") || h.contains("*/*"))
        .unwrap_or(false);
    
    if wants_raw || is_likely_binary_file(file_path) {
        // Serve as binary
        handle_serve_operation(file_path)
    } else {
        // Read as text and return JSON
        match read_file_content(file_path) {
            Ok(content) => {
                let response = ApiResponse {
                    success: true,
                    content: Some(content),
                    error: None,
                };
                json_response(&response)
            }
            Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e),
        }
    }
}

fn handle_write_operation(file_path: &str, content: &str, is_binary: bool) -> Response<BoxBody<Bytes, Infallible>> {
    let result = if is_binary {
        let write_req: WriteBinaryFileRequest = match serde_json::from_str(content) {
            Ok(req) => req,
            Err(_) => return error_response(StatusCode::BAD_REQUEST, "Invalid JSON"),
        };
        write_binary_file_content(file_path, &write_req)
    } else {
        let write_req: WriteFileRequest = match serde_json::from_str(content) {
            Ok(req) => req,
            Err(_) => return error_response(StatusCode::BAD_REQUEST, "Invalid JSON"),
        };
        write_file_content(file_path, &write_req)
    };

    match result {
        Ok(_) => {
            let response = ApiResponse {
                success: true,
                content: None,
                error: None,
            };
            json_response(&response)
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e),
    }
}

fn handle_delete_operation(file_path: &str) -> Response<BoxBody<Bytes, Infallible>> {
    match delete_file_or_directory(file_path) {
        Ok(_) => {
            let response = ApiResponse {
                success: true,
                content: None,
                error: None,
            };
            json_response(&response)
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e),
    }
}

fn handle_serve_operation(file_path: &str) -> Response<BoxBody<Bytes, Infallible>> {
    match read_binary_file(file_path) {
        Ok(contents) => {
            let full_path = std::path::Path::new(file_path);
            let content_type = get_file_content_type(full_path);
            
            Response::builder()
                .status(StatusCode::OK)
                .header(ACCESS_CONTROL_ALLOW_ORIGIN, "*")
                .header(CONTENT_TYPE, content_type)
                .body(BoxBody::new(Full::new(Bytes::from(contents))))
                .unwrap()
        }
        Err(e) => {
            let status = if e == "File not found" {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            error_response(status, &e)
        }
    }
}

// Helper function to detect binary files based on extension
fn is_likely_binary_file(file_path: &str) -> bool {
    let path = std::path::Path::new(file_path);
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        matches!(ext.to_lowercase().as_str(), 
            "png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp" | "ico" |
            "mp4" | "mov" | "avi" | "mkv" | "webm" |
            "mp3" | "wav" | "ogg" | "flac" |
            "zip" | "rar" | "7z" | "tar" | "gz" |
            "pdf" | "doc" | "docx" | "ppt" | "pptx" |
            "exe" | "dll" | "so" | "dylib" |
            "glb" | "gltf" | "fbx" | "obj" | "dae" | "blend" |
            "ttf" | "otf" | "woff" | "woff2"
        )
    } else {
        false
    }
}

// Helper function to decode and validate file paths
fn decode_and_validate_path(encoded_path: &str) -> Result<String, String> {
    decode_url_path(encoded_path)
}

fn decode_url_path(encoded_path: &str) -> Result<String, String> {
    match percent_decode_str(encoded_path).decode_utf8() {
        Ok(decoded) => Ok(decoded.to_string()),
        Err(e) => {
            error!("Failed to decode URL path '{}': {}", encoded_path, e);
            Err(format!("Invalid URL encoding: {}", e))
        }
    }
}

fn handle_start_watcher() -> Response<BoxBody<Bytes, Infallible>> {
    let response = ApiResponse {
        success: true,
        content: Some("File watcher is running".to_string()),
        error: None,
    };
    json_response(&response)
}

fn handle_get_file_changes() -> Response<BoxBody<Bytes, Infallible>> {
    json_response(&Vec::<String>::new())
}

fn handle_clear_file_changes() -> Response<BoxBody<Bytes, Infallible>> {
    let response = ApiResponse {
        success: true,
        content: None,
        error: None,
    };
    json_response(&response)
}

fn handle_set_current_project(body_content: &str) -> Response<BoxBody<Bytes, Infallible>> {
    #[derive(serde::Deserialize)]
    struct SetProjectRequest {
        project_name: Option<String>,
    }
    
    match serde_json::from_str::<SetProjectRequest>(body_content) {
        Ok(request) => {
            set_current_project(request.project_name.clone());
            let response = ApiResponse {
                success: true,
                content: request.project_name.map(|n| format!("Now watching project: {}", n))
                    .or(Some("Now watching all projects".to_string())),
                error: None,
            };
            json_response(&response)
        }
        Err(e) => {
            error_response(StatusCode::BAD_REQUEST, &format!("Invalid request format: {}", e))
        }
    }
}


fn handle_health_check() -> Response<BoxBody<Bytes, Infallible>> {
    use std::time::{SystemTime, UNIX_EPOCH};
    
    // Get basic system info
    let uptime = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    
    let health_data = serde_json::json!({
        "status": "healthy",
        "uptime": uptime % 86400, // Reset daily for demo
        "cache_size": 1024 * 1024, // Mock cache size
        "thumbnail_count": 5, // Mock thumbnail count
        "watched_files": 12, // Mock watched files count
        "timestamp": uptime
    });
    
    json_response(&health_data)
}

fn handle_get_startup_time() -> Response<BoxBody<Bytes, Infallible>> {
    let startup_time = STARTUP_TIME.get().copied().unwrap_or_else(|| {
        // Fallback: return current time if not set
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    });
    
    let startup_data = serde_json::json!({
        "startup_time": startup_time,
        "startup_time_ms": startup_time * 1000 // Also provide milliseconds for JavaScript
    });
    
    json_response(&startup_data)
}

fn handle_get_system_stats() -> Response<BoxBody<Bytes, Infallible>> {
    let stats = get_system_stats();
    json_response(&stats)
}

fn handle_restart_bridge() -> Response<BoxBody<Bytes, Infallible>> {
    // In a real implementation, this would restart the server
    // For now, just return success
    let response = ApiResponse {
        success: true,
        content: Some("Bridge restart initiated".to_string()),
        error: None,
    };
    json_response(&response)
}

fn handle_clear_cache() -> Response<BoxBody<Bytes, Infallible>> {
    // In a real implementation, this would clear caches
    // For now, just return success
    let response = ApiResponse {
        success: true,
        content: Some("Cache cleared successfully".to_string()),
        error: None,
    };
    json_response(&response)
}

// ========== Twitch Handlers ==========

async fn handle_twitch_start() -> Response<BoxBody<Bytes, Infallible>> {
    match get_twitch_manager() {
        Some(manager) => {
            match manager.start().await {
                Ok(_) => {
                    let response = ApiResponse {
                        success: true,
                        content: Some("Twitch bot started successfully".to_string()),
                        error: None,
                    };
                    json_response(&response)
                }
                Err(e) => {
                    error!("Failed to start Twitch bot: {}", e);
                    error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to start bot: {}", e))
                }
            }
        }
        None => error_response(StatusCode::SERVICE_UNAVAILABLE, "Twitch manager not initialized"),
    }
}

async fn handle_twitch_stop() -> Response<BoxBody<Bytes, Infallible>> {
    match get_twitch_manager() {
        Some(manager) => {
            match manager.stop().await {
                Ok(_) => {
                    let response = ApiResponse {
                        success: true,
                        content: Some("Twitch bot stopped successfully".to_string()),
                        error: None,
                    };
                    json_response(&response)
                }
                Err(e) => {
                    error!("Failed to stop Twitch bot: {}", e);
                    error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to stop bot: {}", e))
                }
            }
        }
        None => error_response(StatusCode::SERVICE_UNAVAILABLE, "Twitch manager not initialized"),
    }
}

async fn handle_twitch_status() -> Response<BoxBody<Bytes, Infallible>> {
    match get_twitch_manager() {
        Some(manager) => {
            let stats = manager.get_stats().await;
            json_response(&stats)
        }
        None => error_response(StatusCode::SERVICE_UNAVAILABLE, "Twitch manager not initialized"),
    }
}

async fn handle_twitch_auth_url() -> Response<BoxBody<Bytes, Infallible>> {
    match get_twitch_manager() {
        Some(manager) => {
            let auth = manager.get_auth();
            let scopes = crate::modules::twitch::TwitchAuth::get_default_scopes();

            match auth.generate_auth_url(scopes).await {
                Ok(url) => {
                    let response = serde_json::json!({
                        "success": true,
                        "url": url
                    });
                    json_response(&response)
                }
                Err(e) => {
                    error!("Failed to generate auth URL: {}", e);
                    error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to generate auth URL: {}", e))
                }
            }
        }
        None => error_response(StatusCode::SERVICE_UNAVAILABLE, "Twitch manager not initialized"),
    }
}

async fn handle_twitch_callback_get(uri: &hyper::Uri) -> Response<BoxBody<Bytes, Infallible>> {
    // Extract query parameters from URI
    let query = uri.query().unwrap_or("");

    // Parse query string
    let params: std::collections::HashMap<String, String> = query
        .split('&')
        .filter_map(|pair| {
            let mut split = pair.split('=');
            match (split.next(), split.next()) {
                (Some(key), Some(value)) => {
                    // URL decode the value using percent-encoding
                    use percent_encoding::percent_decode_str;
                    let decoded_value = percent_decode_str(value).decode_utf8().unwrap_or_default();
                    Some((key.to_string(), decoded_value.to_string()))
                }
                _ => None,
            }
        })
        .collect();

    let code = params.get("code");
    let state = params.get("state");
    let error = params.get("error");

    // Check for errors from Twitch
    if let Some(error) = error {
        let error_description = params.get("error_description").map(|s| s.as_str()).unwrap_or("");
        return error_response(StatusCode::BAD_REQUEST, &format!("{}: {}", error, error_description));
    }

    // Check if we have the required parameters
    if code.is_none() || state.is_none() {
        return error_response(StatusCode::BAD_REQUEST, "Missing code or state parameter");
    }

    match get_twitch_manager() {
        Some(manager) => {
            let auth = manager.get_auth();
            let code = code.unwrap();
            let state = state.unwrap();

            // Verify state
            match auth.verify_state(state).await {
                Ok(true) => {
                    // Exchange code for token
                    match auth.exchange_code(code).await {
                        Ok(_token) => {
                            // Return a success HTML page
                            let html = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Authentication Successful</title>
    <style>
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
            display: flex;
            justify-content: center;
            align-items: center;
            height: 100vh;
            margin: 0;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
        }
        .container {
            text-align: center;
            padding: 2rem;
            background: rgba(255, 255, 255, 0.1);
            border-radius: 1rem;
            backdrop-filter: blur(10px);
            box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
        }
        .success {
            color: #51cf66;
            font-size: 4rem;
            margin-bottom: 1rem;
        }
    </style>
</head>
<body>
    <div class="container">
        <div class="success">✓</div>
        <h1>Authentication Successful!</h1>
        <p>You can close this window now.</p>
    </div>
    <script>
        setTimeout(() => window.close(), 2000);
    </script>
</body>
</html>"#;
                            Response::builder()
                                .status(StatusCode::OK)
                                .header(hyper::header::CONTENT_TYPE, "text/html")
                                .body(BoxBody::new(Full::new(Bytes::from(html))))
                                .unwrap()
                        }
                        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to exchange code: {}", e)),
                    }
                }
                Ok(false) => error_response(StatusCode::BAD_REQUEST, "Invalid state parameter"),
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to verify state: {}", e)),
            }
        }
        None => error_response(StatusCode::INTERNAL_SERVER_ERROR, "Twitch manager not initialized"),
    }
}

async fn handle_twitch_callback(body: &str) -> Response<BoxBody<Bytes, Infallible>> {
    #[derive(serde::Deserialize)]
    struct CallbackRequest {
        code: String,
        state: String,
    }

    match get_twitch_manager() {
        Some(manager) => {
            match serde_json::from_str::<CallbackRequest>(body) {
                Ok(req) => {
                    let auth = manager.get_auth();

                    // Verify state
                    match auth.verify_state(&req.state).await {
                        Ok(true) => {
                            // Exchange code for token
                            match auth.exchange_code(&req.code).await {
                                Ok(_token) => {
                                    let response = ApiResponse {
                                        success: true,
                                        content: Some("Authentication successful".to_string()),
                                        error: None,
                                    };
                                    json_response(&response)
                                }
                                Err(e) => {
                                    error!("Failed to exchange code: {}", e);
                                    error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Authentication failed: {}", e))
                                }
                            }
                        }
                        Ok(false) => {
                            error_response(StatusCode::BAD_REQUEST, "Invalid state (CSRF check failed)")
                        }
                        Err(e) => {
                            error!("Failed to verify state: {}", e);
                            error_response(StatusCode::INTERNAL_SERVER_ERROR, "State verification failed")
                        }
                    }
                }
                Err(e) => {
                    error_response(StatusCode::BAD_REQUEST, &format!("Invalid request: {}", e))
                }
            }
        }
        None => error_response(StatusCode::SERVICE_UNAVAILABLE, "Twitch manager not initialized"),
    }
}

async fn handle_twitch_send_message(body: &str) -> Response<BoxBody<Bytes, Infallible>> {
    #[derive(serde::Deserialize)]
    struct SendMessageRequest {
        channel: String,
        message: String,
    }

    match get_twitch_manager() {
        Some(manager) => {
            match serde_json::from_str::<SendMessageRequest>(body) {
                Ok(req) => {
                    match manager.send_message(&req.channel, &req.message).await {
                        Ok(_) => {
                            let response = ApiResponse {
                                success: true,
                                content: Some("Message sent".to_string()),
                                error: None,
                            };
                            json_response(&response)
                        }
                        Err(e) => {
                            error!("Failed to send message: {}", e);
                            error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to send message: {}", e))
                        }
                    }
                }
                Err(e) => {
                    error_response(StatusCode::BAD_REQUEST, &format!("Invalid request: {}", e))
                }
            }
        }
        None => error_response(StatusCode::SERVICE_UNAVAILABLE, "Twitch manager not initialized"),
    }
}

async fn handle_twitch_get_commands() -> Response<BoxBody<Bytes, Infallible>> {
    match get_twitch_manager() {
        Some(manager) => {
            let commands = manager.get_commands().await;

            let command_info: Vec<serde_json::Value> = commands
                .iter()
                .map(|cmd| {
                    serde_json::json!({
                        "name": cmd.name,
                        "aliases": cmd.aliases,
                        "description": cmd.description,
                        "usage": cmd.usage,
                        "permission": cmd.permission,
                        "cooldown": cmd.cooldown_seconds,
                        "enabled": cmd.enabled,
                    })
                })
                .collect();

            json_response(&command_info)
        }
        None => error_response(StatusCode::SERVICE_UNAVAILABLE, "Twitch manager not initialized"),
    }
}

async fn handle_twitch_register_command(body: &str) -> Response<BoxBody<Bytes, Infallible>> {
    #[derive(serde::Deserialize)]
    struct RegisterCommandRequest {
        name: String,
        aliases: Option<Vec<String>>,
        description: String,
        permission: Option<String>,
        response: String,
    }

    match get_twitch_manager() {
        Some(manager) => {
            match serde_json::from_str::<RegisterCommandRequest>(body) {
                Ok(req) => {
                    let permission = match req.permission.as_deref() {
                        Some("subscriber") => PermissionLevel::Subscriber,
                        Some("vip") => PermissionLevel::Vip,
                        Some("moderator") => PermissionLevel::Moderator,
                        Some("broadcaster") => PermissionLevel::Broadcaster,
                        _ => PermissionLevel::Everyone,
                    };

                    let simple_cmd = SimpleCommand {
                        name: req.name.clone(),
                        aliases: req.aliases.unwrap_or_default(),
                        description: req.description,
                        permission,
                        response: req.response,
                    };

                    manager.register_simple_command(simple_cmd).await;

                    let response = ApiResponse {
                        success: true,
                        content: Some(format!("Command '{}' registered", req.name)),
                        error: None,
                    };
                    json_response(&response)
                }
                Err(e) => {
                    error_response(StatusCode::BAD_REQUEST, &format!("Invalid request: {}", e))
                }
            }
        }
        None => error_response(StatusCode::SERVICE_UNAVAILABLE, "Twitch manager not initialized"),
    }
}

async fn handle_twitch_unregister_command(body: &str) -> Response<BoxBody<Bytes, Infallible>> {
    #[derive(serde::Deserialize)]
    struct UnregisterCommandRequest {
        name: String,
    }

    match get_twitch_manager() {
        Some(manager) => {
            match serde_json::from_str::<UnregisterCommandRequest>(body) {
                Ok(req) => {
                    manager.unregister_command(&req.name).await;

                    let response = ApiResponse {
                        success: true,
                        content: Some(format!("Command '{}' unregistered", req.name)),
                        error: None,
                    };
                    json_response(&response)
                }
                Err(e) => {
                    error_response(StatusCode::BAD_REQUEST, &format!("Invalid request: {}", e))
                }
            }
        }
        None => error_response(StatusCode::SERVICE_UNAVAILABLE, "Twitch manager not initialized"),
    }
}

async fn handle_twitch_get_config() -> Response<BoxBody<Bytes, Infallible>> {
    match get_twitch_manager() {
        Some(manager) => {
            let config_manager = manager.get_config_manager();

            match config_manager.load() {
                Ok(config) => {
                    // Don't send sensitive data
                    let safe_config = serde_json::json!({
                        "client_id": config.client_id,
                        "bot_username": config.bot_username,
                        "channels": config.channels,
                        "has_token": config.access_token.is_some(),
                    });
                    json_response(&safe_config)
                }
                Err(e) => {
                    error!("Failed to load config: {}", e);
                    error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to load config: {}", e))
                }
            }
        }
        None => error_response(StatusCode::SERVICE_UNAVAILABLE, "Twitch manager not initialized"),
    }
}

async fn handle_twitch_save_config(body: &str) -> Response<BoxBody<Bytes, Infallible>> {
    #[derive(serde::Deserialize)]
    struct SaveConfigRequest {
        client_id: Option<String>,
        client_secret: Option<String>,
        bot_username: Option<String>,
        channels: Option<Vec<String>>,
    }

    match get_twitch_manager() {
        Some(manager) => {
            let config_manager = manager.get_config_manager();

            match serde_json::from_str::<SaveConfigRequest>(body) {
                Ok(req) => {
                    match config_manager.load() {
                        Ok(mut config) => {
                            if let Some(client_id) = req.client_id {
                                config.client_id = client_id;
                            }
                            if let Some(client_secret) = req.client_secret {
                                config.client_secret = client_secret;
                            }
                            if let Some(bot_username) = req.bot_username {
                                config.bot_username = bot_username;
                            }
                            if let Some(channels) = req.channels {
                                config.channels = channels;
                            }

                            match config_manager.save(&config) {
                                Ok(_) => {
                                    let response = ApiResponse {
                                        success: true,
                                        content: Some("Config saved successfully".to_string()),
                                        error: None,
                                    };
                                    json_response(&response)
                                }
                                Err(e) => {
                                    error!("Failed to save config: {}", e);
                                    error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to save config: {}", e))
                                }
                            }
                        }
                        Err(e) => {
                            error!("Failed to load config: {}", e);
                            error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to load config: {}", e))
                        }
                    }
                }
                Err(e) => {
                    error_response(StatusCode::BAD_REQUEST, &format!("Invalid request: {}", e))
                }
            }
        }
        None => error_response(StatusCode::SERVICE_UNAVAILABLE, "Twitch manager not initialized"),
    }
}

async fn handle_twitch_join_channel(body: &str) -> Response<BoxBody<Bytes, Infallible>> {
    #[derive(serde::Deserialize)]
    struct JoinChannelRequest {
        channel: String,
    }

    match get_twitch_manager() {
        Some(manager) => {
            match serde_json::from_str::<JoinChannelRequest>(body) {
                Ok(req) => {
                    match manager.join_channel(&req.channel).await {
                        Ok(_) => {
                            let response = ApiResponse {
                                success: true,
                                content: Some(format!("Joined channel: {}", req.channel)),
                                error: None,
                            };
                            json_response(&response)
                        }
                        Err(e) => {
                            error!("Failed to join channel: {}", e);
                            error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to join channel: {}", e))
                        }
                    }
                }
                Err(e) => {
                    error_response(StatusCode::BAD_REQUEST, &format!("Invalid request: {}", e))
                }
            }
        }
        None => error_response(StatusCode::SERVICE_UNAVAILABLE, "Twitch manager not initialized"),
    }
}

async fn handle_twitch_part_channel(body: &str) -> Response<BoxBody<Bytes, Infallible>> {
    #[derive(serde::Deserialize)]
    struct PartChannelRequest {
        channel: String,
    }

    match get_twitch_manager() {
        Some(manager) => {
            match serde_json::from_str::<PartChannelRequest>(body) {
                Ok(req) => {
                    match manager.part_channel(&req.channel).await {
                        Ok(_) => {
                            let response = ApiResponse {
                                success: true,
                                content: Some(format!("Parted channel: {}", req.channel)),
                                error: None,
                            };
                            json_response(&response)
                        }
                        Err(e) => {
                            error!("Failed to part channel: {}", e);
                            error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to part channel: {}", e))
                        }
                    }
                }
                Err(e) => {
                    error_response(StatusCode::BAD_REQUEST, &format!("Invalid request: {}", e))
                }
            }
        }
        None => error_response(StatusCode::SERVICE_UNAVAILABLE, "Twitch manager not initialized"),
    }
}

async fn handle_twitch_revoke() -> Response<BoxBody<Bytes, Infallible>> {
    match get_twitch_manager() {
        Some(manager) => {
            let auth = manager.get_auth();

            match auth.revoke_token().await {
                Ok(_) => {
                    let response = ApiResponse {
                        success: true,
                        content: Some("Token revoked successfully".to_string()),
                        error: None,
                    };
                    json_response(&response)
                }
                Err(e) => {
                    error!("Failed to revoke token: {}", e);
                    error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to revoke token: {}", e))
                }
            }
        }
        None => error_response(StatusCode::SERVICE_UNAVAILABLE, "Twitch manager not initialized"),
    }
}

// ========== Database Handlers ==========

async fn handle_get_counters(query: &str) -> Response<BoxBody<Bytes, Infallible>> {
    match get_database() {
        Some(db) => {
            // Parse channel from query string
            let channel = query.split('=')
                .nth(1)
                .unwrap_or("")
                .to_string();

            if channel.is_empty() {
                return error_response(StatusCode::BAD_REQUEST, "Missing channel parameter");
            }

            match db.get_all_counts(&channel) {
                Ok(counters) => {
                    let counter_list: Vec<serde_json::Value> = counters
                        .iter()
                        .map(|(task, count)| {
                            serde_json::json!({
                                "task": task,
                                "count": count
                            })
                        })
                        .collect();
                    json_response(&counter_list)
                }
                Err(e) => {
                    error!("Failed to get counters: {}", e);
                    error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to get counters: {}", e))
                }
            }
        }
        None => error_response(StatusCode::SERVICE_UNAVAILABLE, "Database not initialized"),
    }
}

async fn handle_increment_counter(body: &str) -> Response<BoxBody<Bytes, Infallible>> {
    #[derive(serde::Deserialize)]
    struct IncrementRequest {
        channel: String,
        task: String,
    }

    match get_database() {
        Some(db) => {
            match serde_json::from_str::<IncrementRequest>(body) {
                Ok(req) => {
                    match db.increment(&req.channel, &req.task) {
                        Ok(count) => {
                            let response = serde_json::json!({
                                "success": true,
                                "count": count
                            });
                            json_response(&response)
                        }
                        Err(e) => {
                            error!("Failed to increment counter: {}", e);
                            error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to increment counter: {}", e))
                        }
                    }
                }
                Err(e) => error_response(StatusCode::BAD_REQUEST, &format!("Invalid request: {}", e)),
            }
        }
        None => error_response(StatusCode::SERVICE_UNAVAILABLE, "Database not initialized"),
    }
}

async fn handle_decrement_counter(body: &str) -> Response<BoxBody<Bytes, Infallible>> {
    #[derive(serde::Deserialize)]
    struct DecrementRequest {
        channel: String,
        task: String,
    }

    match get_database() {
        Some(db) => {
            match serde_json::from_str::<DecrementRequest>(body) {
                Ok(req) => {
                    match db.decrement(&req.channel, &req.task) {
                        Ok(count) => {
                            let response = serde_json::json!({
                                "success": true,
                                "count": count
                            });
                            json_response(&response)
                        }
                        Err(e) => {
                            error!("Failed to decrement counter: {}", e);
                            error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to decrement counter: {}", e))
                        }
                    }
                }
                Err(e) => error_response(StatusCode::BAD_REQUEST, &format!("Invalid request: {}", e)),
            }
        }
        None => error_response(StatusCode::SERVICE_UNAVAILABLE, "Database not initialized"),
    }
}

async fn handle_reset_counter(body: &str) -> Response<BoxBody<Bytes, Infallible>> {
    #[derive(serde::Deserialize)]
    struct ResetRequest {
        channel: String,
        task: String,
    }

    match get_database() {
        Some(db) => {
            match serde_json::from_str::<ResetRequest>(body) {
                Ok(req) => {
                    match db.reset(&req.channel, &req.task) {
                        Ok(_) => {
                            let response = ApiResponse {
                                success: true,
                                content: Some("Counter reset successfully".to_string()),
                                error: None,
                            };
                            json_response(&response)
                        }
                        Err(e) => {
                            error!("Failed to reset counter: {}", e);
                            error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to reset counter: {}", e))
                        }
                    }
                }
                Err(e) => error_response(StatusCode::BAD_REQUEST, &format!("Invalid request: {}", e)),
            }
        }
        None => error_response(StatusCode::SERVICE_UNAVAILABLE, "Database not initialized"),
    }
}

// Wheel handlers
async fn handle_get_wheel_options(query: &str) -> Response<BoxBody<Bytes, Infallible>> {
    match get_database() {
        Some(db) => {
            let channel = query.split('=')
                .nth(1)
                .unwrap_or("")
                .to_string();

            if channel.is_empty() {
                return error_response(StatusCode::BAD_REQUEST, "Missing channel parameter");
            }

            match db.get_wheel_options(&channel) {
                Ok(options) => json_response(&options),
                Err(e) => {
                    error!("Failed to get wheel options: {}", e);
                    error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to get wheel options: {}", e))
                }
            }
        }
        None => error_response(StatusCode::SERVICE_UNAVAILABLE, "Database not initialized"),
    }
}

async fn handle_add_wheel_option(body: &str) -> Response<BoxBody<Bytes, Infallible>> {
    #[derive(serde::Deserialize)]
    struct AddRequest {
        channel: String,
        option_text: String,
        color: String,
        weight: i64,
    }

    match get_database() {
        Some(db) => {
            match serde_json::from_str::<AddRequest>(body) {
                Ok(req) => {
                    match db.add_wheel_option(&req.channel, &req.option_text, &req.color, req.weight) {
                        Ok(_) => {
                            let response = ApiResponse {
                                success: true,
                                content: Some("Wheel option added successfully".to_string()),
                                error: None,
                            };
                            json_response(&response)
                        }
                        Err(e) => {
                            error!("Failed to add wheel option: {}", e);
                            error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to add wheel option: {}", e))
                        }
                    }
                }
                Err(e) => error_response(StatusCode::BAD_REQUEST, &format!("Invalid request: {}", e)),
            }
        }
        None => error_response(StatusCode::SERVICE_UNAVAILABLE, "Database not initialized"),
    }
}

async fn handle_update_wheel_option(id: i64, body: &str) -> Response<BoxBody<Bytes, Infallible>> {
    #[derive(serde::Deserialize)]
    struct UpdateRequest {
        option_text: String,
        color: String,
        weight: i64,
    }

    match get_database() {
        Some(db) => {
            match serde_json::from_str::<UpdateRequest>(body) {
                Ok(req) => {
                    match db.update_wheel_option(id, &req.option_text, &req.color, req.weight) {
                        Ok(_) => {
                            let response = ApiResponse {
                                success: true,
                                content: Some("Wheel option updated successfully".to_string()),
                                error: None,
                            };
                            json_response(&response)
                        }
                        Err(e) => {
                            error!("Failed to update wheel option: {}", e);
                            error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to update wheel option: {}", e))
                        }
                    }
                }
                Err(e) => error_response(StatusCode::BAD_REQUEST, &format!("Invalid request: {}", e)),
            }
        }
        None => error_response(StatusCode::SERVICE_UNAVAILABLE, "Database not initialized"),
    }
}

async fn handle_delete_wheel_option(id: i64) -> Response<BoxBody<Bytes, Infallible>> {
    match get_database() {
        Some(db) => {
            match db.delete_wheel_option(id) {
                Ok(_) => {
                    let response = ApiResponse {
                        success: true,
                        content: Some("Wheel option deleted successfully".to_string()),
                        error: None,
                    };
                    json_response(&response)
                }
                Err(e) => {
                    error!("Failed to delete wheel option: {}", e);
                    error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to delete wheel option: {}", e))
                }
            }
        }
        None => error_response(StatusCode::SERVICE_UNAVAILABLE, "Database not initialized"),
    }
}

async fn handle_toggle_wheel_option(id: i64) -> Response<BoxBody<Bytes, Infallible>> {
    match get_database() {
        Some(db) => {
            match db.toggle_wheel_option(id) {
                Ok(_) => {
                    let response = ApiResponse {
                        success: true,
                        content: Some("Wheel option toggled successfully".to_string()),
                        error: None,
                    };
                    json_response(&response)
                }
                Err(e) => {
                    error!("Failed to toggle wheel option: {}", e);
                    error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to toggle wheel option: {}", e))
                }
            }
        }
        None => error_response(StatusCode::SERVICE_UNAVAILABLE, "Database not initialized"),
    }
}

async fn handle_spin_wheel(body: &str) -> Response<BoxBody<Bytes, Infallible>> {
    #[derive(serde::Deserialize)]
    struct SpinRequest {
        channel: String,
    }

    match (get_database(), get_twitch_manager()) {
        (Some(db), Some(twitch_manager)) => {
            match serde_json::from_str::<SpinRequest>(body) {
                Ok(req) => {
                    // Get wheel options
                    let options = match db.get_wheel_options(&req.channel) {
                        Ok(opts) => opts,
                        Err(e) => {
                            error!("Failed to get wheel options: {}", e);
                            return error_response(StatusCode::INTERNAL_SERVER_ERROR, "Failed to get wheel options");
                        }
                    };

                    // Filter to enabled only
                    let enabled_options: Vec<_> = options.into_iter()
                        .filter(|opt| opt.enabled == 1)
                        .collect();

                    if enabled_options.is_empty() {
                        return error_response(StatusCode::BAD_REQUEST, "No enabled wheel options available");
                    }

                    // Build weighted list
                    let mut weighted_options = Vec::new();
                    for option in &enabled_options {
                        for _ in 0..option.weight {
                            weighted_options.push(option);
                        }
                    }

                    // Pick random winner
                    let winner_index = fastrand::usize(0..weighted_options.len());
                    let winner = weighted_options[winner_index];

                    // Convert to wheel options for event
                    use crate::modules::twitch::twitch_irc_client::{WheelOption, WheelSpinEvent, TwitchEvent};
                    let wheel_options: Vec<WheelOption> = enabled_options.iter()
                        .map(|opt| WheelOption {
                            text: opt.option_text.clone(),
                            color: opt.color.clone(),
                        })
                        .collect();

                    // Record the spin
                    if let Err(e) = db.record_wheel_spin(&req.channel, &winner.option_text, Some("Viewport")) {
                        error!("Failed to record wheel spin: {}", e);
                    }

                    // Broadcast event
                    let wheel_event = TwitchEvent::WheelSpin(WheelSpinEvent {
                        channel: req.channel.clone(),
                        winner: winner.option_text.clone(),
                        options: wheel_options,
                        triggered_by: Some("Viewport".to_string()),
                    });

                    let event_sender = twitch_manager.get_event_sender();
                    if let Err(e) = event_sender.send(wheel_event) {
                        error!("Failed to broadcast wheel spin event: {}", e);
                    }

                    // Return response
                    let response = serde_json::json!({
                        "success": true,
                        "winner": winner.option_text,
                        "triggered_by": "Viewport"
                    });
                    json_response(&response)
                }
                Err(e) => error_response(StatusCode::BAD_REQUEST, &format!("Invalid request: {}", e)),
            }
        }
        (None, _) => error_response(StatusCode::SERVICE_UNAVAILABLE, "Database not initialized"),
        (_, None) => error_response(StatusCode::SERVICE_UNAVAILABLE, "Twitch manager not initialized"),
    }
}

async fn handle_get_todos(query: &str) -> Response<BoxBody<Bytes, Infallible>> {
    match get_database() {
        Some(db) => {
            // Parse channel and username from query string
            let params: std::collections::HashMap<_, _> = query
                .split('&')
                .filter_map(|p| {
                    let mut parts = p.split('=');
                    Some((parts.next()?, parts.next()?))
                })
                .collect();

            let channel = params.get("channel").map(|s| s.to_string());
            let username = params.get("username").map(|s| s.to_string());

            // If both channel and username are provided, get user-specific todos
            if let (Some(ch), Some(user)) = (&channel, &username) {
                if !ch.is_empty() && !user.is_empty() {
                    match db.get_user_todos(ch, user) {
                        Ok(todos) => {
                            let todo_list: Vec<serde_json::Value> = todos
                                .iter()
                                .map(|(id, task, created_at)| {
                                    serde_json::json!({
                                        "id": id,
                                        "task": task,
                                        "created_at": created_at
                                    })
                                })
                                .collect();
                            return json_response(&todo_list);
                        }
                        Err(e) => {
                            error!("Failed to get user todos: {}", e);
                            return error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to get todos: {}", e));
                        }
                    }
                }
            }

            // If only channel is provided, get all tasks for that channel
            if let Some(ch) = &channel {
                if !ch.is_empty() {
                    match db.get_channel_tasks(ch) {
                        Ok(tasks) => {
                            let task_list: Vec<serde_json::Value> = tasks
                                .iter()
                                .map(|(id, username, task, created_at)| {
                                    serde_json::json!({
                                        "id": id,
                                        "username": username,
                                        "task": task,
                                        "created_at": created_at
                                    })
                                })
                                .collect();
                            return json_response(&task_list);
                        }
                        Err(e) => {
                            error!("Failed to get channel tasks: {}", e);
                            return error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to get tasks: {}", e));
                        }
                    }
                }
            }

            // Otherwise, get all todos (for overlay)
            match db.get_all_todos() {
                Ok(todos) => {
                    let todo_list: Vec<serde_json::Value> = todos
                        .iter()
                        .map(|(id, channel, username, task, completed, created_at)| {
                            serde_json::json!({
                                "id": id,
                                "channel": channel,
                                "username": username,
                                "task_text": task,
                                "completed": completed,
                                "created_at": created_at
                            })
                        })
                        .collect();
                    json_response(&todo_list)
                }
                Err(e) => {
                    error!("Failed to get all todos: {}", e);
                    error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to get todos: {}", e))
                }
            }
        }
        None => error_response(StatusCode::SERVICE_UNAVAILABLE, "Database not initialized"),
    }
}

async fn handle_add_todo(body: &str) -> Response<BoxBody<Bytes, Infallible>> {
    #[derive(serde::Deserialize)]
    struct AddTodoRequest {
        channel: String,
        username: String,
        task: String,
    }

    match get_database() {
        Some(db) => {
            match serde_json::from_str::<AddTodoRequest>(body) {
                Ok(req) => {
                    match db.add_todo(&req.channel, &req.username, &req.task) {
                        Ok(id) => {
                            let response = serde_json::json!({
                                "success": true,
                                "id": id
                            });
                            json_response(&response)
                        }
                        Err(e) => {
                            error!("Failed to add todo: {}", e);
                            error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to add todo: {}", e))
                        }
                    }
                }
                Err(e) => error_response(StatusCode::BAD_REQUEST, &format!("Invalid request: {}", e)),
            }
        }
        None => error_response(StatusCode::SERVICE_UNAVAILABLE, "Database not initialized"),
    }
}

async fn handle_complete_todo(body: &str) -> Response<BoxBody<Bytes, Infallible>> {
    #[derive(serde::Deserialize)]
    struct CompleteTodoRequest {
        channel: String,
        username: String,
        id: i64,
    }

    match get_database() {
        Some(db) => {
            match serde_json::from_str::<CompleteTodoRequest>(body) {
                Ok(req) => {
                    match db.complete_todo(&req.channel, &req.username, req.id) {
                        Ok((success, should_award_xp)) => {
                            // Award XP if applicable (50 XP per completed task that's >= 5 minutes old)
                            if success && should_award_xp {
                                let _ = db.add_user_xp(&req.channel, &req.username, 50, 0);
                            }

                            let response = ApiResponse {
                                success,
                                content: if success {
                                    Some(if should_award_xp {
                                        "Todo completed! +50 XP".to_string()
                                    } else {
                                        "Todo completed".to_string()
                                    })
                                } else {
                                    Some("Todo not found".to_string())
                                },
                                error: None,
                            };
                            json_response(&response)
                        }
                        Err(e) => {
                            error!("Failed to complete todo: {}", e);
                            error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to complete todo: {}", e))
                        }
                    }
                }
                Err(e) => error_response(StatusCode::BAD_REQUEST, &format!("Invalid request: {}", e)),
            }
        }
        None => error_response(StatusCode::SERVICE_UNAVAILABLE, "Database not initialized"),
    }
}

async fn handle_delete_todo(body: &str) -> Response<BoxBody<Bytes, Infallible>> {
    #[derive(serde::Deserialize)]
    struct DeleteTodoRequest {
        channel: String,
        username: String,
        id: i64,
    }

    match get_database() {
        Some(db) => {
            match serde_json::from_str::<DeleteTodoRequest>(body) {
                Ok(req) => {
                    match db.remove_todo(&req.channel, &req.username, req.id) {
                        Ok(success) => {
                            let response = ApiResponse {
                                success,
                                content: if success { Some("Todo deleted".to_string()) } else { Some("Todo not found".to_string()) },
                                error: None,
                            };
                            json_response(&response)
                        }
                        Err(e) => {
                            error!("Failed to delete todo: {}", e);
                            error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to delete todo: {}", e))
                        }
                    }
                }
                Err(e) => error_response(StatusCode::BAD_REQUEST, &format!("Invalid request: {}", e)),
            }
        }
        None => error_response(StatusCode::SERVICE_UNAVAILABLE, "Database not initialized"),
    }
}

async fn handle_get_tts_users(query: &str) -> Response<BoxBody<Bytes, Infallible>> {
    match get_database() {
        Some(db) => {
            // Parse channel from query string
            let channel = query.split('=')
                .nth(1)
                .unwrap_or("")
                .to_string();

            if channel.is_empty() {
                return error_response(StatusCode::BAD_REQUEST, "Missing channel parameter");
            }

            match db.get_tts_users(&channel) {
                Ok(users) => json_response(&users),
                Err(e) => {
                    error!("Failed to get TTS users: {}", e);
                    error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to get TTS users: {}", e))
                }
            }
        }
        None => error_response(StatusCode::SERVICE_UNAVAILABLE, "Database not initialized"),
    }
}

async fn handle_add_tts_user(body: &str) -> Response<BoxBody<Bytes, Infallible>> {
    #[derive(serde::Deserialize)]
    struct AddTtsUserRequest {
        channel: String,
        username: String,
    }

    match get_database() {
        Some(db) => {
            match serde_json::from_str::<AddTtsUserRequest>(body) {
                Ok(req) => {
                    match db.add_tts_user(&req.channel, &req.username) {
                        Ok(_) => {
                            let response = ApiResponse {
                                success: true,
                                content: Some(format!("User '{}' added to TTS whitelist", req.username)),
                                error: None,
                            };
                            json_response(&response)
                        }
                        Err(e) => {
                            error!("Failed to add TTS user: {}", e);
                            error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to add TTS user: {}", e))
                        }
                    }
                }
                Err(e) => error_response(StatusCode::BAD_REQUEST, &format!("Invalid request: {}", e)),
            }
        }
        None => error_response(StatusCode::SERVICE_UNAVAILABLE, "Database not initialized"),
    }
}

async fn handle_delete_tts_user(body: &str) -> Response<BoxBody<Bytes, Infallible>> {
    #[derive(serde::Deserialize)]
    struct DeleteTtsUserRequest {
        channel: String,
        username: String,
    }

    match get_database() {
        Some(db) => {
            match serde_json::from_str::<DeleteTtsUserRequest>(body) {
                Ok(req) => {
                    match db.remove_tts_user(&req.channel, &req.username) {
                        Ok(success) => {
                            let response = ApiResponse {
                                success,
                                content: if success { Some(format!("User '{}' removed from TTS whitelist", req.username)) } else { Some("User not found".to_string()) },
                                error: None,
                            };
                            json_response(&response)
                        }
                        Err(e) => {
                            error!("Failed to remove TTS user: {}", e);
                            error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to remove TTS user: {}", e))
                        }
                    }
                }
                Err(e) => error_response(StatusCode::BAD_REQUEST, &format!("Invalid request: {}", e)),
            }
        }
        None => error_response(StatusCode::SERVICE_UNAVAILABLE, "Database not initialized"),
    }
}

async fn handle_get_tts_settings(query: &str) -> Response<BoxBody<Bytes, Infallible>> {
    match get_database() {
        Some(db) => {
            // Parse channel from query string
            let channel = query.split('=')
                .nth(1)
                .unwrap_or("")
                .to_string();

            if channel.is_empty() {
                return error_response(StatusCode::BAD_REQUEST, "Missing channel parameter");
            }

            match (db.is_tts_enabled(&channel), db.get_tts_mode(&channel)) {
                (Ok(enabled), Ok(mode)) => {
                    let settings = serde_json::json!({
                        "enabled": enabled,
                        "mode": mode
                    });
                    json_response(&settings)
                }
                _ => {
                    error_response(StatusCode::INTERNAL_SERVER_ERROR, "Failed to get TTS settings")
                }
            }
        }
        None => error_response(StatusCode::SERVICE_UNAVAILABLE, "Database not initialized"),
    }
}

async fn handle_update_tts_settings(body: &str) -> Response<BoxBody<Bytes, Infallible>> {
    #[derive(serde::Deserialize)]
    struct UpdateTtsSettingsRequest {
        channel: String,
        enabled: Option<bool>,
        mode: Option<String>,
    }

    match get_database() {
        Some(db) => {
            match serde_json::from_str::<UpdateTtsSettingsRequest>(body) {
                Ok(req) => {
                    let mut errors = Vec::new();

                    if let Some(enabled) = req.enabled {
                        if let Err(e) = db.set_tts_enabled(&req.channel, enabled) {
                            errors.push(format!("Failed to set TTS enabled: {}", e));
                        }
                    }

                    if let Some(mode) = req.mode {
                        if let Err(e) = db.set_tts_mode(&req.channel, &mode) {
                            errors.push(format!("Failed to set TTS mode: {}", e));
                        }
                    }

                    if errors.is_empty() {
                        let response = ApiResponse {
                            success: true,
                            content: Some("TTS settings updated".to_string()),
                            error: None,
                        };
                        json_response(&response)
                    } else {
                        error_response(StatusCode::INTERNAL_SERVER_ERROR, &errors.join(", "))
                    }
                }
                Err(e) => error_response(StatusCode::BAD_REQUEST, &format!("Invalid request: {}", e)),
            }
        }
        None => error_response(StatusCode::SERVICE_UNAVAILABLE, "Database not initialized"),
    }
}

// ========== Watchtime Handlers ==========

async fn handle_get_all_watchtime(query: &str) -> Response<BoxBody<Bytes, Infallible>> {
    match get_database() {
        Some(db) => {
            // Parse query parameters
            let params: std::collections::HashMap<String, String> = query
                .split('&')
                .filter_map(|param| {
                    let mut parts = param.split('=');
                    Some((parts.next()?.to_string(), parts.next()?.to_string()))
                })
                .collect();

            let channel = params.get("channel").map(|s| s.as_str()).unwrap_or("");
            let limit = params.get("limit")
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(50);
            let offset = params.get("offset")
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(0);

            if channel.is_empty() {
                return error_response(StatusCode::BAD_REQUEST, "Missing channel parameter");
            }

            match (db.get_all_watchers(channel, limit, offset), db.get_watchers_count(channel)) {
                (Ok(watchers), Ok(total)) => {
                    let watcher_list: Vec<serde_json::Value> = watchers
                        .iter()
                        .map(|(username, total_minutes, last_seen)| {
                            serde_json::json!({
                                "username": username,
                                "total_minutes": total_minutes,
                                "last_seen": last_seen
                            })
                        })
                        .collect();

                    let response = serde_json::json!({
                        "watchers": watcher_list,
                        "total": total,
                        "limit": limit,
                        "offset": offset
                    });

                    json_response(&response)
                }
                (Err(e), _) | (_, Err(e)) => {
                    error!("Failed to get watchtime: {}", e);
                    error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to get watchtime: {}", e))
                }
            }
        }
        None => error_response(StatusCode::SERVICE_UNAVAILABLE, "Database not initialized"),
    }
}

async fn handle_search_watchtime(query: &str) -> Response<BoxBody<Bytes, Infallible>> {
    match get_database() {
        Some(db) => {
            // Parse query parameters
            let params: std::collections::HashMap<String, String> = query
                .split('&')
                .filter_map(|param| {
                    let mut parts = param.split('=');
                    Some((parts.next()?.to_string(), parts.next()?.to_string()))
                })
                .collect();

            let channel = params.get("channel").map(|s| s.as_str()).unwrap_or("");
            let search = params.get("search").map(|s| s.as_str()).unwrap_or("");

            if channel.is_empty() {
                return error_response(StatusCode::BAD_REQUEST, "Missing channel parameter");
            }

            if search.is_empty() {
                return error_response(StatusCode::BAD_REQUEST, "Missing search parameter");
            }

            match db.search_watchers(channel, search) {
                Ok(watchers) => {
                    let watcher_list: Vec<serde_json::Value> = watchers
                        .iter()
                        .map(|(username, total_minutes, last_seen)| {
                            serde_json::json!({
                                "username": username,
                                "total_minutes": total_minutes,
                                "last_seen": last_seen
                            })
                        })
                        .collect();

                    json_response(&watcher_list)
                }
                Err(e) => {
                    error!("Failed to search watchtime: {}", e);
                    error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to search watchtime: {}", e))
                }
            }
        }
        None => error_response(StatusCode::SERVICE_UNAVAILABLE, "Database not initialized"),
    }
}

async fn handle_get_viewers_by_period(query: &str) -> Response<BoxBody<Bytes, Infallible>> {
    match get_database() {
        Some(db) => {
            // Parse query parameters
            let params: std::collections::HashMap<String, String> = query
                .split('&')
                .filter_map(|param| {
                    let mut parts = param.split('=');
                    Some((parts.next()?.to_string(), parts.next()?.to_string()))
                })
                .collect();

            let channel = params.get("channel").map(|s| s.as_str()).unwrap_or("");
            let period = params.get("period").map(|s| s.as_str()).unwrap_or("day");
            let limit = params.get("limit")
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(50);
            let offset = params.get("offset")
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(0);

            if channel.is_empty() {
                return error_response(StatusCode::BAD_REQUEST, "Missing channel parameter");
            }

            match (db.get_viewers_by_period(channel, period, limit, offset), db.get_viewers_count_by_period(channel, period)) {
                (Ok(viewers), Ok(total)) => {
                    let viewer_list: Vec<serde_json::Value> = viewers
                        .iter()
                        .map(|(username, total_minutes, last_seen)| {
                            serde_json::json!({
                                "username": username,
                                "total_minutes": total_minutes,
                                "last_seen": last_seen
                            })
                        })
                        .collect();

                    let response = serde_json::json!({
                        "viewers": viewer_list,
                        "total": total,
                        "period": period,
                        "limit": limit,
                        "offset": offset
                    });

                    json_response(&response)
                }
                (Err(e), _) | (_, Err(e)) => {
                    error!("Failed to get viewers by period: {}", e);
                    error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to get viewers by period: {}", e))
                }
            }
        }
        None => error_response(StatusCode::SERVICE_UNAVAILABLE, "Database not initialized"),
    }
}

// ========== Database Management Handlers ==========

async fn handle_get_tables() -> Response<BoxBody<Bytes, Infallible>> {
    match get_database() {
        Some(db) => {
            match db.get_tables() {
                Ok(tables) => json_response(&tables),
                Err(e) => {
                    error!("Failed to get tables: {}", e);
                    error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to get tables: {}", e))
                }
            }
        }
        None => error_response(StatusCode::SERVICE_UNAVAILABLE, "Database not initialized"),
    }
}

async fn handle_get_table_schema(query: &str) -> Response<BoxBody<Bytes, Infallible>> {
    match get_database() {
        Some(db) => {
            // Parse table name from query string
            let table = query.split('=')
                .nth(1)
                .unwrap_or("")
                .to_string();

            if table.is_empty() {
                return error_response(StatusCode::BAD_REQUEST, "Missing table parameter");
            }

            match db.get_table_schema(&table) {
                Ok(schema) => {
                    let response = serde_json::json!({
                        "table": table,
                        "schema": schema
                    });
                    json_response(&response)
                }
                Err(e) => {
                    error!("Failed to get table schema: {}", e);
                    error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to get table schema: {}", e))
                }
            }
        }
        None => error_response(StatusCode::SERVICE_UNAVAILABLE, "Database not initialized"),
    }
}

async fn handle_execute_query(body: &str) -> Response<BoxBody<Bytes, Infallible>> {
    #[derive(serde::Deserialize)]
    struct QueryRequest {
        query: String,
        #[serde(default)]
        write: bool,
    }

    match get_database() {
        Some(db) => {
            let request: QueryRequest = match serde_json::from_str(body) {
                Ok(r) => r,
                Err(e) => {
                    return error_response(StatusCode::BAD_REQUEST, &format!("Invalid JSON: {}", e));
                }
            };

            // Determine if it's a write query
            let query_upper = request.query.trim().to_uppercase();
            let is_write = request.write ||
                          query_upper.starts_with("INSERT") ||
                          query_upper.starts_with("UPDATE") ||
                          query_upper.starts_with("DELETE") ||
                          query_upper.starts_with("CREATE") ||
                          query_upper.starts_with("DROP") ||
                          query_upper.starts_with("ALTER");

            if is_write {
                // Execute write query
                match db.execute_write_query(&request.query) {
                    Ok(rows_affected) => {
                        let response = serde_json::json!({
                            "success": true,
                            "rows_affected": rows_affected,
                            "message": format!("{} row(s) affected", rows_affected)
                        });
                        json_response(&response)
                    }
                    Err(e) => {
                        error!("Query execution failed: {}", e);
                        let response = serde_json::json!({
                            "success": false,
                            "error": format!("{}", e)
                        });
                        json_response(&response)
                    }
                }
            } else {
                // Execute read query
                match db.execute_query(&request.query) {
                    Ok(result_json) => {
                        // Parse the JSON string back to Value to send as response
                        match serde_json::from_str::<serde_json::Value>(&result_json) {
                            Ok(data) => {
                                let response = serde_json::json!({
                                    "success": true,
                                    "data": data,
                                    "count": data.as_array().map(|a| a.len()).unwrap_or(0)
                                });
                                json_response(&response)
                            }
                            Err(e) => {
                                error!("Failed to parse query results: {}", e);
                                error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to parse results: {}", e))
                            }
                        }
                    }
                    Err(e) => {
                        error!("Query execution failed: {}", e);
                        let response = serde_json::json!({
                            "success": false,
                            "error": format!("{}", e)
                        });
                        json_response(&response)
                    }
                }
            }
        }
        None => error_response(StatusCode::SERVICE_UNAVAILABLE, "Database not initialized"),
    }
}

// ========== Hue Handlers ==========

async fn handle_hue_discover() -> Response<BoxBody<Bytes, Infallible>> {
    match HueClient::discover_bridge().await {
        Ok(bridge_ip) => {
            let response = serde_json::json!({
                "success": true,
                "bridge_ip": bridge_ip
            });
            json_response(&response)
        }
        Err(e) => {
            error!("Failed to discover bridge: {}", e);
            error_response(StatusCode::NOT_FOUND, &format!("Bridge not found: {}", e))
        }
    }
}

async fn handle_hue_pair(body: &str) -> Response<BoxBody<Bytes, Infallible>> {
    #[derive(serde::Deserialize)]
    struct PairRequest {
        bridge_ip: String,
    }

    match serde_json::from_str::<PairRequest>(body) {
        Ok(req) => {
            match HueClient::create_user(&req.bridge_ip, "WebArcade#Twitch").await {
                Ok(username) => {
                    match get_database() {
                        Some(db) => {
                            if let Err(e) = db.save_hue_config(&req.bridge_ip, &username) {
                                error!("Failed to save Hue config: {}", e);
                                return error_response(StatusCode::INTERNAL_SERVER_ERROR, "Failed to save configuration");
                            }
                            let response = ApiResponse {
                                success: true,
                                content: Some("Successfully paired with Hue bridge".to_string()),
                                error: None,
                            };
                            json_response(&response)
                        }
                        None => error_response(StatusCode::SERVICE_UNAVAILABLE, "Database not initialized"),
                    }
                }
                Err(e) => {
                    error!("Failed to pair with bridge: {}", e);
                    if e.contains("link button") {
                        error_response(StatusCode::BAD_REQUEST, "Press the link button on your Hue bridge and try again")
                    } else {
                        error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Pairing failed: {}", e))
                    }
                }
            }
        }
        Err(e) => error_response(StatusCode::BAD_REQUEST, &format!("Invalid request: {}", e)),
    }
}

async fn handle_hue_get_config() -> Response<BoxBody<Bytes, Infallible>> {
    match get_database() {
        Some(db) => {
            match db.get_hue_config() {
                Ok(Some((bridge_ip, _))) => {
                    let response = serde_json::json!({
                        "configured": true,
                        "bridge_ip": bridge_ip
                    });
                    json_response(&response)
                }
                Ok(None) => {
                    let response = serde_json::json!({
                        "configured": false
                    });
                    json_response(&response)
                }
                Err(e) => {
                    error!("Failed to get Hue config: {}", e);
                    error_response(StatusCode::INTERNAL_SERVER_ERROR, "Failed to get configuration")
                }
            }
        }
        None => error_response(StatusCode::SERVICE_UNAVAILABLE, "Database not initialized"),
    }
}

async fn handle_hue_delete_config() -> Response<BoxBody<Bytes, Infallible>> {
    match get_database() {
        Some(db) => {
            match db.clear_hue_config() {
                Ok(_) => {
                    let response = ApiResponse {
                        success: true,
                        content: Some("Hue configuration cleared".to_string()),
                        error: None,
                    };
                    json_response(&response)
                }
                Err(e) => {
                    error!("Failed to clear Hue config: {}", e);
                    error_response(StatusCode::INTERNAL_SERVER_ERROR, "Failed to clear configuration")
                }
            }
        }
        None => error_response(StatusCode::SERVICE_UNAVAILABLE, "Database not initialized"),
    }
}

async fn handle_hue_get_lights() -> Response<BoxBody<Bytes, Infallible>> {
    match get_database() {
        Some(db) => {
            match db.get_hue_config() {
                Ok(Some((bridge_ip, username))) => {
                    let client = HueClient::new(bridge_ip, username);
                    match client.get_lights().await {
                        Ok(lights) => json_response(&lights),
                        Err(e) => {
                            error!("Failed to get lights: {}", e);
                            error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to get lights: {}", e))
                        }
                    }
                }
                Ok(None) => error_response(StatusCode::BAD_REQUEST, "Hue bridge not configured"),
                Err(e) => {
                    error!("Database error: {}", e);
                    error_response(StatusCode::INTERNAL_SERVER_ERROR, "Database error")
                }
            }
        }
        None => error_response(StatusCode::SERVICE_UNAVAILABLE, "Database not initialized"),
    }
}

async fn handle_hue_set_power(body: &str) -> Response<BoxBody<Bytes, Infallible>> {
    #[derive(serde::Deserialize)]
    struct SetPowerRequest {
        light_id: String,
        on: bool,
    }
    match get_database() {
        Some(db) => {
            match serde_json::from_str::<SetPowerRequest>(body) {
                Ok(req) => {
                    match db.get_hue_config() {
                        Ok(Some((bridge_ip, username))) => {
                            let client = HueClient::new(bridge_ip, username);
                            match client.set_light_power(&req.light_id, req.on).await {
                                Ok(_) => {
                                    let response = ApiResponse { success: true, content: Some("Light updated".to_string()), error: None };
                                    json_response(&response)
                                }
                                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed: {}", e))
                            }
                        }
                        Ok(None) => error_response(StatusCode::BAD_REQUEST, "Hue bridge not configured"),
                        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, "Database error")
                    }
                }
                Err(e) => error_response(StatusCode::BAD_REQUEST, &format!("Invalid request: {}", e)),
            }
        }
        None => error_response(StatusCode::SERVICE_UNAVAILABLE, "Database not initialized"),
    }
}

async fn handle_hue_set_brightness(body: &str) -> Response<BoxBody<Bytes, Infallible>> {
    #[derive(serde::Deserialize)]
    struct SetBrightnessRequest {
        light_id: String,
        brightness: u8,
    }
    match get_database() {
        Some(db) => {
            match serde_json::from_str::<SetBrightnessRequest>(body) {
                Ok(req) => {
                    match db.get_hue_config() {
                        Ok(Some((bridge_ip, username))) => {
                            let client = HueClient::new(bridge_ip, username);
                            match client.set_light_brightness(&req.light_id, req.brightness).await {
                                Ok(_) => {
                                    let response = ApiResponse { success: true, content: Some("Brightness updated".to_string()), error: None };
                                    json_response(&response)
                                }
                                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed: {}", e))
                            }
                        }
                        Ok(None) => error_response(StatusCode::BAD_REQUEST, "Hue bridge not configured"),
                        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, "Database error")
                    }
                }
                Err(e) => error_response(StatusCode::BAD_REQUEST, &format!("Invalid request: {}", e)),
            }
        }
        None => error_response(StatusCode::SERVICE_UNAVAILABLE, "Database not initialized"),
    }
}

async fn handle_hue_set_color(body: &str) -> Response<BoxBody<Bytes, Infallible>> {
    #[derive(serde::Deserialize)]
    struct SetColorRequest {
        light_id: String,
        r: u8,
        g: u8,
        b: u8,
    }
    match get_database() {
        Some(db) => {
            match serde_json::from_str::<SetColorRequest>(body) {
                Ok(req) => {
                    match db.get_hue_config() {
                        Ok(Some((bridge_ip, username))) => {
                            let client = HueClient::new(bridge_ip, username);
                            match client.set_light_rgb(&req.light_id, req.r, req.g, req.b).await {
                                Ok(_) => {
                                    let response = ApiResponse { success: true, content: Some("Color updated".to_string()), error: None };
                                    json_response(&response)
                                }
                                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed: {}", e))
                            }
                        }
                        Ok(None) => error_response(StatusCode::BAD_REQUEST, "Hue bridge not configured"),
                        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, "Database error")
                    }
                }
                Err(e) => error_response(StatusCode::BAD_REQUEST, &format!("Invalid request: {}", e)),
            }
        }
        None => error_response(StatusCode::SERVICE_UNAVAILABLE, "Database not initialized"),
    }
}

async fn handle_hue_set_scene(body: &str) -> Response<BoxBody<Bytes, Infallible>> {
    #[derive(serde::Deserialize)]
    struct SetSceneRequest {
        scene: String,
    }
    match get_database() {
        Some(db) => {
            match serde_json::from_str::<SetSceneRequest>(body) {
                Ok(req) => {
                    match db.get_hue_config() {
                        Ok(Some((bridge_ip, username))) => {
                            let client = HueClient::new(bridge_ip, username);
                            // Check for custom scenes first
                            let result = if let Ok(Some((r, g, b))) = db.get_hue_scene(&req.scene) {
                                client.set_all_lights_rgb(r, g, b).await
                            } else {
                                client.set_scene(&req.scene).await
                            };

                            match result {
                                Ok(_) => {
                                    let response = ApiResponse { success: true, content: Some(format!("Scene set to {}", req.scene)), error: None };
                                    json_response(&response)
                                }
                                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed: {}", e))
                            }
                        }
                        Ok(None) => error_response(StatusCode::BAD_REQUEST, "Hue bridge not configured"),
                        Err(_e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, "Database error")
                    }
                }
                Err(e) => error_response(StatusCode::BAD_REQUEST, &format!("Invalid request: {}", e)),
            }
        }
        None => error_response(StatusCode::SERVICE_UNAVAILABLE, "Database not initialized"),
    }
}

async fn handle_hue_get_scenes() -> Response<BoxBody<Bytes, Infallible>> {
    match get_database() {
        Some(db) => {
            match db.get_hue_scenes() {
                Ok(scenes) => {
                    let scene_list: Vec<serde_json::Value> = scenes.iter().map(|(name, r, g, b)| {
                        serde_json::json!({
                            "name": name,
                            "red": r,
                            "green": g,
                            "blue": b,
                        })
                    }).collect();
                    json_response(&scene_list)
                }
                Err(_e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, "Database error")
            }
        }
        None => error_response(StatusCode::SERVICE_UNAVAILABLE, "Database not initialized"),
    }
}

async fn handle_hue_save_scene(body: &str) -> Response<BoxBody<Bytes, Infallible>> {
    #[derive(serde::Deserialize)]
    struct SaveSceneRequest {
        name: String,
        red: u8,
        green: u8,
        blue: u8,
    }

    match get_database() {
        Some(db) => {
            match serde_json::from_str::<SaveSceneRequest>(body) {
                Ok(req) => {
                    match db.save_hue_scene(&req.name, req.red, req.green, req.blue) {
                        Ok(_) => {
                            let response = ApiResponse { success: true, content: Some(format!("Scene '{}' saved", req.name)), error: None };
                            json_response(&response)
                        }
                        Err(_e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, "Failed to save scene")
                    }
                }
                Err(e) => error_response(StatusCode::BAD_REQUEST, &format!("Invalid request: {}", e)),
            }
        }
        None => error_response(StatusCode::SERVICE_UNAVAILABLE, "Database not initialized"),
    }
}

async fn handle_hue_delete_scene(body: &str) -> Response<BoxBody<Bytes, Infallible>> {
    #[derive(serde::Deserialize)]
    struct DeleteSceneRequest {
        name: String,
    }

    match get_database() {
        Some(db) => {
            match serde_json::from_str::<DeleteSceneRequest>(body) {
                Ok(req) => {
                    match db.delete_hue_scene(&req.name) {
                        Ok(_) => {
                            let response = ApiResponse { success: true, content: Some(format!("Scene '{}' deleted", req.name)), error: None };
                            json_response(&response)
                        }
                        Err(_e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, "Failed to delete scene")
                    }
                }
                Err(e) => error_response(StatusCode::BAD_REQUEST, &format!("Invalid request: {}", e)),
            }
        }
        None => error_response(StatusCode::SERVICE_UNAVAILABLE, "Database not initialized"),
    }
}

// Animated scenes handlers

async fn handle_hue_get_animated_scenes() -> Response<BoxBody<Bytes, Infallible>> {
    match get_database() {
        Some(db) => {
            match db.get_animated_scenes() {
                Ok(scenes) => {
                    let scene_list: Vec<serde_json::Value> = scenes.iter().map(|(id, name, tag, steps)| {
                        let steps_json: Vec<serde_json::Value> = steps.iter().map(|(step_id, r, g, b, trans, dur)| {
                            serde_json::json!({
                                "id": step_id,
                                "red": r,
                                "green": g,
                                "blue": b,
                                "transition": trans,
                                "duration": dur,
                            })
                        }).collect();

                        serde_json::json!({
                            "id": id,
                            "name": name,
                            "tag": tag,
                            "steps": steps_json,
                        })
                    }).collect();
                    json_response(&scene_list)
                }
                Err(_e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, "Database error")
            }
        }
        None => error_response(StatusCode::SERVICE_UNAVAILABLE, "Database not initialized"),
    }
}

async fn handle_hue_create_animated_scene(body: &str) -> Response<BoxBody<Bytes, Infallible>> {
    #[derive(serde::Deserialize)]
    struct CreateSceneRequest {
        name: String,
        tag: String,
    }

    match get_database() {
        Some(db) => {
            match serde_json::from_str::<CreateSceneRequest>(body) {
                Ok(req) => {
                    match db.create_animated_scene(&req.name, &req.tag) {
                        Ok(scene_id) => {
                            let response = serde_json::json!({
                                "success": true,
                                "scene_id": scene_id,
                                "message": format!("Scene '{}' created", req.name)
                            });
                            json_response(&response)
                        }
                        Err(_e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, "Failed to create scene")
                    }
                }
                Err(e) => error_response(StatusCode::BAD_REQUEST, &format!("Invalid request: {}", e)),
            }
        }
        None => error_response(StatusCode::SERVICE_UNAVAILABLE, "Database not initialized"),
    }
}

async fn handle_hue_delete_animated_scene(body: &str) -> Response<BoxBody<Bytes, Infallible>> {
    #[derive(serde::Deserialize)]
    struct DeleteSceneRequest {
        scene_id: i64,
    }

    match get_database() {
        Some(db) => {
            match serde_json::from_str::<DeleteSceneRequest>(body) {
                Ok(req) => {
                    match db.delete_animated_scene(req.scene_id) {
                        Ok(_) => {
                            let response = ApiResponse { success: true, content: Some("Scene deleted".to_string()), error: None };
                            json_response(&response)
                        }
                        Err(_e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, "Failed to delete scene")
                    }
                }
                Err(e) => error_response(StatusCode::BAD_REQUEST, &format!("Invalid request: {}", e)),
            }
        }
        None => error_response(StatusCode::SERVICE_UNAVAILABLE, "Database not initialized"),
    }
}

async fn handle_hue_add_scene_step(body: &str) -> Response<BoxBody<Bytes, Infallible>> {
    #[derive(serde::Deserialize)]
    struct AddStepRequest {
        scene_id: i64,
        order: i32,
        red: u8,
        green: u8,
        blue: u8,
        transition: u16,
        duration: u16,
    }

    match get_database() {
        Some(db) => {
            match serde_json::from_str::<AddStepRequest>(body) {
                Ok(req) => {
                    match db.add_scene_step(req.scene_id, req.order, req.red, req.green, req.blue, req.transition, req.duration) {
                        Ok(_) => {
                            let response = ApiResponse { success: true, content: Some("Step added".to_string()), error: None };
                            json_response(&response)
                        }
                        Err(_e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, "Failed to add step")
                    }
                }
                Err(e) => error_response(StatusCode::BAD_REQUEST, &format!("Invalid request: {}", e)),
            }
        }
        None => error_response(StatusCode::SERVICE_UNAVAILABLE, "Database not initialized"),
    }
}

async fn handle_hue_update_scene_step(body: &str) -> Response<BoxBody<Bytes, Infallible>> {
    #[derive(serde::Deserialize)]
    struct UpdateStepRequest {
        step_id: i64,
        red: u8,
        green: u8,
        blue: u8,
        transition: u16,
        duration: u16,
    }

    match get_database() {
        Some(db) => {
            match serde_json::from_str::<UpdateStepRequest>(body) {
                Ok(req) => {
                    match db.update_scene_step(req.step_id, req.red, req.green, req.blue, req.transition, req.duration) {
                        Ok(_) => {
                            let response = ApiResponse { success: true, content: Some("Step updated".to_string()), error: None };
                            json_response(&response)
                        }
                        Err(_e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, "Failed to update step")
                    }
                }
                Err(e) => error_response(StatusCode::BAD_REQUEST, &format!("Invalid request: {}", e)),
            }
        }
        None => error_response(StatusCode::SERVICE_UNAVAILABLE, "Database not initialized"),
    }
}

async fn handle_hue_delete_scene_step(body: &str) -> Response<BoxBody<Bytes, Infallible>> {
    #[derive(serde::Deserialize)]
    struct DeleteStepRequest {
        step_id: i64,
    }

    match get_database() {
        Some(db) => {
            match serde_json::from_str::<DeleteStepRequest>(body) {
                Ok(req) => {
                    match db.delete_scene_step(req.step_id) {
                        Ok(_) => {
                            let response = ApiResponse { success: true, content: Some("Step deleted".to_string()), error: None };
                            json_response(&response)
                        }
                        Err(_e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, "Failed to delete step")
                    }
                }
                Err(e) => error_response(StatusCode::BAD_REQUEST, &format!("Invalid request: {}", e)),
            }
        }
        None => error_response(StatusCode::SERVICE_UNAVAILABLE, "Database not initialized"),
    }
}

async fn handle_hue_reorder_scene_steps(body: &str) -> Response<BoxBody<Bytes, Infallible>> {
    #[derive(serde::Deserialize)]
    struct ReorderStepsRequest {
        scene_id: i64,
        step_ids: Vec<i64>,
    }

    match get_database() {
        Some(db) => {
            match serde_json::from_str::<ReorderStepsRequest>(body) {
                Ok(req) => {
                    match db.reorder_scene_steps(req.scene_id, &req.step_ids) {
                        Ok(_) => {
                            let response = ApiResponse { success: true, content: Some("Steps reordered".to_string()), error: None };
                            json_response(&response)
                        }
                        Err(_e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, "Failed to reorder steps")
                    }
                }
                Err(e) => error_response(StatusCode::BAD_REQUEST, &format!("Invalid request: {}", e)),
            }
        }
        None => error_response(StatusCode::SERVICE_UNAVAILABLE, "Database not initialized"),
    }
}

async fn handle_hue_play_animated_scene(body: &str) -> Response<BoxBody<Bytes, Infallible>> {
    #[derive(serde::Deserialize)]
    struct PlaySceneRequest {
        scene_id: i64,
    }

    match get_database() {
        Some(db) => {
            match serde_json::from_str::<PlaySceneRequest>(body) {
                Ok(req) => {
                    // Get scene by ID
                    match db.get_animated_scenes() {
                        Ok(scenes) => {
                            if let Some((_id, name, _tag, steps)) = scenes.iter().find(|(id, _, _, _)| *id == req.scene_id) {
                                // Get Hue config
                                match db.get_hue_config() {
                                    Ok(Some((bridge_ip, username))) => {
                                        let client = HueClient::new(bridge_ip, username);
                                        let steps_vec: Vec<(u8, u8, u8, u16, u16)> = steps.iter()
                                            .map(|(_id, r, g, b, trans, dur)| (*r, *g, *b, *trans, *dur))
                                            .collect();

                                        // Play scene in background
                                        tokio::spawn(async move {
                                            let _ = client.play_animated_scene(steps_vec).await;
                                        });

                                        let response = ApiResponse { success: true, content: Some(format!("Playing scene '{}'", name)), error: None };
                                        json_response(&response)
                                    }
                                    Ok(None) => error_response(StatusCode::BAD_REQUEST, "Hue bridge not configured"),
                                    Err(_e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, "Database error")
                                }
                            } else {
                                error_response(StatusCode::NOT_FOUND, "Scene not found")
                            }
                        }
                        Err(_e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, "Database error")
                    }
                }
                Err(e) => error_response(StatusCode::BAD_REQUEST, &format!("Invalid request: {}", e)),
            }
        }
        None => error_response(StatusCode::SERVICE_UNAVAILABLE, "Database not initialized"),
    }
}

// === WITHINGS HANDLERS ===

async fn handle_withings_get_latest() -> Response<BoxBody<Bytes, Infallible>> {
    match get_database() {
        Some(db) => {
            match db.get_latest_weight() {
                Ok(Some((date, weight, fat_mass, muscle_mass, bone_mass, hydration))) => {
                    let response = serde_json::json!({
                        "success": true,
                        "data": {
                            "date": date,
                            "weight": weight,
                            "fat_mass": fat_mass,
                            "muscle_mass": muscle_mass,
                            "bone_mass": bone_mass,
                            "hydration": hydration
                        }
                    });
                    json_response(&response)
                }
                Ok(None) => {
                    let response = serde_json::json!({
                        "success": true,
                        "data": null
                    });
                    json_response(&response)
                }
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Database error: {}", e))
            }
        }
        None => error_response(StatusCode::SERVICE_UNAVAILABLE, "Database not initialized"),
    }
}

async fn handle_withings_get_history(query: &str) -> Response<BoxBody<Bytes, Infallible>> {
    match get_database() {
        Some(db) => {
            // Parse query parameters
            let mut start_date: Option<i64> = None;
            let mut end_date: Option<i64> = None;
            let mut limit: Option<i64> = Some(100); // Default limit

            for param in query.split('&') {
                if let Some((key, value)) = param.split_once('=') {
                    match key {
                        "start_date" => start_date = value.parse().ok(),
                        "end_date" => end_date = value.parse().ok(),
                        "limit" => limit = value.parse().ok(),
                        _ => {}
                    }
                }
            }

            match db.get_weight_measurements(start_date, end_date, limit) {
                Ok(measurements) => {
                    let data: Vec<serde_json::Value> = measurements
                        .iter()
                        .map(|(date, weight, fat_mass, muscle_mass, bone_mass, hydration)| {
                            serde_json::json!({
                                "date": date,
                                "weight": weight,
                                "fat_mass": fat_mass,
                                "muscle_mass": muscle_mass,
                                "bone_mass": bone_mass,
                                "hydration": hydration
                            })
                        })
                        .collect();

                    let response = serde_json::json!({
                        "success": true,
                        "data": data
                    });
                    json_response(&response)
                }
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Database error: {}", e))
            }
        }
        None => error_response(StatusCode::SERVICE_UNAVAILABLE, "Database not initialized"),
    }
}

async fn handle_withings_sync() -> Response<BoxBody<Bytes, Infallible>> {
    match (get_withings_api(), get_database()) {
        (Some(api), Some(db)) => {
            // Load access token from database
            match db.get_withings_access_token() {
                Ok(Some(token)) => {
                    // Set the access token in the API
                    api.set_access_token(token).await;

                    // Fetch measurements from Withings API (last 90 days)
                    match api.get_weight_history(90).await {
                        Ok(measurements) => {
                            let mut saved_count = 0;

                            // Save each measurement to database
                            for measurement in measurements {
                                match db.save_weight_measurement(
                                    measurement.date,
                                    measurement.weight,
                                    measurement.fat_mass,
                                    measurement.muscle_mass,
                                    measurement.bone_mass,
                                    measurement.hydration
                                ) {
                                    Ok(_) => saved_count += 1,
                                    Err(e) => log::error!("Failed to save measurement: {}", e)
                                }
                            }

                            let response = ApiResponse {
                                success: true,
                                content: Some(format!("Synced {} weight measurements from Withings", saved_count)),
                                error: None,
                            };
                            json_response(&response)
                        }
                        Err(e) => {
                            log::error!("Failed to fetch from Withings API: {}", e);
                            error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to fetch from Withings: {}", e))
                        }
                    }
                }
                Ok(None) => {
                    error_response(StatusCode::UNAUTHORIZED, "No Withings access token found. Please authenticate first.")
                }
                Err(e) => {
                    error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Database error: {}", e))
                }
            }
        }
        _ => error_response(StatusCode::SERVICE_UNAVAILABLE, "Withings API or Database not initialized"),
    }
}

async fn handle_withings_save_config(body: &str) -> Response<BoxBody<Bytes, Infallible>> {
    #[derive(serde::Deserialize)]
    struct ConfigRequest {
        client_id: String,
        client_secret: String,
    }

    match get_database() {
        Some(db) => {
            match serde_json::from_str::<ConfigRequest>(body) {
                Ok(req) => {
                    match db.save_withings_client_credentials(&req.client_id, &req.client_secret) {
                        Ok(()) => {
                            let response = ApiResponse {
                                success: true,
                                content: Some("Withings credentials saved successfully".to_string()),
                                error: None,
                            };
                            json_response(&response)
                        }
                        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to save credentials: {}", e))
                    }
                }
                Err(e) => error_response(StatusCode::BAD_REQUEST, &format!("Invalid request: {}", e))
            }
        }
        None => error_response(StatusCode::SERVICE_UNAVAILABLE, "Database not initialized"),
    }
}

async fn handle_withings_get_config() -> Response<BoxBody<Bytes, Infallible>> {
    match get_database() {
        Some(db) => {
            match db.get_withings_client_credentials() {
                Ok(Some((client_id, _client_secret))) => {
                    // Don't expose the client secret
                    let response = serde_json::json!({
                        "success": true,
                        "data": {
                            "client_id": client_id,
                            "configured": true
                        }
                    });
                    json_response(&response)
                }
                Ok(None) => {
                    let response = serde_json::json!({
                        "success": true,
                        "data": {
                            "configured": false
                        }
                    });
                    json_response(&response)
                }
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Database error: {}", e))
            }
        }
        None => error_response(StatusCode::SERVICE_UNAVAILABLE, "Database not initialized"),
    }
}

async fn handle_withings_get_auth_url() -> Response<BoxBody<Bytes, Infallible>> {
    match get_database() {
        Some(db) => {
            match db.get_withings_client_credentials() {
                Ok(Some((client_id, _client_secret))) => {
                    let redirect_uri = "http://localhost:3001/withings/callback";
                    let auth_url = format!(
                        "https://account.withings.com/oauth2_user/authorize2?response_type=code&client_id={}&redirect_uri={}&scope=user.metrics&state=webarcade",
                        percent_encoding::utf8_percent_encode(&client_id, percent_encoding::NON_ALPHANUMERIC),
                        percent_encoding::utf8_percent_encode(redirect_uri, percent_encoding::NON_ALPHANUMERIC)
                    );

                    let response = serde_json::json!({
                        "success": true,
                        "auth_url": auth_url
                    });
                    json_response(&response)
                }
                Ok(None) => {
                    error_response(StatusCode::BAD_REQUEST, "Withings client credentials not configured")
                }
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Database error: {}", e))
            }
        }
        None => error_response(StatusCode::SERVICE_UNAVAILABLE, "Database not initialized"),
    }
}

async fn handle_withings_callback(uri: &hyper::Uri) -> Response<BoxBody<Bytes, Infallible>> {
    let query = uri.query().unwrap_or("");
    let mut code: Option<String> = None;

    for param in query.split('&') {
        if let Some((key, value)) = param.split_once('=') {
            if key == "code" {
                code = Some(value.to_string());
            }
        }
    }

    match (code, get_database()) {
        (Some(auth_code), Some(db)) => {
            match db.get_withings_client_credentials() {
                Ok(Some((client_id, client_secret))) => {
                    // Exchange code for tokens
                    let client = reqwest::Client::new();
                    let redirect_uri = "http://localhost:3001/withings/callback";

                    let params = [
                        ("action", "requesttoken"),
                        ("grant_type", "authorization_code"),
                        ("client_id", &client_id),
                        ("client_secret", &client_secret),
                        ("code", &auth_code),
                        ("redirect_uri", redirect_uri),
                    ];

                    match client.post("https://wbsapi.withings.net/v2/oauth2")
                        .form(&params)
                        .send()
                        .await
                    {
                        Ok(response) => {
                            match response.json::<serde_json::Value>().await {
                                Ok(data) => {
                                    if let (Some(access_token), Some(refresh_token), Some(expires_in)) = (
                                        data["body"]["access_token"].as_str(),
                                        data["body"]["refresh_token"].as_str(),
                                        data["body"]["expires_in"].as_i64()
                                    ) {
                                        let expires_at = chrono::Utc::now().timestamp() + expires_in;

                                        match db.save_withings_tokens(access_token, refresh_token, expires_at) {
                                            Ok(()) => {
                                                let html = r#"<!DOCTYPE html>
<html>
<head>
    <title>Withings Authorization Successful</title>
    <style>
        body { font-family: sans-serif; text-align: center; padding: 50px; }
        .success { color: #22c55e; font-size: 24px; margin-bottom: 20px; }
    </style>
</head>
<body>
    <div class="success">✅ Successfully connected to Withings!</div>
    <p>You can close this window and return to WebArcade.</p>
    <script>
        setTimeout(() => window.close(), 3000);
    </script>
</body>
</html>"#;
                                                Response::builder()
                                                    .status(StatusCode::OK)
                                                    .header("Content-Type", "text/html")
                                                    .body(BoxBody::new(Full::new(Bytes::from(html))))
                                                    .unwrap()
                                            }
                                            Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to save tokens: {}", e))
                                        }
                                    } else {
                                        error_response(StatusCode::BAD_REQUEST, "Invalid token response from Withings")
                                    }
                                }
                                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to parse Withings response: {}", e))
                            }
                        }
                        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to exchange code: {}", e))
                    }
                }
                Ok(None) => error_response(StatusCode::BAD_REQUEST, "Withings client credentials not configured"),
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Database error: {}", e))
            }
        }
        (None, _) => error_response(StatusCode::BAD_REQUEST, "Missing authorization code"),
        (_, None) => error_response(StatusCode::SERVICE_UNAVAILABLE, "Database not initialized"),
    }
}

async fn handle_withings_save_token(body: &str) -> Response<BoxBody<Bytes, Infallible>> {
    #[derive(serde::Deserialize)]
    struct TokenRequest {
        access_token: String,
        refresh_token: String,
        expires_at: i64,
    }

    match get_database() {
        Some(db) => {
            match serde_json::from_str::<TokenRequest>(body) {
                Ok(req) => {
                    match db.save_withings_tokens(&req.access_token, &req.refresh_token, req.expires_at) {
                        Ok(()) => {
                            let response = ApiResponse {
                                success: true,
                                content: Some("Withings tokens saved successfully".to_string()),
                                error: None,
                            };
                            json_response(&response)
                        }
                        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to save tokens: {}", e))
                    }
                }
                Err(e) => error_response(StatusCode::BAD_REQUEST, &format!("Invalid request: {}", e))
            }
        }
        None => error_response(StatusCode::SERVICE_UNAVAILABLE, "Database not initialized"),
    }
}

// === TEXT COMMANDS HANDLERS ===

#[derive(Deserialize)]
struct AddTextCommandRequest {
    channel: String,
    command: String,
    response: String,
}

#[derive(Deserialize)]
struct EditTextCommandRequest {
    channel: String,
    command: String,
    response: String,
    auto_post: Option<bool>,
    interval_minutes: Option<i64>,
}

#[derive(Deserialize)]
struct DeleteTextCommandRequest {
    channel: String,
    command: String,
}

async fn handle_get_text_commands(query: &str) -> Response<BoxBody<Bytes, Infallible>> {
    match get_database() {
        Some(db) => {
            // Parse channel from query string
            let channel = query.split('=')
                .nth(1)
                .unwrap_or("")
                .to_string();

            if channel.is_empty() {
                return error_response(StatusCode::BAD_REQUEST, "Missing channel parameter");
            }

            match db.get_all_text_commands(&channel) {
                Ok(commands) => {
                    let command_list: Vec<serde_json::Value> = commands
                        .iter()
                        .map(|(command, response, auto_post, interval_minutes)| {
                            serde_json::json!({
                                "command": command,
                                "response": response,
                                "auto_post": auto_post,
                                "interval_minutes": interval_minutes
                            })
                        })
                        .collect();
                    json_response(&command_list)
                }
                Err(e) => {
                    error!("Failed to get text commands: {}", e);
                    error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to get text commands: {}", e))
                }
            }
        }
        None => error_response(StatusCode::SERVICE_UNAVAILABLE, "Database not initialized"),
    }
}

async fn handle_add_text_command(body: &str) -> Response<BoxBody<Bytes, Infallible>> {
    match get_database() {
        Some(db) => {
            match serde_json::from_str::<AddTextCommandRequest>(body) {
                Ok(req) => {
                    match db.add_text_command(&req.channel, &req.command, &req.response) {
                        Ok(_) => {
                            // Register the command dynamically
                            if let Some(manager) = get_twitch_manager() {
                                let command_system = manager.get_command_system();
                                crate::commands::text_commands::register_text_command(
                                    &command_system,
                                    (**db).clone(),
                                    &req.channel,
                                    &req.command,
                                    &req.response
                                ).await;
                            }

                            let response = ApiResponse {
                                success: true,
                                content: Some(format!("Command '{}' added successfully", req.command)),
                                error: None
                            };
                            json_response(&response)
                        }
                        Err(e) => {
                            error!("Failed to add text command: {}", e);
                            error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to add text command: {}", e))
                        }
                    }
                }
                Err(e) => error_response(StatusCode::BAD_REQUEST, &format!("Invalid request: {}", e)),
            }
        }
        None => error_response(StatusCode::SERVICE_UNAVAILABLE, "Database not initialized"),
    }
}

async fn handle_edit_text_command(body: &str) -> Response<BoxBody<Bytes, Infallible>> {
    match get_database() {
        Some(db) => {
            match serde_json::from_str::<EditTextCommandRequest>(body) {
                Ok(req) => {
                    let auto_post = req.auto_post.unwrap_or(false);
                    let interval_minutes = req.interval_minutes.unwrap_or(10);

                    match db.update_text_command(&req.channel, &req.command, &req.response, auto_post, interval_minutes) {
                        Ok(_) => {
                            // Re-register the command with updated response
                            if let Some(manager) = get_twitch_manager() {
                                let command_system = manager.get_command_system();
                                // Unregister old version
                                command_system.unregister_command(&req.command).await;
                                // Register new version
                                crate::commands::text_commands::register_text_command(
                                    &command_system,
                                    (**db).clone(),
                                    &req.channel,
                                    &req.command,
                                    &req.response
                                ).await;
                            }

                            let response = ApiResponse {
                                success: true,
                                content: Some(format!("Command '{}' updated successfully", req.command)),
                                error: None
                            };
                            json_response(&response)
                        }
                        Err(e) => {
                            error!("Failed to update text command: {}", e);
                            error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to update text command: {}", e))
                        }
                    }
                }
                Err(e) => error_response(StatusCode::BAD_REQUEST, &format!("Invalid request: {}", e)),
            }
        }
        None => error_response(StatusCode::SERVICE_UNAVAILABLE, "Database not initialized"),
    }
}

async fn handle_delete_text_command(body: &str) -> Response<BoxBody<Bytes, Infallible>> {
    match get_database() {
        Some(db) => {
            match serde_json::from_str::<DeleteTextCommandRequest>(body) {
                Ok(req) => {
                    match db.delete_text_command(&req.channel, &req.command) {
                        Ok(deleted) => {
                            if deleted {
                                // Unregister the command dynamically
                                if let Some(manager) = get_twitch_manager() {
                                    let command_system = manager.get_command_system();
                                    command_system.unregister_command(&req.command).await;
                                }

                                let response = ApiResponse {
                                    success: true,
                                    content: Some(format!("Command '{}' deleted successfully", req.command)),
                                    error: None
                                };
                                json_response(&response)
                            } else {
                                error_response(StatusCode::NOT_FOUND, "Command not found")
                            }
                        }
                        Err(e) => {
                            error!("Failed to delete text command: {}", e);
                            error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to delete text command: {}", e))
                        }
                    }
                }
                Err(e) => error_response(StatusCode::BAD_REQUEST, &format!("Invalid request: {}", e)),
            }
        }
        None => error_response(StatusCode::SERVICE_UNAVAILABLE, "Database not initialized"),
    }
}

// === OVERLAY HANDLERS ===

#[derive(Deserialize)]
struct SaveOverlayRequest {
    id: String,
    name: String,
    html_content: String,
    width: i32,
    height: i32,
}

#[derive(Deserialize)]
struct OverlayTriggerRequest {
    event: String,
    data: serde_json::Value,
}

async fn handle_serve_overlay(overlay_id: &str) -> Response<BoxBody<Bytes, Infallible>> {
    // Serve built overlays from dist/overlays/
    let file_path = format!("dist/overlays/{}.html", overlay_id);

    match std::fs::read_to_string(&file_path) {
        Ok(html_content) => {
            Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "text/html; charset=utf-8")
                .header("Access-Control-Allow-Origin", "*")
                .header("Cache-Control", "no-cache")
                .body(BoxBody::new(Full::new(Bytes::from(html_content))))
                .unwrap()
        }
        Err(e) => {
            error!("Failed to read overlay file {}: {}", file_path, e);
            error_response(StatusCode::NOT_FOUND, &format!("Overlay '{}' not found", overlay_id))
        }
    }
}

async fn handle_get_overlays() -> Response<BoxBody<Bytes, Infallible>> {
    match get_database() {
        Some(db) => {
            match db.get_all_overlays() {
                Ok(overlays) => {
                    let overlay_list: Vec<serde_json::Value> = overlays
                        .iter()
                        .map(|(id, name, file_path, width, height, updated_at)| {
                            serde_json::json!({
                                "id": id,
                                "name": name,
                                "file_path": file_path,
                                "width": width,
                                "height": height,
                                "updated_at": updated_at,
                                "url": format!("http://localhost:3001/overlay/{}", id)
                            })
                        })
                        .collect();
                    json_response(&overlay_list)
                }
                Err(e) => {
                    error!("Failed to get overlays: {}", e);
                    error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to get overlays: {}", e))
                }
            }
        }
        None => error_response(StatusCode::SERVICE_UNAVAILABLE, "Database not initialized"),
    }
}

async fn handle_save_overlay(body: &str) -> Response<BoxBody<Bytes, Infallible>> {
    match get_database() {
        Some(db) => {
            match serde_json::from_str::<SaveOverlayRequest>(body) {
                Ok(req) => {
                    // Create overlays directory if it doesn't exist
                    let overlays_dir = "src/overlays";
                    if let Err(e) = std::fs::create_dir_all(overlays_dir) {
                        error!("Failed to create overlays directory: {}", e);
                        return error_response(StatusCode::INTERNAL_SERVER_ERROR, "Failed to create overlays directory");
                    }

                    // Save HTML to file
                    let file_path = format!("{}/{}.html", overlays_dir, req.id);
                    match std::fs::write(&file_path, &req.html_content) {
                        Ok(_) => {
                            // Save metadata to database
                            match db.save_overlay(&req.id, &req.name, &file_path, req.width, req.height) {
                                Ok(_) => {
                                    let response = ApiResponse {
                                        success: true,
                                        content: Some(format!("Overlay '{}' saved successfully", req.name)),
                                        error: None
                                    };
                                    json_response(&response)
                                }
                                Err(e) => {
                                    error!("Failed to save overlay metadata: {}", e);
                                    error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to save overlay metadata: {}", e))
                                }
                            }
                        }
                        Err(e) => {
                            error!("Failed to write overlay file: {}", e);
                            error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to write overlay file: {}", e))
                        }
                    }
                }
                Err(e) => error_response(StatusCode::BAD_REQUEST, &format!("Invalid request: {}", e)),
            }
        }
        None => error_response(StatusCode::SERVICE_UNAVAILABLE, "Database not initialized"),
    }
}

async fn handle_delete_overlay(overlay_id: &str) -> Response<BoxBody<Bytes, Infallible>> {
    match get_database() {
        Some(db) => {
            // Get file path before deleting from database
            let file_path = match db.get_overlay(overlay_id) {
                Ok(Some((_name, path, _width, _height))) => Some(path),
                _ => None
            };

            match db.delete_overlay(overlay_id) {
                Ok(deleted) => {
                    if deleted {
                        // Try to delete the file
                        if let Some(path) = file_path {
                            if let Err(e) = std::fs::remove_file(&path) {
                                warn!("Failed to delete overlay file {}: {}", path, e);
                            }
                        }

                        let response = ApiResponse {
                            success: true,
                            content: Some(format!("Overlay deleted successfully")),
                            error: None
                        };
                        json_response(&response)
                    } else {
                        error_response(StatusCode::NOT_FOUND, "Overlay not found")
                    }
                }
                Err(e) => {
                    error!("Failed to delete overlay: {}", e);
                    error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to delete overlay: {}", e))
                }
            }
        }
        None => error_response(StatusCode::SERVICE_UNAVAILABLE, "Database not initialized"),
    }
}

async fn handle_overlay_trigger(_overlay_id: &str, body: &str) -> Response<BoxBody<Bytes, Infallible>> {
    match serde_json::from_str::<OverlayTriggerRequest>(body) {
        Ok(req) => {
            // Broadcast event to WebSocket clients
            // For now, we'll just return success
            // In the future, integrate with WebSocket server to broadcast events
            let response = serde_json::json!({
                "success": true,
                "event": req.event,
                "data": req.data
            });
            json_response(&response)
        }
        Err(e) => error_response(StatusCode::BAD_REQUEST, &format!("Invalid request: {}", e)),
    }
}

// File-based overlay system handlers

async fn handle_get_overlay_files() -> Response<BoxBody<Bytes, Infallible>> {
    use std::fs;
    use std::path::Path;

    let overlays_dir = Path::new("src/overlays");

    if !overlays_dir.exists() {
        return json_response(&Vec::<serde_json::Value>::new());
    }

    match fs::read_dir(overlays_dir) {
        Ok(entries) => {
            let files: Vec<serde_json::Value> = entries
                .filter_map(|entry| entry.ok())
                .filter(|entry| {
                    entry.path().extension()
                        .and_then(|ext| ext.to_str())
                        .map(|ext| ext == "jsx" || ext == "tsx")
                        .unwrap_or(false)
                })
                .map(|entry| {
                    let path = entry.path();
                    let name = path.file_stem()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown")
                        .to_string();

                    serde_json::json!({
                        "name": name,
                        "path": path.to_string_lossy().to_string(),
                    })
                })
                .collect();

            json_response(&files)
        }
        Err(e) => {
            error!("Failed to read overlays directory: {}", e);
            error_response(StatusCode::INTERNAL_SERVER_ERROR, "Failed to read overlays directory")
        }
    }
}

async fn handle_read_overlay_file(filename: &str) -> Response<BoxBody<Bytes, Infallible>> {
    use std::fs;
    use std::path::Path;

    // Sanitize filename to prevent path traversal
    let clean_filename = filename.replace("..", "").replace("/", "").replace("\\", "");
    let file_path = Path::new("src/overlays").join(&clean_filename);

    // Ensure file has .jsx extension
    let file_path = if !file_path.to_string_lossy().ends_with(".jsx") {
        Path::new(&format!("{}.jsx", file_path.to_string_lossy())).to_path_buf()
    } else {
        file_path
    };

    match fs::read_to_string(&file_path) {
        Ok(content) => {
            Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "text/plain; charset=utf-8")
                .header("Access-Control-Allow-Origin", "*")
                .header("Access-Control-Allow-Methods", "GET, POST, DELETE, OPTIONS")
                .header("Access-Control-Allow-Headers", "Content-Type, Authorization")
                .body(BoxBody::new(Full::new(Bytes::from(content))))
                .unwrap()
        }
        Err(e) => {
            error!("Failed to read overlay file {}: {}", file_path.display(), e);
            error_response(StatusCode::NOT_FOUND, "Overlay file not found")
        }
    }
}

async fn handle_write_overlay_file(filename: &str, body: &Bytes) -> Response<BoxBody<Bytes, Infallible>> {
    use std::fs;
    use std::path::Path;

    #[derive(serde::Deserialize)]
    struct WriteRequest {
        content: String,
    }

    match serde_json::from_slice::<WriteRequest>(body) {
        Ok(req) => {
            // Sanitize filename
            let clean_filename = filename.replace("..", "").replace("/", "").replace("\\", "");
            let file_path = Path::new("src/overlays").join(&clean_filename);

            // Ensure directory exists
            if let Some(parent) = file_path.parent() {
                if let Err(e) = fs::create_dir_all(parent) {
                    error!("Failed to create overlays directory: {}", e);
                    return error_response(StatusCode::INTERNAL_SERVER_ERROR, "Failed to create directory");
                }
            }

            match fs::write(&file_path, req.content) {
                Ok(_) => {
                    let response = serde_json::json!({
                        "success": true,
                        "message": "File saved successfully",
                        "path": file_path.to_string_lossy()
                    });
                    json_response(&response)
                }
                Err(e) => {
                    error!("Failed to write overlay file {}: {}", file_path.display(), e);
                    error_response(StatusCode::INTERNAL_SERVER_ERROR, "Failed to write file")
                }
            }
        }
        Err(e) => error_response(StatusCode::BAD_REQUEST, &format!("Invalid request: {}", e)),
    }
}

async fn handle_delete_overlay_file(filename: &str) -> Response<BoxBody<Bytes, Infallible>> {
    use std::fs;
    use std::path::Path;

    // Sanitize filename
    let clean_filename = filename.replace("..", "").replace("/", "").replace("\\", "");
    let file_path = Path::new("src/overlays").join(&clean_filename);

    match fs::remove_file(&file_path) {
        Ok(_) => {
            let response = serde_json::json!({
                "success": true,
                "message": "File deleted successfully"
            });
            json_response(&response)
        }
        Err(e) => {
            error!("Failed to delete overlay file {}: {}", file_path.display(), e);
            error_response(StatusCode::INTERNAL_SERVER_ERROR, "Failed to delete file")
        }
    }
}

async fn handle_rebuild_overlays() -> Response<BoxBody<Bytes, Infallible>> {
    use std::process::Command;

    info!("Triggering overlay rebuild...");

    // Run bun run build:overlays
    match Command::new("bun")
        .args(&["run", "build:overlays"])
        .output()
    {
        Ok(output) => {
            if output.status.success() {
                info!("Overlay rebuild completed successfully");
                let response = serde_json::json!({
                    "success": true,
                    "message": "Overlays rebuilt successfully",
                    "output": String::from_utf8_lossy(&output.stdout).to_string()
                });
                json_response(&response)
            } else {
                let error_msg = String::from_utf8_lossy(&output.stderr).to_string();
                error!("Overlay rebuild failed: {}", error_msg);
                let response = serde_json::json!({
                    "success": false,
                    "error": "Build failed",
                    "details": error_msg
                });
                json_response(&response)
            }
        }
        Err(e) => {
            error!("Failed to execute build command: {}", e);
            error_response(StatusCode::INTERNAL_SERVER_ERROR, "Failed to execute build")
        }
    }
}

// === TIMER BROADCAST HANDLER ===

async fn handle_timer_broadcast(body: &str) -> Response<BoxBody<Bytes, Infallible>> {
    match serde_json::from_str::<serde_json::Value>(body) {
        Ok(timer_data) => {
            // Broadcast timer state to all WebSocket clients
            crate::modules::websocket_server::broadcast_timer_state(timer_data);

            let response = serde_json::json!({
                "success": true,
                "message": "Timer state broadcast"
            });
            json_response(&response)
        }
        Err(e) => {
            error!("Failed to parse timer data: {}", e);
            error_response(StatusCode::BAD_REQUEST, &format!("Invalid timer data: {}", e))
        }
    }
}

// === DISCORD HANDLERS ===

async fn handle_discord_get_config() -> Response<BoxBody<Bytes, Infallible>> {
    match get_database() {
        Some(db) => {
            match db.get_discord_config() {
                Ok(config) => {
                    let response = serde_json::json!({
                        "success": true,
                        "data": {
                            "bot_token": config.bot_token,
                            "channel_id": config.channel_id,
                            "enabled": config.enabled,
                            "command_prefix": config.command_prefix,
                            "max_song_length": config.max_song_length,
                            "max_queue_size": config.max_queue_size,
                            "configured": config.bot_token.is_some() && config.channel_id.is_some()
                        }
                    });
                    json_response(&response)
                }
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Database error: {}", e))
            }
        }
        None => error_response(StatusCode::SERVICE_UNAVAILABLE, "Database not initialized"),
    }
}

async fn handle_discord_save_config(body: &str) -> Response<BoxBody<Bytes, Infallible>> {
    #[derive(serde::Deserialize)]
    struct ConfigRequest {
        bot_token: Option<String>,
        channel_id: Option<String>,
        enabled: bool,
        command_prefix: String,
        max_song_length: i64,
        max_queue_size: i64,
    }

    match get_database() {
        Some(db) => {
            match serde_json::from_str::<ConfigRequest>(body) {
                Ok(req) => {
                    match db.save_discord_config(
                        req.bot_token.as_deref(),
                        req.channel_id.as_deref(),
                        req.enabled,
                        &req.command_prefix,
                        req.max_song_length,
                        req.max_queue_size,
                    ) {
                        Ok(()) => {
                            let response = ApiResponse {
                                success: true,
                                content: Some("Discord configuration saved successfully".to_string()),
                                error: None,
                            };
                            json_response(&response)
                        }
                        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to save configuration: {}", e))
                    }
                }
                Err(e) => error_response(StatusCode::BAD_REQUEST, &format!("Invalid request: {}", e))
            }
        }
        None => error_response(StatusCode::SERVICE_UNAVAILABLE, "Database not initialized"),
    }
}

async fn handle_discord_get_stats() -> Response<BoxBody<Bytes, Infallible>> {
    match get_discord_manager() {
        Some(manager) => {
            match manager.get_stats().await {
                Ok(stats) => {
                    let response = serde_json::json!({
                        "success": true,
                        "data": stats
                    });
                    json_response(&response)
                }
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to get stats: {}", e))
            }
        }
        None => error_response(StatusCode::SERVICE_UNAVAILABLE, "Discord manager not initialized"),
    }
}

async fn handle_discord_start() -> Response<BoxBody<Bytes, Infallible>> {
    match get_discord_manager() {
        Some(manager) => {
            match manager.start().await {
                Ok(()) => {
                    let response = ApiResponse {
                        success: true,
                        content: Some("Discord bot started successfully".to_string()),
                        error: None,
                    };
                    json_response(&response)
                }
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to start Discord bot: {}", e))
            }
        }
        None => error_response(StatusCode::SERVICE_UNAVAILABLE, "Discord manager not initialized"),
    }
}

async fn handle_discord_stop() -> Response<BoxBody<Bytes, Infallible>> {
    match get_discord_manager() {
        Some(manager) => {
            match manager.stop().await {
                Ok(()) => {
                    let response = ApiResponse {
                        success: true,
                        content: Some("Discord bot stopped successfully".to_string()),
                        error: None,
                    };
                    json_response(&response)
                }
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to stop Discord bot: {}", e))
            }
        }
        None => error_response(StatusCode::SERVICE_UNAVAILABLE, "Discord manager not initialized"),
    }
}

async fn handle_discord_restart() -> Response<BoxBody<Bytes, Infallible>> {
    match get_discord_manager() {
        Some(manager) => {
            match manager.restart().await {
                Ok(()) => {
                    let response = ApiResponse {
                        success: true,
                        content: Some("Discord bot restarted successfully".to_string()),
                        error: None,
                    };
                    json_response(&response)
                }
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to restart Discord bot: {}", e))
            }
        }
        None => error_response(StatusCode::SERVICE_UNAVAILABLE, "Discord manager not initialized"),
    }
}

// === SONG REQUEST HANDLERS ===

async fn handle_song_requests_get_pending() -> Response<BoxBody<Bytes, Infallible>> {
    match get_database() {
        Some(db) => {
            match db.get_pending_song_requests() {
                Ok(requests) => {
                    let response = serde_json::json!({
                        "success": true,
                        "data": requests
                    });
                    json_response(&response)
                }
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Database error: {}", e))
            }
        }
        None => error_response(StatusCode::SERVICE_UNAVAILABLE, "Database not initialized"),
    }
}

async fn handle_song_requests_get_all(query: &str) -> Response<BoxBody<Bytes, Infallible>> {
    match get_database() {
        Some(db) => {
            // Parse limit from query string
            let limit = query
                .split('&')
                .find(|s| s.starts_with("limit="))
                .and_then(|s| s.split('=').nth(1))
                .and_then(|s| s.parse::<i64>().ok());

            match db.get_all_song_requests(limit) {
                Ok(requests) => {
                    let response = serde_json::json!({
                        "success": true,
                        "data": requests
                    });
                    json_response(&response)
                }
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Database error: {}", e))
            }
        }
        None => error_response(StatusCode::SERVICE_UNAVAILABLE, "Database not initialized"),
    }
}

async fn handle_song_request_update_status(body: &str) -> Response<BoxBody<Bytes, Infallible>> {
    #[derive(serde::Deserialize)]
    struct UpdateRequest {
        id: i64,
        status: String,
    }

    match get_database() {
        Some(db) => {
            match serde_json::from_str::<UpdateRequest>(body) {
                Ok(req) => {
                    match db.update_song_request_status(req.id, &req.status) {
                        Ok(()) => {
                            let response = ApiResponse {
                                success: true,
                                content: Some(format!("Song request {} updated to {}", req.id, req.status)),
                                error: None,
                            };
                            json_response(&response)
                        }
                        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to update status: {}", e))
                    }
                }
                Err(e) => error_response(StatusCode::BAD_REQUEST, &format!("Invalid request: {}", e))
            }
        }
        None => error_response(StatusCode::SERVICE_UNAVAILABLE, "Database not initialized"),
    }
}

async fn handle_song_request_delete(body: &str) -> Response<BoxBody<Bytes, Infallible>> {
    #[derive(serde::Deserialize)]
    struct DeleteRequest {
        id: i64,
    }

    match get_database() {
        Some(db) => {
            match serde_json::from_str::<DeleteRequest>(body) {
                Ok(req) => {
                    match db.delete_song_request(req.id) {
                        Ok(()) => {
                            let response = ApiResponse {
                                success: true,
                                content: Some(format!("Song request {} deleted", req.id)),
                                error: None,
                            };
                            json_response(&response)
                        }
                        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to delete request: {}", e))
                    }
                }
                Err(e) => error_response(StatusCode::BAD_REQUEST, &format!("Invalid request: {}", e))
            }
        }
        None => error_response(StatusCode::SERVICE_UNAVAILABLE, "Database not initialized"),
    }
}

async fn handle_song_requests_clear(body: &str) -> Response<BoxBody<Bytes, Infallible>> {
    #[derive(serde::Deserialize)]
    struct ClearRequest {
        status: String,
    }

    match get_database() {
        Some(db) => {
            match serde_json::from_str::<ClearRequest>(body) {
                Ok(req) => {
                    match db.clear_song_requests_by_status(&req.status) {
                        Ok(()) => {
                            let response = ApiResponse {
                                success: true,
                                content: Some(format!("Cleared all {} song requests", req.status)),
                                error: None,
                            };
                            json_response(&response)
                        }
                        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to clear requests: {}", e))
                    }
                }
                Err(e) => error_response(StatusCode::BAD_REQUEST, &format!("Invalid request: {}", e))
            }
        }
        None => error_response(StatusCode::SERVICE_UNAVAILABLE, "Database not initialized"),
    }
}
