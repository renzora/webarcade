use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Serialize, Deserialize};
use log::{info, warn, debug};
use dashmap::DashMap;

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct CacheEntry {
    value: String,
    created_at: u64,
    ttl_seconds: u64,
}

#[allow(dead_code)]
impl CacheEntry {
    fn new(value: String, ttl_seconds: u64) -> Self {
        let created_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        Self {
            value,
            created_at,
            ttl_seconds,
        }
    }
    
    #[allow(dead_code)]
    fn is_expired(&self) -> bool {
        if self.ttl_seconds == 0 {
            return false; // Never expires
        }
        
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        now - self.created_at > self.ttl_seconds
    }
}

pub struct MemoryCache {
    #[allow(dead_code)]
    storage: DashMap<String, CacheEntry>,
    #[allow(dead_code)]
    enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptSearchResult {
    pub name: String,
    pub path: String,
    pub directory: String,
    pub last_modified: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CachedScriptList {
    pub scripts: Vec<ScriptSearchResult>,
    pub timestamp: u64,
    pub total_count: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectManifest {
    pub project_name: String,
    pub last_scan: u64,
    pub file_count: usize,
    pub checksum: String,
    pub cache_version: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileMetadata {
    pub path: String,
    pub last_modified: u64,
    pub file_size: u64,
    pub hash: String,
    pub processed_at: u64,
    pub file_type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProcessedAsset {
    pub path: String,
    pub file_type: String,
    pub metadata: serde_json::Value,
    pub thumbnail_path: Option<String>,
    pub compressed_path: Option<String>,
    pub extracted_materials: Option<Vec<String>>,
    pub processing_status: String,
    pub processed_at: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CacheValidationResult {
    pub cache_status: String, // "valid", "needs_update", "missing"
    pub changes_detected: usize,
    pub estimated_processing_time: u64,
    pub change_summary: ChangeSummary,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChangeSummary {
    pub new_files: usize,
    pub modified_files: usize,
    pub deleted_files: usize,
    pub moved_files: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CachedAssetNode {
    pub name: String,
    pub path: String,
    pub is_directory: bool,
    pub file_size: Option<u64>,
    pub last_modified: Option<u64>,
    pub extension: Option<String>,
    pub file_type: Option<String>,
    pub thumbnail_url: Option<String>,
    pub children: Option<Vec<CachedAssetNode>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectAssetTree {
    pub project_name: String,
    pub root_path: String,
    pub assets: Vec<CachedAssetNode>,
    pub generated_at: u64,
    pub total_files: usize,
    pub total_directories: usize,
}

impl MemoryCache {
    pub fn new() -> Self {
        info!("💾 Initializing lightweight memory cache with DashMap");
        Self {
            storage: DashMap::new(),
            enabled: true,
        }
    }
    
    #[allow(dead_code)]
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
    
    // Core cache operations
    #[allow(dead_code)]
    pub async fn set_string(&self, key: &str, value: &str) -> Result<(), String> {
        self.set_string_with_ttl(key, value, 300).await // Default 5 minutes TTL
    }
    
    #[allow(dead_code)]
    pub async fn set_string_with_ttl(&self, key: &str, value: &str, ttl_seconds: u64) -> Result<(), String> {
        if !self.enabled {
            return Err("Memory cache not enabled".to_string());
        }
        
        let entry = CacheEntry::new(value.to_string(), ttl_seconds);
        self.storage.insert(key.to_string(), entry);
        debug!("💾 Cached key: {} (TTL: {}s)", key, ttl_seconds);
        Ok(())
    }
    
    #[allow(dead_code)]
    pub async fn get_string(&self, key: &str) -> Result<Option<String>, String> {
        if !self.enabled {
            return Ok(None);
        }
        
        // Check if the key exists and is not expired
        if let Some(entry) = self.storage.get(key) {
            if entry.is_expired() {
                // Remove expired entry
                self.storage.remove(key);
                debug!("💾 Removed expired key: {}", key);
                Ok(None)
            } else {
                Ok(Some(entry.value.clone()))
            }
        } else {
            Ok(None)
        }
    }
    
    #[allow(dead_code)]
    pub fn clear_all_cache(&self) -> bool {
        let count = self.storage.len();
        self.storage.clear();
        info!("💾 Cleared {} cache entries", count);
        true
    }
    
    #[allow(dead_code)]
    pub fn get_cache_stats(&self) -> serde_json::Value {
        let total_keys = self.storage.len();
        let expired_keys = self.storage.iter().filter(|entry| entry.value().is_expired()).count();
        
        serde_json::json!({
            "memory_cache_enabled": true,
            "connection_status": "in_memory",
            "total_keys": total_keys,
            "expired_keys": expired_keys,
            "active_keys": total_keys - expired_keys,
            "cache_type": "dashmap_concurrent"
        })
    }
    
    // Cleanup expired entries periodically
    #[allow(dead_code)]
    pub fn cleanup_expired(&self) -> usize {
        let original_size = self.storage.len();
        self.storage.retain(|_, entry| !entry.is_expired());
        let removed = original_size - self.storage.len();
        
        if removed > 0 {
            debug!("💾 Cleaned up {} expired cache entries", removed);
        }
        removed
    }
    
    
    // Script list caching (compatible with Redis interface)
    #[allow(dead_code)]
    pub async fn cache_script_list(&self, scripts: &[ScriptSearchResult]) -> bool {
        let cached_data = CachedScriptList {
            scripts: scripts.to_vec(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            total_count: scripts.len(),
        };
        
        match serde_json::to_string(&cached_data) {
            Ok(json) => {
                match self.set_string_with_ttl("scripts:list", &json, 300).await {
                    Ok(_) => {
                        debug!("💾 Cached {} scripts in memory", scripts.len());
                        true
                    }
                    Err(e) => {
                        warn!("💾 Failed to cache script list: {}", e);
                        false
                    }
                }
            }
            Err(e) => {
                warn!("💾 Failed to serialize script list: {}", e);
                false
            }
        }
    }
    
    #[allow(dead_code)]
    pub async fn get_cached_script_list(&self) -> Option<Vec<ScriptSearchResult>> {
        match self.get_string("scripts:list").await {
            Ok(Some(json)) => {
                match serde_json::from_str::<CachedScriptList>(&json) {
                    Ok(cached_data) => {
                        debug!("💾 Retrieved {} scripts from memory cache", cached_data.scripts.len());
                        Some(cached_data.scripts)
                    }
                    Err(e) => {
                        warn!("💾 Failed to deserialize cached script list: {}", e);
                        None
                    }
                }
            }
            Ok(None) => {
                debug!("💾 No cached script list found");
                None
            }
            Err(e) => {
                warn!("💾 Failed to get cached script list: {}", e);
                None
            }
        }
    }
    
    // Project asset cache methods
    #[allow(dead_code)]
    pub async fn cache_project_manifest(&self, manifest: &ProjectManifest) -> bool {
        let key = format!("project:{}:manifest", manifest.project_name);
        match serde_json::to_string(manifest) {
            Ok(json) => {
                match self.set_string_with_ttl(&key, &json, 86400).await { // 24 hours
                    Ok(_) => {
                        info!("💾 Cached project manifest for: {}", manifest.project_name);
                        true
                    }
                    Err(e) => {
                        warn!("💾 Failed to cache project manifest: {}", e);
                        false
                    }
                }
            }
            Err(e) => {
                warn!("💾 Failed to serialize project manifest: {}", e);
                false
            }
        }
    }
    
    #[allow(dead_code)]
    pub async fn get_project_manifest(&self, project_name: &str) -> Option<ProjectManifest> {
        let key = format!("project:{}:manifest", project_name);
        match self.get_string(&key).await {
            Ok(Some(json)) => {
                match serde_json::from_str::<ProjectManifest>(&json) {
                    Ok(manifest) => {
                        info!("💾 Retrieved project manifest from cache: {}", project_name);
                        Some(manifest)
                    }
                    Err(e) => {
                        warn!("💾 Failed to deserialize project manifest: {}", e);
                        None
                    }
                }
            }
            Ok(None) => {
                debug!("💾 No cached project manifest found: {}", project_name);
                None
            }
            Err(e) => {
                warn!("💾 Failed to get project manifest: {}", e);
                None
            }
        }
    }
    
    #[allow(dead_code)]
    pub async fn cache_project_asset_tree(&self, tree: &ProjectAssetTree) -> bool {
        let key = format!("project:{}:asset_tree", tree.project_name);
        match serde_json::to_string(tree) {
            Ok(json) => {
                match self.set_string_with_ttl(&key, &json, 86400).await { // 24 hours
                    Ok(_) => {
                        info!("💾 Cached project asset tree for: {} ({} files, {} directories)", 
                              tree.project_name, tree.total_files, tree.total_directories);
                        true
                    }
                    Err(e) => {
                        warn!("💾 Failed to cache project asset tree: {}", e);
                        false
                    }
                }
            }
            Err(e) => {
                warn!("💾 Failed to serialize project asset tree: {}", e);
                false
            }
        }
    }
    
    #[allow(dead_code)]
    pub async fn get_project_asset_tree(&self, project_name: &str) -> Option<ProjectAssetTree> {
        let key = format!("project:{}:asset_tree", project_name);
        match self.get_string(&key).await {
            Ok(Some(json)) => {
                match serde_json::from_str::<ProjectAssetTree>(&json) {
                    Ok(tree) => {
                        info!("💾 Retrieved project asset tree from cache: {} ({} files, {} directories)", 
                              project_name, tree.total_files, tree.total_directories);
                        Some(tree)
                    }
                    Err(e) => {
                        warn!("💾 Failed to deserialize project asset tree: {}", e);
                        None
                    }
                }
            }
            Ok(None) => {
                debug!("💾 No cached project asset tree found: {}", project_name);
                None
            }
            Err(e) => {
                warn!("💾 Failed to get project asset tree: {}", e);
                None
            }
        }
    }
    
    #[allow(dead_code)]
    pub fn clear_project_cache(&self, project_name: &str) -> bool {
        let keys = vec![
            format!("project:{}:manifest", project_name),
            format!("project:{}:files", project_name),
            format!("project:{}:processed", project_name),
            format!("project:{}:asset_tree", project_name),
        ];
        
        let mut cleared = 0;
        for key in &keys {
            if self.storage.remove(key).is_some() {
                cleared += 1;
            }
        }
        
        info!("💾 Cleared {} cache entries for project: {}", cleared, project_name);
        true
    }
    
    // File metadata caching methods
    #[allow(dead_code)]
    pub async fn cache_file_metadata(&self, project_name: &str, file_metadata: &[FileMetadata]) -> bool {
        let key = format!("project:{}:files", project_name);
        
        // Store as JSON for simplicity (in real Redis this was a hash)
        let metadata_map: std::collections::HashMap<String, FileMetadata> = file_metadata
            .iter()
            .map(|meta| (meta.path.clone(), meta.clone()))
            .collect();
        
        match serde_json::to_string(&metadata_map) {
            Ok(json) => {
                match self.set_string_with_ttl(&key, &json, 86400).await { // 24 hours
                    Ok(_) => {
                        info!("💾 Cached {} file metadata entries for project: {}", file_metadata.len(), project_name);
                        true
                    }
                    Err(e) => {
                        warn!("💾 Failed to cache file metadata: {}", e);
                        false
                    }
                }
            }
            Err(e) => {
                warn!("💾 Failed to serialize file metadata: {}", e);
                false
            }
        }
    }
    
    #[allow(dead_code)]
    pub async fn get_all_file_metadata(&self, project_name: &str) -> Vec<FileMetadata> {
        let key = format!("project:{}:files", project_name);
        match self.get_string(&key).await {
            Ok(Some(json)) => {
                match serde_json::from_str::<std::collections::HashMap<String, FileMetadata>>(&json) {
                    Ok(metadata_map) => {
                        let metadata_list: Vec<FileMetadata> = metadata_map.into_values().collect();
                        if !metadata_list.is_empty() {
                            info!("💾 Retrieved {} file metadata entries from cache for project: {}", metadata_list.len(), project_name);
                        }
                        metadata_list
                    }
                    Err(e) => {
                        warn!("💾 Failed to deserialize file metadata: {}", e);
                        Vec::new()
                    }
                }
            }
            Ok(None) => Vec::new(),
            Err(e) => {
                warn!("💾 Failed to get file metadata: {}", e);
                Vec::new()
            }
        }
    }
    
    // Processed asset caching methods  
    #[allow(dead_code)]
    pub async fn cache_processed_asset(&self, project_name: &str, asset: &ProcessedAsset) -> bool {
        let key = format!("project:{}:processed", project_name);
        
        // Get existing processed assets
        let mut assets_map: std::collections::HashMap<String, ProcessedAsset> = 
            match self.get_string(&key).await {
                Ok(Some(json)) => {
                    serde_json::from_str(&json).unwrap_or_default()
                }
                _ => std::collections::HashMap::new(),
            };
        
        // Add or update the asset
        assets_map.insert(asset.path.clone(), asset.clone());
        
        // Store back
        match serde_json::to_string(&assets_map) {
            Ok(json) => {
                match self.set_string_with_ttl(&key, &json, 86400).await { // 24 hours
                    Ok(_) => {
                        info!("💾 Cached processed asset: {} (thumbnail: {:?})", asset.path, asset.thumbnail_path);
                        true
                    }
                    Err(e) => {
                        warn!("💾 Failed to cache processed asset: {}", e);
                        false
                    }
                }
            }
            Err(e) => {
                warn!("💾 Failed to serialize processed asset: {}", e);
                false
            }
        }
    }
    
    #[allow(dead_code)]
    pub async fn get_all_processed_assets(&self, project_name: &str) -> Vec<ProcessedAsset> {
        let key = format!("project:{}:processed", project_name);
        match self.get_string(&key).await {
            Ok(Some(json)) => {
                match serde_json::from_str::<std::collections::HashMap<String, ProcessedAsset>>(&json) {
                    Ok(assets_map) => {
                        let assets: Vec<ProcessedAsset> = assets_map.into_values().collect();
                        if !assets.is_empty() {
                            info!("💾 Retrieved {} processed assets from cache for project: {}", assets.len(), project_name);
                            let with_thumbnails = assets.iter().filter(|a| a.thumbnail_path.is_some()).count();
                            info!("🖼️ Assets with thumbnails: {}/{}", with_thumbnails, assets.len());
                        }
                        assets
                    }
                    Err(e) => {
                        warn!("💾 Failed to deserialize processed assets: {}", e);
                        Vec::new()
                    }
                }
            }
            Ok(None) => Vec::new(),
            Err(e) => {
                warn!("💾 Failed to get processed assets: {}", e);
                Vec::new()
            }
        }
    }
}