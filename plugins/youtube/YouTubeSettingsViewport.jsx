import { Show, createSignal, onMount } from 'solid-js';
import { IconCheck, IconAlertCircle, IconBrandYoutube } from '@tabler/icons-solidjs';
import youtubeStore from './YouTubeStore.jsx';

export default function YouTubeSettingsViewport() {
  const [clientId, setClientId] = createSignal('');
  const [clientSecret, setClientSecret] = createSignal('');
  const [saveStatus, setSaveStatus] = createSignal(null);

  onMount(async () => {
    // Load existing credentials if any
    try {
      const response = await fetch('http://localhost:3001/youtube/config');
      if (response.ok) {
        const data = await response.json();
        if (data.client_id) setClientId(data.client_id);
        // Don't show client secret for security
      }
    } catch (error) {
      console.error('Failed to load config:', error);
    }
  });

  const saveCredentials = async () => {
    try {
      setSaveStatus('saving');

      const response = await fetch('http://localhost:3001/config', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          youtube_client_id: clientId(),
          youtube_client_secret: clientSecret(),
        }),
      });

      if (response.ok) {
        setSaveStatus('success');
        setTimeout(() => setSaveStatus(null), 3000);
      } else {
        setSaveStatus('error');
      }
    } catch (error) {
      console.error('Failed to save credentials:', error);
      setSaveStatus('error');
    }
  };

  const handleAuthenticate = async () => {
    await youtubeStore.authenticate();
  };

  const handleRevoke = async () => {
    if (confirm('Are you sure you want to revoke YouTube access?')) {
      await youtubeStore.revokeAuth();
    }
  };

  return (
    <div class="h-full overflow-y-auto bg-base-200">
      <div class="max-w-4xl mx-auto p-6 space-y-6">
        {/* Header */}
        <div class="flex items-center gap-4 p-6 bg-gradient-to-r from-red-600 to-red-700 rounded-xl shadow-lg">
          <div class="p-4 bg-white/20 rounded-lg backdrop-blur-sm">
            <IconBrandYoutube size={40} class="text-white" />
          </div>
          <div class="flex-1">
            <h1 class="text-3xl font-bold text-white">YouTube Settings</h1>
            <p class="text-red-100">Configure YouTube API credentials and authentication</p>
          </div>
        </div>

        {/* OAuth Credentials */}
        <div class="card bg-base-100 shadow-xl">
          <div class="card-body">
            <h2 class="card-title">API Credentials</h2>
            <p class="text-sm opacity-70">
              Configure your YouTube Data API credentials. Get them from the{' '}
              <a
                href="https://console.cloud.google.com/apis/credentials"
                target="_blank"
                class="link link-primary"
              >
                Google Cloud Console
              </a>
            </p>

            <div class="form-control">
              <label class="label">
                <span class="label-text font-semibold">Client ID</span>
                <span class="label-text-alt">Required</span>
              </label>
              <input
                type="text"
                value={clientId()}
                onInput={(e) => setClientId(e.target.value)}
                placeholder="Your OAuth 2.0 Client ID"
                class="input input-bordered"
              />
            </div>

            <div class="form-control">
              <label class="label">
                <span class="label-text font-semibold">Client Secret</span>
                <span class="label-text-alt">Required (stored securely)</span>
              </label>
              <input
                type="password"
                value={clientSecret()}
                onInput={(e) => setClientSecret(e.target.value)}
                placeholder="Your OAuth 2.0 Client Secret"
                class="input input-bordered"
              />
            </div>

            <div class="form-control">
              <label class="label">
                <span class="label-text font-semibold">Redirect URI</span>
              </label>
              <input
                type="text"
                value="http://localhost:3000/api/plugin/youtube/auth/callback"
                disabled
                class="input input-bordered bg-base-200"
              />
              <label class="label">
                <span class="label-text-alt">
                  Add this URI to your authorized redirect URIs in Google Cloud Console
                </span>
              </label>
            </div>

            <div class="card-actions justify-end mt-4">
              <button
                onClick={saveCredentials}
                disabled={saveStatus() === 'saving'}
                class={`btn btn-primary gap-2 ${saveStatus() === 'saving' ? 'loading' : ''}`}
              >
                <Show when={saveStatus() === 'success'}>
                  <IconCheck size={16} />
                </Show>
                {saveStatus() === 'saving' ? 'Saving...' : 'Save Credentials'}
              </button>
            </div>

            <Show when={saveStatus() === 'success'}>
              <div class="alert alert-success shadow-lg">
                <IconCheck />
                <span>Credentials saved successfully!</span>
              </div>
            </Show>

            <Show when={saveStatus() === 'error'}>
              <div class="alert alert-error shadow-lg">
                <IconAlertCircle />
                <span>Failed to save credentials. Please try again.</span>
              </div>
            </Show>
          </div>
        </div>

        {/* Authentication Status */}
        <div class="card bg-base-100 shadow-xl">
          <div class="card-body">
            <h2 class="card-title">Authentication</h2>
            <p class="text-sm opacity-70">
              Connect your YouTube account to access analytics
            </p>

            <Show
              when={youtubeStore.authenticated}
              fallback={
                <div class="space-y-4">
                  <div class="alert">
                    <IconAlertCircle size={16} />
                    <span>
                      You need to authenticate with YouTube to access your channels and analytics.
                    </span>
                  </div>
                  <button onClick={handleAuthenticate} class="btn btn-primary gap-2">
                    <IconBrandYoutube size={16} />
                    Connect YouTube Account
                  </button>
                </div>
              }
            >
              <div class="space-y-4">
                <div class="alert alert-success">
                  <IconCheck size={16} />
                  <span>Connected as: {youtubeStore.userId}</span>
                </div>
                <button onClick={handleRevoke} class="btn btn-error">
                  Disconnect Account
                </button>
              </div>
            </Show>
          </div>
        </div>

        {/* Setup Instructions */}
        <div class="card bg-base-100 shadow-xl">
          <div class="card-body">
            <h2 class="card-title">Setup Instructions</h2>
            <ol class="list-decimal list-inside space-y-2 text-sm">
              <li>Go to the <a href="https://console.cloud.google.com/" target="_blank" class="link link-primary">Google Cloud Console</a></li>
              <li>Create a new project or select an existing one</li>
              <li>Enable the YouTube Data API v3 and YouTube Analytics API</li>
              <li>Create OAuth 2.0 credentials (Web application)</li>
              <li>Add the redirect URI shown above to authorized redirect URIs</li>
              <li>Copy the Client ID and Client Secret to the fields above</li>
              <li>Save the credentials and click "Connect YouTube Account"</li>
            </ol>
          </div>
        </div>

        <Show when={youtubeStore.error}>
          <div class="alert alert-error shadow-lg">
            <IconAlertCircle size={16} />
            <span>{youtubeStore.error}</span>
          </div>
        </Show>
      </div>
    </div>
  );
}
