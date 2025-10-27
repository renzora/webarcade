import { createPlugin } from '@/api/plugin';
import { createSignal, onMount, createEffect, onCleanup, Show, For } from 'solid-js';
import { IconNews } from '@tabler/icons-solidjs';

const WEBARCADE_API = 'http://localhost:3001';

const TickerViewport = () => {
  const [messages, setMessages] = createSignal([]);
  const [streamStartDate, setStreamStartDate] = createSignal('');
  const [tickerSpeed, setTickerSpeed] = createSignal(30);
  const [tempTickerSpeed, setTempTickerSpeed] = createSignal(30);
  const [maxTickerItems, setMaxTickerItems] = createSignal(20);
  const [tempMaxTickerItems, setTempMaxTickerItems] = createSignal(20);
  const [currentDate, setCurrentDate] = createSignal(new Date());
  const [newMessage, setNewMessage] = createSignal('');
  const [editingId, setEditingId] = createSignal(null);
  const [editMessage, setEditMessage] = createSignal('');
  const [loading, setLoading] = createSignal(false);
  const [eventsConfig, setEventsConfig] = createSignal({
    show_followers: true,
    show_subscribers: true,
    show_raids: true,
    show_donations: true,
    show_gifted_subs: true,
    show_cheers: true
  });

  let speedUpdateTimeout;
  let maxItemsUpdateTimeout;

  // Load ticker messages
  const loadMessages = async () => {
    try {
      const response = await fetch(`${WEBARCADE_API}/api/ticker/messages`);
      const data = await response.json();
      setMessages(data);
    } catch (error) {
      console.error('Failed to load ticker messages:', error);
    }
  };

  // Load status config (start date, ticker speed, and max items)
  const loadStatusConfig = async () => {
    try {
      const response = await fetch(`${WEBARCADE_API}/api/status/config`);
      const data = await response.json();
      setStreamStartDate(data.stream_start_date || '');
      setTickerSpeed(data.ticker_speed || 30);
      setTempTickerSpeed(data.ticker_speed || 30);
      setMaxTickerItems(data.max_ticker_items || 20);
      setTempMaxTickerItems(data.max_ticker_items || 20);
    } catch (error) {
      console.error('Failed to load status config:', error);
    }
  };

  // Load ticker events config
  const loadEventsConfig = async () => {
    try {
      const response = await fetch(`${WEBARCADE_API}/api/ticker/events/config`);
      const data = await response.json();
      setEventsConfig(data);
    } catch (error) {
      console.error('Failed to load events config:', error);
    }
  };

  // Toggle event type
  const toggleEventType = async (eventType) => {
    const newConfig = {
      ...eventsConfig(),
      [eventType]: !eventsConfig()[eventType]
    };

    setLoading(true);
    try {
      await fetch(`${WEBARCADE_API}/api/ticker/events/config`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          ...newConfig,
          updated_at: Math.floor(Date.now() / 1000)
        })
      });
      setEventsConfig(newConfig);
    } catch (error) {
      console.error('Failed to update events config:', error);
    }
    setLoading(false);
  };

  // Add new message
  const addMessage = async () => {
    const msg = newMessage().trim();
    if (!msg) return;

    setLoading(true);
    try {
      await fetch(`${WEBARCADE_API}/api/ticker/messages`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ message: msg })
      });
      setNewMessage('');
      await loadMessages();
    } catch (error) {
      console.error('Failed to add message:', error);
    }
    setLoading(false);
  };

  // Update message
  const updateMessage = async (id) => {
    const msg = editMessage().trim();
    if (!msg) return;

    setLoading(true);
    try {
      const message = messages().find(m => m.id === id);
      await fetch(`${WEBARCADE_API}/api/ticker/messages`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ id, message: msg, enabled: message.enabled, is_sticky: message.is_sticky })
      });
      setEditingId(null);
      setEditMessage('');
      await loadMessages();
    } catch (error) {
      console.error('Failed to update message:', error);
    }
    setLoading(false);
  };

  // Toggle message sticky state
  const toggleMessageSticky = async (id) => {
    setLoading(true);
    try {
      await fetch(`${WEBARCADE_API}/api/ticker/messages/toggle-sticky`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ id })
      });
      await loadMessages();
    } catch (error) {
      console.error('Failed to toggle sticky:', error);
    }
    setLoading(false);
  };

  // Delete message
  const deleteMessage = async (id) => {
    if (!confirm('Delete this ticker message?')) return;

    setLoading(true);
    try {
      await fetch(`${WEBARCADE_API}/api/ticker/messages`, {
        method: 'DELETE',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ id })
      });
      await loadMessages();
    } catch (error) {
      console.error('Failed to delete message:', error);
    }
    setLoading(false);
  };

  // Toggle message enabled/disabled
  const toggleMessage = async (id) => {
    setLoading(true);
    try {
      await fetch(`${WEBARCADE_API}/api/ticker/messages/toggle`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ id })
      });
      await loadMessages();
    } catch (error) {
      console.error('Failed to toggle message:', error);
    }
    setLoading(false);
  };

  // Update stream start date
  const updateStreamStartDate = async (date) => {
    setLoading(true);
    try {
      await fetch(`${WEBARCADE_API}/api/status/start-date`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ start_date: date || null })
      });
      setStreamStartDate(date);
    } catch (error) {
      console.error('Failed to update stream start date:', error);
    }
    setLoading(false);
  };

  // Update ticker speed (debounced)
  const updateTickerSpeed = async (speed) => {
    try {
      await fetch(`${WEBARCADE_API}/api/status/ticker-speed`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ speed: parseInt(speed) })
      });
      setTickerSpeed(speed);
    } catch (error) {
      console.error('Failed to update ticker speed:', error);
    }
  };

  // Handle slider change with debounce
  const handleSpeedSliderChange = (value) => {
    setTempTickerSpeed(value);

    // Clear existing timeout
    if (speedUpdateTimeout) {
      clearTimeout(speedUpdateTimeout);
    }

    // Set new timeout to update after 500ms of no changes
    speedUpdateTimeout = setTimeout(() => {
      updateTickerSpeed(value);
    }, 500);
  };

  // Update max ticker items
  const updateMaxTickerItems = async (maxItems) => {
    try {
      await fetch(`${WEBARCADE_API}/api/status/max-ticker-items`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ max_items: parseInt(maxItems) })
      });
      setMaxTickerItems(maxItems);
    } catch (error) {
      console.error('Failed to update max ticker items:', error);
    }
  };

  // Handle max items slider change with debounce
  const handleMaxItemsSliderChange = (value) => {
    setTempMaxTickerItems(value);

    // Clear existing timeout
    if (maxItemsUpdateTimeout) {
      clearTimeout(maxItemsUpdateTimeout);
    }

    // Set new timeout to update after 500ms of no changes
    maxItemsUpdateTimeout = setTimeout(() => {
      updateMaxTickerItems(value);
    }, 500);
  };

  // Calculate days since start date (reactive)
  const daysSinceStart = () => {
    if (!streamStartDate()) return 0;
    const startDate = new Date(streamStartDate());
    return Math.floor((currentDate() - startDate) / (1000 * 60 * 60 * 24));
  };

  onMount(() => {
    loadMessages();
    loadStatusConfig();
    loadEventsConfig();

    // Update current date every minute to keep days counter accurate
    const dateInterval = setInterval(() => {
      setCurrentDate(new Date());
    }, 60000);

    onCleanup(() => {
      clearInterval(dateInterval);
    });
  });

  return (
    <div class="p-6 pb-24 space-y-6 overflow-y-auto max-h-screen">
      {/* Stream Start Date */}
      <div class="bg-base-200 rounded-lg p-4 shadow">
        <h2 class="text-xl font-bold mb-4">🔴 Stream Start Date</h2>
        <p class="text-sm text-gray-400 mb-4">
          Set the date when your 24/7 stream started. The day counter will update automatically.
        </p>
        <div class="flex items-center gap-4">
          <input
            type="date"
            value={streamStartDate()}
            onInput={(e) => updateStreamStartDate(e.target.value)}
            class="input input-bordered w-48"
            disabled={loading()}
          />
          <Show when={streamStartDate()}>
            <span class="text-sm text-gray-400">
              ({daysSinceStart()} days ago)
            </span>
          </Show>
        </div>
      </div>

      {/* Ticker Speed */}
      <div class="bg-base-200 rounded-lg p-4 shadow">
        <h2 class="text-xl font-bold mb-4">⚡ Ticker Speed</h2>
        <p class="text-sm text-gray-400 mb-4">
          Control how fast the ticker scrolls across the screen. Changes apply live!
        </p>
        <div class="space-y-3">
          <div class="flex items-center gap-4">
            <input
              type="range"
              min="3"
              max="60"
              value={tempTickerSpeed()}
              onInput={(e) => handleSpeedSliderChange(parseInt(e.target.value))}
              class="range range-primary flex-1"
              disabled={loading()}
            />
            <span class="text-lg font-mono w-20 text-right">{tempTickerSpeed()}s</span>
          </div>
          <div class="flex justify-between text-xs text-gray-400">
            <span>Fastest (3s)</span>
            <span>Slowest (60s)</span>
          </div>
        </div>
      </div>

      {/* Maximum Ticker Items */}
      <div class="bg-base-200 rounded-lg p-4 shadow">
        <h2 class="text-xl font-bold mb-4">🔢 Maximum Ticker Items</h2>
        <p class="text-sm text-gray-400 mb-4">
          Set the maximum number of items (messages + events) shown in the ticker. When this limit is reached, the oldest event notifications will be removed as new ones arrive. Permanent messages are not removed.
        </p>
        <div class="space-y-3">
          <div class="flex items-center gap-4">
            <input
              type="range"
              min="5"
              max="50"
              value={tempMaxTickerItems()}
              onInput={(e) => handleMaxItemsSliderChange(parseInt(e.target.value))}
              class="range range-primary flex-1"
              disabled={loading()}
            />
            <span class="text-lg font-mono w-20 text-right">{tempMaxTickerItems()} items</span>
          </div>
          <div class="flex justify-between text-xs text-gray-400">
            <span>Min (5 items)</span>
            <span>Max (50 items)</span>
          </div>
        </div>
      </div>

      {/* Ticker Messages */}
      <div class="bg-base-200 rounded-lg p-4 shadow">
        <h2 class="text-xl font-bold mb-4">📰 Ticker Messages</h2>

        {/* Add New Message */}
        <div class="mb-4">
          <div class="flex gap-2">
            <input
              type="text"
              value={newMessage()}
              onInput={(e) => setNewMessage(e.target.value)}
              placeholder="Enter new ticker message..."
              class="input input-bordered flex-1"
              onKeyPress={(e) => e.key === 'Enter' && addMessage()}
              disabled={loading()}
            />
            <button
              onClick={addMessage}
              class="btn btn-primary"
              disabled={loading() || !newMessage().trim()}
            >
              Add Message
            </button>
          </div>
        </div>

        {/* Messages List */}
        <div class="space-y-2">
          <Show when={messages().length === 0}>
            <p class="text-gray-400 text-center py-4">
              No ticker messages yet. Add one above!
            </p>
          </Show>

          <For each={messages()}>
            {(msg) => (
              <div class={`p-3 rounded flex items-center gap-3 ${msg.enabled ? 'bg-base-300' : 'bg-base-100 opacity-50'}`}>
                {/* Enable/Disable Toggle */}
                <input
                  type="checkbox"
                  checked={msg.enabled}
                  onChange={() => toggleMessage(msg.id)}
                  class="checkbox checkbox-primary"
                  disabled={loading()}
                />

                {/* Message Text */}
                <Show
                  when={editingId() === msg.id}
                  fallback={
                    <span class="flex-1 font-mono">{msg.message}</span>
                  }
                >
                  <input
                    type="text"
                    value={editMessage()}
                    onInput={(e) => setEditMessage(e.target.value)}
                    class="input input-bordered input-sm flex-1"
                    onKeyPress={(e) => e.key === 'Enter' && updateMessage(msg.id)}
                    disabled={loading()}
                    autofocus
                  />
                </Show>

                {/* Action Buttons */}
                <div class="flex gap-2">
                  <Show
                    when={editingId() === msg.id}
                    fallback={
                      <>
                        <button
                          onClick={() => toggleMessageSticky(msg.id)}
                          class={`btn btn-sm ${msg.is_sticky ? 'btn-warning' : 'btn-ghost'}`}
                          disabled={loading()}
                          title={msg.is_sticky ? "Unpin (sticky item)" : "Pin (make sticky)"}
                        >
                          {msg.is_sticky ? '📌' : '📍'}
                        </button>
                        <button
                          onClick={() => {
                            setEditingId(msg.id);
                            setEditMessage(msg.message);
                          }}
                          class="btn btn-sm btn-ghost"
                          disabled={loading()}
                        >
                          ✏️ Edit
                        </button>
                        <button
                          onClick={() => deleteMessage(msg.id)}
                          class="btn btn-sm btn-error btn-ghost"
                          disabled={loading()}
                        >
                          🗑️ Delete
                        </button>
                      </>
                    }
                  >
                    <button
                      onClick={() => updateMessage(msg.id)}
                      class="btn btn-sm btn-success"
                      disabled={loading()}
                    >
                      💾 Save
                    </button>
                    <button
                      onClick={() => {
                        setEditingId(null);
                        setEditMessage('');
                      }}
                      class="btn btn-sm btn-ghost"
                      disabled={loading()}
                    >
                      ❌ Cancel
                    </button>
                  </Show>
                </div>
              </div>
            )}
          </For>
        </div>

        <div class="mt-4 p-3 bg-info/20 rounded text-sm">
          <p><strong>💡 Tip:</strong> Enabled messages will scroll across the ticker overlay.</p>
          <p>Use the checkbox to toggle messages on/off without deleting them.</p>
        </div>
      </div>

      {/* Ticker Event Notifications */}
      <div class="bg-base-200 rounded-lg p-4 shadow">
        <h2 class="text-xl font-bold mb-4">🎉 Event Notifications</h2>
        <p class="text-sm text-gray-400 mb-4">
          Choose which Twitch events should appear in the ticker overlay.
        </p>

        <div class="grid grid-cols-2 gap-4">
          <div class="form-control">
            <label class="label cursor-pointer">
              <span class="label-text">👥 New Followers</span>
              <input
                type="checkbox"
                checked={eventsConfig().show_followers}
                onChange={() => toggleEventType('show_followers')}
                class="checkbox checkbox-primary"
                disabled={loading()}
              />
            </label>
          </div>

          <div class="form-control">
            <label class="label cursor-pointer">
              <span class="label-text">⭐ Subscribers</span>
              <input
                type="checkbox"
                checked={eventsConfig().show_subscribers}
                onChange={() => toggleEventType('show_subscribers')}
                class="checkbox checkbox-primary"
                disabled={loading()}
              />
            </label>
          </div>

          <div class="form-control">
            <label class="label cursor-pointer">
              <span class="label-text">🎯 Raids</span>
              <input
                type="checkbox"
                checked={eventsConfig().show_raids}
                onChange={() => toggleEventType('show_raids')}
                class="checkbox checkbox-primary"
                disabled={loading()}
              />
            </label>
          </div>

          <div class="form-control">
            <label class="label cursor-pointer">
              <span class="label-text">💰 Donations</span>
              <input
                type="checkbox"
                checked={eventsConfig().show_donations}
                onChange={() => toggleEventType('show_donations')}
                class="checkbox checkbox-primary"
                disabled={loading()}
              />
            </label>
          </div>

          <div class="form-control">
            <label class="label cursor-pointer">
              <span class="label-text">🎁 Gifted Subs</span>
              <input
                type="checkbox"
                checked={eventsConfig().show_gifted_subs}
                onChange={() => toggleEventType('show_gifted_subs')}
                class="checkbox checkbox-primary"
                disabled={loading()}
              />
            </label>
          </div>

          <div class="form-control">
            <label class="label cursor-pointer">
              <span class="label-text">💎 Cheers/Bits</span>
              <input
                type="checkbox"
                checked={eventsConfig().show_cheers}
                onChange={() => toggleEventType('show_cheers')}
                class="checkbox checkbox-primary"
                disabled={loading()}
              />
            </label>
          </div>
        </div>

        <div class="mt-4 p-3 bg-info/20 rounded text-sm">
          <p><strong>💡 Tip:</strong> Enabled events will automatically appear in the ticker when they occur on your stream.</p>
        </div>
      </div>
    </div>
  );
};

export default createPlugin({
  id: 'ticker-plugin',
  name: 'Ticker Manager',
  version: '1.0.0',
  description: 'Manage ticker messages and stream status',
  author: 'WebArcade Team',

  async onInit() {
    console.log('[Ticker Plugin] Initializing...');
  },

  async onStart(api) {
    console.log('[Ticker Plugin] Starting...');

    // Register Ticker viewport
    api.viewport('ticker-manager', {
      label: 'Ticker',
      component: TickerViewport,
      icon: IconNews,
      description: 'Manage ticker messages and stream status'
    });
  }
});
