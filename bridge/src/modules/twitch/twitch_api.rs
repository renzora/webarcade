use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use super::twitch_auth::TwitchAuth;
use super::twitch_config::TwitchConfigManager;

const TWITCH_API_BASE: &str = "https://api.twitch.tv/helix";

/// Twitch user information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwitchUser {
    pub id: String,
    pub login: String,
    pub display_name: String,
    #[serde(rename = "type")]
    pub user_type: String,
    pub broadcaster_type: String,
    pub description: String,
    pub profile_image_url: String,
    pub offline_image_url: String,
    pub view_count: u64,
    pub created_at: String,
}

/// Stream information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamInfo {
    pub id: String,
    pub user_id: String,
    pub user_login: String,
    pub user_name: String,
    pub game_id: String,
    pub game_name: String,
    #[serde(rename = "type")]
    pub stream_type: String,
    pub title: String,
    pub viewer_count: u64,
    pub started_at: String,
    pub language: String,
    pub thumbnail_url: String,
    pub is_mature: bool,
    #[serde(default)]
    pub tag_ids: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Channel information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelInfo {
    pub broadcaster_id: String,
    pub broadcaster_login: String,
    pub broadcaster_name: String,
    pub broadcaster_language: String,
    pub game_id: String,
    pub game_name: String,
    pub title: String,
    pub delay: u32,
}

/// Follower information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Follower {
    pub user_id: String,
    pub user_login: String,
    pub user_name: String,
    pub followed_at: String,
}

/// Chatter information (user currently in chat)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chatter {
    pub user_id: String,
    pub user_login: String,
    pub user_name: String,
}

/// Subscription information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subscription {
    pub user_id: String,
    pub user_login: String,
    pub user_name: String,
    pub tier: String,
    pub is_gift: bool,
}

/// Generic API response wrapper
#[derive(Debug, Deserialize)]
struct ApiResponse<T> {
    data: Vec<T>,
    #[serde(default)]
    pagination: Option<Pagination>,
}

#[derive(Debug, Deserialize)]
struct Pagination {
    cursor: Option<String>,
}

/// Twitch API client
pub struct TwitchAPI {
    config_manager: Arc<TwitchConfigManager>,
    auth: Arc<TwitchAuth>,
    http_client: Client,
}

impl TwitchAPI {
    /// Create a new Twitch API client
    pub fn new(config_manager: Arc<TwitchConfigManager>, auth: Arc<TwitchAuth>) -> Self {
        Self {
            config_manager,
            auth,
            http_client: Client::new(),
        }
    }

    /// Make an authenticated API request
    async fn make_request<T: for<'de> Deserialize<'de>>(
        &self,
        endpoint: &str,
    ) -> Result<ApiResponse<T>> {
        let config = self.config_manager.load()?;
        let access_token = self.auth.get_valid_token().await?;

        let url = format!("{}{}", TWITCH_API_BASE, endpoint);

        let response = self
            .http_client
            .get(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Client-Id", &config.client_id)
            .send()
            .await
            .context("Failed to make API request")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            log::error!("Twitch API request failed - Status: {}, URL: {}, Error: {}", status, url, error_text);
            anyhow::bail!("API request failed ({}): {}", status, error_text);
        }

        let response_text = response.text().await.context("Failed to read response body")?;
        log::debug!("API Response body: {}", response_text);

        serde_json::from_str::<ApiResponse<T>>(&response_text)
            .with_context(|| format!("Failed to parse API response: {}", response_text))
    }

    /// Get user information by username
    pub async fn get_user_by_login(&self, login: &str) -> Result<Option<TwitchUser>> {
        let endpoint = format!("/users?login={}", login);
        let response: ApiResponse<TwitchUser> = self.make_request(&endpoint).await?;

        Ok(response.data.into_iter().next())
    }

    /// Get user information by user ID
    pub async fn get_user_by_id(&self, user_id: &str) -> Result<Option<TwitchUser>> {
        let endpoint = format!("/users?id={}", user_id);
        let response: ApiResponse<TwitchUser> = self.make_request(&endpoint).await?;

        Ok(response.data.into_iter().next())
    }

    /// Get authenticated user information
    pub async fn get_authenticated_user(&self) -> Result<TwitchUser> {
        log::info!("Getting authenticated user info...");
        let endpoint = "/users";

        match self.make_request::<TwitchUser>(endpoint).await {
            Ok(response) => {
                log::info!("Got API response with {} users", response.data.len());
                response
                    .data
                    .into_iter()
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("No user data returned"))
            }
            Err(e) => {
                log::error!("Failed to make API request for authenticated user: {:?}", e);
                Err(e)
            }
        }
    }

    /// Get stream information for a user
    pub async fn get_stream(&self, user_login: &str) -> Result<Option<StreamInfo>> {
        let endpoint = format!("/streams?user_login={}", user_login);
        let response: ApiResponse<StreamInfo> = self.make_request(&endpoint).await?;

        Ok(response.data.into_iter().next())
    }

    /// Get multiple streams
    pub async fn get_streams(&self, user_logins: Vec<&str>) -> Result<Vec<StreamInfo>> {
        let logins_param = user_logins
            .iter()
            .map(|login| format!("user_login={}", login))
            .collect::<Vec<_>>()
            .join("&");

        let endpoint = format!("/streams?{}", logins_param);
        let response: ApiResponse<StreamInfo> = self.make_request(&endpoint).await?;

        Ok(response.data)
    }

    /// Check if a user is live
    pub async fn is_live(&self, user_login: &str) -> Result<bool> {
        Ok(self.get_stream(user_login).await?.is_some())
    }

    /// Get channel information
    pub async fn get_channel(&self, broadcaster_id: &str) -> Result<Option<ChannelInfo>> {
        let endpoint = format!("/channels?broadcaster_id={}", broadcaster_id);
        let response: ApiResponse<ChannelInfo> = self.make_request(&endpoint).await?;

        Ok(response.data.into_iter().next())
    }

    /// Get followers for a broadcaster
    pub async fn get_followers(
        &self,
        broadcaster_id: &str,
        limit: Option<u32>,
    ) -> Result<Vec<Follower>> {
        let limit = limit.unwrap_or(20).min(100);
        let endpoint = format!(
            "/channels/followers?broadcaster_id={}&first={}",
            broadcaster_id, limit
        );

        let response: ApiResponse<Follower> = self.make_request(&endpoint).await?;

        Ok(response.data)
    }

    /// Check if a specific user follows the broadcaster and when they followed
    /// Returns None if the user doesn't follow, or Some(followed_at) if they do
    pub async fn get_user_follow_status(
        &self,
        broadcaster_id: &str,
        user_id: &str,
    ) -> Result<Option<String>> {
        let endpoint = format!(
            "/channels/followers?broadcaster_id={}&user_id={}",
            broadcaster_id, user_id
        );

        let response: ApiResponse<Follower> = self.make_request(&endpoint).await?;

        // If data is not empty, user is following
        Ok(response.data.into_iter().next().map(|f| f.followed_at))
    }

    /// Get list of users currently in chat (including lurkers)
    /// Requires moderator:read:chatters scope
    /// Returns up to 1000 chatters (can paginate for more)
    pub async fn get_chatters(
        &self,
        broadcaster_id: &str,
        moderator_id: &str,
    ) -> Result<Vec<Chatter>> {
        let endpoint = format!(
            "/chat/chatters?broadcaster_id={}&moderator_id={}&first=1000",
            broadcaster_id, moderator_id
        );

        let response: ApiResponse<Chatter> = self.make_request(&endpoint).await?;

        Ok(response.data)
    }

    /// Get subscriber count (requires appropriate scope)
    pub async fn get_subscriptions(
        &self,
        broadcaster_id: &str,
        limit: Option<u32>,
    ) -> Result<Vec<Subscription>> {
        let limit = limit.unwrap_or(20).min(100);
        let endpoint = format!(
            "/subscriptions?broadcaster_id={}&first={}",
            broadcaster_id, limit
        );

        let response: ApiResponse<Subscription> = self.make_request(&endpoint).await?;

        Ok(response.data)
    }

    /// Update channel information (title, game, etc.)
    pub async fn update_channel(
        &self,
        broadcaster_id: &str,
        title: Option<&str>,
        game_id: Option<&str>,
    ) -> Result<()> {
        let config = self.config_manager.load()?;
        let access_token = self.auth.get_valid_token().await?;

        let url = format!(
            "{}/channels?broadcaster_id={}",
            TWITCH_API_BASE, broadcaster_id
        );

        #[derive(Serialize)]
        struct UpdateRequest<'a> {
            #[serde(skip_serializing_if = "Option::is_none")]
            title: Option<&'a str>,
            #[serde(skip_serializing_if = "Option::is_none")]
            game_id: Option<&'a str>,
        }

        let body = UpdateRequest { title, game_id };

        let response = self
            .http_client
            .patch(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Client-Id", &config.client_id)
            .json(&body)
            .send()
            .await
            .context("Failed to update channel")?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("Failed to update channel: {}", error_text);
        }

        log::info!("Successfully updated channel information");

        Ok(())
    }

    /// Search for games
    pub async fn search_games(&self, query: &str) -> Result<Vec<GameInfo>> {
        let endpoint = format!("/search/categories?query={}", query);
        let response: ApiResponse<GameInfo> = self.make_request(&endpoint).await?;

        Ok(response.data)
    }

    /// Get game information by ID
    pub async fn get_game(&self, game_id: &str) -> Result<Option<GameInfo>> {
        let endpoint = format!("/games?id={}", game_id);
        let response: ApiResponse<GameInfo> = self.make_request(&endpoint).await?;

        Ok(response.data.into_iter().next())
    }

    /// Ban a user from chat (requires moderator permissions)
    pub async fn ban_user(
        &self,
        broadcaster_id: &str,
        moderator_id: &str,
        user_id: &str,
        reason: Option<&str>,
        duration: Option<u32>,
    ) -> Result<()> {
        let config = self.config_manager.load()?;
        let access_token = self.auth.get_valid_token().await?;

        let url = format!("{}/moderation/bans", TWITCH_API_BASE);

        #[derive(Serialize)]
        struct BanRequest<'a> {
            user_id: &'a str,
            #[serde(skip_serializing_if = "Option::is_none")]
            reason: Option<&'a str>,
            #[serde(skip_serializing_if = "Option::is_none")]
            duration: Option<u32>,
        }

        #[derive(Serialize)]
        struct BanBody<'a> {
            data: BanRequest<'a>,
        }

        let body = BanBody {
            data: BanRequest {
                user_id,
                reason,
                duration,
            },
        };

        let response = self
            .http_client
            .post(&url)
            .query(&[
                ("broadcaster_id", broadcaster_id),
                ("moderator_id", moderator_id),
            ])
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Client-Id", &config.client_id)
            .json(&body)
            .send()
            .await
            .context("Failed to ban user")?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("Failed to ban user: {}", error_text);
        }

        log::info!("Successfully banned user: {}", user_id);

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameInfo {
    pub id: String,
    pub name: String,
    pub box_art_url: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_twitch_user_deserialization() {
        let json = r#"{
            "id": "12345",
            "login": "testuser",
            "display_name": "TestUser",
            "user_type": "",
            "broadcaster_type": "affiliate",
            "description": "Test description",
            "profile_image_url": "https://example.com/image.jpg",
            "offline_image_url": "https://example.com/offline.jpg",
            "view_count": 1000,
            "created_at": "2020-01-01T00:00:00Z"
        }"#;

        let user: TwitchUser = serde_json::from_str(json).unwrap();
        assert_eq!(user.id, "12345");
        assert_eq!(user.login, "testuser");
    }
}
