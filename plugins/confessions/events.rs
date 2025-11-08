use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfessionSubmittedEvent {
    pub id: i64,
    pub status: String,
    pub submitted_by: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfessionApprovedEvent {
    pub id: i64,
    pub approved_by: Option<String>,
    pub auto_approved: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfessionRejectedEvent {
    pub id: i64,
    pub rejected_by: String,
    pub rejection_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfessionDisplayedEvent {
    pub id: i64,
    pub confession_text: String,
    pub display_count: i64,
}
