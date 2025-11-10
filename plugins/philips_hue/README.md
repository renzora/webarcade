# Philips Hue Plugin

Control your Philips Hue smart lights directly from WebArcade!

## Features

### 🌉 Bridge Management
- Add multiple Hue Bridges by IP address
- Easy one-click pairing process
- Automatic light and room discovery

### 💡 Individual Light Control
- Toggle lights on/off
- Adjust brightness with smooth sliders
- Change colors with preset buttons and color wheel (for color bulbs)
- White temperature control (for white ambiance bulbs)
- Real-time state synchronization

### 🏠 Room/Group Control
- Control all lights in a room simultaneously
- Quick brightness presets (Low/Medium/Full)
- Group on/off toggle

## Getting Started

### 1. Find Your Hue Bridge IP Address

You can find your bridge's IP address in several ways:

- **Router Method**: Check your router's DHCP client list for a device named "Philips-hue"
- **Discovery Service**: Visit https://discovery.meethue.com/
- **Mobile App**: Check the Hue app settings under "Bridge" → "Network Settings"

### 2. Add Your Bridge

1. Open the **Hue Bridge Setup** widget in your dashboard
2. Click the **+** button to add a new bridge
3. Enter a friendly name (e.g., "Living Room Bridge")
4. Enter your bridge's IP address (e.g., `192.168.1.100`)
5. Click **Add**

### 3. Pair with Your Bridge

1. Click the **Pair** button next to your newly added bridge
2. **Immediately press the physical link button** on top of your Hue Bridge
3. Wait for the confirmation message
4. Once paired, the bridge status will show "✓ Paired"

### 4. Discover Your Lights

1. Click the **Discover Lights** button
2. The plugin will automatically find all lights and rooms connected to your bridge
3. Your lights will now appear in the **Light Control** and **Room Control** widgets

## Widgets

### Hue Bridge Setup
**File**: `widgets/HueBridgeSetup.jsx`

Manage your Hue Bridge connections. Add, pair, and discover lights from your bridges.

### Light Control
**File**: `widgets/LightControl.jsx`

Control individual lights with full color and brightness control.

**Features:**
- On/Off toggle
- Brightness slider (0-100%)
- Color presets (Red, Green, Blue, Yellow, Purple, Cyan)
- Full color wheel slider
- White temperature control (for compatible bulbs)
- Reachability status indicator

### Room Control
**File**: `widgets/RoomControl.jsx`

Control entire rooms or groups of lights at once.

**Features:**
- Group on/off toggle
- Brightness slider
- Quick brightness presets (Low/Medium/Full)
- Light count indicator

## API Endpoints

All API endpoints are available at `http://localhost:3001/philips-hue/`

### Bridge Management
- `GET /bridges` - List all configured bridges
- `POST /bridges` - Add a new bridge
  ```json
  {
    "name": "Living Room Bridge",
    "ip_address": "192.168.1.100"
  }
  ```
- `DELETE /bridges/:bridge_id` - Delete a bridge
- `POST /bridges/:bridge_id/pair` - Pair with a bridge (press button first!)
- `GET /bridges/:bridge_id/discover` - Discover lights from a bridge

### Light Control
- `GET /lights` - List all discovered lights
- `GET /lights/:light_id` - Get a specific light's state
- `PUT /lights/:light_id/state` - Update a light's state
  ```json
  {
    "on": true,
    "bri": 254,
    "hue": 25500,
    "sat": 254
  }
  ```

### Group Control
- `GET /groups` - List all groups/rooms
- `PUT /groups/:group_id/state` - Update a group's state
  ```json
  {
    "on": true,
    "bri": 127
  }
  ```

## Database Schema

The plugin creates three tables to store your Hue configuration:

### `hue_bridges`
Stores configured Hue Bridge information.

### `hue_lights`
Caches light states for faster loading and offline reference.

### `hue_groups`
Caches room/group information and states.

## Troubleshooting

### Bridge won't pair
- Make sure you pressed the physical button on the bridge within 30 seconds
- Check that your bridge IP address is correct
- Ensure your computer and bridge are on the same network

### Lights not appearing
- Make sure you've paired with the bridge first
- Click "Discover Lights" to refresh the light list
- Check that your lights are powered on and reachable

### Can't control lights
- Verify the bridge is still online
- Check that the lights are marked as "reachable"
- Try re-discovering lights from the bridge

### Colors not changing
- Some Hue bulbs only support white/temperature control
- Make sure you're using color-capable bulbs (e.g., Hue Color, Hue Color Ambiance)

## Technical Details

### Backend (Rust)
- **Location**: `plugins/philips_hue/`
- **Main Module**: `mod.rs` - Plugin initialization and database setup
- **Router**: `router.rs` - HTTP API endpoints for Hue control
- **Dependencies**: Uses `reqwest` for HTTP communication with Hue Bridge

### Frontend (SolidJS)
- **Location**: `plugins/philips_hue/`
- **Entry Point**: `index.jsx`
- **Widgets**: Auto-discovered from `widgets/` directory
- **State Management**: Local component state with API synchronization

### Hue API
This plugin uses the official Philips Hue REST API (v1). The API documentation is available at:
https://developers.meethue.com/develop/hue-api/

## License

Part of the WebArcade framework.

## Support

For issues or feature requests, please check the main WebArcade documentation.
