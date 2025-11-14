// Database utilities for the plugin system
// Each plugin gets its own connection and manages its own schema

use anyhow::Result;
use std::path::PathBuf;

/// Get the path to the main database file
pub fn get_database_path() -> PathBuf {
    // Use platform-specific data directory
    let data_dir = dirs::data_local_dir()
        .or_else(|| dirs::data_dir())
        .expect("Could not determine data directory");

    let db_path = data_dir.join("WebArcade").join("database.db");

    // Ensure the directory exists
    if let Some(parent) = db_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    db_path
}

/// Ensure the database directory exists
pub fn ensure_database_dir() -> Result<()> {
    let db_path = get_database_path();
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    Ok(())
}
