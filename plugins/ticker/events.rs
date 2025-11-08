use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TickerMessageAddedEvent {
    pub message: String,
    pub is_sticky: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TickerEventAddedEvent {
    pub event_type: String,
    pub display_text: String,
}
