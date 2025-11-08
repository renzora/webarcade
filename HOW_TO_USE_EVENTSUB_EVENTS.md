# How to Use EventSub Events in Plugins and Overlays

This guide shows you how to subscribe to and use EventSub events in your plugins and overlays.

## Available Events

After running auto-setup, you have **35+ EventSub events** available:

### Channel Events
- `twitch.channel_update` - Title/game changes
- `twitch.channel_follow` - New followers
- `twitch.channel_subscribe` - New subscriptions
- `twitch.channel_subscription_end` - Subscription ended
- `twitch.channel_subscription_gift` - Gift subscriptions
- `twitch.channel_subscription_message` - Resub messages
- `twitch.channel_cheer` - Bits/cheers
- `twitch.channel_raid` - Incoming raids
- `twitch.channel_ban` - User banned
- `twitch.channel_unban` - User unbanned
- `twitch.channel_moderator_add` - Mod added
- `twitch.channel_moderator_remove` - Mod removed

### Channel Points
- `twitch.channel_channel_points_custom_reward_add` - Reward created
- `twitch.channel_channel_points_custom_reward_update` - Reward updated
- `twitch.channel_channel_points_custom_reward_remove` - Reward deleted
- `twitch.channel_channel_points_custom_reward_redemption_add` - Reward redeemed
- `twitch.channel_channel_points_custom_reward_redemption_update` - Redemption updated

### Polls & Predictions
- `twitch.channel_poll_begin` - Poll started
- `twitch.channel_poll_progress` - Poll voting
- `twitch.channel_poll_end` - Poll ended
- `twitch.channel_prediction_begin` - Prediction started
- `twitch.channel_prediction_progress` - Prediction voting
- `twitch.channel_prediction_lock` - Prediction locked
- `twitch.channel_prediction_end` - Prediction ended

### Hype Trains
- `twitch.channel_hype_train_begin` - Hype train started
- `twitch.channel_hype_train_progress` - Hype train progress
- `twitch.channel_hype_train_end` - Hype train ended

### Charity
- `twitch.channel_charity_campaign_donate` - Charity donation
- `twitch.channel_charity_campaign_start` - Charity started
- `twitch.channel_charity_campaign_progress` - Charity progress
- `twitch.channel_charity_campaign_stop` - Charity ended

### Stream
- `twitch.stream_online` - Stream went live
- `twitch.stream_offline` - Stream ended

## How to Use Events in Backend Plugins (Rust)

### Example 1: Simple Event Listener

```rust
use crate::core::plugin::{Plugin, PluginMetadata};
use crate::core::plugin_context::PluginContext;
use async_trait::async_trait;
use std::sync::Arc;
use anyhow::Result;

pub struct MyAlertPlugin;

#[async_trait]
impl Plugin for MyAlertPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            id: "my_alerts".to_string(),
            name: "My Alert Plugin".to_string(),
            version: "1.0.0".to_string(),
            description: "Custom alert system".to_string(),
            author: "You".to_string(),
            dependencies: vec!["twitch".to_string()],
        }
    }

    async fn init(&self, ctx: &PluginContext) -> Result<()> {
        log::info!("[MyAlerts] Initializing...");
        Ok(())
    }

    async fn start(&self, ctx: Arc<PluginContext>) -> Result<()> {
        log::info!("[MyAlerts] Starting...");

        // Listen for follow events
        let ctx_follow = ctx.clone();
        tokio::spawn(async move {
            let mut events = ctx_follow.subscribe_to("twitch.channel_follow").await;

            while let Ok(event) = events.recv().await {
                // Get follower data
                let username = event.payload["username"].as_str().unwrap_or("Unknown");
                let user_id = event.payload["user_id"].as_str().unwrap_or("");

                log::info!("[MyAlerts] New follower: {}", username);

                // Emit to frontend overlay
                ctx_follow.emit("alerts.show", &serde_json::json!({
                    "type": "follow",
                    "username": username,
                    "user_id": user_id,
                    "message": format!("Thanks for the follow, {}!", username)
                }));
            }
        });

        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        Ok(())
    }
}
```

### Example 2: Multiple Event Types

```rust
async fn start(&self, ctx: Arc<PluginContext>) -> Result<()> {
    // Listen for follows
    let ctx_follow = ctx.clone();
    tokio::spawn(async move {
        let mut events = ctx_follow.subscribe_to("twitch.channel_follow").await;
        while let Ok(event) = events.recv().await {
            handle_follow_alert(&ctx_follow, event.payload).await;
        }
    });

    // Listen for subs
    let ctx_sub = ctx.clone();
    tokio::spawn(async move {
        let mut events = ctx_sub.subscribe_to("twitch.channel_subscribe").await;
        while let Ok(event) = events.recv().await {
            handle_sub_alert(&ctx_sub, event.payload).await;
        }
    });

    // Listen for raids
    let ctx_raid = ctx.clone();
    tokio::spawn(async move {
        let mut events = ctx_raid.subscribe_to("twitch.channel_raid").await;
        while let Ok(event) = events.recv().await {
            handle_raid_alert(&ctx_raid, event.payload).await;
        }
    });

    // Listen for channel points
    let ctx_points = ctx.clone();
    tokio::spawn(async move {
        let mut events = ctx_points.subscribe_to("twitch.channel_channel_points_custom_reward_redemption_add").await;
        while let Ok(event) = events.recv().await {
            handle_channel_points(&ctx_points, event.payload).await;
        }
    });

    Ok(())
}

async fn handle_follow_alert(ctx: &PluginContext, payload: serde_json::Value) {
    let username = payload["username"].as_str().unwrap_or("Unknown");

    ctx.emit("alerts.show", &serde_json::json!({
        "type": "follow",
        "username": username,
        "animation": "heart_explosion",
        "sound": "follow_sound.mp3"
    }));
}

async fn handle_sub_alert(ctx: &PluginContext, payload: serde_json::Value) {
    let username = payload["user_name"].as_str().unwrap_or("Unknown");
    let tier = payload["tier"].as_str().unwrap_or("1000");

    ctx.emit("alerts.show", &serde_json::json!({
        "type": "subscription",
        "username": username,
        "tier": tier,
        "animation": "confetti",
        "sound": "sub_sound.mp3"
    }));
}

async fn handle_raid_alert(ctx: &PluginContext, payload: serde_json::Value) {
    let username = payload["from_broadcaster_user_name"].as_str().unwrap_or("Unknown");
    let viewers = payload["viewers"].as_i64().unwrap_or(0);

    ctx.emit("alerts.show", &serde_json::json!({
        "type": "raid",
        "username": username,
        "viewers": viewers,
        "animation": "raid_incoming",
        "sound": "raid_sound.mp3"
    }));
}

async fn handle_channel_points(ctx: &PluginContext, payload: serde_json::Value) {
    let username = payload["user_name"].as_str().unwrap_or("Unknown");
    let reward_title = payload["reward"]["title"].as_str().unwrap_or("Unknown Reward");
    let user_input = payload["user_input"].as_str();

    log::info!("[Alerts] {} redeemed: {}", username, reward_title);

    // Custom logic based on reward
    match reward_title {
        "Hydrate" => {
            ctx.emit("alerts.show", &serde_json::json!({
                "type": "channel_points",
                "message": format!("{} says: Time to drink water!", username),
                "icon": "💧"
            }));
        }
        "Play a Sound" => {
            if let Some(sound_name) = user_input {
                ctx.emit("alerts.play_sound", &serde_json::json!({
                    "sound": sound_name
                }));
            }
        }
        _ => {
            ctx.emit("alerts.show", &serde_json::json!({
                "type": "channel_points",
                "username": username,
                "reward": reward_title
            }));
        }
    }
}
```

## How to Use Events in Frontend Overlays (React/SolidJS)

### Example: Alerts Overlay

Your alerts overlay can listen to WebSocket events from the backend:

```jsx
// plugins/my-alerts/overlay.jsx
import { createSignal, onMount, Show } from 'solid-js';

export default function MyAlertsOverlay() {
  const [alert, setAlert] = createSignal(null);
  const [visible, setVisible] = createSignal(false);

  onMount(() => {
    // Connect to WebSocket
    const ws = new WebSocket('ws://localhost:3001/ws');

    ws.onmessage = (event) => {
      const data = JSON.parse(event.data);

      // Listen for alert events
      if (data.event === 'alerts.show') {
        showAlert(data.payload);
      }
    };

    ws.onerror = (error) => {
      console.error('WebSocket error:', error);
    };

    return () => ws.close();
  });

  const showAlert = (alertData) => {
    setAlert(alertData);
    setVisible(true);

    // Play sound if specified
    if (alertData.sound) {
      const audio = new Audio(`/sounds/${alertData.sound}`);
      audio.play();
    }

    // Hide after 5 seconds
    setTimeout(() => {
      setVisible(false);
      setTimeout(() => setAlert(null), 500); // Clear after fade out
    }, 5000);
  };

  return (
    <div class="alerts-container">
      <Show when={visible() && alert()}>
        <div class={`alert alert-${alert().type} ${visible() ? 'show' : 'hide'}`}>
          <Show when={alert().type === 'follow'}>
            <div class="follow-alert">
              <div class="icon">❤️</div>
              <div class="message">
                <h2>New Follower!</h2>
                <p>Thanks for the follow, <strong>{alert().username}</strong>!</p>
              </div>
            </div>
          </Show>

          <Show when={alert().type === 'subscription'}>
            <div class="sub-alert">
              <div class="icon">⭐</div>
              <div class="message">
                <h2>New Subscriber!</h2>
                <p><strong>{alert().username}</strong> just subscribed!</p>
                <Show when={alert().tier === '2000' || alert().tier === '3000'}>
                  <p class="tier">Tier {parseInt(alert().tier) / 1000} Sub!</p>
                </Show>
              </div>
            </div>
          </Show>

          <Show when={alert().type === 'raid'}>
            <div class="raid-alert">
              <div class="icon">🎉</div>
              <div class="message">
                <h2>Incoming Raid!</h2>
                <p><strong>{alert().username}</strong> is raiding with <strong>{alert().viewers}</strong> viewers!</p>
              </div>
            </div>
          </Show>

          <Show when={alert().type === 'channel_points'}>
            <div class="points-alert">
              <div class="icon">{alert().icon || '🎁'}</div>
              <div class="message">
                <p>{alert().message || `${alert().username} redeemed ${alert().reward}`}</p>
              </div>
            </div>
          </Show>
        </div>
      </Show>
    </div>
  );
}
```

### CSS for Alerts

```css
/* plugins/my-alerts/styles.css */
.alerts-container {
  position: fixed;
  top: 20%;
  left: 50%;
  transform: translateX(-50%);
  width: 600px;
  z-index: 9999;
}

.alert {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
  padding: 30px;
  border-radius: 20px;
  box-shadow: 0 10px 40px rgba(0, 0, 0, 0.3);
  opacity: 0;
  transform: scale(0.8) translateY(-50px);
  transition: all 0.5s cubic-bezier(0.68, -0.55, 0.265, 1.55);
}

.alert.show {
  opacity: 1;
  transform: scale(1) translateY(0);
}

.alert.hide {
  opacity: 0;
  transform: scale(0.8) translateY(-50px);
}

.follow-alert,
.sub-alert,
.raid-alert,
.points-alert {
  display: flex;
  align-items: center;
  gap: 20px;
}

.icon {
  font-size: 60px;
  animation: bounce 1s infinite;
}

@keyframes bounce {
  0%, 100% { transform: translateY(0); }
  50% { transform: translateY(-10px); }
}

.message h2 {
  margin: 0;
  font-size: 32px;
  font-weight: bold;
  text-shadow: 2px 2px 4px rgba(0, 0, 0, 0.3);
}

.message p {
  margin: 10px 0 0 0;
  font-size: 24px;
}

.tier {
  color: #ffd700;
  font-weight: bold;
}
```

## Example: Using Stream Status Events

```rust
// Listen for stream online/offline
let ctx_stream = ctx.clone();
tokio::spawn(async move {
    let mut online_events = ctx_stream.subscribe_to("twitch.stream_online").await;
    let mut offline_events = ctx_stream.subscribe_to("twitch.stream_offline").await;

    loop {
        tokio::select! {
            Ok(event) = online_events.recv() => {
                let started_at = event.payload["started_at"].as_i64().unwrap_or(0);
                log::info!("[Stream] Stream went live at {}", started_at);

                // Update database, send notifications, etc.
                ctx_stream.emit("stream.status_changed", &serde_json::json!({
                    "status": "online",
                    "started_at": started_at
                }));
            }
            Ok(event) = offline_events.recv() => {
                log::info!("[Stream] Stream went offline");

                ctx_stream.emit("stream.status_changed", &serde_json::json!({
                    "status": "offline"
                }));
            }
        }
    }
});
```

## Event Payload Examples

### Follow Event
```json
{
  "user_id": "123456789",
  "username": "cool_viewer",
  "followed_at": 1704067200
}
```

### Subscription Event
```json
{
  "user_id": "123456789",
  "user_name": "cool_viewer",
  "tier": "1000",
  "is_gift": false
}
```

### Raid Event
```json
{
  "from_broadcaster_user_id": "123456789",
  "from_broadcaster_user_name": "raiding_streamer",
  "to_broadcaster_user_id": "987654321",
  "to_broadcaster_user_name": "your_channel",
  "viewers": 50
}
```

### Channel Points Event
```json
{
  "id": "redemption-id",
  "broadcaster_user_id": "987654321",
  "broadcaster_user_name": "your_channel",
  "user_id": "123456789",
  "user_name": "cool_viewer",
  "user_input": "optional user text",
  "status": "unfulfilled",
  "reward": {
    "id": "reward-id",
    "title": "Hydrate",
    "cost": 100,
    "prompt": "Make the streamer drink water"
  },
  "redeemed_at": "2024-01-01T12:00:00Z"
}
```

## Tips

1. **Always handle errors**: Events might be malformed
   ```rust
   let username = event.payload["username"]
       .as_str()
       .unwrap_or("Unknown");
   ```

2. **Use tokio::select!** for multiple event streams
   ```rust
   tokio::select! {
       Ok(follow) = follow_events.recv() => { /* ... */ }
       Ok(sub) = sub_events.recv() => { /* ... */ }
   }
   ```

3. **Emit to frontend** for visual overlays
   ```rust
   ctx.emit("alerts.show", &alert_data);
   ```

4. **Store important events** in database for history
   ```rust
   conn.execute(
       "INSERT INTO event_history (event_type, username, data, timestamp) VALUES (?1, ?2, ?3, ?4)",
       params![event_type, username, data, timestamp]
   )?;
   ```

5. **Test with Twitch CLI** before going live
   ```bash
   twitch event trigger channel.follow -F http://localhost:3001/twitch/eventsub/webhook
   ```

## Next Steps

1. Create your plugin directory in `plugins/your_plugin/`
2. Add backend code (`mod.rs`, `router.rs`) in the plugin directory
3. Subscribe to events in the plugin's `start()` method
4. Emit events to your overlay
5. Create overlay component (`overlay.jsx`) in the same plugin directory
6. Style and animate!

You now have access to **every Twitch event** in real-time! 🚀
