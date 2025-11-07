import { createSignal, onMount, For, Show } from 'solid-js';
import twitchStore from '../twitch/TwitchStore.jsx';
import { bridgeFetch } from '@/api/bridge.js';
import { IconVolume, IconUserPlus, IconTrash, IconAlertCircle, IconSettings } from '@tabler/icons-solidjs';

export default function TTSWhitelistViewport() {
  const [users, setUsers] = createSignal([]);
  const [loading, setLoading] = createSignal(true);
  const [selectedChannel, setSelectedChannel] = createSignal('');
  const [newUsername, setNewUsername] = createSignal('');
  const [ttsEnabled, setTtsEnabled] = createSignal(false);
  const [ttsMode, setTtsMode] = createSignal('broadcaster');
  const [status, setStatus] = createSignal({ status: 'disconnected', connected_channels: [] });

  onMount(async () => {
    const currentStatus = await twitchStore.fetchStatus();
    if (currentStatus) {
      setStatus({ ...currentStatus, connected_channels: currentStatus.connected_channels || [] });
      if (currentStatus.connected_channels && currentStatus.connected_channels.length > 0) {
        setSelectedChannel(currentStatus.connected_channels[0]);
        await loadTTSSettings(currentStatus.connected_channels[0]);
        await loadUsers(currentStatus.connected_channels[0]);
      }
    }
    setLoading(false);
  });

  const loadTTSSettings = async (channel) => {
    if (!channel) return;

    try {
      const response = await bridgeFetch(`/tts/settings?channel=${channel}`);
      const data = await response.json();
      setTtsEnabled(data.enabled);
      setTtsMode(data.mode);
    } catch (e) {
      console.error('Failed to load TTS settings:', e);
    }
  };

  const loadUsers = async (channel) => {
    if (!channel) return;

    try {
      setLoading(true);
      const response = await bridgeFetch(`/tts/whitelist?channel=${channel}`);
      const data = await response.json();
      setUsers(data);
    } catch (e) {
      console.error('Failed to load TTS users:', e);
    } finally {
      setLoading(false);
    }
  };

  const handleChannelChange = async (channel) => {
    setSelectedChannel(channel);
    await loadTTSSettings(channel);
    await loadUsers(channel);
  };

  const addUser = async () => {
    const username = newUsername().trim();
    if (!username || !selectedChannel()) return;

    try {
      const response = await bridgeFetch('/tts/whitelist/add', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          channel: selectedChannel(),
          username,
        }),
      });

      if (response.ok) {
        setNewUsername('');
        await loadUsers(selectedChannel());
      }
    } catch (e) {
      console.error('Failed to add TTS user:', e);
    }
  };

  const removeUser = async (username) => {
    try {
      const response = await bridgeFetch('/tts/whitelist', {
        method: 'DELETE',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          channel: selectedChannel(),
          username,
        }),
      });

      if (response.ok) {
        await loadUsers(selectedChannel());
      }
    } catch (e) {
      console.error('Failed to remove TTS user:', e);
    }
  };

  const updateTTSSettings = async (enabled, mode) => {
    try {
      const response = await bridgeFetch('/tts/settings', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          channel: selectedChannel(),
          enabled,
          mode,
        }),
      });

      if (response.ok) {
        setTtsEnabled(enabled);
        setTtsMode(mode);
      }
    } catch (e) {
      console.error('Failed to update TTS settings:', e);
    }
  };

  const toggleTTS = () => {
    updateTTSSettings(!ttsEnabled(), ttsMode());
  };

  const changeTTSMode = (mode) => {
    updateTTSSettings(ttsEnabled(), mode);
  };

  return (
    <div class="h-full flex flex-col bg-base-200">
      {/* Header */}
      <div class="flex items-center justify-between bg-base-100 border-b border-base-300 px-4 py-3">
        <div class="flex items-center gap-3 flex-1">
          <IconVolume size={20} class="text-primary" />
          <h2 class="text-lg font-semibold">TTS Settings</h2>
        </div>

        <Show when={status().connected_channels?.length > 0}>
          <select
            class="select select-bordered select-sm"
            value={selectedChannel()}
            onChange={(e) => handleChannelChange(e.target.value)}
          >
            {status().connected_channels?.map((channel) => (
              <option value={channel}>#{channel}</option>
            ))}
          </select>
        </Show>
      </div>

      {/* TTS Settings */}
      <div class="p-4 bg-base-100 border-b border-base-300 space-y-4">
        {/* Enable/Disable Toggle */}
        <div class="flex items-center justify-between">
          <div>
            <label class="label-text font-semibold">TTS Enabled</label>
            <p class="text-xs text-base-content/60">Enable text-to-speech for chat messages</p>
          </div>
          <input
            type="checkbox"
            class="toggle toggle-primary"
            checked={ttsEnabled()}
            onChange={toggleTTS}
            disabled={!selectedChannel()}
          />
        </div>

        {/* Mode Selection */}
        <div>
          <label class="label-text font-semibold flex items-center gap-2 mb-2">
            <IconSettings size={16} />
            TTS Mode
          </label>
          <div class="flex gap-2 flex-wrap">
            <button
              class={`btn btn-sm ${ttsMode() === 'broadcaster' ? 'btn-primary' : 'btn-outline'}`}
              onClick={() => changeTTSMode('broadcaster')}
              disabled={!selectedChannel()}
            >
              Broadcaster Only
            </button>
            <button
              class={`btn btn-sm ${ttsMode() === 'whitelist' ? 'btn-primary' : 'btn-outline'}`}
              onClick={() => changeTTSMode('whitelist')}
              disabled={!selectedChannel()}
            >
              Whitelist
            </button>
            <button
              class={`btn btn-sm ${ttsMode() === 'everyone' ? 'btn-primary' : 'btn-outline'}`}
              onClick={() => changeTTSMode('everyone')}
              disabled={!selectedChannel()}
            >
              Everyone
            </button>
          </div>
        </div>

        {/* Mode Description */}
        <Show when={ttsMode() === 'broadcaster'}>
          <div class="alert alert-info alert-sm">
            <IconAlertCircle size={16} />
            <span class="text-xs">Only the broadcaster's messages will be read aloud</span>
          </div>
        </Show>
        <Show when={ttsMode() === 'whitelist'}>
          <div class="alert alert-info alert-sm">
            <IconAlertCircle size={16} />
            <span class="text-xs">Only whitelisted users' messages will be read aloud</span>
          </div>
        </Show>
        <Show when={ttsMode() === 'everyone'}>
          <div class="alert alert-warning alert-sm">
            <IconAlertCircle size={16} />
            <span class="text-xs">All chat messages will be read aloud (use with caution!)</span>
          </div>
        </Show>
      </div>

      {/* Whitelist Management (only shown in whitelist mode) */}
      <Show when={ttsMode() === 'whitelist'}>
        <div class="p-4 bg-base-100 border-b border-base-300">
          <div class="flex gap-2">
            <input
              type="text"
              placeholder="Add username to whitelist..."
              class="input input-bordered input-sm flex-1"
              value={newUsername()}
              onInput={(e) => setNewUsername(e.target.value)}
              onKeyPress={(e) => e.key === 'Enter' && addUser()}
            />
            <button
              class="btn btn-primary btn-sm gap-2"
              onClick={addUser}
              disabled={!newUsername().trim() || !selectedChannel()}
            >
              <IconUserPlus size={16} />
              Add
            </button>
          </div>
        </div>

        {/* Users List */}
        <div class="flex-1 overflow-y-auto p-4">
          <Show
            when={!loading() && selectedChannel()}
            fallback={
              <div class="flex items-center justify-center h-full">
                <div class="text-center">
                  <IconAlertCircle size={48} class="mx-auto mb-4 opacity-30" />
                  <p class="text-sm text-base-content/60">
                    {loading() ? 'Loading whitelist...' : 'Select a channel'}
                  </p>
                </div>
              </div>
            }
          >
            <Show
              when={users().length > 0}
              fallback={
                <div class="text-center py-8">
                  <IconVolume size={48} class="mx-auto mb-4 opacity-30" />
                  <p class="text-sm font-semibold mb-2">No users in whitelist</p>
                  <p class="text-xs text-base-content/60">Add users above to allow TTS</p>
                </div>
              }
            >
              <div class="space-y-2">
                <For each={users()}>
                  {(username) => (
                    <div class="card bg-base-100 shadow-sm hover:shadow-md transition-shadow">
                      <div class="card-body p-3">
                        <div class="flex items-center justify-between">
                          <div class="flex items-center gap-2">
                            <div class="avatar placeholder">
                              <div class="bg-primary text-primary-content rounded-full w-8">
                                <span class="text-xs">{username[0].toUpperCase()}</span>
                              </div>
                            </div>
                            <span class="font-medium text-sm">{username}</span>
                          </div>

                          <button
                            class="btn btn-circle btn-sm btn-ghost"
                            onClick={() => removeUser(username)}
                            title="Remove"
                          >
                            <IconTrash size={16} />
                          </button>
                        </div>
                      </div>
                    </div>
                  )}
                </For>
              </div>
            </Show>
          </Show>
        </div>
      </Show>
    </div>
  );
}
