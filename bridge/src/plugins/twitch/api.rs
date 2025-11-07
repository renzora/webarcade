// Twitch API client implementation
// Handles calls to the Twitch Helix API

use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwitchApiClient {
    pub client_id: String,
    pub access_token: String,
    pub api_base_url: String,
}

impl TwitchApiClient {
    pub fn new(client_id: String, access_token: String) -> Self {
        Self {
            client_id,
            access_token,
            api_base_url: "https://api.twitch.tv/helix".to_string(),
        }
    }

    pub async fn get_authenticated_user(&self) -> Result<serde_json::Value> {
        let client = Client::new();
        let url = format!("{}/users", self.api_base_url);

        log::info!("[Twitch API] Getting authenticated user");

        let response = client
            .get(&url)
            .header("Client-ID", &self.client_id)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            anyhow::bail!("Failed to get authenticated user: {}", error_text);
        }

        Ok(response.json().await?)
    }

    pub async fn get_user_by_login(&self, login: &str) -> Result<serde_json::Value> {
        let client = Client::new();
        let url = format!("{}/users?login={}", self.api_base_url, login);

        log::info!("[Twitch API] Getting user: {}", login);

        let response = client
            .get(&url)
            .header("Client-ID", &self.client_id)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            anyhow::bail!("Failed to get user: {}", error_text);
        }

        Ok(response.json().await?)
    }

    pub async fn get_channel_info(&self, broadcaster_id: &str) -> Result<serde_json::Value> {
        let client = Client::new();
        let url = format!("{}/channels?broadcaster_id={}", self.api_base_url, broadcaster_id);

        log::info!("[Twitch API] Getting channel info: {}", broadcaster_id);

        let response = client
            .get(&url)
            .header("Client-ID", &self.client_id)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            anyhow::bail!("Failed to get channel info: {}", error_text);
        }

        Ok(response.json().await?)
    }

    pub async fn get_streams(&self, user_login: &str) -> Result<serde_json::Value> {
        let client = Client::new();
        let url = format!("{}/streams?user_login={}", self.api_base_url, user_login);

        log::info!("[Twitch API] Getting streams for: {}", user_login);

        let response = client
            .get(&url)
            .header("Client-ID", &self.client_id)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            anyhow::bail!("Failed to get streams: {}", error_text);
        }

        Ok(response.json().await?)
    }

    pub async fn get_schedule(&self, broadcaster_id: &str) -> Result<serde_json::Value> {
        let client = Client::new();
        let url = format!("{}/schedule?broadcaster_id={}", self.api_base_url, broadcaster_id);

        log::info!("[Twitch API] Getting schedule for: {}", broadcaster_id);

        let response = client
            .get(&url)
            .header("Client-ID", &self.client_id)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .send()
            .await?;

        if !response.status().is_success() {
            // Schedule might not be set up, return empty schedule instead of error
            if response.status() == 404 {
                return Ok(serde_json::json!({
                    "data": {
                        "segments": [],
                        "broadcaster_id": broadcaster_id,
                        "broadcaster_name": "",
                        "broadcaster_login": "",
                        "vacation": null
                    }
                }));
            }
            let error_text = response.text().await?;
            anyhow::bail!("Failed to get schedule: {}", error_text);
        }

        Ok(response.json().await?)
    }

    pub async fn modify_channel_info(&self, broadcaster_id: &str, title: Option<&str>, game_id: Option<&str>) -> Result<()> {
        let client = Client::new();
        let url = format!("{}/channels?broadcaster_id={}", self.api_base_url, broadcaster_id);

        let mut body = serde_json::Map::new();
        if let Some(t) = title {
            body.insert("title".to_string(), serde_json::Value::String(t.to_string()));
        }
        if let Some(g) = game_id {
            body.insert("game_id".to_string(), serde_json::Value::String(g.to_string()));
        }

        log::info!("[Twitch API] Modifying channel: {}", broadcaster_id);

        let response = client
            .patch(&url)
            .header("Client-ID", &self.client_id)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            anyhow::bail!("Failed to modify channel: {}", error_text);
        }

        Ok(())
    }

    pub async fn send_chat_announcement(&self, broadcaster_id: &str, moderator_id: &str, message: &str) -> Result<()> {
        let client = Client::new();
        let url = format!(
            "{}/chat/announcements?broadcaster_id={}&moderator_id={}",
            self.api_base_url, broadcaster_id, moderator_id
        );

        let body = serde_json::json!({
            "message": message
        });

        log::info!("[Twitch API] Sending announcement to {}: {}", broadcaster_id, message);

        let response = client
            .post(&url)
            .header("Client-ID", &self.client_id)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            anyhow::bail!("Failed to send announcement: {}", error_text);
        }

        Ok(())
    }

    pub async fn get_channel_followers(&self, broadcaster_id: &str) -> Result<i64> {
        let client = Client::new();
        let url = format!("{}/channels/followers?broadcaster_id={}", self.api_base_url, broadcaster_id);

        log::info!("[Twitch API] Getting follower count for broadcaster: {}", broadcaster_id);

        let response = client
            .get(&url)
            .header("Client-ID", &self.client_id)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            anyhow::bail!("Failed to get followers: {}", error_text);
        }

        let json: serde_json::Value = response.json().await?;
        let total = json.get("total").and_then(|v| v.as_i64()).unwrap_or(0);

        Ok(total)
    }

    pub async fn get_broadcaster_subscriptions(&self, broadcaster_id: &str) -> Result<i64> {
        let client = Client::new();
        let url = format!("{}/subscriptions?broadcaster_id={}", self.api_base_url, broadcaster_id);

        log::info!("[Twitch API] Getting subscriber count for broadcaster: {}", broadcaster_id);

        let response = client
            .get(&url)
            .header("Client-ID", &self.client_id)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            anyhow::bail!("Failed to get subscribers: {}", error_text);
        }

        let json: serde_json::Value = response.json().await?;
        // The subscriptions endpoint returns points, not count directly
        // We need to count the data array items and handle pagination if needed
        let data = json.get("data").and_then(|v| v.as_array());
        let points = json.get("points").and_then(|v| v.as_i64()).unwrap_or(0);

        // If we have points, that's the total subscription points
        // Otherwise, count the data array
        if points > 0 {
            Ok(points)
        } else if let Some(arr) = data {
            Ok(arr.len() as i64)
        } else {
            Ok(0)
        }
    }
}
