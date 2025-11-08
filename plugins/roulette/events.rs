use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameStartedEvent {
    pub game_id: i64,
    pub channel: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BetPlacedEvent {
    pub channel: String,
    pub username: String,
    pub amount: i64,
    pub bet_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WheelSpunEvent {
    pub channel: String,
    pub winning_number: i64,
    pub winning_color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameEndedEvent {
    pub channel: String,
    pub game_id: i64,
    pub winning_number: i64,
    pub total_bets: usize,
    pub total_payout: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BetWonEvent {
    pub username: String,
    pub amount: i64,
    pub payout: i64,
    pub bet_type: String,
}
