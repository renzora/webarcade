// Database utilities for the plugin system
// Each plugin gets its own connection and manages its own schema

use anyhow::Result;
use std::path::PathBuf;

/// Get the path to the main database file
pub fn get_database_path() -> PathBuf {
    // ALWAYS use data/counters.db in the project root
    // We'll look for the project root by finding where package.json exists
    let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    // Try to find project root by looking for package.json
    let mut search_dir = current_dir.clone();
    loop {
        if search_dir.join("package.json").exists() {
            // Found project root
            return search_dir.join("data").join("counters.db");
        }

        // Move up one directory
        if let Some(parent) = search_dir.parent() {
            search_dir = parent.to_path_buf();
        } else {
            // Couldn't find project root, fall back to current_dir/data/counters.db
            break;
        }
    }

    current_dir.join("data").join("counters.db")
}

/// Ensure the database directory exists
pub fn ensure_database_dir() -> Result<()> {
    let db_path = get_database_path();
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    Ok(())
}
