import { createSignal, onMount, Show } from 'solid-js';
import twitchStore from './TwitchStore.jsx';
import TwitchAccountManager from './TwitchAccountManager.jsx';
import { IconBrandTwitch, IconCheck, IconX, IconKey, IconUser, IconHash, IconUsers, IconAlertCircle, IconExternalLink, IconCopy } from '@tabler/icons-solidjs';

export default function TwitchSettingsViewport() {
  const [config, setConfig] = createSignal({
    client_id: '',
    client_secret: '',
    bot_username: '',
    channels: [],
    has_token: false
  });

  const [clientId, setClientId] = createSignal('');
  const [clientSecret, setClientSecret] = createSignal('');
  const [botUsername, setBotUsername] = createSignal('');
  const [channels, setChannels] = createSignal('');
  const [newChannel, setNewChannel] = createSignal('');

  const [saving, setSaving] = createSignal(false);
  const [saveMessage, setSaveMessage] = createSignal('');
  const [loading, setLoading] = createSignal(true);

  onMount(async () => {
    const loadedConfig = await twitchStore.fetchConfig();
    if (loadedConfig) {
      setConfig(loadedConfig);
      setClientId(loadedConfig.client_id || '');
      setBotUsername(loadedConfig.bot_username || '');
      setChannels(loadedConfig.channels.join(', ') || '');
    }
    setLoading(false);
  });

  const handleSaveConfig = async () => {
    setSaving(true);
    setSaveMessage('');

    try {
      const channelList = channels()
        .split(',')
        .map((ch) => ch.trim())
        .filter((ch) => ch.length > 0);

      const configToSave = {
        client_id: clientId(),
        bot_username: botUsername(),
        channels: channelList
      };

      if (clientSecret()) {
        configToSave.client_secret = clientSecret();
      }

      await twitchStore.saveConfig(configToSave);
      setSaveMessage('success');
      setTimeout(() => setSaveMessage(''), 3000);

      const loadedConfig = await twitchStore.fetchConfig();
      if (loadedConfig) {
        setConfig(loadedConfig);
      }
    } catch (e) {
      setSaveMessage(`error:${e.message}`);
      setTimeout(() => setSaveMessage(''), 5000);
    } finally {
      setSaving(false);
    }
  };

  const handleAddChannel = async () => {
    const channel = newChannel().trim();
    if (!channel) return;

    try {
      await twitchStore.joinChannel(channel);
      setNewChannel('');

      const loadedConfig = await twitchStore.fetchConfig();
      if (loadedConfig) {
        setConfig(loadedConfig);
        setChannels(loadedConfig.channels.join(', '));
      }
    } catch (e) {
      alert(`Failed to join channel: ${e.message}`);
    }
  };

  return (
    <div class="h-full overflow-y-auto bg-base-200">
      <Show when={!loading()} fallback={
        <div class="flex items-center justify-center h-full">
          <span class="loading loading-spinner loading-lg"></span>
        </div>
      }>
        <div class="max-w-4xl mx-auto p-6 space-y-6">
          {/* Header */}
          <div class="flex items-center gap-4 p-6 bg-gradient-to-r from-purple-600 to-purple-700 rounded-xl shadow-lg">
            <div class="p-4 bg-white/20 rounded-lg backdrop-blur-sm">
              <IconBrandTwitch size={40} class="text-white" />
            </div>
            <div class="flex-1">
              <h1 class="text-3xl font-bold text-white">Twitch Bot Settings</h1>
              <p class="text-purple-100">Configure your Twitch application and manage accounts</p>
            </div>
          </div>

          {/* Save Message Alert */}
          <Show when={saveMessage()}>
            <div class={`alert ${saveMessage() === 'success' ? 'alert-success' : 'alert-error'} shadow-lg`}>
              <Show when={saveMessage() === 'success'} fallback={
                <>
                  <IconAlertCircle />
                  <span>{saveMessage().replace('error:', '')}</span>
                </>
              }>
                <IconCheck />
                <span>Configuration saved successfully!</span>
              </Show>
            </div>
          </Show>

          {/* Account Management */}
          <div class="card bg-base-100 shadow-xl">
            <div class="card-body">
              <TwitchAccountManager />
            </div>
          </div>

          {/* Bot Configuration */}
          <div class="card bg-base-100 shadow-xl">
            <div class="card-body">
              <h2 class="card-title">
                <IconUser size={24} />
                Bot Configuration
              </h2>

              <div class="form-control">
                <label class="label">
                  <span class="label-text font-semibold">Client ID</span>
                  <span class="label-text-alt">Required</span>
                </label>
                <input
                  type="text"
                  placeholder="Your Twitch Application Client ID"
                  class="input input-bordered"
                  value={clientId()}
                  onInput={(e) => setClientId(e.target.value)}
                />
              </div>

              <div class="form-control">
                <label class="label">
                  <span class="label-text font-semibold">Client Secret</span>
                  <span class="label-text-alt">Required (stored securely)</span>
                </label>
                <input
                  type="password"
                  placeholder="Your Twitch Application Client Secret"
                  class="input input-bordered"
                  value={clientSecret()}
                  onInput={(e) => setClientSecret(e.target.value)}
                />
                <label class="label">
                  <span class="label-text-alt text-warning">
                    💡 Leave blank if already configured
                  </span>
                </label>
              </div>

              <div class="form-control">
                <label class="label">
                  <span class="label-text font-semibold">Bot Username</span>
                  <span class="label-text-alt">Required</span>
                </label>
                <input
                  type="text"
                  placeholder="Your bot's Twitch username"
                  class="input input-bordered"
                  value={botUsername()}
                  onInput={(e) => setBotUsername(e.target.value)}
                />
              </div>

              <div class="form-control">
                <label class="label">
                  <span class="label-text font-semibold">Channels</span>
                  <span class="label-text-alt">Comma-separated</span>
                </label>
                <textarea
                  placeholder="channel1, channel2, channel3"
                  class="textarea textarea-bordered h-20"
                  value={channels()}
                  onInput={(e) => setChannels(e.target.value)}
                />
              </div>

              <div class="card-actions justify-end mt-4">
                <button
                  class={`btn btn-primary gap-2 ${saving() ? 'loading' : ''}`}
                  onClick={handleSaveConfig}
                  disabled={saving()}
                >
                  {!saving() && <IconCheck size={20} />}
                  {saving() ? 'Saving...' : 'Save Configuration'}
                </button>
              </div>
            </div>
          </div>

          {/* Quick Channel Management */}
          <div class="card bg-base-100 shadow-xl">
            <div class="card-body">
              <h2 class="card-title">
                <IconUsers size={24} />
                Quick Channel Management
              </h2>

              <div class="join w-full">
                <input
                  type="text"
                  placeholder="Channel name"
                  class="input input-bordered join-item flex-1"
                  value={newChannel()}
                  onInput={(e) => setNewChannel(e.target.value)}
                  onKeyPress={(e) => {
                    if (e.key === 'Enter') handleAddChannel();
                  }}
                />
                <button
                  class="btn btn-primary join-item"
                  onClick={handleAddChannel}
                  disabled={!newChannel().trim()}
                >
                  Join Channel
                </button>
              </div>

              <Show when={config().channels.length > 0}>
                <div class="divider">Connected Channels</div>
                <div class="flex flex-wrap gap-2">
                  {config().channels.map((channel) => (
                    <div class="badge badge-lg badge-primary gap-2">
                      <IconHash size={14} />
                      {channel}
                    </div>
                  ))}
                </div>
              </Show>
            </div>
          </div>

          {/* Setup Guide */}
          <div class="card bg-gradient-to-br from-primary/10 to-secondary/10 border-2 border-primary/20">
            <div class="card-body">
              <h2 class="card-title">
                <IconExternalLink size={24} />
                Getting Started Guide
              </h2>
              <div class="steps steps-vertical lg:steps-horizontal w-full">
                <div class="step step-primary">Create Twitch App</div>
                <div class="step step-primary">Configure Settings</div>
                <div class="step step-primary">Add Accounts</div>
                <div class="step">Start Bot</div>
              </div>
              <ol class="list-decimal list-inside space-y-2 mt-4 text-sm">
                <li>Go to <a href="https://dev.twitch.tv/console/apps" target="_blank" class="link link-primary">dev.twitch.tv/console/apps</a></li>
                <li>Click "Register Your Application"</li>
                <li>Set OAuth Redirect URL: <code class="bg-base-300 px-2 py-1 rounded">https://localhost:3000/twitch/callback</code></li>
                <li>Copy Client ID and Client Secret</li>
                <li>Fill in Bot Configuration and save</li>
                <li>Add Bot account in "Authenticated Accounts" section</li>
                <li>Add Broadcaster account for stream title/category control</li>
                <li>Go to Twitch menu → Start Bot</li>
              </ol>
            </div>
          </div>
        </div>
      </Show>
    </div>
  );
}
