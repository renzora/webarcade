use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackOpenedEvent {
    pub user_id: String,
    pub username: String,
    pub pack_id: i64,
    pub pack_name: String,
    pub items: Vec<PackItem>,
    pub cost: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackItem {
    pub name: String,
    pub description: Option<String>,
    pub rarity: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackPurchaseRequestEvent {
    pub user_id: String,
    pub username: String,
    pub pack_id: i64,
}
