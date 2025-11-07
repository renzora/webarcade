import { createSignal, onMount, Show, For } from 'solid-js';
import twitchStore from './TwitchStore.jsx';
import TwitchAccountManager from './TwitchAccountManager.jsx';
import { IconBrandTwitch, IconCheck, IconX, IconKey, IconUser, IconHash, IconUsers, IconAlertCircle, IconExternalLink, IconCopy, IconWebhook, IconRefresh, IconTrash } from '@tabler/icons-solidjs';

export default function TwitchSettingsViewport() {
  const [config, setConfig] = createSignal({
    client_id: '',
    client_secret: '',
    has_token: false
  });

  const [clientId, setClientId] = createSignal('');
  const [clientSecret, setClientSecret] = createSignal('');

  const [saving, setSaving] = createSignal(false);
  const [saveMessage, setSaveMessage] = createSignal('');
  const [loading, setLoading] = createSignal(true);

  // EventSub webhook state
  const [webhookUrl, setWebhookUrl] = createSignal('');
  const [subscriptions, setSubscriptions] = createSignal([]);
  const [settingUpWebhook, setSettingUpWebhook] = createSignal(false);
  const [webhookMessage, setWebhookMessage] = createSignal('');
  const [loadingSubscriptions, setLoadingSubscriptions] = createSignal(false);

  // Ngrok state
  const [ngrokRunning, setNgrokRunning] = createSignal(false);
  const [ngrokStarting, setNgrokStarting] = createSignal(false);
  const [ngrokStopping, setNgrokStopping] = createSignal(false);
  const [ngrokPublicUrl, setNgrokPublicUrl] = createSignal('');

  onMount(async () => {
    const loadedConfig = await twitchStore.fetchConfig();
    if (loadedConfig) {
      setConfig(loadedConfig);
      setClientId(loadedConfig.client_id || '');
    }
    setLoading(false);

    // Load existing subscriptions
    await loadSubscriptions();

    // Check ngrok status
    await checkNgrokStatus();
  });

  const loadSubscriptions = async () => {
    setLoadingSubscriptions(true);
    try {
      const response = await fetch('http://localhost:3001/twitch/eventsub/subscriptions');
      const data = await response.json();
      setSubscriptions(data.subscriptions || []);
    } catch (e) {
      console.error('Failed to load subscriptions:', e);
    } finally {
      setLoadingSubscriptions(false);
    }
  };

  const handleSaveConfig = async () => {
    setSaving(true);
    setSaveMessage('');

    try {
      const configToSave = {
        client_id: clientId()
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

  const handleAutoSetupWebhook = async () => {
    if (!webhookUrl()) {
      setWebhookMessage('error:Please enter a webhook URL');
      setTimeout(() => setWebhookMessage(''), 3000);
      return;
    }

    setSettingUpWebhook(true);
    setWebhookMessage('');

    try {
      const response = await fetch('http://localhost:3001/twitch/eventsub/auto-setup', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json'
        },
        body: JSON.stringify({
          callback_url: webhookUrl()
        })
      });

      const data = await response.json();

      if (!response.ok) {
        throw new Error(data.error || 'Failed to setup webhook');
      }

      setWebhookMessage(`success:${data.message || `Created ${data.count || 0} subscriptions`}`);
      setTimeout(() => setWebhookMessage(''), 5000);

      // Reload subscriptions
      await loadSubscriptions();
    } catch (e) {
      setWebhookMessage(`error:${e.message}`);
      setTimeout(() => setWebhookMessage(''), 5000);
    } finally {
      setSettingUpWebhook(false);
    }
  };

  const handleDeleteAllSubscriptions = async () => {
    if (!confirm('Are you sure you want to delete all EventSub subscriptions? This will stop receiving follow notifications.')) {
      return;
    }

    setLoadingSubscriptions(true);
    try {
      const response = await fetch('http://localhost:3001/twitch/eventsub/subscriptions', {
        method: 'DELETE'
      });

      if (!response.ok) {
        throw new Error('Failed to delete subscriptions');
      }

      setWebhookMessage('success:All subscriptions deleted');
      setTimeout(() => setWebhookMessage(''), 3000);

      // Reload subscriptions
      await loadSubscriptions();
    } catch (e) {
      setWebhookMessage(`error:${e.message}`);
      setTimeout(() => setWebhookMessage(''), 5000);
    } finally {
      setLoadingSubscriptions(false);
    }
  };

  const checkNgrokStatus = async () => {
    try {
      const response = await fetch('http://localhost:3001/twitch/ngrok/status');
      const data = await response.json();

      setNgrokRunning(data.running);
      if (data.running && data.public_url) {
        setNgrokPublicUrl(data.public_url);
        // Auto-fill webhook URL
        setWebhookUrl(`${data.public_url}/twitch/eventsub/webhook`);
      }
    } catch (e) {
      console.error('Failed to check ngrok status:', e);
      setNgrokRunning(false);
    }
  };

  const handleStartNgrok = async () => {
    setNgrokStarting(true);
    setWebhookMessage('');

    try {
      const response = await fetch('http://localhost:3001/twitch/ngrok/start', {
        method: 'POST'
      });

      const data = await response.json();

      if (!response.ok) {
        throw new Error(data.error || 'Failed to start ngrok');
      }

      setNgrokRunning(true);
      setNgrokPublicUrl(data.public_url);
      setWebhookUrl(`${data.public_url}/twitch/eventsub/webhook`);
      setWebhookMessage(`success:Ngrok started! URL: ${data.public_url}`);
      setTimeout(() => setWebhookMessage(''), 5000);
    } catch (e) {
      setWebhookMessage(`error:${e.message}. Make sure ngrok is installed.`);
      setTimeout(() => setWebhookMessage(''), 5000);
    } finally {
      setNgrokStarting(false);
    }
  };

  const handleStopNgrok = async () => {
    setNgrokStopping(true);
    setWebhookMessage('');

    try {
      const response = await fetch('http://localhost:3001/twitch/ngrok/stop', {
        method: 'POST'
      });

      if (!response.ok) {
        throw new Error('Failed to stop ngrok');
      }

      setNgrokRunning(false);
      setNgrokPublicUrl('');
      setWebhookMessage('success:Ngrok stopped');
      setTimeout(() => setWebhookMessage(''), 3000);
    } catch (e) {
      setWebhookMessage(`error:${e.message}`);
      setTimeout(() => setWebhookMessage(''), 5000);
    } finally {
      setNgrokStopping(false);
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

          {/* Application Configuration */}
          <div class="card bg-base-100 shadow-xl">
            <div class="card-body">
              <h2 class="card-title">
                <IconKey size={24} />
                Twitch Application Settings
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

          {/* EventSub Webhook Configuration */}
          <div class="card bg-base-100 shadow-xl">
            <div class="card-body">
              <h2 class="card-title">
                <IconWebhook size={24} />
                EventSub Webhook Configuration
              </h2>
              <p class="text-sm opacity-70">
                Configure EventSub webhooks to receive real-time notifications for follows, channel points, and more.
              </p>

              {/* Webhook Message Alert */}
              <Show when={webhookMessage()}>
                <div class={`alert ${webhookMessage().startsWith('success') ? 'alert-success' : 'alert-error'} shadow-lg`}>
                  <Show when={webhookMessage().startsWith('success')} fallback={
                    <>
                      <IconAlertCircle />
                      <span>{webhookMessage().replace('error:', '')}</span>
                    </>
                  }>
                    <IconCheck />
                    <span>{webhookMessage().includes(':') ? webhookMessage().split(':')[1] : 'EventSub setup successful! Subscriptions created.'}</span>
                  </Show>
                </div>
              </Show>

              {/* Ngrok Controls */}
              <div class="card bg-base-300/50 border-2 border-primary/20">
                <div class="card-body p-4">
                  <div class="flex items-center justify-between">
                    <div class="flex items-center gap-3">
                      <div class={`badge ${ngrokRunning() ? 'badge-success' : 'badge-ghost'} gap-2`}>
                        {ngrokRunning() ? <IconCheck size={16} /> : <IconX size={16} />}
                        {ngrokRunning() ? 'Running' : 'Stopped'}
                      </div>
                      <Show when={ngrokRunning() && ngrokPublicUrl()}>
                        <span class="text-sm font-mono">{ngrokPublicUrl()}</span>
                      </Show>
                    </div>
                    <div class="flex gap-2">
                      <Show
                        when={ngrokRunning()}
                        fallback={
                          <button
                            class={`btn btn-primary btn-sm gap-2 ${ngrokStarting() ? 'loading' : ''}`}
                            onClick={handleStartNgrok}
                            disabled={ngrokStarting()}
                          >
                            {!ngrokStarting() && <IconRefresh size={16} />}
                            {ngrokStarting() ? 'Starting...' : 'Start Ngrok'}
                          </button>
                        }
                      >
                        <button
                          class={`btn btn-error btn-sm btn-outline gap-2 ${ngrokStopping() ? 'loading' : ''}`}
                          onClick={handleStopNgrok}
                          disabled={ngrokStopping()}
                        >
                          {!ngrokStopping() && <IconX size={16} />}
                          {ngrokStopping() ? 'Stopping...' : 'Stop Ngrok'}
                        </button>
                      </Show>
                    </div>
                  </div>
                  <Show when={!ngrokRunning()}>
                    <p class="text-xs text-base-content/60 mt-2">
                      Click "Start Ngrok" to automatically create a public tunnel. No manual ngrok setup needed!
                    </p>
                  </Show>
                </div>
              </div>

              <div class="form-control">
                <label class="label">
                  <span class="label-text font-semibold">Webhook URL</span>
                  <span class="label-text-alt">From ngrok or public URL</span>
                </label>
                <input
                  type="text"
                  placeholder="https://your-ngrok-url.ngrok-free.app/twitch/eventsub/webhook"
                  class="input input-bordered"
                  value={webhookUrl()}
                  onInput={(e) => setWebhookUrl(e.target.value)}
                />
                <label class="label">
                  <span class="label-text-alt text-info">
                    💡 Use ngrok to get a public URL: <code class="bg-base-300 px-1 rounded">ngrok http 3001</code>
                  </span>
                </label>
              </div>

              <div class="card-actions justify-between mt-4">
                <button
                  class={`btn btn-primary gap-2 ${settingUpWebhook() ? 'loading' : ''}`}
                  onClick={handleAutoSetupWebhook}
                  disabled={settingUpWebhook() || !config().has_token}
                >
                  {!settingUpWebhook() && <IconCheck size={20} />}
                  {settingUpWebhook() ? 'Setting Up...' : 'Auto-Setup All EventSub Events'}
                </button>

                <div class="flex gap-2">
                  <button
                    class={`btn btn-ghost gap-2 ${loadingSubscriptions() ? 'loading' : ''}`}
                    onClick={loadSubscriptions}
                    disabled={loadingSubscriptions()}
                  >
                    {!loadingSubscriptions() && <IconRefresh size={20} />}
                    Refresh
                  </button>
                  <Show when={subscriptions().length > 0}>
                    <button
                      class="btn btn-error btn-outline gap-2"
                      onClick={handleDeleteAllSubscriptions}
                    >
                      <IconTrash size={20} />
                      Delete All
                    </button>
                  </Show>
                </div>
              </div>

              {/* Active Subscriptions */}
              <Show when={subscriptions().length > 0}>
                <div class="divider">Active Subscriptions</div>
                <div class="space-y-2">
                  <For each={subscriptions()}>
                    {(sub) => (
                      <div class="alert alert-success">
                        <IconCheck size={20} />
                        <div class="flex-1">
                          <div class="font-semibold">{sub.type || sub.subscription_type}</div>
                          <div class="text-xs opacity-70">
                            Status: {sub.status} • Created: {new Date(sub.created_at).toLocaleString()}
                          </div>
                        </div>
                      </div>
                    )}
                  </For>
                </div>
              </Show>

              <Show when={!config().has_token}>
                <div class="alert alert-warning mt-4">
                  <IconAlertCircle size={20} />
                  <div class="text-sm">
                    <strong>Please authenticate first!</strong> You need to connect your Twitch account before setting up EventSub.
                  </div>
                </div>
              </Show>

              <div class="alert alert-info mt-4">
                <IconAlertCircle size={20} />
                <div class="text-sm space-y-2">
                  <p><strong>What are EventSub webhooks?</strong></p>
                  <p>EventSub lets your bot receive real-time notifications for:</p>
                  <ul class="list-disc list-inside ml-4">
                    <li><strong>Follows, Subs, Cheers, Raids</strong> - Viewer interactions</li>
                    <li><strong>Channel Points</strong> - All reward redemptions</li>
                    <li><strong>Polls & Predictions</strong> - Community voting</li>
                    <li><strong>Hype Trains</strong> - Community hype events</li>
                    <li><strong>Charity Campaigns</strong> - Donation tracking</li>
                    <li><strong>Bans, Mods, Updates</strong> - Channel management</li>
                    <li><strong>Stream Status</strong> - Online/offline events</li>
                  </ul>
                  <p class="mt-2">
                    <strong>How to use:</strong>
                  </p>
                  <ol class="list-decimal list-inside ml-4">
                    <li>Run <code class="bg-base-300 px-1 rounded">ngrok http 3001</code> to get a public URL</li>
                    <li>Copy the HTTPS URL (e.g., https://abc123.ngrok-free.app)</li>
                    <li>Paste it above and add <code class="bg-base-300 px-1 rounded">/twitch/eventsub/webhook</code> at the end</li>
                    <li>Click "Auto-Setup All EventSub Events" - this creates 70+ subscriptions!</li>
                  </ol>
                </div>
              </div>
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
                <div class="step step-primary">Authenticate</div>
                <div class="step">Done!</div>
              </div>
              <ol class="list-decimal list-inside space-y-2 mt-4 text-sm">
                <li>Go to <a href="https://dev.twitch.tv/console/apps" target="_blank" class="link link-primary">dev.twitch.tv/console/apps</a></li>
                <li>Click "Register Your Application"</li>
                <li>Set OAuth Redirect URL: <code class="bg-base-300 px-2 py-1 rounded">http://localhost:3001/twitch/auth/callback</code></li>
                <li>Copy Client ID and Client Secret to the form above</li>
                <li>Click "Authenticate with Twitch" to connect your broadcaster account</li>
                <li>Make sure to give the WebArcade bot moderator permissions in your channel</li>
              </ol>
              <div class="alert alert-info mt-4">
                <IconAlertCircle size={20} />
                <div class="text-sm">
                  <strong>Note:</strong> After authenticating, your channel will be automatically configured.
                  The WebArcade bot will handle all chat commands and interactions.
                </div>
              </div>
            </div>
          </div>
        </div>
      </Show>
    </div>
  );
}
