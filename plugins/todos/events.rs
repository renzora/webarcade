use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoCreatedEvent {
    pub todo_id: i64,
    pub channel: String,
    pub username: String,
    pub task: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoCompletedEvent {
    pub todo_id: i64,
    pub channel: String,
    pub task: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoDeletedEvent {
    pub todo_id: i64,
    pub channel: String,
}
