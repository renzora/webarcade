use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessageEvent {
    pub channel: String,
    pub user_id: String,
    pub username: String,
    pub message: String,
    pub badges: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandEvent {
    pub command: String,
    pub args: Vec<String>,
    pub channel: String,
    pub user_id: String,
    pub username: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FollowEvent {
    pub user_id: String,
    pub username: String,
    pub followed_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionEvent {
    pub user_id: String,
    pub username: String,
    pub tier: String,
    pub is_gift: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamOnlineEvent {
    pub broadcaster_id: String,
    pub broadcaster_name: String,
    pub started_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamOfflineEvent {
    pub broadcaster_id: String,
    pub broadcaster_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaidEvent {
    pub from_broadcaster_id: String,
    pub from_broadcaster_name: String,
    pub to_broadcaster_id: String,
    pub to_broadcaster_name: String,
    pub viewers: i64,
}
