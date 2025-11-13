// Core infrastructure modules
pub mod types;
pub mod errors;
pub mod file_watcher;
pub mod file_sync;
pub mod system_monitor;
pub mod memory_cache;
pub mod websocket_server;
pub mod system_api;

// Exports
pub use file_watcher::{initialize_file_watcher, get_base_path, get_projects_path};
pub use system_monitor::initialize_system_monitor;
pub use memory_cache::MemoryCache;
pub use websocket_server::start_websocket_server;
