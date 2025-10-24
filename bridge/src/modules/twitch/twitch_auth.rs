use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use url::Url;

use super::twitch_config::{TwitchConfig, TwitchConfigManager};

const TWITCH_AUTH_URL: &str = "https://id.twitch.tv/oauth2/authorize";
const TWITCH_TOKEN_URL: &str = "https://id.twitch.tv/oauth2/token";
const TWITCH_VALIDATE_URL: &str = "https://id.twitch.tv/oauth2/validate";
const REDIRECT_URI: &str = "http://localhost:3001/twitch/callback";

/// OAuth token response from Twitch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
    pub scope: Vec<String>,
    pub token_type: String,
}

/// Token validation response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateResponse {
    pub client_id: String,
    pub login: String,
    pub scopes: Vec<String>,
    pub user_id: String,
    pub expires_in: i64,
}

/// OAuth state for CSRF protection
#[derive(Debug, Clone)]
pub struct OAuthState {
    pub state: String,
    pub created_at: i64,
}

/// Twitch OAuth authentication manager
pub struct TwitchAuth {
    config_manager: Arc<TwitchConfigManager>,
    http_client: Client,
    oauth_state: Arc<RwLock<Option<OAuthState>>>,
}

impl TwitchAuth {
    /// Create a new Twitch authentication manager
    pub fn new(config_manager: Arc<TwitchConfigManager>) -> Self {
        Self {
            config_manager,
            http_client: Client::new(),
            oauth_state: Arc::new(RwLock::new(None)),
        }
    }

    /// Generate OAuth authorization URL
    pub async fn generate_auth_url(&self, scopes: Vec<String>) -> Result<String> {
        let config = self.config_manager.load()?;

        if config.client_id.is_empty() {
            anyhow::bail!("Client ID not configured");
        }

        // Generate random state for CSRF protection
        let state: String = (0..32)
            .map(|_| format!("{:02x}", rand::random::<u8>()))
            .collect();

        // Store state
        {
            let mut oauth_state = self.oauth_state.write().await;
            *oauth_state = Some(OAuthState {
                state: state.clone(),
                created_at: chrono::Utc::now().timestamp(),
            });
        }

        let scope_string = scopes.join(" ");

        let mut url = Url::parse(TWITCH_AUTH_URL)?;
        url.query_pairs_mut()
            .append_pair("client_id", &config.client_id)
            .append_pair("redirect_uri", REDIRECT_URI)
            .append_pair("response_type", "code")
            .append_pair("scope", &scope_string)
            .append_pair("state", &state);

        Ok(url.to_string())
    }

    /// Verify OAuth state (CSRF protection)
    pub async fn verify_state(&self, state: &str) -> Result<bool> {
        let oauth_state_guard = self.oauth_state.read().await;

        if let Some(ref stored_state) = *oauth_state_guard {
            // Check if state matches and is not too old (10 minutes)
            let now = chrono::Utc::now().timestamp();
            let is_valid = stored_state.state == state && (now - stored_state.created_at) < 600;

            Ok(is_valid)
        } else {
            Ok(false)
        }
    }

    /// Exchange authorization code for access token
    pub async fn exchange_code(&self, code: &str) -> Result<TokenResponse> {
        let config = self.config_manager.load()?;

        if config.client_id.is_empty() || config.client_secret.is_empty() {
            anyhow::bail!("Client ID or Secret not configured");
        }

        let params = [
            ("client_id", config.client_id.as_str()),
            ("client_secret", config.client_secret.as_str()),
            ("code", code),
            ("grant_type", "authorization_code"),
            ("redirect_uri", REDIRECT_URI),
        ];

        let response = self
            .http_client
            .post(TWITCH_TOKEN_URL)
            .form(&params)
            .send()
            .await
            .context("Failed to exchange code for token")?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("Token exchange failed: {}", error_text);
        }

        let token_response: TokenResponse = response
            .json()
            .await
            .context("Failed to parse token response")?;

        // Save tokens to config
        self.config_manager.update_tokens(
            token_response.access_token.clone(),
            token_response.refresh_token.clone(),
            token_response.expires_in,
        )?;

        // Clear OAuth state
        {
            let mut oauth_state = self.oauth_state.write().await;
            *oauth_state = None;
        }

        log::info!("Successfully obtained Twitch OAuth tokens");

        Ok(token_response)
    }

    /// Refresh access token using refresh token
    pub async fn refresh_token(&self) -> Result<TokenResponse> {
        let config = self.config_manager.load()?;

        let refresh_token = config
            .refresh_token
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No refresh token available"))?;

        if config.client_id.is_empty() || config.client_secret.is_empty() {
            anyhow::bail!("Client ID or Secret not configured");
        }

        let params = [
            ("client_id", config.client_id.as_str()),
            ("client_secret", config.client_secret.as_str()),
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token.as_str()),
        ];

        let response = self
            .http_client
            .post(TWITCH_TOKEN_URL)
            .form(&params)
            .send()
            .await
            .context("Failed to refresh token")?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("Token refresh failed: {}", error_text);
        }

        let token_response: TokenResponse = response
            .json()
            .await
            .context("Failed to parse refresh token response")?;

        // Save new tokens
        self.config_manager.update_tokens(
            token_response.access_token.clone(),
            token_response.refresh_token.clone(),
            token_response.expires_in,
        )?;

        log::info!("Successfully refreshed Twitch OAuth tokens");

        Ok(token_response)
    }

    /// Validate current access token
    pub async fn validate_token(&self) -> Result<ValidateResponse> {
        let config = self.config_manager.load()?;

        let access_token = config
            .access_token
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No access token available"))?;

        let response = self
            .http_client
            .get(TWITCH_VALIDATE_URL)
            .header("Authorization", format!("OAuth {}", access_token))
            .send()
            .await
            .context("Failed to validate token")?;

        if !response.status().is_success() {
            anyhow::bail!("Token validation failed");
        }

        let validate_response: ValidateResponse = response
            .json()
            .await
            .context("Failed to parse validation response")?;

        Ok(validate_response)
    }

    /// Get valid access token, refreshing if necessary
    pub async fn get_valid_token(&self) -> Result<String> {
        // Check if token is expired
        if self.config_manager.is_token_expired()? {
            log::info!("Access token expired, refreshing...");
            let token_response = self.refresh_token().await?;
            Ok(token_response.access_token)
        } else {
            let config = self.config_manager.load()?;
            config
                .access_token
                .ok_or_else(|| anyhow::anyhow!("No access token available"))
        }
    }

    /// Revoke access token (logout)
    pub async fn revoke_token(&self) -> Result<()> {
        let config = self.config_manager.load()?;

        if let Some(access_token) = config.access_token {
            let params = [
                ("client_id", config.client_id.as_str()),
                ("token", access_token.as_str()),
            ];

            let response = self
                .http_client
                .post("https://id.twitch.tv/oauth2/revoke")
                .form(&params)
                .send()
                .await
                .context("Failed to revoke token")?;

            if !response.status().is_success() {
                log::warn!("Token revocation failed, but continuing...");
            }
        }

        // Clear tokens from config
        let mut config = self.config_manager.load()?;
        config.access_token = None;
        config.refresh_token = None;
        config.token_expires_at = None;
        self.config_manager.save(&config)?;

        log::info!("Successfully revoked and cleared Twitch tokens");

        Ok(())
    }

    /// Check if user is authenticated
    pub fn is_authenticated(&self) -> Result<bool> {
        let config = self.config_manager.load()?;
        let has_token = config.access_token.is_some();
        let is_expired = self.config_manager.is_token_expired()?;

        log::info!("Authentication check - has_token: {}, is_expired: {}", has_token, is_expired);

        if has_token && is_expired {
            log::warn!("Token is expired, attempting to refresh...");
        }

        Ok(has_token && !is_expired)
    }

    /// Get default scopes for bot functionality (ALL SCOPES)
    pub fn get_default_scopes() -> Vec<String> {
        vec![
            // Analytics
            "analytics:read:extensions".to_string(),
            "analytics:read:games".to_string(),

            // Bits
            "bits:read".to_string(),

            // Channel
            "channel:bot".to_string(),
            "channel:edit:commercial".to_string(),
            "channel:manage:ads".to_string(),
            "channel:manage:broadcast".to_string(),
            "channel:manage:extensions".to_string(),
            "channel:manage:moderators".to_string(),
            "channel:manage:polls".to_string(),
            "channel:manage:predictions".to_string(),
            "channel:manage:raids".to_string(),
            "channel:manage:redemptions".to_string(),
            "channel:manage:schedule".to_string(),
            "channel:manage:videos".to_string(),
            "channel:manage:vips".to_string(),
            "channel:moderate".to_string(),
            "channel:read:ads".to_string(),
            "channel:read:charity".to_string(),
            "channel:read:editors".to_string(),
            "channel:read:goals".to_string(),
            "channel:read:guest_star".to_string(),
            "channel:read:hype_train".to_string(),
            "channel:read:polls".to_string(),
            "channel:read:predictions".to_string(),
            "channel:read:redemptions".to_string(),
            "channel:read:stream_key".to_string(),
            "channel:read:subscriptions".to_string(),
            "channel:read:vips".to_string(),

            // Chat
            "chat:edit".to_string(),
            "chat:read".to_string(),

            // Clips
            "clips:edit".to_string(),

            // Moderation
            "moderation:read".to_string(),
            "moderator:manage:announcements".to_string(),
            "moderator:manage:automod".to_string(),
            "moderator:manage:automod_settings".to_string(),
            "moderator:manage:banned_users".to_string(),
            "moderator:manage:blocked_terms".to_string(),
            "moderator:manage:chat_messages".to_string(),
            "moderator:manage:chat_settings".to_string(),
            "moderator:manage:guest_star".to_string(),
            "moderator:manage:shield_mode".to_string(),
            "moderator:manage:shoutouts".to_string(),
            "moderator:manage:unban_requests".to_string(),
            "moderator:manage:warnings".to_string(),
            "moderator:read:automod_settings".to_string(),
            "moderator:read:blocked_terms".to_string(),
            "moderator:read:chat_settings".to_string(),
            "moderator:read:chatters".to_string(),
            "moderator:read:followers".to_string(),
            "moderator:read:guest_star".to_string(),
            "moderator:read:shield_mode".to_string(),
            "moderator:read:shoutouts".to_string(),
            "moderator:read:suspicious_users".to_string(),
            "moderator:read:unban_requests".to_string(),
            "moderator:read:warnings".to_string(),

            // User
            "user:bot".to_string(),
            "user:edit".to_string(),
            "user:edit:broadcast".to_string(),
            "user:edit:follows".to_string(),
            "user:manage:blocked_users".to_string(),
            "user:manage:chat_color".to_string(),
            "user:manage:whispers".to_string(),
            "user:read:blocked_users".to_string(),
            "user:read:broadcast".to_string(),
            "user:read:email".to_string(),
            "user:read:follows".to_string(),
            "user:read:subscriptions".to_string(),
            "user:write:chat".to_string(),

            // Whispers
            "whispers:edit".to_string(),
            "whispers:read".to_string(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_generate_auth_url() {
        let config_manager = Arc::new(TwitchConfigManager::new());

        // Create a test config
        let mut config = TwitchConfig::default();
        config.client_id = "test_client_id".to_string();
        config_manager.save(&config).unwrap();

        let auth = TwitchAuth::new(config_manager);
        let scopes = vec!["chat:read".to_string(), "chat:edit".to_string()];

        let url = auth.generate_auth_url(scopes).await.unwrap();

        assert!(url.contains("client_id=test_client_id"));
        assert!(url.contains("chat:read"));
        assert!(url.contains("state="));
    }

    #[tokio::test]
    async fn test_state_verification() {
        let config_manager = Arc::new(TwitchConfigManager::new());
        let auth = TwitchAuth::new(config_manager);

        // Set a test state
        {
            let mut oauth_state = auth.oauth_state.write().await;
            *oauth_state = Some(OAuthState {
                state: "test_state_123".to_string(),
                created_at: chrono::Utc::now().timestamp(),
            });
        }

        // Verify correct state
        assert!(auth.verify_state("test_state_123").await.unwrap());

        // Verify incorrect state
        assert!(!auth.verify_state("wrong_state").await.unwrap());
    }
}
