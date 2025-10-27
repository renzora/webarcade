import { createSignal, onMount, Show } from 'solid-js';
import { bridgeFetch } from '@/api/bridge.js';
import { IconBrandDiscord, IconSettings, IconCheck, IconX, IconPlayerPlay, IconPlayerStop, IconRefresh } from '@tabler/icons-solidjs';

export default function DiscordViewport() {
  const [config, setConfig] = createSignal(null);
  const [stats, setStats] = createSignal(null);
  const [loading, setLoading] = createSignal(true);
  const [saving, setSaving] = createSignal(false);
  const [showConfig, setShowConfig] = createSignal(false);

  // Form fields
  const [botToken, setBotToken] = createSignal('');
  const [channelId, setChannelId] = createSignal('');
  const [enabled, setEnabled] = createSignal(false);
  const [commandPrefix, setCommandPrefix] = createSignal('!sr');
  const [maxSongLength, setMaxSongLength] = createSignal(600);
  const [maxQueueSize, setMaxQueueSize] = createSignal(50);

  onMount(async () => {
    await loadConfig();
    await loadStats();
    setLoading(false);
  });

  const loadConfig = async () => {
    try {
      const response = await bridgeFetch('/discord/config');
      const data = await response.json();
      if (data.success && data.data) {
        setConfig(data.data);
        if (data.data.bot_token) setBotToken(data.data.bot_token);
        if (data.data.channel_id) setChannelId(data.data.channel_id);
        setEnabled(data.data.enabled || false);
        setCommandPrefix(data.data.command_prefix || '!sr');
        setMaxSongLength(data.data.max_song_length || 600);
        setMaxQueueSize(data.data.max_queue_size || 50);
      }
    } catch (e) {
      console.error('Failed to load Discord config:', e);
    }
  };

  const loadStats = async () => {
    try {
      const response = await bridgeFetch('/discord/stats');
      const data = await response.json();
      if (data.success && data.data) {
        setStats(data.data);
      }
    } catch (e) {
      console.error('Failed to load Discord stats:', e);
    }
  };

  const saveConfig = async () => {
    setSaving(true);
    try {
      const response = await bridgeFetch('/discord/config', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          bot_token: botToken() || null,
          channel_id: channelId() || null,
          enabled: enabled(),
          command_prefix: commandPrefix(),
          max_song_length: maxSongLength(),
          max_queue_size: maxQueueSize(),
        }),
      });

      const data = await response.json();
      if (data.success) {
        alert('Discord configuration saved successfully!');
        setShowConfig(false);
        await loadConfig();
      } else {
        alert(data.error || 'Failed to save configuration');
      }
    } catch (e) {
      console.error('Failed to save config:', e);
      alert(`Failed to save config: ${e.message}`);
    } finally {
      setSaving(false);
    }
  };

  const startBot = async () => {
    try {
      const response = await bridgeFetch('/discord/start', { method: 'POST' });
      const data = await response.json();
      if (data.success) {
        alert('Discord bot started!');
        await loadStats();
      } else {
        alert(data.error || 'Failed to start bot');
      }
    } catch (e) {
      console.error('Failed to start bot:', e);
      alert(`Failed to start bot: ${e.message}`);
    }
  };

  const stopBot = async () => {
    try {
      const response = await bridgeFetch('/discord/stop', { method: 'POST' });
      const data = await response.json();
      if (data.success) {
        alert('Discord bot stopped!');
        await loadStats();
      } else {
        alert(data.error || 'Failed to stop bot');
      }
    } catch (e) {
      console.error('Failed to stop bot:', e);
      alert(`Failed to stop bot: ${e.message}`);
    }
  };

  const restartBot = async () => {
    try {
      const response = await bridgeFetch('/discord/restart', { method: 'POST' });
      const data = await response.json();
      if (data.success) {
        alert('Discord bot restarted!');
        await loadStats();
      } else {
        alert(data.error || 'Failed to restart bot');
      }
    } catch (e) {
      console.error('Failed to restart bot:', e);
      alert(`Failed to restart bot: ${e.message}`);
    }
  };

  const getStatusColor = () => {
    if (!stats()) return 'gray';
    switch (stats().status) {
      case 'connected': return 'green';
      case 'connecting': return 'yellow';
      case 'error': return 'red';
      default: return 'gray';
    }
  };

  return (
    <div class="p-6 max-w-4xl mx-auto">
      <div class="flex items-center justify-between mb-6">
        <div class="flex items-center gap-3">
          <IconBrandDiscord size={32} class="text-indigo-500" />
          <h1 class="text-2xl font-bold">Discord Song Requests</h1>
        </div>
        <button
          onClick={() => setShowConfig(!showConfig())}
          class="px-4 py-2 bg-gray-700 hover:bg-gray-600 rounded-lg flex items-center gap-2 transition-colors"
        >
          <IconSettings size={20} />
          {showConfig() ? 'Hide' : 'Show'} Settings
        </button>
      </div>

      <Show when={loading()}>
        <div class="text-center py-8">
          <div class="animate-spin w-8 h-8 border-4 border-indigo-500 border-t-transparent rounded-full mx-auto"></div>
          <p class="mt-4 text-gray-400">Loading Discord settings...</p>
        </div>
      </Show>

      <Show when={!loading()}>
        {/* Status Card */}
        <div class="bg-gray-800 rounded-lg p-6 mb-6">
          <h2 class="text-lg font-semibold mb-4">Bot Status</h2>
          <div class="flex items-center gap-4">
            <div class="flex items-center gap-2">
              <div class={`w-3 h-3 rounded-full bg-${getStatusColor()}-500`}></div>
              <span class="capitalize">{stats()?.status || 'disconnected'}</span>
            </div>
            <div class="text-gray-400">
              Queue: {stats()?.queue_size || 0} songs
            </div>
          </div>

          <div class="flex gap-2 mt-4">
            <Show when={stats()?.status !== 'connected'}>
              <button
                onClick={startBot}
                disabled={!config()?.configured}
                class="px-4 py-2 bg-green-600 hover:bg-green-500 disabled:bg-gray-600 disabled:cursor-not-allowed rounded-lg flex items-center gap-2 transition-colors"
              >
                <IconPlayerPlay size={20} />
                Start Bot
              </button>
            </Show>
            <Show when={stats()?.status === 'connected'}>
              <button
                onClick={stopBot}
                class="px-4 py-2 bg-red-600 hover:bg-red-500 rounded-lg flex items-center gap-2 transition-colors"
              >
                <IconPlayerStop size={20} />
                Stop Bot
              </button>
              <button
                onClick={restartBot}
                class="px-4 py-2 bg-yellow-600 hover:bg-yellow-500 rounded-lg flex items-center gap-2 transition-colors"
              >
                <IconRefresh size={20} />
                Restart Bot
              </button>
            </Show>
          </div>
        </div>

        {/* Configuration Panel */}
        <Show when={showConfig()}>
          <div class="bg-gray-800 rounded-lg p-6">
            <h2 class="text-lg font-semibold mb-4">Configuration</h2>

            <div class="space-y-4">
              <div>
                <label class="block text-sm font-medium mb-2">Bot Token</label>
                <input
                  type="password"
                  value={botToken()}
                  onInput={(e) => setBotToken(e.target.value)}
                  placeholder="Your Discord bot token"
                  class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-lg focus:outline-none focus:border-indigo-500"
                />
                <p class="text-sm text-gray-400 mt-1">
                  Get your bot token from <a href="https://discord.com/developers/applications" target="_blank" class="text-indigo-400 hover:underline">Discord Developer Portal</a>
                </p>
              </div>

              <div>
                <label class="block text-sm font-medium mb-2">Channel ID</label>
                <input
                  type="text"
                  value={channelId()}
                  onInput={(e) => setChannelId(e.target.value)}
                  placeholder="Discord channel ID for song requests"
                  class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-lg focus:outline-none focus:border-indigo-500"
                />
                <p class="text-sm text-gray-400 mt-1">
                  Enable Developer Mode in Discord, right-click the channel, and select "Copy ID"
                </p>
              </div>

              <div>
                <label class="block text-sm font-medium mb-2">Command Prefix</label>
                <input
                  type="text"
                  value={commandPrefix()}
                  onInput={(e) => setCommandPrefix(e.target.value)}
                  placeholder="!sr"
                  class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-lg focus:outline-none focus:border-indigo-500"
                />
                <p class="text-sm text-gray-400 mt-1">
                  Users will type: {commandPrefix()} Song Name
                </p>
              </div>

              <div>
                <label class="block text-sm font-medium mb-2">Max Queue Size</label>
                <input
                  type="number"
                  value={maxQueueSize()}
                  onInput={(e) => setMaxQueueSize(parseInt(e.target.value))}
                  class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-lg focus:outline-none focus:border-indigo-500"
                />
              </div>

              <div>
                <label class="block text-sm font-medium mb-2">Max Song Length (seconds)</label>
                <input
                  type="number"
                  value={maxSongLength()}
                  onInput={(e) => setMaxSongLength(parseInt(e.target.value))}
                  class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-lg focus:outline-none focus:border-indigo-500"
                />
              </div>

              <button
                onClick={saveConfig}
                disabled={saving()}
                class="w-full px-4 py-2 bg-indigo-600 hover:bg-indigo-500 disabled:bg-gray-600 disabled:cursor-not-allowed rounded-lg flex items-center justify-center gap-2 transition-colors"
              >
                <Show when={!saving()}>
                  <IconCheck size={20} />
                  Save Configuration
                </Show>
                <Show when={saving()}>
                  <div class="animate-spin w-5 h-5 border-2 border-white border-t-transparent rounded-full"></div>
                  Saving...
                </Show>
              </button>
            </div>
          </div>
        </Show>

        {/* Quick Setup Guide */}
        <Show when={!config()?.configured}>
          <div class="bg-yellow-900/30 border border-yellow-700 rounded-lg p-6 mt-6">
            <h3 class="font-semibold mb-2 flex items-center gap-2">
              <IconSettings size={20} />
              Setup Required
            </h3>
            <ol class="list-decimal list-inside space-y-2 text-sm text-gray-300">
              <li>Create a bot at <a href="https://discord.com/developers/applications" target="_blank" class="text-indigo-400 hover:underline">Discord Developer Portal</a></li>
              <li>Enable "Message Content Intent" in Bot settings</li>
              <li>Copy your bot token and paste it above</li>
              <li>Invite the bot to your server using OAuth2 URL Generator (bot scope + Send Messages permission)</li>
              <li>Copy the channel ID where you want song requests</li>
              <li>Save configuration and start the bot</li>
            </ol>
          </div>
        </Show>
      </Show>
    </div>
  );
}
