pub mod router;

use api::{Plugin, PluginMetadata};

pub struct SystemMonitorPlugin;

impl Plugin for SystemMonitorPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            id: "systemMonitor".to_string(),
            name: "System Monitor".to_string(),
            version: "1.0.0".to_string(),
            description: "System resource monitoring (CPU, RAM, GPU)".to_string(),
            author: "WebArcade Team".to_string(),
            dependencies: vec![],
        }
    }
}
