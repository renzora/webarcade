import { createSignal, onMount, For, Show } from 'solid-js';
import { IconHome, IconBulb, IconBulbOff, IconRefresh } from '@tabler/icons-solidjs';

export default function RoomControl() {
  const [groups, setGroups] = createSignal([]);
  const [loading, setLoading] = createSignal(false);

  const loadGroups = async () => {
    setLoading(true);
    try {
      const response = await fetch('http://localhost:3001/philips-hue/groups');
      const data = await response.json();
      setGroups(data.groups || []);
    } catch (error) {
      console.error('Failed to load groups:', error);
    } finally {
      setLoading(false);
    }
  };

  const toggleGroup = async (group) => {
    try {
      const newState = !group.state.on;
      await fetch(`http://localhost:3001/philips-hue/groups/${group.id}/state`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ on: newState }),
      });

      // Update local state
      setGroups(groups().map(g =>
        g.id === group.id
          ? { ...g, state: { ...g.state, on: newState } }
          : g
      ));
    } catch (error) {
      console.error('Failed to toggle group:', error);
    }
  };

  const setBrightness = async (group, brightness) => {
    try {
      await fetch(`http://localhost:3001/philips-hue/groups/${group.id}/state`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ on: true, bri: brightness }),
      });

      // Update local state
      setGroups(groups().map(g =>
        g.id === group.id
          ? { ...g, state: { ...g.state, on: true, bri: brightness } }
          : g
      ));
    } catch (error) {
      console.error('Failed to set brightness:', error);
    }
  };

  onMount(() => {
    loadGroups();
  });

  return (
    <div class="card bg-gradient-to-br from-blue-500/20 to-blue-500/5 bg-base-100 shadow-lg h-full flex flex-col p-3">
      {/* Header */}
      <div class="flex items-center justify-between mb-2">
        <div class="flex items-center gap-1.5">
          <IconHome size={16} class="text-blue-500 opacity-80" />
          <span class="text-xs font-medium opacity-70">Room Control</span>
        </div>
        <button
          class="btn btn-xs btn-ghost"
          onClick={loadGroups}
          disabled={loading()}
          title="Refresh"
        >
          <IconRefresh size={14} />
        </button>
      </div>

      {/* Groups List */}
      <div class="flex-1 overflow-y-auto">
        {loading() && groups().length === 0 ? (
          <div class="text-center text-xs opacity-50 py-4">Loading...</div>
        ) : groups().length === 0 ? (
          <div class="text-center text-xs opacity-50 py-4">
            No rooms/groups found. Discover lights from your Hue Bridge first.
          </div>
        ) : (
          <For each={groups()}>
            {(group) => (
              <div class="bg-base-200 rounded-lg p-2 mb-2">
                <div class="flex items-center justify-between mb-2">
                  <div class="flex items-center gap-2 flex-1">
                    {group.state.on ? (
                      <IconBulb size={16} class="text-amber-400" />
                    ) : (
                      <IconBulbOff size={16} class="opacity-50" />
                    )}
                    <div class="flex-1">
                      <div class="font-medium text-xs">{group.name}</div>
                      <div class="text-xs opacity-50">
                        {group.type} • {group.lights.length} lights
                      </div>
                    </div>
                  </div>
                  <button
                    class={`btn btn-xs ${group.state.on ? 'btn-primary' : 'btn-outline'}`}
                    onClick={() => toggleGroup(group)}
                  >
                    {group.state.on ? 'ON' : 'OFF'}
                  </button>
                </div>

                {/* Brightness Control */}
                <Show when={group.state.on && group.state.bri !== null}>
                  <div class="form-control">
                    <label class="label py-0">
                      <span class="label-text text-xs">Brightness</span>
                      <span class="label-text text-xs opacity-50">
                        {Math.round((group.state.bri / 254) * 100)}%
                      </span>
                    </label>
                    <input
                      type="range"
                      min="1"
                      max="254"
                      value={group.state.bri || 254}
                      class="range range-xs range-primary"
                      onInput={(e) => setBrightness(group, parseInt(e.target.value))}
                    />
                  </div>
                </Show>

                {/* Quick Actions */}
                <Show when={group.state.on}>
                  <div class="flex gap-1 mt-2">
                    <button
                      class="btn btn-xs btn-outline flex-1"
                      onClick={() => setBrightness(group, 50)}
                    >
                      Low
                    </button>
                    <button
                      class="btn btn-xs btn-outline flex-1"
                      onClick={() => setBrightness(group, 127)}
                    >
                      Medium
                    </button>
                    <button
                      class="btn btn-xs btn-outline flex-1"
                      onClick={() => setBrightness(group, 254)}
                    >
                      Full
                    </button>
                  </div>
                </Show>
              </div>
            )}
          </For>
        )}
      </div>

      {/* Info */}
      <div class="text-xs opacity-50 mt-2 border-t border-base-300 pt-2">
        🏠 Control all lights in a room at once
      </div>
    </div>
  );
}
