use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightStateChangedEvent {
    pub light_id: String,
    pub light_name: String,
    pub on: Option<bool>,
    pub brightness: Option<i64>,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneActivatedEvent {
    pub scene_id: i64,
    pub scene_name: String,
    pub triggered_by: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationTriggeredEvent {
    pub automation_id: i64,
    pub automation_name: String,
    pub trigger_event: String,
}
