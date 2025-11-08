use crate::core::plugin::{Plugin, PluginMetadata};
use crate::core::plugin_context::PluginContext;
use async_trait::async_trait;
use std::sync::Arc;
use anyhow::Result;

pub struct AlertsPlugin;

#[async_trait]
impl Plugin for AlertsPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            id: "alerts".to_string(),
            name: "Stream Alerts".to_string(),
            version: "1.0.0".to_string(),
            description: "Transforms Twitch events into alerts for the overlay".to_string(),
            author: "WebArcade Team".to_string(),
            dependencies: vec!["twitch".to_string()],
        }
    }

    async fn init(&self, _ctx: &PluginContext) -> Result<()> {
        log::info!("[Alerts] Initializing plugin...");
        Ok(())
    }

    async fn start(&self, ctx: Arc<PluginContext>) -> Result<()> {
        log::info!("[Alerts] Starting plugin...");
        log::info!("[Alerts] Subscribing to Twitch events for alert transformation");

        // ============================================================================
        // FOLLOW EVENTS (EventSub)
        // ============================================================================
        let ctx_follow = ctx.clone();
        tokio::spawn(async move {
            log::info!("[Alerts] Subscribed to twitch.channel_follow events");
            let mut events = ctx_follow.subscribe_to("twitch.channel_follow").await;

            while let Ok(event) = events.recv().await {
                log::info!("[Alerts] ✨ FOLLOW EVENT RECEIVED!");
                log::info!("[Alerts] Follow event payload: {:?}", event.payload);

                // Extract data from EventSub format
                let username = event.payload
                    .get("user_name")
                    .or_else(|| event.payload.get("username"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown");

                let user_id = event.payload
                    .get("user_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                // Transform to alert format
                let alert = serde_json::json!({
                    "type": "follow",
                    "user_name": username,
                    "display_name": username,
                    "username": username,
                    "user_id": user_id,
                    "message": format!("Thanks for the follow, {}!", username)
                });

                log::info!("[Alerts] 📤 Emitting alert.show event for follow: {}", username);
                log::info!("[Alerts] Alert data: {:?}", alert);
                ctx_follow.emit("alert.show", &alert);
                log::info!("[Alerts] ✅ alert.show event emitted successfully");
            }
        });

        // ============================================================================
        // SUBSCRIPTION EVENTS (IRC)
        // ============================================================================
        let ctx_sub = ctx.clone();
        tokio::spawn(async move {
            log::info!("[Alerts] Subscribed to twitch.subscription events (IRC)");
            let mut events = ctx_sub.subscribe_to("twitch.subscription").await;

            while let Ok(event) = events.recv().await {
                log::info!("[Alerts] ✨ SUBSCRIPTION EVENT RECEIVED (IRC)!");
                log::info!("[Alerts] Subscription event payload: {:?}", event.payload);

                let username = event.payload
                    .get("username")
                    .or_else(|| event.payload.get("display_name"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown");

                let display_name = event.payload
                    .get("display_name")
                    .or_else(|| event.payload.get("username"))
                    .and_then(|v| v.as_str())
                    .unwrap_or(username);

                let tier = event.payload
                    .get("tier")
                    .and_then(|v| v.as_str())
                    .unwrap_or("1000");

                let months = event.payload
                    .get("months")
                    .or_else(|| event.payload.get("cumulative_months"))
                    .and_then(|v| v.as_i64())
                    .unwrap_or(1);

                let is_resub = event.payload
                    .get("is_resub")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);

                let message = event.payload
                    .get("message")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                // Transform to alert format
                let alert_type = if is_resub || months > 1 { "resubscription" } else { "subscription" };

                let alert = serde_json::json!({
                    "type": alert_type,
                    "user_name": username,
                    "display_name": display_name,
                    "username": username,
                    "tier": tier,
                    "months": months,
                    "cumulative_months": months,
                    "message": message
                });

                log::info!("[Alerts] 📤 Emitting alert.show event for subscription: {}", username);
                log::info!("[Alerts] Alert data: {:?}", alert);
                ctx_sub.emit("alert.show", &alert);
                log::info!("[Alerts] ✅ alert.show event emitted successfully");
            }
        });

        // ============================================================================
        // SUBSCRIPTION EVENTS (EventSub)
        // ============================================================================
        let ctx_sub_eventsub = ctx.clone();
        tokio::spawn(async move {
            log::info!("[Alerts] Subscribed to twitch.channel_subscribe events (EventSub)");
            let mut events = ctx_sub_eventsub.subscribe_to("twitch.channel_subscribe").await;

            while let Ok(event) = events.recv().await {
                log::info!("[Alerts] ✨ SUBSCRIPTION EVENT RECEIVED (EventSub)!");
                log::info!("[Alerts] EventSub subscription event payload: {:?}", event.payload);

                let username = event.payload
                    .get("user_name")
                    .or_else(|| event.payload.get("username"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown");

                let user_id = event.payload
                    .get("user_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                let tier = event.payload
                    .get("tier")
                    .and_then(|v| v.as_str())
                    .unwrap_or("1000");

                let is_gift = event.payload
                    .get("is_gift")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);

                let alert = serde_json::json!({
                    "type": "subscription",
                    "user_name": username,
                    "display_name": username,
                    "username": username,
                    "user_id": user_id,
                    "tier": tier,
                    "is_gift": is_gift
                });

                log::info!("[Alerts] 📤 Emitting alert.show event for EventSub subscription: {}", username);
                log::info!("[Alerts] Alert data: {:?}", alert);
                ctx_sub_eventsub.emit("alert.show", &alert);
                log::info!("[Alerts] ✅ alert.show event emitted successfully");
            }
        });

        // ============================================================================
        // GIFT SUBSCRIPTION EVENTS (IRC)
        // ============================================================================
        let ctx_gift = ctx.clone();
        tokio::spawn(async move {
            log::info!("[Alerts] Subscribed to twitch.subscription_gift events");
            let mut events = ctx_gift.subscribe_to("twitch.subscription_gift").await;

            while let Ok(event) = events.recv().await {
                log::info!("[Alerts] ✨ GIFT SUBSCRIPTION EVENT RECEIVED!");
                log::info!("[Alerts] Gift subscription event payload: {:?}", event.payload);

                let gifter_name = event.payload
                    .get("gifter_display_name")
                    .or_else(|| event.payload.get("gifter"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("Someone");

                let recipient_name = event.payload
                    .get("recipient_display_name")
                    .or_else(|| event.payload.get("recipient"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("Someone");

                let tier = event.payload
                    .get("tier")
                    .and_then(|v| v.as_str())
                    .unwrap_or("1000");

                let alert = serde_json::json!({
                    "type": "gift_subscription",
                    "user_name": gifter_name,
                    "gifter_name": gifter_name,
                    "gifter_display_name": gifter_name,
                    "recipient_name": recipient_name,
                    "recipient_user_name": recipient_name,
                    "tier": tier
                });

                log::info!("[Alerts] 📤 Emitting alert.show event for gift sub: {} → {}", gifter_name, recipient_name);
                log::info!("[Alerts] Alert data: {:?}", alert);
                ctx_gift.emit("alert.show", &alert);
                log::info!("[Alerts] ✅ alert.show event emitted successfully");
            }
        });

        // ============================================================================
        // RAID EVENTS (IRC)
        // ============================================================================
        let ctx_raid = ctx.clone();
        tokio::spawn(async move {
            log::info!("[Alerts] Subscribed to twitch.raid events (IRC)");
            let mut events = ctx_raid.subscribe_to("twitch.raid").await;

            while let Ok(event) = events.recv().await {
                log::info!("[Alerts] ✨ RAID EVENT RECEIVED (IRC)!");
                log::info!("[Alerts] Raid event payload: {:?}", event.payload);

                let raider = event.payload
                    .get("raider")
                    .or_else(|| event.payload.get("from_broadcaster_user_name"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown");

                let viewers = event.payload
                    .get("viewer_count")
                    .or_else(|| event.payload.get("viewers"))
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0);

                let alert = serde_json::json!({
                    "type": "raid",
                    "from_broadcaster_user_name": raider,
                    "display_name": raider,
                    "username": raider,
                    "viewers": viewers
                });

                log::info!("[Alerts] 📤 Emitting alert.show event for raid (IRC): {} with {} viewers", raider, viewers);
                log::info!("[Alerts] Alert data: {:?}", alert);
                ctx_raid.emit("alert.show", &alert);
                log::info!("[Alerts] ✅ alert.show event emitted successfully");
            }
        });

        // ============================================================================
        // RAID EVENTS (EventSub)
        // ============================================================================
        let ctx_raid_eventsub = ctx.clone();
        tokio::spawn(async move {
            log::info!("[Alerts] Subscribed to twitch.channel_raid events (EventSub)");
            let mut events = ctx_raid_eventsub.subscribe_to("twitch.channel_raid").await;

            while let Ok(event) = events.recv().await {
                log::info!("[Alerts] ✨ RAID EVENT RECEIVED (EventSub)!");
                log::info!("[Alerts] EventSub raid event payload: {:?}", event.payload);

                let raider = event.payload
                    .get("from_broadcaster_user_name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown");

                let viewers = event.payload
                    .get("viewers")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0);

                let alert = serde_json::json!({
                    "type": "raid",
                    "from_broadcaster_user_name": raider,
                    "display_name": raider,
                    "username": raider,
                    "viewers": viewers
                });

                log::info!("[Alerts] 📤 Emitting alert.show event for raid (EventSub): {} with {} viewers", raider, viewers);
                log::info!("[Alerts] Alert data: {:?}", alert);
                ctx_raid_eventsub.emit("alert.show", &alert);
                log::info!("[Alerts] ✅ alert.show event emitted successfully");
            }
        });

        // ============================================================================
        // BITS/CHEER EVENTS (IRC)
        // ============================================================================
        let ctx_bits = ctx.clone();
        tokio::spawn(async move {
            log::info!("[Alerts] Subscribed to twitch.bits events (IRC)");
            let mut events = ctx_bits.subscribe_to("twitch.bits").await;

            while let Ok(event) = events.recv().await {
                log::info!("[Alerts] ✨ BITS EVENT RECEIVED!");
                log::info!("[Alerts] Bits event payload: {:?}", event.payload);

                let username = event.payload
                    .get("display_name")
                    .or_else(|| event.payload.get("username"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown");

                let bits = event.payload
                    .get("bits")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0);

                let message = event.payload
                    .get("message")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                let alert = serde_json::json!({
                    "type": "cheer",
                    "user_name": username,
                    "display_name": username,
                    "username": username,
                    "bits": bits,
                    "message": message
                });

                log::info!("[Alerts] 📤 Emitting alert.show event for bits: {} cheered {} bits", username, bits);
                log::info!("[Alerts] Alert data: {:?}", alert);
                ctx_bits.emit("alert.show", &alert);
                log::info!("[Alerts] ✅ alert.show event emitted successfully");
            }
        });

        // ============================================================================
        // BITS/CHEER EVENTS (EventSub)
        // ============================================================================
        let ctx_cheer_eventsub = ctx.clone();
        tokio::spawn(async move {
            log::info!("[Alerts] Subscribed to twitch.channel_cheer events (EventSub)");
            let mut events = ctx_cheer_eventsub.subscribe_to("twitch.channel_cheer").await;

            while let Ok(event) = events.recv().await {
                log::info!("[Alerts] ✨ CHEER EVENT RECEIVED (EventSub)!");
                log::info!("[Alerts] EventSub cheer event payload: {:?}", event.payload);

                let username = event.payload
                    .get("user_name")
                    .or_else(|| event.payload.get("user_login"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("Anonymous");

                let bits = event.payload
                    .get("bits")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0);

                let message = event.payload
                    .get("message")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                let alert = serde_json::json!({
                    "type": "cheer",
                    "user_name": username,
                    "display_name": username,
                    "username": username,
                    "bits": bits,
                    "message": message
                });

                log::info!("[Alerts] 📤 Emitting alert.show event for cheer (EventSub): {} cheered {} bits", username, bits);
                log::info!("[Alerts] Alert data: {:?}", alert);
                ctx_cheer_eventsub.emit("alert.show", &alert);
                log::info!("[Alerts] ✅ alert.show event emitted successfully");
            }
        });

        // ============================================================================
        // CHANNEL POINTS REDEMPTION (EventSub)
        // ============================================================================
        let ctx_points = ctx.clone();
        tokio::spawn(async move {
            log::info!("[Alerts] Subscribed to twitch.channel_channel_points_custom_reward_redemption_add events");
            let mut events = ctx_points.subscribe_to("twitch.channel_channel_points_custom_reward_redemption_add").await;

            while let Ok(event) = events.recv().await {
                log::info!("[Alerts] ✨ CHANNEL POINTS REDEMPTION EVENT RECEIVED!");
                log::info!("[Alerts] Channel points event payload: {:?}", event.payload);

                let username = event.payload
                    .get("user_name")
                    .or_else(|| event.payload.get("user_login"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown");

                let reward_title = event.payload
                    .get("reward")
                    .and_then(|r| r.get("title"))
                    .or_else(|| event.payload.get("reward_title"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown Reward");

                let user_input = event.payload
                    .get("user_input")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                let alert = serde_json::json!({
                    "type": "channel_points_redemption",
                    "user_name": username,
                    "display_name": username,
                    "username": username,
                    "reward": {
                        "title": reward_title
                    },
                    "reward_title": reward_title,
                    "user_input": user_input
                });

                log::info!("[Alerts] 📤 Emitting alert.show event for channel points: {} redeemed {}", username, reward_title);
                log::info!("[Alerts] Alert data: {:?}", alert);
                ctx_points.emit("alert.show", &alert);
                log::info!("[Alerts] ✅ alert.show event emitted successfully");
            }
        });

        log::info!("[Alerts] Plugin started successfully - listening for Twitch events");
        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        log::info!("[Alerts] Stopping plugin...");
        Ok(())
    }
}
