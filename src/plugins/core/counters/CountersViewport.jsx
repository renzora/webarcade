import { createSignal, onMount, For, Show } from 'solid-js';
import twitchStore from '../twitch/TwitchStore.jsx';
import { bridgeFetch } from '@/api/bridge.js';
import { IconPlus, IconMinus, IconRefresh, IconList, IconAlertCircle } from '@tabler/icons-solidjs';

export default function CountersViewport() {
  const [counters, setCounters] = createSignal([]);
  const [loading, setLoading] = createSignal(true);
  const [selectedChannel, setSelectedChannel] = createSignal('');
  const [newCounterName, setNewCounterName] = createSignal('');
  const [status, setStatus] = createSignal({ status: 'disconnected', connected_channels: [] });

  onMount(async () => {
    const currentStatus = await twitchStore.fetchStatus();
    if (currentStatus) {
      setStatus(currentStatus);
      if (currentStatus.connected_channels && currentStatus.connected_channels.length > 0) {
        setSelectedChannel(currentStatus.connected_channels[0]);
        await loadCounters(currentStatus.connected_channels[0]);
      }
    }
    setLoading(false);
  });

  const loadCounters = async (channel) => {
    if (!channel) return;

    try {
      setLoading(true);
      const response = await bridgeFetch(`/database/counters?channel=${channel}`);
      const data = await response.json();
      setCounters(data);
    } catch (e) {
      console.error('Failed to load counters:', e);
    } finally {
      setLoading(false);
    }
  };

  const handleChannelChange = async (channel) => {
    setSelectedChannel(channel);
    await loadCounters(channel);
  };

  const incrementCounter = async (task) => {
    try {
      const response = await bridgeFetch('/database/counters/increment', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ channel: selectedChannel(), task }),
      });
      const data = await response.json();

      if (data.success) {
        await loadCounters(selectedChannel());
      }
    } catch (e) {
      console.error('Failed to increment counter:', e);
    }
  };

  const decrementCounter = async (task) => {
    try {
      const response = await bridgeFetch('/database/counters/decrement', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ channel: selectedChannel(), task }),
      });
      const data = await response.json();

      if (data.success) {
        await loadCounters(selectedChannel());
      }
    } catch (e) {
      console.error('Failed to decrement counter:', e);
    }
  };

  const resetCounter = async (task) => {
    if (!confirm(`Reset "${task}" counter to 0?`)) return;

    try {
      const response = await bridgeFetch('/database/counters/reset', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ channel: selectedChannel(), task }),
      });

      if (response.ok) {
        await loadCounters(selectedChannel());
      }
    } catch (e) {
      console.error('Failed to reset counter:', e);
    }
  };

  const createCounter = async () => {
    const counterName = newCounterName().trim();
    if (!counterName) return;

    await incrementCounter(counterName);
    setNewCounterName('');
  };

  return (
    <div class="h-full flex flex-col bg-base-200">
      {/* Header */}
      <div class="flex items-center justify-between bg-base-100 border-b border-base-300 px-4 py-3">
        <div class="flex items-center gap-3 flex-1">
          <IconList size={20} class="text-primary" />
          <h2 class="text-lg font-semibold">Stream Counters</h2>
        </div>

        <Show when={status().connected_channels.length > 0}>
          <select
            class="select select-bordered select-sm"
            value={selectedChannel()}
            onChange={(e) => handleChannelChange(e.target.value)}
          >
            {status().connected_channels.map((channel) => (
              <option value={channel}>#{channel}</option>
            ))}
          </select>
        </Show>
      </div>

      {/* Add Counter */}
      <div class="p-4 bg-base-100 border-b border-base-300">
        <div class="flex gap-2">
          <input
            type="text"
            placeholder="New counter name..."
            class="input input-bordered input-sm flex-1"
            value={newCounterName()}
            onInput={(e) => setNewCounterName(e.target.value)}
            onKeyPress={(e) => e.key === 'Enter' && createCounter()}
          />
          <button
            class="btn btn-primary btn-sm"
            onClick={createCounter}
            disabled={!newCounterName().trim() || !selectedChannel()}
          >
            Add Counter
          </button>
        </div>
      </div>

      {/* Counters List */}
      <div class="flex-1 overflow-y-auto p-4">
        <Show
          when={!loading() && selectedChannel()}
          fallback={
            <div class="flex items-center justify-center h-full">
              <div class="text-center">
                <IconAlertCircle size={48} class="mx-auto mb-4 opacity-30" />
                <p class="text-sm text-base-content/60">
                  {loading() ? 'Loading counters...' : 'Select a channel to view counters'}
                </p>
              </div>
            </div>
          }
        >
          <Show
            when={counters().length > 0}
            fallback={
              <div class="text-center py-8">
                <IconList size={48} class="mx-auto mb-4 opacity-30" />
                <p class="text-sm font-semibold mb-2">No counters yet</p>
                <p class="text-xs text-base-content/60">Create your first counter above</p>
              </div>
            }
          >
            <div class="grid gap-3">
              <For each={counters()}>
                {(counter) => (
                  <div class="card bg-base-100 shadow-sm">
                    <div class="card-body p-4">
                      <div class="flex items-center justify-between">
                        <div class="flex-1">
                          <h3 class="font-semibold text-sm">{counter.task}</h3>
                          <div class="text-2xl font-bold text-primary mt-1">
                            {counter.count}
                          </div>
                        </div>

                        <div class="flex gap-2">
                          <button
                            class="btn btn-circle btn-sm btn-error btn-outline"
                            onClick={() => decrementCounter(counter.task)}
                            disabled={counter.count <= 0}
                            title="Decrement"
                          >
                            <IconMinus size={16} />
                          </button>
                          <button
                            class="btn btn-circle btn-sm btn-success btn-outline"
                            onClick={() => incrementCounter(counter.task)}
                            title="Increment"
                          >
                            <IconPlus size={16} />
                          </button>
                          <button
                            class="btn btn-circle btn-sm btn-ghost"
                            onClick={() => resetCounter(counter.task)}
                            title="Reset"
                          >
                            <IconRefresh size={16} />
                          </button>
                        </div>
                      </div>
                    </div>
                  </div>
                )}
              </For>
            </div>
          </Show>
        </Show>
      </div>
    </div>
  );
}
