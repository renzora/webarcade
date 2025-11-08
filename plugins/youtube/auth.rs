use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

const YOUTUBE_CLIENT_ID_KEY: &str = "youtube_client_id";
const YOUTUBE_CLIENT_SECRET_KEY: &str = "youtube_client_secret";
const YOUTUBE_REDIRECT_URI: &str = "http://localhost:3001/youtube/auth/callback";

// OAuth scopes needed for YouTube analytics
const YOUTUBE_SCOPES: &[&str] = &[
    "https://www.googleapis.com/auth/youtube.readonly",
    "https://www.googleapis.com/auth/yt-analytics.readonly",
    "https://www.googleapis.com/auth/userinfo.profile",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YouTubeAuth {
    pub user_id: String,
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: i64,
    pub scopes: Vec<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: i64,
    pub scope: String,
    pub token_type: String,
}

#[derive(Debug, Deserialize)]
pub struct UserInfo {
    pub id: String,
    pub name: Option<String>,
    pub email: Option<String>,
}

/// Get the YouTube OAuth authorization URL
pub fn get_auth_url(client_id: &str, state: Option<String>) -> String {
    let scopes = YOUTUBE_SCOPES.join(" ");
    let state_param = state.unwrap_or_else(|| "state".to_string());

    format!(
        "https://accounts.google.com/o/oauth2/v2/auth?\
        client_id={}&\
        redirect_uri={}&\
        response_type=code&\
        scope={}&\
        access_type=offline&\
        prompt=consent&\
        state={}",
        urlencoding::encode(client_id),
        urlencoding::encode(YOUTUBE_REDIRECT_URI),
        urlencoding::encode(&scopes),
        urlencoding::encode(&state_param)
    )
}

/// Exchange authorization code for access token
pub async fn exchange_code_for_token(
    code: &str,
    client_id: &str,
    client_secret: &str,
) -> Result<TokenResponse> {
    let client = reqwest::Client::new();

    let params = [
        ("code", code),
        ("client_id", client_id),
        ("client_secret", client_secret),
        ("redirect_uri", YOUTUBE_REDIRECT_URI),
        ("grant_type", "authorization_code"),
    ];

    let response = client
        .post("https://oauth2.googleapis.com/token")
        .form(&params)
        .send()
        .await?;

    if !response.status().is_success() {
        let error_text = response.text().await?;
        return Err(anyhow!("Failed to exchange code: {}", error_text));
    }

    let token_response: TokenResponse = response.json().await?;
    Ok(token_response)
}

/// Refresh the access token
pub async fn refresh_access_token(
    refresh_token: &str,
    client_id: &str,
    client_secret: &str,
) -> Result<TokenResponse> {
    let client = reqwest::Client::new();

    let params = [
        ("refresh_token", refresh_token),
        ("client_id", client_id),
        ("client_secret", client_secret),
        ("grant_type", "refresh_token"),
    ];

    let response = client
        .post("https://oauth2.googleapis.com/token")
        .form(&params)
        .send()
        .await?;

    if !response.status().is_success() {
        let error_text = response.text().await?;
        return Err(anyhow!("Failed to refresh token: {}", error_text));
    }

    let token_response: TokenResponse = response.json().await?;
    Ok(token_response)
}

/// Validate the access token
pub async fn validate_token(access_token: &str) -> Result<bool> {
    let client = reqwest::Client::new();

    let response = client
        .get("https://www.googleapis.com/oauth2/v1/tokeninfo")
        .query(&[("access_token", access_token)])
        .send()
        .await?;

    Ok(response.status().is_success())
}

/// Revoke the access token
pub async fn revoke_token(token: &str) -> Result<()> {
    let client = reqwest::Client::new();

    let params = [("token", token)];

    let response = client
        .post("https://oauth2.googleapis.com/revoke")
        .form(&params)
        .send()
        .await?;

    if !response.status().is_success() {
        let error_text = response.text().await?;
        return Err(anyhow!("Failed to revoke token: {}", error_text));
    }

    Ok(())
}

/// Get user info from Google
pub async fn get_user_info(access_token: &str) -> Result<UserInfo> {
    let client = reqwest::Client::new();

    let response = client
        .get("https://www.googleapis.com/oauth2/v2/userinfo")
        .bearer_auth(access_token)
        .send()
        .await?;

    if !response.status().is_success() {
        let error_text = response.text().await?;
        return Err(anyhow!("Failed to get user info: {}", error_text));
    }

    let user_info: UserInfo = response.json().await?;
    Ok(user_info)
}

/// Get current timestamp in seconds
pub fn current_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

/// Check if token is expired
pub fn is_token_expired(expires_at: i64) -> bool {
    current_timestamp() >= expires_at
}
