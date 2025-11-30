use api::{HttpRequest, HttpResponse, json, json_response};
use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Deserialize)]
struct ListFilesRequest {
    path: String,
}

#[derive(Deserialize)]
struct ReadFileRequest {
    path: String,
}

#[derive(Deserialize)]
struct WriteFileRequest {
    path: String,
    content: String,
}

#[derive(Deserialize)]
struct CreateFileRequest {
    path: String,
    is_dir: Option<bool>,
}

#[derive(Deserialize)]
struct DeleteFileRequest {
    path: String,
}

#[derive(Deserialize)]
struct RenameFileRequest {
    old_path: String,
    new_path: String,
}

/// List files and directories in a given path
pub async fn handle_list_files(req: HttpRequest) -> HttpResponse {
    let body: ListFilesRequest = match req.body_json() {
        Ok(v) => v,
        Err(e) => {
            return json_response(&json!({
                "error": format!("Invalid request: {}", e)
            }));
        }
    };

    let path = Path::new(&body.path);

    if !path.exists() {
        return json_response(&json!({
            "error": "Path does not exist"
        }));
    }

    if !path.is_dir() {
        return json_response(&json!({
            "error": "Path is not a directory"
        }));
    }

    let mut files = Vec::new();

    // Read directory entries (only immediate children, not recursive)
    match fs::read_dir(path) {
        Ok(entries) => {
            for entry in entries.filter_map(|e| e.ok()) {
                let file_path = entry.path();
                let file_name = entry.file_name().to_string_lossy().to_string();

                // Skip hidden files/folders (starting with .)
                if file_name.starts_with('.') {
                    continue;
                }

                // Skip common build/dependency directories
                let skip_dirs = ["node_modules", "target", "dist", ".git", "__pycache__", ".next", ".nuxt", "build"];
                if file_path.is_dir() && skip_dirs.contains(&file_name.as_str()) {
                    continue;
                }

                let is_dir = file_path.is_dir();
                let size = if is_dir {
                    0
                } else {
                    fs::metadata(&file_path).map(|m| m.len()).unwrap_or(0)
                };

                files.push(json!({
                    "name": file_name,
                    "path": file_path.to_string_lossy().to_string(),
                    "is_dir": is_dir,
                    "size": size
                }));
            }
        }
        Err(e) => {
            return json_response(&json!({
                "error": format!("Failed to read directory: {}", e)
            }));
        }
    }

    json_response(&json!({
        "files": files,
        "path": body.path
    }))
}

/// Read file contents
pub async fn handle_read_file(req: HttpRequest) -> HttpResponse {
    let body: ReadFileRequest = match req.body_json() {
        Ok(v) => v,
        Err(e) => {
            return json_response(&json!({
                "error": format!("Invalid request: {}", e)
            }));
        }
    };

    let path = Path::new(&body.path);

    if !path.exists() {
        return json_response(&json!({
            "error": "File does not exist"
        }));
    }

    if !path.is_file() {
        return json_response(&json!({
            "error": "Path is not a file"
        }));
    }

    // Check file size - limit to 10MB
    let metadata = match fs::metadata(path) {
        Ok(m) => m,
        Err(e) => {
            return json_response(&json!({
                "error": format!("Failed to read file metadata: {}", e)
            }));
        }
    };

    if metadata.len() > 10 * 1024 * 1024 {
        return json_response(&json!({
            "error": "File is too large (max 10MB)"
        }));
    }

    match fs::read_to_string(path) {
        Ok(content) => {
            json_response(&json!({
                "content": content,
                "path": body.path,
                "size": metadata.len()
            }))
        }
        Err(e) => {
            // Try reading as binary and return error for binary files
            json_response(&json!({
                "error": format!("Failed to read file (may be binary): {}", e)
            }))
        }
    }
}

/// Write file contents
pub async fn handle_write_file(req: HttpRequest) -> HttpResponse {
    let body: WriteFileRequest = match req.body_json() {
        Ok(v) => v,
        Err(e) => {
            return json_response(&json!({
                "error": format!("Invalid request: {}", e)
            }));
        }
    };

    let path = Path::new(&body.path);

    // Create parent directories if they don't exist
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            if let Err(e) = fs::create_dir_all(parent) {
                return json_response(&json!({
                    "error": format!("Failed to create parent directories: {}", e)
                }));
            }
        }
    }

    match fs::write(path, &body.content) {
        Ok(_) => {
            json_response(&json!({
                "success": true,
                "path": body.path,
                "size": body.content.len()
            }))
        }
        Err(e) => {
            json_response(&json!({
                "success": false,
                "error": format!("Failed to write file: {}", e)
            }))
        }
    }
}

/// Open folder picker dialog
pub async fn handle_pick_folder(_req: HttpRequest) -> HttpResponse {
    // Use rfd for native file dialog
    let folder = rfd::FileDialog::new()
        .set_title("Select Folder")
        .pick_folder();

    match folder {
        Some(path) => {
            json_response(&json!({
                "success": true,
                "path": path.to_string_lossy().to_string()
            }))
        }
        None => {
            json_response(&json!({
                "success": false,
                "error": "No folder selected"
            }))
        }
    }
}

/// Create a new file or directory
pub async fn handle_create_file(req: HttpRequest) -> HttpResponse {
    let body: CreateFileRequest = match req.body_json() {
        Ok(v) => v,
        Err(e) => {
            return json_response(&json!({
                "error": format!("Invalid request: {}", e)
            }));
        }
    };

    let path = Path::new(&body.path);
    let is_dir = body.is_dir.unwrap_or(false);

    if path.exists() {
        return json_response(&json!({
            "success": false,
            "error": "Path already exists"
        }));
    }

    if is_dir {
        match fs::create_dir_all(path) {
            Ok(_) => json_response(&json!({
                "success": true,
                "path": body.path
            })),
            Err(e) => json_response(&json!({
                "success": false,
                "error": format!("Failed to create directory: {}", e)
            }))
        }
    } else {
        // Create parent directories if needed
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                if let Err(e) = fs::create_dir_all(parent) {
                    return json_response(&json!({
                        "success": false,
                        "error": format!("Failed to create parent directories: {}", e)
                    }));
                }
            }
        }

        match fs::write(path, "") {
            Ok(_) => json_response(&json!({
                "success": true,
                "path": body.path
            })),
            Err(e) => json_response(&json!({
                "success": false,
                "error": format!("Failed to create file: {}", e)
            }))
        }
    }
}

/// Delete a file or directory
pub async fn handle_delete_file(req: HttpRequest) -> HttpResponse {
    let body: DeleteFileRequest = match req.body_json() {
        Ok(v) => v,
        Err(e) => {
            return json_response(&json!({
                "error": format!("Invalid request: {}", e)
            }));
        }
    };

    let path = Path::new(&body.path);

    if !path.exists() {
        return json_response(&json!({
            "success": false,
            "error": "Path does not exist"
        }));
    }

    let result = if path.is_dir() {
        fs::remove_dir_all(path)
    } else {
        fs::remove_file(path)
    };

    match result {
        Ok(_) => json_response(&json!({
            "success": true,
            "path": body.path
        })),
        Err(e) => json_response(&json!({
            "success": false,
            "error": format!("Failed to delete: {}", e)
        }))
    }
}

/// Rename a file or directory
pub async fn handle_rename_file(req: HttpRequest) -> HttpResponse {
    let body: RenameFileRequest = match req.body_json() {
        Ok(v) => v,
        Err(e) => {
            return json_response(&json!({
                "error": format!("Invalid request: {}", e)
            }));
        }
    };

    let old_path = Path::new(&body.old_path);
    let new_path = Path::new(&body.new_path);

    if !old_path.exists() {
        return json_response(&json!({
            "success": false,
            "error": "Source path does not exist"
        }));
    }

    if new_path.exists() {
        return json_response(&json!({
            "success": false,
            "error": "Destination path already exists"
        }));
    }

    match fs::rename(old_path, new_path) {
        Ok(_) => json_response(&json!({
            "success": true,
            "old_path": body.old_path,
            "new_path": body.new_path
        })),
        Err(e) => json_response(&json!({
            "success": false,
            "error": format!("Failed to rename: {}", e)
        }))
    }
}
