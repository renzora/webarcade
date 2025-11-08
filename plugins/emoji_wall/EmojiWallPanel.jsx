import { createSignal, createEffect, onCleanup, Show, For } from 'solid-js';
import {
  IconSparkles,
  IconPlayerPlay,
  IconPlayerPause,
  IconTrash,
  IconSettings,
  IconChartBar
} from '@tabler/icons-solidjs';
import { WEBARCADE_WS } from '@/api/bridge';

const BRIDGE_URL = 'http://localhost:3001';

export default function EmojiWallPanel() {
  const [enabled, setEnabled] = createSignal(true);
  const [gravity, setGravity] = createSignal(0.5);
  const [bounce, setBounce] = createSignal(0.7);
  const [size, setSize] = createSignal(64);
  const [lifetime, setLifetime] = createSignal(5000);
  const [maxEmotes, setMaxEmotes] = createSignal(50);
  const [customEmoteId, setCustomEmoteId] = createSignal('');
  const [recentEmotes, setRecentEmotes] = createSignal({});
  const [totalEmotesSpawned, setTotalEmotesSpawned] = createSignal(0);

  let ws;

  // Connect to WebSocket to listen for chat messages
  const connectWebSocket = () => {
    ws = new WebSocket(WEBARCADE_WS);

    ws.onopen = () => {
      console.log('✅ Emoji Wall Panel connected to WebSocket');
    };

    ws.onclose = () => {
      console.log('❌ Emoji Wall Panel disconnected from WebSocket');
      setTimeout(connectWebSocket, 3000);
    };

    ws.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data);

        // Track Twitch emotes from chat
        if (data.type === 'twitch_event' && data.event?.type === 'chat_message') {
          const emotes = data.event.emotes || [];

          if (emotes && emotes.length > 0) {
            // Count total emote instances
            let totalInstances = 0;
            emotes.forEach(emote => {
              totalInstances += emote.positions.length;
            });
            setTotalEmotesSpawned(prev => prev + totalInstances);

            // Update emote counts
            setRecentEmotes(prev => {
              const updated = { ...prev };
              emotes.forEach(emote => {
                const key = emote.id;
                if (!updated[key]) {
                  updated[key] = {
                    id: emote.id,
                    name: emote.name || 'emote',
                    count: 0
                  };
                }
                updated[key].count += emote.positions.length;
              });
              return updated;
            });
          }
        }
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
        type: 'emote_wall_update',
        settings: {
          enabled: enabled(),
          gravity: gravity(),
          bounce: bounce(),
          size: size(),
          lifetime: lifetime(),
          maxEmotes: maxEmotes()
        }
      })
    }).catch(err => console.error('Failed to broadcast settings:', err));
  };

  // Toggle enabled state
  const toggleEnabled = () => {
    setEnabled(!enabled());
    broadcastSettings();
  };

  // Update gravity
  const updateGravity = (value) => {
    setGravity(value);
    broadcastSettings();
  };

  // Update bounce
  const updateBounce = (value) => {
    setBounce(value);
    broadcastSettings();
  };

  // Update size
  const updateSize = (value) => {
    setSize(value);
    broadcastSettings();
  };

  // Update lifetime
  const updateLifetime = (value) => {
    setLifetime(value);
    broadcastSettings();
  };

  // Update max emotes
  const updateMaxEmotes = (value) => {
    setMaxEmotes(value);
    broadcastSettings();
  };

  // Spawn a single Twitch emote manually
  const spawnEmote = (emoteId, emoteName) => {
    fetch(`${BRIDGE_URL}/system/broadcast`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        type: 'emote_wall_spawn',
        emoteId: emoteId,
        emoteName: emoteName
      })
    }).catch(err => console.error('Failed to spawn emote:', err));
  };

  // Clear all emotes
  const clearEmotes = () => {
    fetch(`${BRIDGE_URL}/system/broadcast`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        type: 'emote_wall_clear'
      })
    }).catch(err => console.error('Failed to clear emotes:', err));
  };

  // Clear stats
  const clearStats = () => {
    setRecentEmotes({});
    setTotalEmotesSpawned(0);
  };

  // Add custom emote manually
  const addCustomEmote = () => {
    const emoteId = customEmoteId().trim();
    if (emoteId) {
      spawnEmote(emoteId, 'Custom');
      setCustomEmoteId('');
    }
  };

  // Get top emotes sorted by count
  const getTopEmotes = () => {
    const emotes = recentEmotes();
    return Object.values(emotes)
      .sort((a, b) => b.count - a.count)
      .slice(0, 10);
  };

  return (
    <div class="h-full flex flex-col bg-base-200">
      {/* Header */}
      <div class="p-4 bg-base-100 border-b border-base-300">
        <h2 class="text-lg font-semibold flex items-center gap-2">
          <IconSparkles size={24} />
          Emote Wall Control
        </h2>
      </div>

      {/* Controls */}
      <div class="flex-1 overflow-y-auto p-4 space-y-4">
        {/* Enable/Disable Toggle */}
        <div class="card bg-base-100 shadow-sm">
          <div class="card-body p-4">
            <h3 class="font-semibold mb-3">Status</h3>
            <div class="flex items-center justify-between">
              <span class="text-sm">Emote Wall Enabled</span>
              <button
                class={`btn btn-sm gap-2 ${enabled() ? 'btn-error' : 'btn-success'}`}
                onClick={toggleEnabled}
              >
                {enabled() ? (
                  <>
                    <IconPlayerPause size={18} />
                    Stop
                  </>
                ) : (
                  <>
                    <IconPlayerPlay size={18} />
                    Start
                  </>
                )}
              </button>
            </div>
          </div>
        </div>

        {/* Stats */}
        <div class="card bg-base-100 shadow-sm">
          <div class="card-body p-4">
            <h3 class="font-semibold mb-3 flex items-center gap-2">
              <IconChartBar size={20} />
              Chat Emote Stats
            </h3>

            <div class="stats shadow w-full mb-3">
              <div class="stat py-3 px-4">
                <div class="stat-title text-xs">Total Spawned</div>
                <div class="stat-value text-2xl">{totalEmotesSpawned()}</div>
              </div>
            </div>

            <Show when={getTopEmotes().length > 0}>
              <div class="text-sm mb-2 font-semibold">Top Emotes from Chat:</div>
              <div class="flex flex-wrap gap-2 mb-3">
                <For each={getTopEmotes()}>
                  {(emote) => (
                    <div class="badge badge-lg gap-2 py-3">
                      <img
                        src={`https://static-cdn.jtvnw.net/emoticons/v2/${emote.id}/default/dark/1.0`}
                        alt={emote.name}
                        class="w-6 h-6"
                        style={{ 'image-rendering': 'pixelated' }}
                      />
                      <span class="text-xs opacity-70">×{emote.count}</span>
                    </div>
                  )}
                </For>
              </div>
            </Show>

            <button class="btn btn-sm btn-ghost w-full" onClick={clearStats}>
              Clear Stats
            </button>
          </div>
        </div>

        {/* Settings */}
        <div class="card bg-base-100 shadow-sm">
          <div class="card-body p-4">
            <h3 class="font-semibold mb-3 flex items-center gap-2">
              <IconSettings size={20} />
              Physics Settings
            </h3>

            {/* Gravity */}
            <div class="form-control">
              <label class="label">
                <span class="label-text">Gravity</span>
                <span class="label-text-alt">{gravity().toFixed(1)}</span>
              </label>
              <input
                type="range"
                min="0.1"
                max="2"
                step="0.1"
                value={gravity()}
                onInput={(e) => updateGravity(parseFloat(e.target.value))}
                class="range range-sm range-primary"
              />
              <div class="w-full flex justify-between text-xs px-2 text-base-content/60">
                <span>Light</span>
                <span>Heavy</span>
              </div>
            </div>

            {/* Bounce */}
            <div class="form-control mt-4">
              <label class="label">
                <span class="label-text">Bounciness</span>
                <span class="label-text-alt">{(bounce() * 100).toFixed(0)}%</span>
              </label>
              <input
                type="range"
                min="0.1"
                max="0.95"
                step="0.05"
                value={bounce()}
                onInput={(e) => updateBounce(parseFloat(e.target.value))}
                class="range range-sm range-secondary"
              />
              <div class="w-full flex justify-between text-xs px-2 text-base-content/60">
                <span>Dead</span>
                <span>Bouncy</span>
              </div>
            </div>

            {/* Size */}
            <div class="form-control mt-4">
              <label class="label">
                <span class="label-text">Emote Size</span>
                <span class="label-text-alt">{size()}px</span>
              </label>
              <input
                type="range"
                min="24"
                max="128"
                step="4"
                value={size()}
                onInput={(e) => updateSize(parseFloat(e.target.value))}
                class="range range-sm range-accent"
              />
              <div class="w-full flex justify-between text-xs px-2 text-base-content/60">
                <span>Small</span>
                <span>Large</span>
              </div>
            </div>

            {/* Lifetime */}
            <div class="form-control mt-4">
              <label class="label">
                <span class="label-text">Lifetime</span>
                <span class="label-text-alt">{(lifetime() / 1000).toFixed(1)}s</span>
              </label>
              <input
                type="range"
                min="2000"
                max="15000"
                step="500"
                value={lifetime()}
                onInput={(e) => updateLifetime(parseFloat(e.target.value))}
                class="range range-sm range-warning"
              />
              <div class="w-full flex justify-between text-xs px-2 text-base-content/60">
                <span>Quick</span>
                <span>Long</span>
              </div>
            </div>

            {/* Max Emotes */}
            <div class="form-control mt-4">
              <label class="label">
                <span class="label-text">Max Emotes</span>
                <span class="label-text-alt">{maxEmotes()}</span>
              </label>
              <input
                type="range"
                min="10"
                max="200"
                step="10"
                value={maxEmotes()}
                onInput={(e) => updateMaxEmotes(parseFloat(e.target.value))}
                class="range range-sm range-info"
              />
              <div class="w-full flex justify-between text-xs px-2 text-base-content/60">
                <span>Few</span>
                <span>Many</span>
              </div>
            </div>
          </div>
        </div>

        {/* Manual Spawn */}
        <div class="card bg-base-100 shadow-sm">
          <div class="card-body p-4">
            <h3 class="font-semibold mb-3">Test Emote</h3>

            {/* Add Custom Emote */}
            <div class="form-control">
              <label class="label">
                <span class="label-text">Spawn a test emote (enter Emote ID)</span>
              </label>
              <div class="join w-full">
                <input
                  type="text"
                  class="input input-bordered join-item flex-1"
                  placeholder="e.g. 25 for Kappa"
                  value={customEmoteId()}
                  onInput={(e) => setCustomEmoteId(e.target.value)}
                  onKeyPress={(e) => e.key === 'Enter' && addCustomEmote()}
                />
                <button
                  class="btn btn-primary join-item"
                  onClick={addCustomEmote}
                >
                  Spawn
                </button>
              </div>
              <div class="label">
                <span class="label-text-alt">Common IDs: 25 (Kappa), 354 (4Head), 425618 (LUL)</span>
              </div>
            </div>
          </div>
        </div>

        {/* Actions */}
        <div class="card bg-base-100 shadow-sm">
          <div class="card-body p-4">
            <h3 class="font-semibold mb-3">Actions</h3>
            <button
              class="btn btn-error btn-block gap-2"
              onClick={clearEmotes}
            >
              <IconTrash size={18} />
              Clear All Emotes
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
