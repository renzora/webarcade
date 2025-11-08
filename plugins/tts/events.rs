use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TtsRequestEvent {
    pub text: String,
    pub voice: Option<String>,
    pub priority: Option<i64>,
    pub requested_by: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TtsQueuedEvent {
    pub id: i64,
    pub text: String,
    pub voice: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TtsProcessingEvent {
    pub id: i64,
    pub text: String,
    pub voice: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TtsCompletedEvent {
    pub id: i64,
    pub text: String,
    pub voice: String,
}
