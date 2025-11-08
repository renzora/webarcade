import { createSignal, createEffect, onCleanup } from 'solid-js';
import {
  IconHighlight,
  IconPlayerPlay,
  IconPlayerPause,
  IconTrash,
  IconSettings,
  IconTestPipe
} from '@tabler/icons-solidjs';
import { WEBARCADE_WS } from '@/api/bridge';

const BRIDGE_URL = 'http://localhost:3001';

export default function ChatHighlightPanel() {
  const [enabled, setEnabled] = createSignal(true);
  const [duration, setDuration] = createSignal(8000);
  const [maxHighlights, setMaxHighlights] = createSignal(5);
  const [position, setPosition] = createSignal('top');
  const [animation, setAnimation] = createSignal('slide');
  const [testMessage, setTestMessage] = createSignal('This is a test highlight message!');
  const [testUsername, setTestUsername] = createSignal('TestUser');

  let ws;

  // Connect to WebSocket
  const connectWebSocket = () => {
    ws = new WebSocket(WEBARCADE_WS);

    ws.onopen = () => {
      console.log('✅ Chat Highlight Panel connected to WebSocket');
    };

    ws.onclose = () => {
      console.log('❌ Chat Highlight Panel disconnected from WebSocket');
      setTimeout(connectWebSocket, 3000);
    };

    ws.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data);
        // Handle any incoming messages if needed
      } catch (error) {
        console.error('Error parsing WebSocket message:', error);
      }
    };
  };

  // Connect on mount
  createEffect(() => {
    connectWebSocket();
    onCleanup(() => {
      if (ws) {
        ws.close();
      }
    });
  });

  // Send settings update to overlay
  const broadcastSettings = () => {
    fetch(`${BRIDGE_URL}/system/broadcast`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        type: 'chat_highlight_settings',
        settings: {
          enabled: enabled(),
          duration: duration(),
          maxHighlights: maxHighlights(),
          position: position(),
          animation: animation(),
        }
      })
    }).catch(err => console.error('Failed to broadcast settings:', err));
  };

  // Toggle enabled state
  const toggleEnabled = () => {
    setEnabled(!enabled());
    broadcastSettings();
  };

  // Update setting and broadcast
  const updateSetting = (setter) => (value) => {
    setter(value);
    broadcastSettings();
  };

  // Send a test highlight
  const sendTestHighlight = () => {
    fetch(`${BRIDGE_URL}/system/broadcast`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        type: 'chat_highlight',
        username: testUsername().toLowerCase(),
        display_name: testUsername(),
        message: testMessage(),
        color: '#9147ff',
        profile_image_url: null,
        badges: ['Test'],
      })
    }).catch(err => console.error('Failed to send test highlight:', err));
  };

  // Clear all highlights
  const clearHighlights = () => {
    fetch(`${BRIDGE_URL}/system/broadcast`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        type: 'chat_highlight_clear'
      })
    }).catch(err => console.error('Failed to clear highlights:', err));
  };

  return (
    <div class="h-full flex flex-col bg-base-200 overflow-y-auto">
      <div class="p-6 space-y-6">
        {/* Header */}
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-3">
            <IconHighlight class="w-8 h-8 text-primary" />
            <div>
              <h2 class="text-2xl font-bold">Chat Highlight</h2>
              <p class="text-sm text-base-content/60">
                Display highlighted messages with !highlight command
              </p>
            </div>
          </div>
          <button
            onClick={toggleEnabled}
            class={`btn ${enabled() ? 'btn-success' : 'btn-error'} gap-2`}
          >
            {enabled() ? <IconPlayerPlay /> : <IconPlayerPause />}
            {enabled() ? 'Enabled' : 'Disabled'}
          </button>
        </div>

        {/* Quick Actions */}
        <div class="card bg-base-100 shadow-xl">
          <div class="card-body">
            <h3 class="card-title text-lg flex items-center gap-2">
              <IconTestPipe class="w-5 h-5" />
              Quick Actions
            </h3>

            <div class="space-y-3">
              <button
                onClick={sendTestHighlight}
                class="btn btn-primary btn-block gap-2"
              >
                <IconTestPipe />
                Send Test Highlight
              </button>

              <button
                onClick={clearHighlights}
                class="btn btn-error btn-outline btn-block gap-2"
              >
                <IconTrash />
                Clear All Highlights
              </button>
            </div>
          </div>
        </div>

        {/* Test Settings */}
        <div class="card bg-base-100 shadow-xl">
          <div class="card-body">
            <h3 class="card-title text-lg">Test Message Settings</h3>

            <div class="form-control">
              <label class="label">
                <span class="label-text">Test Username</span>
              </label>
              <input
                type="text"
                value={testUsername()}
                onInput={(e) => setTestUsername(e.target.value)}
                class="input input-bordered"
                placeholder="TestUser"
              />
            </div>

            <div class="form-control">
              <label class="label">
                <span class="label-text">Test Message</span>
              </label>
              <textarea
                value={testMessage()}
                onInput={(e) => setTestMessage(e.target.value)}
                class="textarea textarea-bordered h-24"
                placeholder="Enter test message..."
              />
            </div>
          </div>
        </div>

        {/* Overlay Settings */}
        <div class="card bg-base-100 shadow-xl">
          <div class="card-body">
            <h3 class="card-title text-lg flex items-center gap-2">
              <IconSettings class="w-5 h-5" />
              Overlay Settings
            </h3>

            {/* Duration */}
            <div class="form-control">
              <label class="label">
                <span class="label-text">Display Duration</span>
                <span class="label-text-alt">{(duration() / 1000).toFixed(1)}s</span>
              </label>
              <input
                type="range"
                min="3000"
                max="30000"
                step="1000"
                value={duration()}
                onInput={(e) => updateSetting(setDuration)(parseInt(e.target.value))}
                class="range range-primary"
              />
              <div class="w-full flex justify-between text-xs px-2 mt-1">
                <span>3s</span>
                <span>15s</span>
                <span>30s</span>
              </div>
            </div>

            {/* Max Highlights */}
            <div class="form-control">
              <label class="label">
                <span class="label-text">Max Concurrent Highlights</span>
                <span class="label-text-alt">{maxHighlights()}</span>
              </label>
              <input
                type="range"
                min="1"
                max="10"
                step="1"
                value={maxHighlights()}
                onInput={(e) => updateSetting(setMaxHighlights)(parseInt(e.target.value))}
                class="range range-primary"
              />
              <div class="w-full flex justify-between text-xs px-2 mt-1">
                <span>1</span>
                <span>5</span>
                <span>10</span>
              </div>
            </div>

            {/* Position */}
            <div class="form-control">
              <label class="label">
                <span class="label-text">Position</span>
              </label>
              <select
                value={position()}
                onChange={(e) => updateSetting(setPosition)(e.target.value)}
                class="select select-bordered"
              >
                <option value="top">Top</option>
                <option value="center">Center</option>
                <option value="bottom">Bottom</option>
              </select>
            </div>

            {/* Animation */}
            <div class="form-control">
              <label class="label">
                <span class="label-text">Animation Style</span>
              </label>
              <select
                value={animation()}
                onChange={(e) => updateSetting(setAnimation)(e.target.value)}
                class="select select-bordered"
              >
                <option value="slide">Slide</option>
                <option value="fade">Fade</option>
                <option value="zoom">Zoom</option>
              </select>
            </div>
          </div>
        </div>

        {/* Command Info */}
        <div class="card bg-base-100 shadow-xl border-2 border-primary">
          <div class="card-body">
            <h3 class="card-title text-lg">Chat Command</h3>
            <div class="space-y-2">
              <div class="bg-base-200 p-3 rounded-lg font-mono text-sm">
                <strong class="text-primary">!highlight</strong> [your message here]
              </div>
              <p class="text-sm text-base-content/70">
                Users can type this command in chat to display a highlighted message on stream.
              </p>
              <div class="text-xs text-base-content/60 space-y-1">
                <p><strong>Example:</strong> !highlight Check out my new project!</p>
                <p><strong>Note:</strong> The message will be displayed prominently on the overlay.</p>
              </div>
            </div>
          </div>
        </div>

        {/* Info */}
        <div class="alert alert-info">
          <div class="flex flex-col gap-1">
            <span class="font-semibold">How it works:</span>
            <span class="text-sm">
              When a viewer types !highlight followed by a message in chat, their message
              will be displayed prominently on the stream overlay with animations and effects.
            </span>
          </div>
        </div>
      </div>
    </div>
  );
}
