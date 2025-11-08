use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrencyEarnedEvent {
    pub user_id: String,
    pub username: String,
    pub amount: i64,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrencySpentEvent {
    pub user_id: String,
    pub username: String,
    pub amount: i64,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrencyTransferredEvent {
    pub from_user: String,
    pub to_user: String,
    pub amount: i64,
}
