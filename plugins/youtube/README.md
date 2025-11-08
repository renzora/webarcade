# YouTube Plugin for WebArcade

A full-featured YouTube integration plugin that allows you to connect to your YouTube channels and view detailed analytics.

## Features

- **OAuth 2.0 Authentication**: Secure YouTube account connection
- **Channel Management**: View and manage multiple YouTube channels
- **Analytics Dashboard**: Comprehensive analytics including:
  - Views and watch time
  - Subscriber changes
  - Average view duration
  - Likes, comments, and shares
  - Custom date ranges (7, 30, 90 days, or custom)
  - Detailed analytics reports

## Setup Instructions

### 1. Create Google Cloud Project

1. Go to [Google Cloud Console](https://console.cloud.google.com/)
2. Create a new project or select an existing one
3. Enable the following APIs:
   - YouTube Data API v3
   - YouTube Analytics API

### 2. Create OAuth 2.0 Credentials

1. Navigate to **APIs & Services** > **Credentials**
2. Click **Create Credentials** > **OAuth client ID**
3. Select **Web application** as the application type
4. Add the following to **Authorized redirect URIs**:
   ```
   http://localhost:3000/api/plugin/youtube/auth/callback
   ```
5. Copy the **Client ID** and **Client Secret**

### 3. Configure the Plugin

1. Start your WebArcade application
2. Navigate to **YouTube Settings** in the plugin menu
3. Enter your **Client ID** and **Client Secret**
4. Click **Save Credentials**
5. Click **Connect YouTube Account** to authenticate

### 4. Required OAuth Scopes

The plugin requests the following scopes:
- `https://www.googleapis.com/auth/youtube.readonly` - Read YouTube account data
- `https://www.googleapis.com/auth/yt-analytics.readonly` - Read YouTube Analytics data
- `https://www.googleapis.com/auth/userinfo.profile` - Basic profile information

## Plugin Structure

### Backend (Rust)
- **`mod.rs`**: Main plugin entry point with database migrations
- **`auth.rs`**: OAuth authentication flow and token management
- **`api.rs`**: YouTube Data API and Analytics API integration
- **`router.rs`**: HTTP route handlers for REST API endpoints
- **`events.rs`**: Event handling and emission

### Frontend (SolidJS)
- **`index.jsx`**: Plugin registration and lifecycle
- **`YouTubeStore.jsx`**: State management and API client
- **`YouTubeSettingsViewport.jsx`**: OAuth configuration UI
- **`YouTubeAnalyticsViewport.jsx`**: Analytics dashboard
- **`YouTubeChannelsPanel.jsx`**: Channel list panel

## API Endpoints

### Authentication
- `GET /api/plugin/youtube/auth/url` - Get OAuth authorization URL
- `GET /api/plugin/youtube/auth/callback` - OAuth callback handler
- `POST /api/plugin/youtube/auth/refresh` - Refresh access token
- `POST /api/plugin/youtube/auth/revoke` - Revoke access token
- `GET /api/plugin/youtube/auth/status` - Get authentication status

### Channels
- `GET /api/plugin/youtube/channels` - Get authenticated user's channels
- `GET /api/plugin/youtube/channels/:id` - Get specific channel details

### Analytics
- `GET /api/plugin/youtube/analytics/:channel_id` - Get channel analytics summary
  - Query params: `start_date`, `end_date`
- `GET /api/plugin/youtube/analytics/:channel_id/report` - Get detailed analytics report
  - Query params: `start_date`, `end_date`, `metrics`, `dimensions`

## Database Schema

### `youtube_auth`
Stores OAuth authentication tokens for the connected YouTube account.

### `youtube_channels`
Caches channel information including subscriber counts, video counts, and view counts.

### `youtube_analytics_cache`
Caches analytics data to reduce API calls.

## Usage

1. **Connect Your Account**: Go to YouTube Settings and authenticate
2. **View Channels**: Your channels will appear in the YouTube panel
3. **Select a Channel**: Click on a channel to select it
4. **View Analytics**: Navigate to YouTube Analytics to see detailed metrics
5. **Customize Date Range**: Choose from preset ranges or set a custom date range

## Analytics Metrics

The plugin provides the following metrics:

- **Views**: Total video views
- **Watch Time**: Total minutes watched
- **Subscriber Change**: Net subscriber gain/loss
- **Average View Duration**: Average time viewers watch your videos
- **Likes**: Total likes received
- **Comments**: Total comments received
- **Shares**: Total shares of your videos

## Troubleshooting

### Authentication Issues
- Ensure your Client ID and Client Secret are correct
- Verify the redirect URI is exactly: `http://localhost:3000/api/plugin/youtube/auth/callback`
- Check that the YouTube Data API v3 and YouTube Analytics API are enabled

### API Quota Limits
- YouTube API has daily quota limits
- Analytics data is cached to reduce API calls
- If you hit quota limits, wait 24 hours for the quota to reset

### Token Expiration
- Access tokens are automatically refreshed when they expire
- If you encounter authentication errors, try disconnecting and reconnecting your account

## Development

To modify the plugin:

1. **Backend changes**: Edit files in `plugins/youtube/`
2. **Frontend changes**: Edit files in `plugins/youtube/*.jsx`
3. **Rebuild**: Run `bun run discover` to register changes
4. **Compile**: Run `cargo build --manifest-path bridge/Cargo.toml`

## License

Part of the WebArcade project.
