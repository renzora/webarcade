# Twitch EventSub Setup Guide

This guide explains how to set up and use the Twitch EventSub system for receiving follow events and other Twitch notifications.

## What is EventSub?

EventSub is Twitch's webhook system that sends real-time notifications to your server when events occur on your channel (follows, subscriptions, channel points, etc.). Unlike IRC, which handles chat messages, EventSub is required for follow notifications and other channel events.

## Features

### ✅ Implemented EventSub Events
- **Follows** (`channel.follow`) - Triggers when someone follows your channel
- **Subscriptions** (`channel.subscribe`) - New subscriptions
- **Channel Points** (`channel.channel_points_custom_reward_redemption.add`) - Channel points redemptions
- **Stream Online/Offline** (`stream.online`, `stream.offline`) - Stream status changes

### Follower Auto-Thank Plugin

The follower plugin automatically:
1. Listens for `twitch.follow` events
2. Stores follower data in the database
3. Sends an automatic thank you message in chat: `"Thank you for the follow, @username! Welcome to the community!"`

## Setup Instructions

### 1. Prerequisites

- You need to have authenticated with Twitch (broadcaster account)
- Your WebArcade bridge must be running on `localhost:3001`
- For production, you'll need a public URL with HTTPS (use ngrok or similar)

### 2. Configure EventSub Webhook

The EventSub webhook is available at:
```
http://localhost:3001/twitch/eventsub/webhook
```

For testing with Twitch's actual API, you need a publicly accessible URL. Use a tunneling service like ngrok:

```bash
ngrok http 3001
```

This gives you a public URL like `https://abc123.ngrok.io`.

### 3. Create Follow Subscription

Use the API endpoint to create a follow subscription:

**Endpoint:** `POST /twitch/eventsub/subscriptions`

**Request Body:**
```json
{
  "type": "channel.follow",
  "version": "2",
  "condition": {
    "broadcaster_user_id": "YOUR_USER_ID",
    "moderator_user_id": "YOUR_USER_ID"
  },
  "callback_url": "https://YOUR_NGROK_URL/twitch/eventsub/webhook"
}
```

Replace:
- `YOUR_USER_ID` - Your Twitch user ID (found in auth status)
- `YOUR_NGROK_URL` - Your public ngrok URL (e.g., `abc123.ngrok.io`)

### 4. Testing

#### Test with Postman/Curl

```bash
curl -X POST http://localhost:3001/twitch/eventsub/subscriptions \
  -H "Content-Type: application/json" \
  -d '{
    "type": "channel.follow",
    "version": "2",
    "condition": {
      "broadcaster_user_id": "123456",
      "moderator_user_id": "123456"
    }
  }'
```

#### Check Active Subscriptions

```bash
curl http://localhost:3001/twitch/eventsub/subscriptions
```

#### Delete All Subscriptions

```bash
curl -X DELETE http://localhost:3001/twitch/eventsub/subscriptions
```

## How It Works

### EventSub Flow

1. **Subscription Creation:**
   - You create a subscription via the API
   - Twitch sends a verification challenge to your webhook
   - Your server responds with the challenge to confirm

2. **Event Delivery:**
   - When someone follows your channel, Twitch sends a POST request to your webhook
   - The webhook verifies the HMAC signature
   - Parses the event and emits a `twitch.follow` event

3. **Follower Plugin:**
   - Listens for `twitch.follow` events
   - Stores follower data in the `followers` table
   - Sends a thank you message via `twitch.send_message` event
   - The Twitch plugin picks up the message and sends it to IRC

### Event Emission

When a follow occurs, this event is emitted:
```json
{
  "event": "twitch.follow",
  "payload": {
    "user_id": "123456",
    "username": "cool_viewer",
    "followed_at": 1704067200
  }
}
```

Any plugin can subscribe to this event:
```rust
let mut events = ctx.subscribe_to("twitch.follow").await;
while let Ok(event) = events.recv().await {
    let username = event.payload["username"].as_str().unwrap();
    // Do something with the follow event
}
```

## Database Schema

### Followers Table
```sql
CREATE TABLE followers (
    user_id TEXT PRIMARY KEY,
    username TEXT NOT NULL,
    followed_at INTEGER NOT NULL,
    thanked INTEGER NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL
);
```

## API Endpoints

### EventSub Management

- `GET /twitch/eventsub/subscriptions` - List all subscriptions
- `POST /twitch/eventsub/subscriptions` - Create new subscription
- `DELETE /twitch/eventsub/subscriptions` - Delete all subscriptions
- `POST /twitch/eventsub/webhook` - Webhook callback (for Twitch only)

### Follower Plugin

- `POST /followers/list` - Get follower list (via service call)
- `POST /followers/count` - Get total follower count (via service call)

## Troubleshooting

### EventSub Not Receiving Events

1. **Check subscription status:**
   ```bash
   curl http://localhost:3001/twitch/eventsub/subscriptions
   ```
   Status should be `"enabled"`, not `"webhook_callback_verification_pending"`

2. **Verify webhook is publicly accessible:**
   - Twitch must be able to reach your webhook URL
   - Test with curl from outside your network

3. **Check logs:**
   ```
   [Twitch EventSub] Webhook verification challenge received
   [Twitch EventSub] Subscription created successfully: sub_xyz...
   ```

### Signature Verification Failures

- Make sure the `eventsub_secret` in the database matches what you sent to Twitch
- Check that your clock is synchronized (Twitch includes timestamps in signatures)

### Follower Not Thanked

1. **Check if IRC is connected:**
   - The thank you message is sent via Twitch IRC
   - Verify IRC connection in logs

2. **Check database:**
   - See if the follower was stored in the `followers` table
   - If stored but not thanked, there may be an IRC issue

3. **Check channel name:**
   - The plugin uses the authenticated user's channel
   - Verify your Twitch auth is set up correctly

## Local Testing (Without Twitch)

You can simulate a follow event for testing:

```rust
ctx.emit("twitch.follow", &serde_json::json!({
    "user_id": "999999",
    "username": "test_follower",
    "followed_at": chrono::Utc::now().timestamp()
}));
```

This will trigger the follower plugin without needing an actual Twitch follow.

## Security Notes

- The EventSub webhook uses HMAC-SHA256 signature verification
- Never expose your `eventsub_secret` publicly
- For production, always use HTTPS (Twitch requires it)
- The webhook automatically generates a secret if none exists

## Next Steps

To add more EventSub events:

1. Add the event type to `eventsub_webhook.rs` in the `handle_event_notification` function
2. Create subscriptions via the API for the new event types
3. Create plugins that listen for the emitted events

Example events you can add:
- `channel.update` - Stream title/game changes
- `channel.raid` - Incoming raids (already handled via IRC)
- `channel.poll.begin` - Polls starting
- `channel.prediction.begin` - Predictions starting
