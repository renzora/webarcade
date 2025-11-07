// Twitch command system
// Handles command parsing, permissions, and cooldowns

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct CommandManager {
    pub cooldowns: Arc<RwLock<HashMap<String, i64>>>,
}

impl CommandManager {
    pub fn new() -> Self {
        Self {
            cooldowns: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn check_cooldown(&self, command: &str, user_id: &str, cooldown_seconds: i64) -> bool {
        let key = format!("{}:{}", command, user_id);
        let now = current_timestamp();

        let mut cooldowns = self.cooldowns.write().await;

        if let Some(&last_used) = cooldowns.get(&key) {
            if now - last_used < cooldown_seconds {
                return false; // Still on cooldown
            }
        }

        cooldowns.insert(key, now);
        true
    }

    pub fn check_permission(&self, user_level: &str, required_level: &str) -> bool {
        let levels = ["everyone", "subscriber", "vip", "moderator", "broadcaster"];

        let user_idx = levels.iter().position(|&l| l == user_level).unwrap_or(0);
        let required_idx = levels.iter().position(|&l| l == required_level).unwrap_or(0);

        user_idx >= required_idx
    }
}

impl Default for CommandManager {
    fn default() -> Self {
        Self::new()
    }
}

fn current_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}
