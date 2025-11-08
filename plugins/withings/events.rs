use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeightRecordedEvent {
    pub user_id: String,
    pub weight_kg: f64,
    pub measured_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityRecordedEvent {
    pub user_id: String,
    pub date: String,
    pub steps: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SleepRecordedEvent {
    pub user_id: String,
    pub date: String,
    pub total_sleep_minutes: i64,
    pub sleep_score: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalAchievedEvent {
    pub user_id: String,
    pub goal_type: String,
    pub goal_value: f64,
}
