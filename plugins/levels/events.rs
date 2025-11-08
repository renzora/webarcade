use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XpEarnedEvent {
    pub user_id: String,
    pub username: String,
    pub amount: i64,
    pub total_xp: i64,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LevelUpEvent {
    pub user_id: String,
    pub username: String,
    pub old_level: i64,
    pub new_level: i64,
    pub total_xp: i64,
}
