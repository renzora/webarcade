pub mod types;
pub mod errors;
pub mod handlers;
pub mod file_watcher;
pub mod file_sync;
pub mod system_monitor;
pub mod memory_cache;
pub mod websocket_server;
pub mod twitch;
pub mod withings_api;
pub mod discord;

// Export only what's needed by main.rs
pub use handlers::{handle_http_request, set_startup_time, set_memory_cache, set_database, set_twitch_manager, set_withings_api, set_discord_manager};
pub use file_watcher::{initialize_file_watcher, get_base_path, get_projects_path};
pub use system_monitor::{initialize_system_monitor};
pub use memory_cache::MemoryCache;
pub use websocket_server::start_websocket_server;
pub use twitch::TwitchManager;
pub use withings_api::WithingsAPI;
pub use discord::DiscordManager;
