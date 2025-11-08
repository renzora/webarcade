// Twitch OAuth2 authentication implementation

use anyhow::Result;
use serde::{Deserialize, Serialize};
use reqwest::Client;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwitchAuth {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwitchTokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
    pub scope: Vec<String>,
    pub token_type: String,
}

#[derive(Debug, Deserialize)]
struct TwitchTokenApiResponse {
    access_token: String,
    refresh_token: Option<String>,
    expires_in: i64,
    scope: Vec<String>,
    token_type: String,
}

impl TwitchAuth {
    pub fn new(client_id: String, client_secret: String, redirect_uri: String) -> Self {
        Self {
            client_id,
            client_secret,
            redirect_uri,
        }
    }

    pub fn get_authorization_url(&self, scopes: &[&str], state: &str) -> String {
        let scope_str = scopes.join(" ");
        format!(
            "https://id.twitch.tv/oauth2/authorize?client_id={}&redirect_uri={}&response_type=code&scope={}&state={}",
            self.client_id,
            urlencoding::encode(&self.redirect_uri),
            urlencoding::encode(&scope_str),
            urlencoding::encode(state)
        )
    }

    pub async fn exchange_code_for_token(&self, code: &str) -> Result<TwitchTokenResponse> {
        let client = Client::new();

        let params = [
            ("client_id", self.client_id.as_str()),
            ("client_secret", self.client_secret.as_str()),
            ("code", code),
            ("grant_type", "authorization_code"),
            ("redirect_uri", self.redirect_uri.as_str()),
        ];

        log::info!("[Twitch Auth] Exchanging code for token");

        let response = client
            .post("https://id.twitch.tv/oauth2/token")
            .form(&params)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            anyhow::bail!("Failed to exchange code for token: {}", error_text);
        }

        let api_response: TwitchTokenApiResponse = response.json().await?;

        Ok(TwitchTokenResponse {
            access_token: api_response.access_token,
            refresh_token: api_response.refresh_token.unwrap_or_default(),
            expires_in: api_response.expires_in,
            scope: api_response.scope,
            token_type: api_response.token_type,
        })
    }

    pub async fn refresh_access_token(&self, refresh_token: &str) -> Result<TwitchTokenResponse> {
        let client = Client::new();

        let params = [
            ("client_id", self.client_id.as_str()),
            ("client_secret", self.client_secret.as_str()),
            ("refresh_token", refresh_token),
            ("grant_type", "refresh_token"),
        ];

        log::info!("[Twitch Auth] Refreshing access token");

        let response = client
            .post("https://id.twitch.tv/oauth2/token")
            .form(&params)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            anyhow::bail!("Failed to refresh token: {}", error_text);
        }

        let api_response: TwitchTokenApiResponse = response.json().await?;

        Ok(TwitchTokenResponse {
            access_token: api_response.access_token,
            refresh_token: api_response.refresh_token.unwrap_or_else(|| refresh_token.to_string()),
            expires_in: api_response.expires_in,
            scope: api_response.scope,
            token_type: api_response.token_type,
        })
    }

    pub async fn validate_token(&self, access_token: &str) -> Result<bool> {
        let client = Client::new();

        log::info!("[Twitch Auth] Validating token");

        let response = client
            .get("https://id.twitch.tv/oauth2/validate")
            .header("Authorization", format!("OAuth {}", access_token))
            .send()
            .await?;

        Ok(response.status().is_success())
    }

    pub async fn revoke_token(&self, access_token: &str) -> Result<()> {
        let client = Client::new();

        let params = [
            ("client_id", self.client_id.as_str()),
            ("token", access_token),
        ];

        log::info!("[Twitch Auth] Revoking token");

        let response = client
            .post("https://id.twitch.tv/oauth2/revoke")
            .form(&params)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            anyhow::bail!("Failed to revoke token: {}", error_text);
        }

        Ok(())
    }
}
