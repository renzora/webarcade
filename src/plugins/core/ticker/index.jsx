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
  const [breakingNewsActive, setBreakingNewsActive] = createSignal(false);
  const [breakingNewsMessage, setBreakingNewsMessage] = createSignal('');

  // Segment management
  const [segments, setSegments] = createSignal([]);
  const [segmentDuration, setSegmentDuration] = createSignal(15);
  const [tempSegmentDuration, setTempSegmentDuration] = createSignal(15);
  const [newSegmentType, setNewSegmentType] = createSignal('custom');
  const [newSegmentContent, setNewSegmentContent] = createSignal('');
  const [editingSegmentId, setEditingSegmentId] = createSignal(null);
  const [draggedSegmentId, setDraggedSegmentId] = createSignal(null);
  const [dragOverSegmentId, setDragOverSegmentId] = createSignal(null);

  let speedUpdateTimeout;
  let maxItemsUpdateTimeout;
  let segmentDurationTimeout;

  // Load ticker messages
  const loadMessages = async () => {
    try {
      const response = await fetch(`${WEBARCADE_API}/ticker/messages`);
      const data = await response.json();
      setMessages(data.messages || data || []);
    } catch (error) {
      console.error('Failed to load ticker messages:', error);
    }
  };

  // Load status config (start date, ticker speed, and max items)
  const loadStatusConfig = async () => {
    try {
      const response = await fetch(`${WEBARCADE_API}/status/config`);
      const data = await response.json();
      setStreamStartDate(data.stream_start_date || '');
      setTickerSpeed(data.ticker_speed || 30);
      setTempTickerSpeed(data.ticker_speed || 30);
      setMaxTickerItems(data.max_ticker_items || 20);
      setTempMaxTickerItems(data.max_ticker_items || 20);
      setSegmentDuration(data.segment_duration || 15);
      setTempSegmentDuration(data.segment_duration || 15);
      setBreakingNewsActive(data.breaking_news_active || false);
      setBreakingNewsMessage(data.breaking_news_message || '');
    } catch (error) {
      console.error('Failed to load status config:', error);
    }
  };

  // Load ticker segments
  const loadSegments = async () => {
    try {
      const response = await fetch(`${WEBARCADE_API}/ticker/segments`);
      const data = await response.json();
      setSegments(data);
    } catch (error) {
      console.error('Failed to load segments:', error);
    }
  };

  // Add new segment
  const addSegment = async () => {
    const content = newSegmentContent().trim();

    // Only check for content if not messages type
    if (newSegmentType() !== 'messages' && !content) {
      alert('Please enter content for this segment type');
      return;
    }

    setLoading(true);
    try {
      const segmentData = {
        type: newSegmentType(),
        enabled: true,
        content: {}
      };

      // Parse content based on type
      switch (newSegmentType()) {
        case 'schedule':
          try {
            segmentData.content = { schedule: JSON.parse(content) };
          } catch {
            alert('Invalid JSON format for schedule');
            setLoading(false);
            return;
          }
          break;
        case 'commands':
          try {
            segmentData.content = { commands: JSON.parse(content) };
          } catch {
            alert('Invalid JSON format for commands');
            setLoading(false);
            return;
          }
          break;
        case 'messages':
          segmentData.content = {};
          break;
        case 'custom':
          segmentData.content = { text: content };
          break;
      }

      await fetch(`${WEBARCADE_API}/ticker/segments`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(segmentData)
      });
      setNewSegmentContent('');
      await loadSegments();
    } catch (error) {
      console.error('Failed to add segment:', error);
    }
    setLoading(false);
  };

  // Delete segment
  const deleteSegment = async (id) => {
    if (!confirm('Delete this ticker segment?')) return;

    setLoading(true);
    try {
      await fetch(`${WEBARCADE_API}/ticker/segments/${id}`, {
        method: 'DELETE'
      });
      await loadSegments();
    } catch (error) {
      console.error('Failed to delete segment:', error);
    }
    setLoading(false);
  };

  // Toggle segment enabled/disabled
  const toggleSegment = async (id) => {
    setLoading(true);
    try {
      const segment = segments().find(s => s.id === id);
      await fetch(`${WEBARCADE_API}/ticker/segments/${id}`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          type: segment.type,
          enabled: !segment.enabled,
          content: segment.content,
          position: segment.position
        })
      });
      await loadSegments();
    } catch (error) {
      console.error('Failed to toggle segment:', error);
    }
    setLoading(false);
  };

  // Handle drag start
  const handleDragStart = (e, segmentId) => {
    setDraggedSegmentId(segmentId);
    e.dataTransfer.effectAllowed = 'move';
  };

  // Handle drag over
  const handleDragOver = (e, segmentId) => {
    e.preventDefault();
    e.dataTransfer.dropEffect = 'move';
    setDragOverSegmentId(segmentId);
  };

  // Handle drag leave
  const handleDragLeave = () => {
    setDragOverSegmentId(null);
  };

  // Handle drop
  const handleDrop = async (e, targetSegmentId) => {
    e.preventDefault();
    const sourceId = draggedSegmentId();

    if (sourceId === targetSegmentId) {
      setDraggedSegmentId(null);
      setDragOverSegmentId(null);
      return;
    }

    // Reorder segments array
    const segmentsCopy = [...segments()];
    const sourceIndex = segmentsCopy.findIndex(s => s.id === sourceId);
    const targetIndex = segmentsCopy.findIndex(s => s.id === targetSegmentId);

    if (sourceIndex === -1 || targetIndex === -1) return;

    // Remove source and insert at target position
    const [movedSegment] = segmentsCopy.splice(sourceIndex, 1);
    segmentsCopy.splice(targetIndex, 0, movedSegment);

    // Update local state immediately for smooth UX
    setSegments(segmentsCopy);

    // Send reorder request to backend
    try {
      const segmentIds = segmentsCopy.map(s => s.id);
      await fetch(`${WEBARCADE_API}/ticker/segments/reorder`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ segment_ids: segmentIds })
      });
    } catch (error) {
      console.error('Failed to reorder segments:', error);
      // Reload on error to restore correct order
      await loadSegments();
    }

    setDraggedSegmentId(null);
    setDragOverSegmentId(null);
  };

  // Handle drag end
  const handleDragEnd = () => {
    setDraggedSegmentId(null);
    setDragOverSegmentId(null);
  };

  // Update segment duration
  const updateSegmentDuration = async (duration) => {
    try {
      await fetch(`${WEBARCADE_API}/status/segment-duration`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ duration: parseInt(duration) })
      });
      setSegmentDuration(duration);
    } catch (error) {
      console.error('Failed to update segment duration:', error);
    }
  };

  // Handle segment duration slider change with debounce
  const handleSegmentDurationSliderChange = (value) => {
    setTempSegmentDuration(value);

    if (segmentDurationTimeout) {
      clearTimeout(segmentDurationTimeout);
    }

    segmentDurationTimeout = setTimeout(() => {
      updateSegmentDuration(value);
    }, 500);
  };

  // Load ticker events config
  const loadEventsConfig = async () => {
    try {
      const response = await fetch(`${WEBARCADE_API}/ticker/events/config`);
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
      await fetch(`${WEBARCADE_API}/ticker/events/config`, {
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
      await fetch(`${WEBARCADE_API}/ticker/messages`, {
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
      await fetch(`${WEBARCADE_API}/ticker/messages`, {
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
      await fetch(`${WEBARCADE_API}/ticker/messages/toggle-sticky`, {
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
      await fetch(`${WEBARCADE_API}/ticker/messages`, {
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
      await fetch(`${WEBARCADE_API}/ticker/messages/toggle`, {
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
      await fetch(`${WEBARCADE_API}/status/start-date`, {
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
      await fetch(`${WEBARCADE_API}/status/ticker-speed`, {
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
      await fetch(`${WEBARCADE_API}/status/max-ticker-items`, {
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

  // Update breaking news
  const updateBreakingNews = async (active, message) => {
    setLoading(true);
    try {
      await fetch(`${WEBARCADE_API}/status/breaking-news`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ active, message: message || null })
      });
      setBreakingNewsActive(active);
      if (!active) {
        setBreakingNewsMessage('');
      }
    } catch (error) {
      console.error('Failed to update breaking news:', error);
    }
    setLoading(false);
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
    loadSegments();

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

      {/* Ticker Segments */}
      <div class="bg-base-200 rounded-lg p-4 shadow">
        <h2 class="text-xl font-bold mb-4">🎬 Ticker Segments</h2>
        <p class="text-sm text-gray-400 mb-4">
          Create different segments that cycle through the ticker with animated transitions. Each segment displays for a set duration before transitioning to the next one.
        </p>

        {/* Segment Duration Control */}
        <div class="mb-6 p-4 bg-base-300 rounded-lg">
          <h3 class="text-lg font-bold mb-3">⏱️ Max Segment Duration</h3>
          <p class="text-sm text-gray-400 mb-3">
            Segments automatically advance when the ticker completes scrolling. This setting is a fallback maximum duration.
          </p>
          <div class="space-y-3">
            <div class="flex items-center gap-4">
              <input
                type="range"
                min="15"
                max="120"
                value={tempSegmentDuration()}
                onInput={(e) => handleSegmentDurationSliderChange(parseInt(e.target.value))}
                class="range range-primary flex-1"
                disabled={loading()}
              />
              <span class="text-lg font-mono w-20 text-right">{tempSegmentDuration()}s</span>
            </div>
            <div class="flex justify-between text-xs text-gray-400">
              <span>Short (15s)</span>
              <span>Long (120s)</span>
            </div>
          </div>
        </div>

        {/* Add New Segment */}
        <div class="mb-4">
          <h3 class="text-lg font-bold mb-3">➕ Add New Segment</h3>
          <div class="space-y-3">
            <select
              value={newSegmentType()}
              onChange={(e) => setNewSegmentType(e.target.value)}
              class="select select-bordered w-full"
              disabled={loading()}
            >
              <option value="messages">📰 Regular Messages (scrolling ticker)</option>
              <option value="schedule">📅 Stream Schedule</option>
              <option value="commands">💬 Chat Commands</option>
              <option value="custom">✏️ Custom Text</option>
            </select>

            <Show when={newSegmentType() === 'schedule'}>
              <div class="text-sm text-gray-400 mb-2">
                📅 Enter schedule as JSON array: [{"{"}"day": "Mon-Fri", "time": "6PM-12AM EST"{"}"}]<br/>
                Will scroll continuously until one loop completes, then advance to next segment.
              </div>
            </Show>

            <Show when={newSegmentType() === 'commands'}>
              <div class="text-sm text-gray-400 mb-2">
                💬 Enter commands as JSON array: [{"{"}"name": "discord", "description": "Join our Discord!"{"}"}]<br/>
                Will scroll continuously until one loop completes, then advance to next segment.
              </div>
            </Show>

            <Show when={newSegmentType() === 'custom'}>
              <div class="text-sm text-gray-400 mb-2">
                ✏️ Enter custom text to display<br/>
                Will scroll once from right to left, then advance to next segment when fully off-screen.
              </div>
            </Show>

            <Show when={newSegmentType() === 'messages'}>
              <div class="text-sm text-gray-400 mb-2">
                📰 This segment will show your ticker messages (configured below)<br/>
                Will scroll continuously until one loop completes, then advance to next segment.
              </div>
            </Show>

            <div class="flex gap-2">
              <textarea
                value={newSegmentContent()}
                onInput={(e) => setNewSegmentContent(e.target.value)}
                placeholder={
                  newSegmentType() === 'messages'
                    ? 'No content needed - uses your ticker messages'
                    : newSegmentType() === 'custom'
                    ? 'Enter your custom text...'
                    : 'Enter JSON data...'
                }
                class="textarea textarea-bordered flex-1"
                rows="3"
                disabled={loading() || newSegmentType() === 'messages'}
              />
            </div>

            <button
              onClick={addSegment}
              class="btn btn-primary w-full"
              disabled={loading()}
            >
              Add Segment
            </button>
          </div>
        </div>

        {/* Segments List */}
        <div class="space-y-2">
          <h3 class="text-lg font-bold mb-3">📋 Current Segments ({segments().length})</h3>

          <Show when={segments().length === 0}>
            <p class="text-gray-400 text-center py-4">
              No segments configured. Add one above to enable segment cycling!
            </p>
          </Show>

          <For each={segments()}>
            {(segment, index) => (
              <div
                draggable="true"
                onDragStart={(e) => handleDragStart(e, segment.id)}
                onDragOver={(e) => handleDragOver(e, segment.id)}
                onDragLeave={handleDragLeave}
                onDrop={(e) => handleDrop(e, segment.id)}
                onDragEnd={handleDragEnd}
                class={`p-4 rounded flex items-start gap-3 cursor-move transition-all ${
                  segment.enabled ? 'bg-base-300' : 'bg-base-100 opacity-50'
                } ${
                  draggedSegmentId() === segment.id ? 'opacity-50 scale-95' : ''
                } ${
                  dragOverSegmentId() === segment.id && draggedSegmentId() !== segment.id ? 'ring-2 ring-primary' : ''
                }`}
              >
                {/* Drag Handle */}
                <div class="cursor-grab active:cursor-grabbing text-gray-400 hover:text-white mt-1">
                  <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <line x1="8" y1="6" x2="21" y2="6"></line>
                    <line x1="8" y1="12" x2="21" y2="12"></line>
                    <line x1="8" y1="18" x2="21" y2="18"></line>
                    <line x1="3" y1="6" x2="3.01" y2="6"></line>
                    <line x1="3" y1="12" x2="3.01" y2="12"></line>
                    <line x1="3" y1="18" x2="3.01" y2="18"></line>
                  </svg>
                </div>

                {/* Enable/Disable Toggle */}
                <input
                  type="checkbox"
                  checked={segment.enabled}
                  onChange={() => toggleSegment(segment.id)}
                  class="checkbox checkbox-primary mt-1"
                  disabled={loading()}
                  onClick={(e) => e.stopPropagation()}
                />

                {/* Segment Info */}
                <div class="flex-1">
                  <div class="flex items-center gap-2 mb-2">
                    <span class="badge badge-primary">#{index() + 1}</span>
                    <span class="font-bold">
                      {segment.type === 'messages' && '📰 Regular Messages'}
                      {segment.type === 'schedule' && '📅 Stream Schedule'}
                      {segment.type === 'commands' && '💬 Chat Commands'}
                      {segment.type === 'custom' && '✏️ Custom Text'}
                    </span>
                  </div>

                  <div class="text-sm text-gray-400">
                    {segment.type === 'messages' && '🔄 Displays your ticker messages with continuous scrolling'}
                    {segment.type === 'schedule' && `🔄 Schedule: ${JSON.stringify(segment.content?.schedule || [])}`}
                    {segment.type === 'commands' && `🔄 Commands: ${segment.content?.commands?.map(c => `!${c.name}`).join(', ') || 'None'}`}
                    {segment.type === 'custom' && `➡️ "${segment.content?.text || ''}" (scrolls once)`}
                  </div>
                </div>

                {/* Action Buttons */}
                <div class="flex gap-2">
                  <button
                    onClick={() => deleteSegment(segment.id)}
                    class="btn btn-sm btn-error btn-ghost"
                    disabled={loading()}
                  >
                    🗑️ Delete
                  </button>
                </div>
              </div>
            )}
          </For>
        </div>

        <div class="mt-4 p-3 bg-info/20 rounded text-sm">
          <p><strong>💡 Tip:</strong> Drag segments using the handle on the left to reorder them! All segments scroll like a ticker.</p>
          <p class="mt-1">Segments automatically advance when scrolling completes (or after {segmentDuration()}s max).</p>
        </div>
      </div>

      {/* Breaking News */}
      <div class={`bg-base-200 rounded-lg p-4 shadow ${breakingNewsActive() ? 'ring-2 ring-red-500' : ''}`}>
        <h2 class="text-xl font-bold mb-4">🚨 Breaking News</h2>
        <p class="text-sm text-gray-400 mb-4">
          Activate breaking news mode to replace the ticker with an urgent message. The ticker background will turn red.
        </p>

        <div class="space-y-4">
          {/* Breaking News Toggle */}
          <div class="form-control">
            <label class="label cursor-pointer">
              <span class="label-text font-bold">Breaking News Active</span>
              <input
                type="checkbox"
                checked={breakingNewsActive()}
                onChange={(e) => {
                  const active = e.target.checked;
                  if (!active) {
                    updateBreakingNews(false, '');
                  } else if (breakingNewsMessage().trim()) {
                    updateBreakingNews(true, breakingNewsMessage());
                  } else {
                    alert('Please enter a breaking news message first');
                    e.target.checked = false;
                  }
                }}
                class="toggle toggle-error toggle-lg"
                disabled={loading()}
              />
            </label>
          </div>

          {/* Breaking News Message Input */}
          <div class="space-y-2">
            <label class="label">
              <span class="label-text">Breaking News Message</span>
            </label>
            <div class="flex gap-2">
              <input
                type="text"
                value={breakingNewsMessage()}
                onInput={(e) => setBreakingNewsMessage(e.target.value)}
                placeholder="Enter breaking news message..."
                class="input input-bordered flex-1"
                disabled={loading()}
                onKeyPress={(e) => {
                  if (e.key === 'Enter' && breakingNewsMessage().trim()) {
                    updateBreakingNews(true, breakingNewsMessage());
                  }
                }}
              />
              <button
                onClick={() => updateBreakingNews(true, breakingNewsMessage())}
                class="btn btn-error"
                disabled={loading() || !breakingNewsMessage().trim()}
              >
                Activate
              </button>
            </div>
          </div>

          <Show when={breakingNewsActive()}>
            <div class="alert alert-error">
              <svg xmlns="http://www.w3.org/2000/svg" class="stroke-current shrink-0 h-6 w-6" fill="none" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
              </svg>
              <span><strong>Breaking News is currently ACTIVE!</strong> The ticker is showing: "{breakingNewsMessage()}"</span>
            </div>
          </Show>
        </div>

        <div class="mt-4 p-3 bg-warning/20 rounded text-sm">
          <p><strong>⚠️ Note:</strong> When breaking news is active, the normal ticker messages will be hidden and replaced with your breaking news message. The ticker background will turn red.</p>
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
