pub mod router;

use api::{Plugin, PluginMetadata};

pub struct DemoPlugin;

impl Plugin for DemoPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            id: "demo".to_string(),
            name: "Demo Plugin".to_string(),
            version: "1.0.0".to_string(),
            description: "Showcases all UI components with native Rust backend".to_string(),
            author: "WebArcade Team".to_string(),
            dependencies: vec![],
        }
    }
}
