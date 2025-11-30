pub mod router;

use api::{Plugin, PluginMetadata};

pub struct CodePlugin;

impl Plugin for CodePlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            id: "code".to_string(),
            name: "Code Editor".to_string(),
            version: "1.0.0".to_string(),
            description: "Monaco-based code editor with file tree navigation".to_string(),
            author: "WebArcade Team".to_string(),
            dependencies: vec![],
        }
    }
}
