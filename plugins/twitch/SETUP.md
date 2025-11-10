# Twitch Plugin Setup Guide

## Quick Start

The Twitch plugin is now installed and ready to configure! Follow these steps:

### 1. Open Twitch Setup Plugin

1. Click the **menu button** in WebArcade (top navigation)
2. Select **"Twitch Setup"** from the plugin list
3. The Twitch Setup interface will open with a full-screen wizard

### 2. Configure Twitch App Credentials (Step 1 in Setup)

1. Follow the instructions in the **left panel** of the setup interface
2. Visit https://dev.twitch.tv/console
3. Register a new Twitch application
4. Add OAuth redirect URL: `http://localhost:3001/twitch/auth/callback`
5. Copy your Client ID and Client Secret
6. Paste them into the setup form and click **"Save Credentials"**

### 3. Connect Your Accounts (Step 2 in Setup)

1. In the **right panel**, click **"Connect Broadcaster"**
   - This opens your browser for OAuth authorization
   - Authorize the app with your main Twitch account
2. (Optional) Click **"Connect Bot"**
   - Use a separate account for bot functionality
   - If you don't connect a bot, the broadcaster account will be used

### 4. You're Done!

The plugin will automatically:
- Connect to Twitch IRC for chat
- Set up EventSub for real-time events
- Refresh tokens automatically
- Provide services to other plugins

## Available Components

### Twitch Setup Plugin
A dedicated plugin with full viewport for configuring Twitch integration:
- Step-by-step setup wizard
- App credentials configuration
- Account connection interface
- Real-time status monitoring
- Visual progress tracking

Access it from the main menu under **"Twitch Setup"**.

### Widgets

### TwitchChat.jsx
IRC chat interface:
- View recent chat messages
- Send messages to chat
- Auto-refreshing message feed

### TwitchEvents.jsx
EventSub event monitor:
- View active subscriptions
- Monitor received events
- JSON event data display

## Using Twitch Integration in Other Plugins

### Send Chat Messages (Rust)

```rust
let result = ctx.call_service("twitch", "send_chat_message",
    serde_json::json!({
        "channel": "channel_name",
        "message": "Hello from my plugin!"
    })
).await?;
```

### Get Channel Info (Rust)

```rust
let info = ctx.call_service("twitch", "get_channel_info",
    serde_json::json!({
        "channel": "channel_name"
    })
).await?;
```

### Get Access Token (Rust)

```rust
let token_data = ctx.call_service("twitch", "get_broadcaster_token",
    serde_json::json!({})
).await?;

let access_token = token_data["access_token"].as_str().unwrap();
```

### Listen to Chat Messages (Rust)

```rust
let mut rx = ctx.subscribe_to("twitch:chat-message").await;
tokio::spawn(async move {
    while let Ok(event) = rx.recv().await {
        if let Some(payload) = event.payload {
            log::info!("Chat: {} - {}",
                payload["username"].as_str().unwrap_or("unknown"),
                payload["message"].as_str().unwrap_or("")
            );
        }
    }
});
```

### From Frontend (JavaScript)

```javascript
import { bridgeFetch } from '@/api/bridge';

// Send a chat message
await bridgeFetch('/twitch/irc/send', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    channel: 'channel_name',
    message: 'Hello from JavaScript!'
  })
});

// Get recent messages
const response = await bridgeFetch('/twitch/irc/messages');
const messages = await response.json();
```

## Troubleshooting

### "Failed to load configuration status"
- Make sure the backend server is running
- Check database permissions

### OAuth Callback Not Working
- Verify redirect URI matches exactly: `http://localhost:3001/twitch/auth/callback`
- Ensure the bridge is running on port 3001
- Check browser popup blockers

### IRC Not Connecting
- Ensure broadcaster account is connected
- Check that tokens are valid (try refreshing in TwitchAuth widget)
- Verify broadcaster username is correct
- Check backend logs for connection errors

### EventSub Not Receiving Events
- Confirm broadcaster account has required scopes
- Check EventSub subscriptions are active
- Verify WebSocket connection in logs
- Ensure client ID is correct

## Development

### File Structure

```
plugins/twitch/
├── index.jsx              # Frontend plugin entry
├── mod.rs                 # Backend plugin implementation
├── router.rs              # HTTP API routes
├── twitch_api.rs          # OAuth & Twitch API integration
├── twitch_irc.rs          # IRC client
├── twitch_eventsub.rs     # EventSub WebSocket client
├── widgets/
│   ├── TwitchSetup.jsx    # App credentials setup
│   ├── TwitchAuth.jsx     # OAuth account connection
│   ├── TwitchStatus.jsx   # Connection status dashboard
│   ├── TwitchChat.jsx     # IRC chat viewer/sender
│   └── TwitchEvents.jsx   # EventSub event viewer
└── README.md              # Full documentation
```

### Database Tables

- `twitch_accounts` - OAuth tokens and account info
- `twitch_irc_messages` - IRC chat message history
- `twitch_eventsub_subscriptions` - EventSub subscription tracking
- `twitch_events` - EventSub event history
- `twitch_settings` - Plugin settings (client_id, client_secret, etc.)

### API Endpoints

See README.md for full API documentation.

## Support

For issues and questions, refer to:
- README.md - Full plugin documentation
- Main WebArcade documentation
- GitHub repository issues

## Security Notes

- Client Secret is stored in the database - keep your database secure
- Never share your Client Secret publicly
- Tokens are automatically refreshed to maintain connections
- OAuth tokens are stored securely in the local database
