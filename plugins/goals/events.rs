use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalCreatedEvent {
    pub goal_id: i64,
    pub channel: String,
    pub title: String,
    pub goal_type: String,
    pub target: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalProgressEvent {
    pub goal_id: i64,
    pub channel: String,
    pub current: i64,
    pub target: i64,
    pub percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalCompletedEvent {
    pub goal_id: i64,
    pub channel: String,
    pub goal_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalDeletedEvent {
    pub goal_id: i64,
    pub channel: String,
}
