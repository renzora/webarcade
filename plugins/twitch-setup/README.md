# Twitch Setup Plugin

A dedicated plugin for configuring the Twitch integration in WebArcade. This plugin provides a comprehensive viewport for managing Twitch app credentials and account connections.

## Features

- **Interactive Setup Wizard**: Step-by-step configuration process
- **Visual Progress Tracking**: See your setup completion at a glance
- **Account Management**: Connect and manage broadcaster and bot accounts
- **Real-time Status**: Monitor IRC and EventSub connections
- **Side Panel**: Quick status overview in the right panel

## What This Plugin Does

The Twitch Setup plugin serves as the central configuration hub for all Twitch integration features:

1. **App Credentials Configuration** (Step 1)
   - Enter Twitch app Client ID
   - Enter Twitch app Client Secret
   - Stores credentials securely in database
   - Provides instructions for creating a Twitch app

2. **Account Connection** (Step 2)
   - OAuth flow for broadcaster account
   - Optional bot account connection
   - Account removal and management
   - Token refresh functionality

3. **Connection Status** (Step 3)
   - IRC connection status
   - EventSub WebSocket status
   - Channel information
   - Feature availability

## Usage

### Opening the Plugin

1. Click the menu button in WebArcade
2. Select "Twitch Setup" from the plugin list
3. The setup viewport will open with a comprehensive interface

### Setup Process

**Step 1: Configure App Credentials**
1. Visit [Twitch Developer Console](https://dev.twitch.tv/console)
2. Create or select an application
3. Add OAuth redirect URL: `http://localhost:3001/twitch/auth/callback`
4. Copy Client ID and Client Secret
5. Paste into the form and save

**Step 2: Connect Accounts**
1. Click "Connect Broadcaster" button
2. Authorize in the browser popup
3. (Optional) Click "Connect Bot" for a separate bot account
4. Verify connections appear with checkmarks

**Step 3: Verify Status**
1. Check that IRC shows "Connected"
2. Verify your channel name appears
3. Confirm setup progress shows 100%

### Right Panel

The side panel provides a quick status overview:
- Setup progress percentage
- Checklist of completed steps
- Quick status indicators
- Overall setup health

## Layout

### Viewport (Main View)

The viewport uses a two-column layout:

**Left Column:**
- App credentials form
- Setup instructions
- Progress flow diagram

**Right Column:**
- Account connection cards
- Connection status
- Quick links and next steps

### Panel (Side View)

Compact status display showing:
- Progress bar
- Status checklist
- Overall health indicator
- Quick tips

## Integration with Twitch Plugin

This plugin works in tandem with the main `twitch` plugin:

- **Twitch Plugin**: Provides the backend (IRC, EventSub, API)
- **Twitch Setup Plugin**: Provides the UI for configuration

All configuration is stored in the database and shared between plugins.

## Widgets vs Plugin

Unlike the widget-based approach, this plugin:
- ✅ Has its own dedicated viewport
- ✅ Provides a full-screen configuration experience
- ✅ Includes a side panel for quick status
- ✅ Appears in the main plugin menu
- ✅ Offers better UX for initial setup
- ✅ Can be opened independently

## Development

### File Structure

```
plugins/twitch-setup/
├── index.jsx     # Plugin entry point
├── Viewport.jsx  # Main setup interface
├── Panel.jsx     # Side panel status
└── README.md     # This file
```

### Key Components

**Viewport.jsx**
- Two-column responsive layout
- Real-time status updates
- Form validation and error handling
- OAuth popup integration
- Auto-refresh every 30 seconds

**Panel.jsx**
- Compact status display
- Progress tracking
- Status checklist
- Auto-refresh every 10 seconds

### State Management

The plugin uses Solid.js signals for reactive state:
- `isConfigured` - App credentials status
- `accounts` - Connected accounts array
- `ircStatus` - IRC connection state
- `loading`, `saving` - UI state
- `error`, `success` - User feedback

### API Endpoints Used

- `GET /twitch/setup/status` - Check configuration
- `POST /twitch/setup` - Save credentials
- `GET /twitch/accounts` - List accounts
- `GET /twitch/auth/start` - Start OAuth flow
- `DELETE /twitch/accounts/:type` - Remove account
- `POST /twitch/auth/refresh` - Refresh tokens
- `GET /twitch/irc/status` - IRC status

## Screenshots

### Main Viewport
- Full-screen setup interface
- Visual step-by-step process
- Real-time status indicators

### Side Panel
- Compact progress view
- Quick status check
- Minimal UI for at-a-glance info

## Tips

1. **Complete Setup in Order**: Follow steps 1→2→3 for best results
2. **Use the Panel**: Keep an eye on status without opening the full viewport
3. **Refresh Tokens**: If connections fail, try refreshing tokens
4. **Browser Popups**: Allow popups for OAuth authorization
5. **Check Logs**: Backend logs show detailed connection info

## Troubleshooting

### "Failed to load configuration data"
- Ensure backend is running
- Check database connection
- Verify API endpoints are accessible

### OAuth Not Working
- Check redirect URI matches exactly
- Verify Client ID and Secret are correct
- Ensure browser allows popups

### IRC Not Connecting
- Verify broadcaster account is connected
- Check that tokens are valid
- Try refreshing tokens manually

## Related

- **Main Plugin**: `plugins/twitch/` - Backend implementation
- **Chat Widget**: `plugins/twitch/widgets/TwitchChat.jsx`
- **Events Widget**: `plugins/twitch/widgets/TwitchEvents.jsx`
- **Documentation**: `plugins/twitch/README.md`

## License

Part of the WebArcade project.
