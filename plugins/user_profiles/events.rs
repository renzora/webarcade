use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BirthdayEvent {
    pub user_id: String,
    pub username: String,
    pub birthday: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileUpdatedEvent {
    pub user_id: String,
    pub username: String,
    pub fields_updated: Vec<String>,
}
