// Auction plugin event types
// These are NOT in core - they're specific to this plugin

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuctionCreatedEvent {
    pub auction_id: i64,
    pub item_name: String,
    pub item_rarity: String,
    pub starting_bid: i64,
    pub created_by: String,
    pub ends_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BidPlacedEvent {
    pub auction_id: i64,
    pub bidder: String,
    pub amount: i64,
    pub previous_bid: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuctionEndedEvent {
    pub auction_id: i64,
    pub winner: Option<String>,
    pub final_bid: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAuctionRequest {
    pub item_id: String,
    pub item_name: String,
    pub item_rarity: String,
    pub starting_bid: i64,
    pub created_by: String,
    pub duration_seconds: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaceBidRequest {
    pub auction_id: i64,
    pub user_id: String,
    pub username: String,
    pub amount: i64,
}
