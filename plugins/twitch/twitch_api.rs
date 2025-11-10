use anyhow::{Result, anyhow};
use serde_json::Value;
use crate::core::plugin_context::PluginContext;

// Twitch OAuth configuration - now loaded from database
const TWITCH_REDIRECT_URI: &str = "http://localhost:3001/twitch/auth/callback";

// Twitch OAuth scopes
const BROADCASTER_SCOPES: &[&str] = &[
    "channel:read:subscriptions",
    "channel:read:redemptions",
    "channel:manage:redemptions",
    "channel:read:hype_train",
    "channel:read:polls",
    "channel:manage:polls",
    "channel:read:predictions",
    "channel:manage:predictions",
    "moderator:read:followers",
    "moderator:read:chatters",
    "chat:read",
    "chat:edit",
    "whispers:read",
    "whispers:edit",
];

const BOT_SCOPES: &[&str] = &[
    "chat:read",
    "chat:edit",
    "whispers:read",
    "whispers:edit",
];

// Helper function to get client ID from database
fn get_client_id() -> Result<String> {
    let db_path = crate::core::database::get_database_path();
    let conn = rusqlite::Connection::open(&db_path)?;

    let client_id: String = conn.query_row(
        "SELECT value FROM twitch_settings WHERE key = 'client_id'",
        [],
        |row| row.get(0)
    )?;

    Ok(client_id)
}

// Helper function to get client secret from database
fn get_client_secret() -> Result<String> {
    let db_path = crate::core::database::get_database_path();
    let conn = rusqlite::Connection::open(&db_path)?;

    let client_secret: String = conn.query_row(
        "SELECT value FROM twitch_settings WHERE key = 'client_secret'",
        [],
        |row| row.get(0)
    )?;

    Ok(client_secret)
}

pub fn generate_auth_url(account_type: &str) -> Result<String> {
    let client_id = get_client_id()?;

    let scopes = if account_type == "broadcaster" {
        BROADCASTER_SCOPES.join(" ")
    } else {
        BOT_SCOPES.join(" ")
    };

    Ok(format!(
        "https://id.twitch.tv/oauth2/authorize?client_id={}&redirect_uri={}&response_type=code&scope={}&state={}",
        client_id,
        urlencoding::encode(TWITCH_REDIRECT_URI),
        urlencoding::encode(&scopes),
        account_type
    ))
}

pub async fn exchange_code_for_token(code: &str, account_type: &str) -> Result<Value> {
    let client_id = get_client_id()?;
    let client_secret = get_client_secret()?;
    let client = reqwest::Client::new();

    let response = client
        .post("https://id.twitch.tv/oauth2/token")
        .form(&[
            ("client_id", client_id.as_str()),
            ("client_secret", client_secret.as_str()),
            ("code", code),
            ("grant_type", "authorization_code"),
            ("redirect_uri", TWITCH_REDIRECT_URI),
        ])
        .send()
        .await?;

    if !response.status().is_success() {
        let error_text = response.text().await?;
        return Err(anyhow!("Failed to exchange code: {}", error_text));
    }

    let token_data: Value = response.json().await?;

    let access_token = token_data["access_token"]
        .as_str()
        .ok_or_else(|| anyhow!("Missing access_token"))?;
    let refresh_token_str = token_data["refresh_token"]
        .as_str()
        .ok_or_else(|| anyhow!("Missing refresh_token"))?;
    let expires_in = token_data["expires_in"]
        .as_i64()
        .ok_or_else(|| anyhow!("Missing expires_in"))?;

    // Get user info
    let user_info = get_user_info_from_token(access_token).await?;

    let username = user_info["login"]
        .as_str()
        .ok_or_else(|| anyhow!("Missing username"))?;
    let user_id = user_info["id"]
        .as_str()
        .ok_or_else(|| anyhow!("Missing user_id"))?;

    // Save to database
    let db_path = crate::core::database::get_database_path();
    let conn = rusqlite::Connection::open(&db_path)?;

    let timestamp = current_timestamp();
    let expires_at = timestamp + expires_in;
    let scopes = if account_type == "broadcaster" {
        BROADCASTER_SCOPES.join(",")
    } else {
        BOT_SCOPES.join(",")
    };

    // Check if account already exists
    let exists: bool = conn.query_row(
        "SELECT COUNT(*) FROM twitch_accounts WHERE account_type = ?1",
        [account_type],
        |row| {
            let count: i64 = row.get(0)?;
            Ok(count > 0)
        }
    )?;

    if exists {
        // Update existing account
        conn.execute(
            "UPDATE twitch_accounts
             SET username = ?2, user_id = ?3, access_token = ?4, refresh_token = ?5,
                 token_expires_at = ?6, scopes = ?7, updated_at = ?8
             WHERE account_type = ?1",
            rusqlite::params![
                account_type,
                username,
                user_id,
                access_token,
                refresh_token_str,
                expires_at,
                scopes,
                timestamp
            ]
        )?;
    } else {
        // Insert new account
        conn.execute(
            "INSERT INTO twitch_accounts
             (account_type, username, user_id, access_token, refresh_token, token_expires_at, scopes, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            rusqlite::params![
                account_type,
                username,
                user_id,
                access_token,
                refresh_token_str,
                expires_at,
                scopes,
                timestamp,
                timestamp
            ]
        )?;
    }

    Ok(serde_json::json!({
        "success": true,
        "account_type": account_type,
        "username": username,
        "user_id": user_id
    }))
}

pub async fn get_user_info_from_token(access_token: &str) -> Result<Value> {
    let client_id = get_client_id()?;
    let client = reqwest::Client::new();

    let response = client
        .get("https://api.twitch.tv/helix/users")
        .header("Authorization", format!("Bearer {}", access_token))
        .header("Client-Id", client_id)
        .send()
        .await?;

    if !response.status().is_success() {
        let error_text = response.text().await?;
        return Err(anyhow!("Failed to get user info: {}", error_text));
    }

    let data: Value = response.json().await?;

    data["data"]
        .get(0)
        .cloned()
        .ok_or_else(|| anyhow!("No user data returned"))
}

pub async fn do_refresh_token(refresh_token_str: &str) -> Result<(String, String, i64)> {
    let client_id = get_client_id()?;
    let client_secret = get_client_secret()?;
    let client = reqwest::Client::new();

    let response = client
        .post("https://id.twitch.tv/oauth2/token")
        .form(&[
            ("client_id", client_id.as_str()),
            ("client_secret", client_secret.as_str()),
            ("refresh_token", refresh_token_str),
            ("grant_type", "refresh_token"),
        ])
        .send()
        .await?;

    if !response.status().is_success() {
        let error_text = response.text().await?;
        return Err(anyhow!("Failed to refresh token: {}", error_text));
    }

    let token_data: Value = response.json().await?;

    let access_token = token_data["access_token"]
        .as_str()
        .ok_or_else(|| anyhow!("Missing access_token"))?
        .to_string();
    let new_refresh_token = token_data["refresh_token"]
        .as_str()
        .ok_or_else(|| anyhow!("Missing refresh_token"))?
        .to_string();
    let expires_in = token_data["expires_in"]
        .as_i64()
        .ok_or_else(|| anyhow!("Missing expires_in"))?;

    Ok((access_token, new_refresh_token, expires_in))
}

pub async fn refresh_all_tokens() -> Result<Value> {
    let db_path = crate::core::database::get_database_path();
    let conn = rusqlite::Connection::open(&db_path)?;

    // Collect all accounts first
    let mut accounts = Vec::new();
    {
        let mut stmt = conn.prepare(
            "SELECT account_type, refresh_token FROM twitch_accounts WHERE refresh_token IS NOT NULL"
        )?;

        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;

        for row in rows {
            accounts.push(row?);
        }
    }

    // Release the connection before async operations
    drop(conn);

    let mut refreshed = Vec::new();

    for (account_type, old_refresh_token) in accounts {
        match do_refresh_token(&old_refresh_token).await {
            Ok((access_token, new_refresh_token, expires_in)) => {
                let timestamp = current_timestamp();
                let expires_at = timestamp + expires_in;

                // Open a new connection for each update
                let conn = rusqlite::Connection::open(&db_path)?;
                conn.execute(
                    "UPDATE twitch_accounts
                     SET access_token = ?1, refresh_token = ?2, token_expires_at = ?3, updated_at = ?4
                     WHERE account_type = ?5",
                    rusqlite::params![access_token, new_refresh_token, expires_at, timestamp, account_type]
                )?;

                refreshed.push(account_type);
            }
            Err(e) => {
                log::error!("[Twitch API] Failed to refresh {} token: {}", account_type, e);
            }
        }
    }

    Ok(serde_json::json!({
        "success": true,
        "refreshed": refreshed
    }))
}

pub async fn get_account_token(ctx: &PluginContext, account_type: &str) -> Result<Value> {
    let conn = ctx.db()?;

    let result: Result<(String, i64), rusqlite::Error> = conn.query_row(
        "SELECT access_token, token_expires_at FROM twitch_accounts WHERE account_type = ?1",
        [account_type],
        |row| Ok((row.get(0)?, row.get(1)?))
    );

    match result {
        Ok((access_token, expires_at)) => {
            // Check if token is expired or about to expire (within 5 minutes)
            let current_time = current_timestamp();
            if expires_at - current_time < 300 {
                // Token expired or expiring soon, refresh it
                let old_refresh_token: String = conn.query_row(
                    "SELECT refresh_token FROM twitch_accounts WHERE account_type = ?1",
                    [account_type],
                    |row| row.get(0)
                )?;

                match do_refresh_token(&old_refresh_token).await {
                    Ok((new_access_token, new_refresh_token, expires_in)) => {
                        let new_expires_at = current_time + expires_in;
                        conn.execute(
                            "UPDATE twitch_accounts
                             SET access_token = ?1, refresh_token = ?2, token_expires_at = ?3, updated_at = ?4
                             WHERE account_type = ?5",
                            rusqlite::params![
                                &new_access_token,
                                &new_refresh_token,
                                new_expires_at,
                                current_time,
                                account_type
                            ]
                        )?;

                        Ok(serde_json::json!({
                            "access_token": new_access_token,
                            "expires_at": new_expires_at
                        }))
                    }
                    Err(e) => Err(anyhow!("Failed to refresh token: {}", e))
                }
            } else {
                Ok(serde_json::json!({
                    "access_token": access_token,
                    "expires_at": expires_at
                }))
            }
        }
        Err(_) => Err(anyhow!("No {} account configured", account_type))
    }
}

pub async fn get_channel_info(ctx: &PluginContext, channel: &str) -> Result<Value> {
    let client_id = get_client_id()?;
    let token_data = get_account_token(ctx, "broadcaster").await?;
    let access_token = token_data["access_token"]
        .as_str()
        .ok_or_else(|| anyhow!("Missing access token"))?;

    let client = reqwest::Client::new();

    let response = client
        .get("https://api.twitch.tv/helix/channels")
        .query(&[("broadcaster_login", channel)])
        .header("Authorization", format!("Bearer {}", access_token))
        .header("Client-Id", client_id)
        .send()
        .await?;

    if !response.status().is_success() {
        let error_text = response.text().await?;
        return Err(anyhow!("Failed to get channel info: {}", error_text));
    }

    let data: Value = response.json().await?;

    data["data"]
        .get(0)
        .cloned()
        .ok_or_else(|| anyhow!("Channel not found"))
}

pub fn get_client_id_public() -> Result<String> {
    get_client_id()
}

fn current_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}
