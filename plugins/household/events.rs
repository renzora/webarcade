use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskCreatedEvent {
    pub task_id: i64,
    pub task_name: String,
    pub assigned_to: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskCompletedEvent {
    pub task_id: i64,
    pub task_name: String,
    pub completed_by: String,
    pub completion_id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskAssignedEvent {
    pub task_id: i64,
    pub task_name: String,
    pub assigned_to: String,
}
