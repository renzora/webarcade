import { createSignal, createEffect, Show, For } from 'solid-js';
import { bridgeFetch } from '@/api/bridge';
import {
  IconBrandTwitch,
  IconKey,
  IconCheck,
  IconAlertCircle,
  IconExternalLink,
  IconRefresh,
  IconTrash,
  IconX
} from '@tabler/icons-solidjs';

export default function TwitchSetupViewport() {
  // Setup state
  const [clientId, setClientId] = createSignal('');
  const [clientSecret, setClientSecret] = createSignal('');
  const [isConfigured, setIsConfigured] = createSignal(false);

  // Accounts state
  const [accounts, setAccounts] = createSignal([]);

  // IRC status
  const [ircStatus, setIrcStatus] = createSignal(null);

  // UI state
  const [loading, setLoading] = createSignal(false);
  const [saving, setSaving] = createSignal(false);
  const [error, setError] = createSignal('');
  const [success, setSuccess] = createSignal('');

  const loadAll = async () => {
    setLoading(true);
    setError('');

    try {
      const [setupResponse, accountsResponse, ircResponse] = await Promise.all([
        bridgeFetch('/twitch/setup/status'),
        bridgeFetch('/twitch/accounts'),
        bridgeFetch('/twitch/irc/status')
      ]);

      const setupData = await setupResponse.json();
      const accountsData = await accountsResponse.json();
      const ircData = await ircResponse.json();

      setIsConfigured(setupData.is_configured);
      setAccounts(accountsData);
      setIrcStatus(ircData);
    } catch (e) {
      console.error('Failed to load data:', e);
      setError('Failed to load configuration data');
    } finally {
      setLoading(false);
    }
  };

  const saveCredentials = async (e) => {
    e.preventDefault();

    const id = clientId().trim();
    const secret = clientSecret().trim();

    if (!id || !secret) {
      setError('Both Client ID and Client Secret are required');
      return;
    }

    setSaving(true);
    setError('');
    setSuccess('');

    try {
      const response = await bridgeFetch('/twitch/setup', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          client_id: id,
          client_secret: secret
        })
      });

      const data = await response.json();

      if (data.success) {
        setSuccess('Credentials saved successfully!');
        setClientId('');
        setClientSecret('');
        await loadAll();
      } else {
        setError('Failed to save credentials');
      }
    } catch (e) {
      console.error('Failed to save credentials:', e);
      setError('Failed to save credentials');
    } finally {
      setSaving(false);
    }
  };

  const startAuth = async (accountType) => {
    try {
      const response = await bridgeFetch(`/twitch/auth/start?type=${accountType}`);
      const data = await response.json();

      if (data.auth_url) {
        window.open(data.auth_url, '_blank');
        // Reload after a delay to check if auth completed
        setTimeout(loadAll, 3000);
      }
    } catch (e) {
      console.error('Failed to start auth:', e);
      setError('Failed to start authentication');
    }
  };

  const deleteAccount = async (accountType) => {
    if (!confirm(`Are you sure you want to remove the ${accountType} account?`)) {
      return;
    }

    try {
      await bridgeFetch(`/twitch/accounts/${accountType}`, {
        method: 'DELETE'
      });
      await loadAll();
      setSuccess(`${accountType} account removed`);
    } catch (e) {
      console.error('Failed to delete account:', e);
      setError('Failed to delete account');
    }
  };

  const refreshTokens = async () => {
    setLoading(true);
    try {
      await bridgeFetch('/twitch/auth/refresh', {
        method: 'POST'
      });
      await loadAll();
      setSuccess('Tokens refreshed successfully');
    } catch (e) {
      console.error('Failed to refresh tokens:', e);
      setError('Failed to refresh tokens');
    } finally {
      setLoading(false);
    }
  };

  createEffect(() => {
    loadAll();

    // Auto-refresh every 30 seconds
    const interval = setInterval(loadAll, 30000);
    return () => clearInterval(interval);
  });

  const getAccount = (type) => {
    return accounts().find(acc => acc.account_type === type);
  };

  return (
    <div class="h-full w-full bg-base-200 overflow-y-auto">
      <div class="max-w-6xl mx-auto p-8 space-y-6">
        {/* Header */}
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-3">
            <IconBrandTwitch class="w-10 h-10 text-primary" />
            <div>
              <h1 class="text-3xl font-bold">Twitch Setup</h1>
              <p class="text-base-content/60">Configure your Twitch integration</p>
            </div>
          </div>
          <button
            class="btn btn-circle btn-ghost"
            onClick={loadAll}
            disabled={loading()}
          >
            <IconRefresh class={`w-6 h-6 ${loading() ? 'animate-spin' : ''}`} />
          </button>
        </div>

        {/* Alerts */}
        <Show when={error()}>
          <div class="alert alert-error shadow-lg">
            <IconAlertCircle class="w-6 h-6" />
            <span>{error()}</span>
            <button class="btn btn-sm btn-ghost" onClick={() => setError('')}>
              <IconX class="w-4 h-4" />
            </button>
          </div>
        </Show>

        <Show when={success()}>
          <div class="alert alert-success shadow-lg">
            <IconCheck class="w-6 h-6" />
            <span>{success()}</span>
            <button class="btn btn-sm btn-ghost" onClick={() => setSuccess('')}>
              <IconX class="w-4 h-4" />
            </button>
          </div>
        </Show>

        <Show when={!loading()} fallback={
          <div class="flex justify-center items-center h-64">
            <span class="loading loading-spinner loading-lg"></span>
          </div>
        }>
          <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
            {/* Left Column - Setup */}
            <div class="space-y-6">
              {/* Step 1: App Credentials */}
              <div class="card bg-base-100 shadow-xl">
                <div class="card-body">
                  <div class="flex items-center gap-2 mb-4">
                    <div class="badge badge-primary badge-lg">Step 1</div>
                    <h2 class="card-title">App Credentials</h2>
                    <Show when={isConfigured()}>
                      <div class="badge badge-success gap-1 ml-auto">
                        <IconCheck class="w-3 h-3" />
                        Configured
                      </div>
                    </Show>
                  </div>

                  <div class="alert alert-info mb-4">
                    <IconKey class="w-5 h-5" />
                    <div class="text-sm">
                      <p class="font-medium">Create a Twitch Application</p>
                      <p class="text-xs">You need a Twitch app to use this integration</p>
                    </div>
                  </div>

                  <form onSubmit={saveCredentials} class="space-y-4">
                    <div>
                      <label class="label">
                        <span class="label-text font-medium">Client ID</span>
                      </label>
                      <input
                        type="text"
                        class="input input-bordered w-full font-mono text-sm"
                        placeholder="Enter your Twitch app Client ID"
                        value={clientId()}
                        onInput={(e) => setClientId(e.target.value)}
                        disabled={saving()}
                        required
                      />
                    </div>

                    <div>
                      <label class="label">
                        <span class="label-text font-medium">Client Secret</span>
                      </label>
                      <input
                        type="password"
                        class="input input-bordered w-full font-mono text-sm"
                        placeholder="Enter your Twitch app Client Secret"
                        value={clientSecret()}
                        onInput={(e) => setClientSecret(e.target.value)}
                        disabled={saving()}
                        required
                      />
                    </div>

                    <button
                      type="submit"
                      class="btn btn-primary w-full"
                      disabled={saving()}
                    >
                      {saving() ? (
                        <>
                          <span class="loading loading-spinner loading-sm"></span>
                          Saving...
                        </>
                      ) : (
                        <>
                          <IconKey class="w-4 h-4" />
                          Save Credentials
                        </>
                      )}
                    </button>
                  </form>

                  <div class="divider">How to get credentials</div>

                  <ol class="text-sm space-y-2 list-decimal list-inside">
                    <li>
                      Visit{' '}
                      <a
                        href="https://dev.twitch.tv/console"
                        target="_blank"
                        rel="noopener noreferrer"
                        class="link link-primary inline-flex items-center gap-1"
                      >
                        Twitch Developer Console
                        <IconExternalLink class="w-3 h-3" />
                      </a>
                    </li>
                    <li>Click "Register Your Application"</li>
                    <li>
                      Add OAuth Redirect URL:{' '}
                      <code class="bg-base-200 px-2 py-1 rounded text-xs">
                        http://localhost:3001/twitch/auth/callback
                      </code>
                    </li>
                    <li>Copy Client ID and generate Client Secret</li>
                    <li>Paste them above and save</li>
                  </ol>
                </div>
              </div>

              {/* Instructions Card */}
              <div class="card bg-base-300">
                <div class="card-body">
                  <h3 class="font-bold text-lg mb-2">Setup Flow</h3>
                  <ol class="space-y-2 text-sm">
                    <li class="flex items-start gap-2">
                      <div class={`badge ${isConfigured() ? 'badge-success' : 'badge-neutral'}`}>1</div>
                      <div>
                        <strong>Configure App</strong> - Enter Client ID & Secret above
                      </div>
                    </li>
                    <li class="flex items-start gap-2">
                      <div class={`badge ${getAccount('broadcaster') ? 'badge-success' : 'badge-neutral'}`}>2</div>
                      <div>
                        <strong>Connect Broadcaster</strong> - Authorize your main account
                      </div>
                    </li>
                    <li class="flex items-start gap-2">
                      <div class={`badge ${getAccount('bot') ? 'badge-success' : 'badge-neutral'}`}>3</div>
                      <div>
                        <strong>(Optional) Connect Bot</strong> - Add a separate bot account
                      </div>
                    </li>
                    <li class="flex items-start gap-2">
                      <div class={`badge ${ircStatus()?.connected ? 'badge-success' : 'badge-neutral'}`}>4</div>
                      <div>
                        <strong>Done!</strong> - IRC and EventSub will auto-connect
                      </div>
                    </li>
                  </ol>
                </div>
              </div>
            </div>

            {/* Right Column - Accounts & Status */}
            <div class="space-y-6">
              {/* Step 2: Connect Accounts */}
              <div class="card bg-base-100 shadow-xl">
                <div class="card-body">
                  <div class="flex items-center gap-2 mb-4">
                    <div class="badge badge-primary badge-lg">Step 2</div>
                    <h2 class="card-title">Connect Accounts</h2>
                  </div>

                  <Show when={!isConfigured()}>
                    <div class="alert alert-warning">
                      <IconAlertCircle class="w-5 h-5" />
                      <span class="text-sm">Complete Step 1 first</span>
                    </div>
                  </Show>

                  <div class="space-y-4">
                    {/* Broadcaster Account */}
                    <div class="card bg-base-200">
                      <div class="card-body">
                        <h3 class="font-bold flex items-center gap-2">
                          <IconBrandTwitch class="w-5 h-5" />
                          Broadcaster Account
                        </h3>
                        <Show
                          when={getAccount('broadcaster')}
                          fallback={
                            <div class="space-y-2">
                              <p class="text-sm text-base-content/60">Not connected</p>
                              <button
                                class="btn btn-primary btn-sm"
                                onClick={() => startAuth('broadcaster')}
                                disabled={!isConfigured()}
                              >
                                Connect Broadcaster
                              </button>
                            </div>
                          }
                        >
                          <div class="space-y-2">
                            <div class="flex items-center gap-2">
                              <IconCheck class="w-4 h-4 text-success" />
                              <span class="font-medium">{getAccount('broadcaster')?.username}</span>
                            </div>
                            <div class="text-xs text-base-content/60">
                              User ID: {getAccount('broadcaster')?.user_id}
                            </div>
                            <button
                              class="btn btn-error btn-xs"
                              onClick={() => deleteAccount('broadcaster')}
                            >
                              <IconTrash class="w-3 h-3" />
                              Disconnect
                            </button>
                          </div>
                        </Show>
                      </div>
                    </div>

                    {/* Bot Account */}
                    <div class="card bg-base-200">
                      <div class="card-body">
                        <h3 class="font-bold flex items-center gap-2">
                          <IconBrandTwitch class="w-5 h-5" />
                          Bot Account (Optional)
                        </h3>
                        <Show
                          when={getAccount('bot')}
                          fallback={
                            <div class="space-y-2">
                              <p class="text-sm text-base-content/60">Not connected</p>
                              <button
                                class="btn btn-secondary btn-sm"
                                onClick={() => startAuth('bot')}
                                disabled={!isConfigured()}
                              >
                                Connect Bot
                              </button>
                            </div>
                          }
                        >
                          <div class="space-y-2">
                            <div class="flex items-center gap-2">
                              <IconCheck class="w-4 h-4 text-success" />
                              <span class="font-medium">{getAccount('bot')?.username}</span>
                            </div>
                            <div class="text-xs text-base-content/60">
                              User ID: {getAccount('bot')?.user_id}
                            </div>
                            <button
                              class="btn btn-error btn-xs"
                              onClick={() => deleteAccount('bot')}
                            >
                              <IconTrash class="w-3 h-3" />
                              Disconnect
                            </button>
                          </div>
                        </Show>
                      </div>
                    </div>

                    <button
                      class="btn btn-outline btn-sm w-full"
                      onClick={refreshTokens}
                      disabled={accounts().length === 0}
                    >
                      <IconRefresh class="w-4 h-4" />
                      Refresh Tokens
                    </button>
                  </div>
                </div>
              </div>

              {/* Connection Status */}
              <div class="card bg-base-100 shadow-xl">
                <div class="card-body">
                  <h2 class="card-title">Connection Status</h2>

                  <div class="space-y-3">
                    <div class="flex items-center justify-between">
                      <span class="text-sm">IRC Chat</span>
                      <div class="flex items-center gap-2">
                        <Show
                          when={ircStatus()?.connected}
                          fallback={
                            <>
                              <IconX class="w-4 h-4 text-error" />
                              <span class="text-sm text-error">Disconnected</span>
                            </>
                          }
                        >
                          <IconCheck class="w-4 h-4 text-success" />
                          <span class="text-sm text-success">Connected</span>
                        </Show>
                      </div>
                    </div>

                    <Show when={ircStatus()?.channel}>
                      <div class="text-xs text-base-content/60">
                        Channel: #{ircStatus()?.channel}
                      </div>
                    </Show>

                    <div class="divider my-2"></div>

                    <div class="space-y-1">
                      <div class="flex items-center gap-2 text-sm">
                        <IconCheck class="w-4 h-4 text-success" />
                        <span>EventSub WebSocket</span>
                      </div>
                      <div class="flex items-center gap-2 text-sm">
                        <IconCheck class="w-4 h-4 text-success" />
                        <span>Automatic Token Refresh</span>
                      </div>
                      <div class="flex items-center gap-2 text-sm">
                        <IconCheck class="w-4 h-4 text-success" />
                        <span>Service API Available</span>
                      </div>
                    </div>
                  </div>
                </div>
              </div>

              {/* Quick Links */}
              <div class="card bg-primary text-primary-content">
                <div class="card-body">
                  <h3 class="font-bold">Next Steps</h3>
                  <div class="text-sm space-y-2">
                    <p>Once configured, you can:</p>
                    <ul class="list-disc list-inside space-y-1 text-xs">
                      <li>View chat in the Twitch Chat widget</li>
                      <li>Monitor events in Twitch Events widget</li>
                      <li>Build plugins that use Twitch integration</li>
                      <li>Access Twitch API from other plugins</li>
                    </ul>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </Show>
      </div>
    </div>
  );
}
