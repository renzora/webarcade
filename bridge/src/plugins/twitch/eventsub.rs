// Twitch EventSub implementation
// Handles EventSub webhooks and subscriptions

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventSubSubscription {
    pub id: String,
    #[serde(rename = "type")]
    pub subscription_type: String,
    pub version: String,
    pub condition: serde_json::Value,
    pub transport: EventSubTransport,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventSubTransport {
    pub method: String,
    pub callback: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secret: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CreateSubscriptionResponse {
    data: Vec<EventSubSubscription>,
}

#[derive(Debug, Deserialize)]
struct ListSubscriptionsResponse {
    data: Vec<EventSubSubscription>,
    total: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventSubWebhookPayload {
    pub subscription: EventSubSubscription,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub challenge: Option<String>,
}

pub struct EventSubManager {
    pub client_id: String,
    pub access_token: String,
    pub webhook_secret: String,
    pub callback_url: String,
    pub http_client: reqwest::Client,
}

impl EventSubManager {
    pub fn new(client_id: String, access_token: String, webhook_secret: String, callback_url: String) -> Self {
        Self {
            client_id,
            access_token,
            webhook_secret,
            callback_url,
            http_client: reqwest::Client::new(),
        }
    }

    pub async fn create_subscription(&self, subscription_type: &str, version: &str, condition: serde_json::Value) -> Result<EventSubSubscription> {
        log::info!("[Twitch EventSub] Creating subscription: {} (version {})", subscription_type, version);

        let body = serde_json::json!({
            "type": subscription_type,
            "version": version,
            "condition": condition,
            "transport": {
                "method": "webhook",
                "callback": self.callback_url,
                "secret": self.webhook_secret
            }
        });

        let response = self.http_client
            .post("https://api.twitch.tv/helix/eventsub/subscriptions")
            .header("Client-ID", &self.client_id)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await?;
            return Err(anyhow!("Failed to create EventSub subscription: {} - {}", status, error_text));
        }

        let result: CreateSubscriptionResponse = response.json().await?;

        if result.data.is_empty() {
            return Err(anyhow!("No subscription returned from Twitch API"));
        }

        log::info!("[Twitch EventSub] Subscription created successfully: {}", result.data[0].id);
        Ok(result.data[0].clone())
    }

    pub async fn delete_subscription(&self, subscription_id: &str) -> Result<()> {
        log::info!("[Twitch EventSub] Deleting subscription: {}", subscription_id);

        let response = self.http_client
            .delete(format!("https://api.twitch.tv/helix/eventsub/subscriptions?id={}", subscription_id))
            .header("Client-ID", &self.client_id)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await?;
            return Err(anyhow!("Failed to delete EventSub subscription: {} - {}", status, error_text));
        }

        log::info!("[Twitch EventSub] Subscription deleted successfully");
        Ok(())
    }

    pub async fn list_subscriptions(&self) -> Result<Vec<EventSubSubscription>> {
        log::info!("[Twitch EventSub] Listing subscriptions");

        let response = self.http_client
            .get("https://api.twitch.tv/helix/eventsub/subscriptions")
            .header("Client-ID", &self.client_id)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await?;
            return Err(anyhow!("Failed to list EventSub subscriptions: {} - {}", status, error_text));
        }

        let result: ListSubscriptionsResponse = response.json().await?;
        log::info!("[Twitch EventSub] Found {} subscriptions", result.total);
        Ok(result.data)
    }

    pub async fn delete_all_subscriptions(&self) -> Result<()> {
        let subscriptions = self.list_subscriptions().await?;

        for sub in subscriptions {
            if let Err(e) = self.delete_subscription(&sub.id).await {
                log::error!("[Twitch EventSub] Failed to delete subscription {}: {}", sub.id, e);
            }
        }

        Ok(())
    }

    pub fn verify_signature(&self, body: &str, signature: &str, message_id: &str, timestamp: &str) -> bool {
        // Construct the message that Twitch signed
        let message = format!("{}{}{}", message_id, timestamp, body);

        // Create HMAC
        let mut mac = match HmacSha256::new_from_slice(self.webhook_secret.as_bytes()) {
            Ok(m) => m,
            Err(e) => {
                log::error!("[Twitch EventSub] Failed to create HMAC: {}", e);
                return false;
            }
        };

        mac.update(message.as_bytes());

        // Get the expected signature
        let expected_signature = format!("sha256={}", hex::encode(mac.finalize().into_bytes()));

        // Compare signatures (constant-time comparison to prevent timing attacks)
        expected_signature == signature
    }
}
