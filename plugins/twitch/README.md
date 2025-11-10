# Twitch Integration Plugin

A comprehensive Twitch integration plugin for WebArcade that provides broadcaster and bot account support, IRC chat capabilities, and EventSub event subscriptions.

## Features

- **Dual Account Support**: Connect both broadcaster and bot accounts
- **IRC Chat Integration**: Read and send messages in Twitch chat
- **EventSub Support**: Subscribe to and receive Twitch events in real-time
- **Service API**: Expose Twitch functionality to other plugins
- **OAuth Authentication**: Secure OAuth flow for account authorization
- **Automatic Token Refresh**: Keep connections alive with automatic token refresh

## Setup

**Use the "Twitch Setup" plugin to configure this integration:**

1. Open WebArcade and click the **menu button**
2. Select **"Twitch Setup"** from the plugin list
3. Follow the step-by-step setup wizard:
   - **Step 1**: Create a Twitch app and enter Client ID & Secret
   - **Step 2**: Connect your broadcaster account (and optionally bot account)
   - **Step 3**: Verify IRC and EventSub connections

The setup plugin provides a visual interface with instructions for creating your Twitch application at the [Twitch Developer Console](https://dev.twitch.tv/console).

Once configured, you can use the widgets below to interact with Twitch.

## Widgets

The plugin includes two widgets accessible from the system settings:

### TwitchChat.jsx
IRC chat viewer and sender:

- View recent chat messages
- Send messages to chat
- Auto-refresh messages
- Color-coded usernames

### TwitchEvents.jsx
EventSub event viewer:

- View active EventSub subscriptions
- Monitor received events in real-time
- Event type filtering
- JSON event data display

## API Endpoints

### Account Management

- `GET /twitch/accounts` - List connected accounts
- `POST /twitch/auth/start?type=broadcaster|bot` - Start OAuth flow
- `POST /twitch/auth/callback?code=...&state=...` - OAuth callback
- `POST /twitch/auth/refresh` - Refresh all tokens
- `DELETE /twitch/accounts/:type` - Remove account

### IRC

- `GET /twitch/irc/status` - Get IRC connection status
- `POST /twitch/irc/send` - Send chat message
  ```json
  {
    "channel": "channel_name",
    "message": "Hello chat!"
  }
  ```
- `GET /twitch/irc/messages` - Get recent messages
- `POST /twitch/irc/connect` - Connect to IRC
- `POST /twitch/irc/disconnect` - Disconnect from IRC

### EventSub

- `GET /twitch/eventsub/subscriptions` - List subscriptions
- `POST /twitch/eventsub/subscribe` - Create subscription
  ```json
  {
    "type": "channel.follow",
    "condition": {
      "broadcaster_user_id": "123456"
    }
  }
  ```
- `DELETE /twitch/eventsub/subscribe/:id` - Delete subscription
- `GET /twitch/eventsub/events` - Get recent events

### Channel Info

- `GET /twitch/channel/info?channel=username` - Get channel information
- `GET /twitch/user/info?username=username` - Get user information

### Settings

- `GET /twitch/settings` - Get plugin settings
- `POST /twitch/settings` - Update plugin settings

## Service API (For Other Plugins)

Other plugins can use the Twitch integration through the service API:

### Send Chat Message

```rust
let result = ctx.call_service("twitch", "send_chat_message",
    serde_json::json!({
        "channel": "channel_name",
        "message": "Hello from another plugin!"
    })
).await?;
```

### Get Channel Info

```rust
let info = ctx.call_service("twitch", "get_channel_info",
    serde_json::json!({
        "channel": "channel_name"
    })
).await?;
```

### Get Broadcaster Token

```rust
let token_data = ctx.call_service("twitch", "get_broadcaster_token",
    serde_json::json!({})
).await?;

let access_token = token_data["access_token"].as_str().unwrap();
```

## Events

The plugin emits the following events that other plugins can subscribe to:

### chat-message

Emitted when a chat message is received via IRC.

```json
{
  "channel": "channel_name",
  "username": "user123",
  "message": "Hello world!",
  "tags": {
    "color": "#FF0000",
    "display-name": "User123",
    "user-id": "123456"
  }
}
```

### eventsub-event

Emitted when an EventSub event is received.

```json
{
  "type": "channel.follow",
  "data": {
    "user_id": "123456",
    "user_name": "follower123",
    "broadcaster_user_id": "789",
    "broadcaster_user_name": "broadcaster"
  }
}
```

## Database Schema

### twitch_accounts
Stores OAuth tokens and account information for broadcaster and bot accounts.

### twitch_irc_messages
Stores received IRC chat messages with metadata (badges, color, emotes, etc.).

### twitch_eventsub_subscriptions
Tracks active EventSub subscriptions.

### twitch_events
Stores received EventSub events.

### twitch_settings
Plugin configuration storage.

## OAuth Scopes

### Broadcaster Account
- `channel:read:subscriptions` - Read subscription information
- `channel:read:redemptions` - Read channel point redemptions
- `channel:manage:redemptions` - Manage channel point redemptions
- `channel:read:hype_train` - Read hype train events
- `channel:read:polls` - Read polls
- `channel:manage:polls` - Manage polls
- `channel:read:predictions` - Read predictions
- `channel:manage:predictions` - Manage predictions
- `moderator:read:followers` - Read follower information
- `moderator:read:chatters` - Read chatters list
- `chat:read` - Read chat messages
- `chat:edit` - Send chat messages
- `whispers:read` - Read whispers
- `whispers:edit` - Send whispers

### Bot Account
- `chat:read` - Read chat messages
- `chat:edit` - Send chat messages
- `whispers:read` - Read whispers
- `whispers:edit` - Send whispers

## Usage Examples

### From Frontend (JavaScript)

```javascript
import { bridgeFetch } from '@/api/bridge';

// Send a chat message
await bridgeFetch('/twitch/irc/send', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    channel: 'channel_name',
    message: 'Hello from WebArcade!'
  })
});

// Get recent messages
const response = await bridgeFetch('/twitch/irc/messages');
const messages = await response.json();
```

### From Backend (Rust)

```rust
// Emit a chat message event
ctx.emit("twitch:chat-message", &serde_json::json!({
    "channel": "channel_name",
    "username": "user123",
    "message": "Hello!"
}));

// Listen to Twitch events
let mut rx = ctx.subscribe_to("twitch:chat-message").await;
tokio::spawn(async move {
    while let Ok(event) = rx.recv().await {
        log::info!("Chat message: {:?}", event.payload);
    }
});
```

## Architecture

```
┌─────────────────────────────────────────────────────┐
│                  Twitch Plugin                       │
├─────────────────────────────────────────────────────┤
│  Frontend (Widgets)                                  │
│  ├── TwitchAuth.jsx      (OAuth setup)              │
│  ├── TwitchStatus.jsx    (Connection status)        │
│  ├── TwitchChat.jsx      (IRC chat viewer)          │
│  └── TwitchEvents.jsx    (EventSub viewer)          │
├─────────────────────────────────────────────────────┤
│  Backend (Rust)                                      │
│  ├── mod.rs              (Plugin entry point)       │
│  ├── router.rs           (HTTP API routes)          │
│  ├── twitch_api.rs       (OAuth & Twitch API)       │
│  ├── twitch_irc.rs       (IRC client)               │
│  └── twitch_eventsub.rs  (EventSub WebSocket)       │
├─────────────────────────────────────────────────────┤
│  External Services                                   │
│  ├── Twitch OAuth        (Authentication)           │
│  ├── Twitch IRC          (irc.chat.twitch.tv)       │
│  ├── Twitch EventSub     (WebSocket events)         │
│  └── Twitch Helix API    (REST API)                 │
└─────────────────────────────────────────────────────┘
```

## Troubleshooting

### IRC Not Connecting

1. Ensure broadcaster account is connected
2. Check that OAuth tokens are valid
3. Verify the broadcaster username is correct
4. Check backend logs for connection errors

### OAuth Callback Not Working

1. Verify redirect URI matches Twitch app settings exactly
2. Ensure Client ID and Secret are correct
3. Check that the bridge is running on port 3001

### EventSub Not Receiving Events

1. Confirm broadcaster account has required scopes
2. Check EventSub subscriptions are active
3. Verify WebSocket connection in logs
4. Ensure broadcaster user ID is correct in subscriptions

## Dependencies

The plugin requires these Rust crates (add to `Cargo.toml` if needed):

```toml
[dependencies]
reqwest = { version = "0.11", features = ["json"] }
tokio-tungstenite = "0.21"
futures-util = "0.3"
lazy_static = "1.4"
urlencoding = "2.1"
```

## License

Part of the WebArcade project.

## Support

For issues and questions, refer to the main WebArcade documentation or create an issue in the repository.
