import { Show, For, createSignal, createEffect, onMount } from 'solid-js';
import {
  IconEye,
  IconClock,
  IconUsers,
  IconThumbUp,
  IconMessage,
  IconShare,
  IconTrendingUp,
  IconTrendingDown,
  IconAlertCircle,
  IconRefresh,
  IconBrandYoutube,
  IconSettings,
  IconCheck,
  IconX,
} from '@tabler/icons-solidjs';
import youtubeStore from './YouTubeStore.jsx';

export default function YouTubeViewport() {
  const [showSettings, setShowSettings] = createSignal(false);
  const [dateRange, setDateRange] = createSignal('30days');
  const [startDate, setStartDate] = createSignal('');
  const [endDate, setEndDate] = createSignal('');
  const [analytics, setAnalytics] = createSignal(null);

  // Settings state
  const [clientId, setClientId] = createSignal('');
  const [clientSecret, setClientSecret] = createSignal('');
  const [saveStatus, setSaveStatus] = createSignal(null);
  const [hasCredentials, setHasCredentials] = createSignal(false);

  onMount(async () => {
    // Check if credentials exist
    try {
      const response = await fetch('http://localhost:3001/database/config');
      if (response.ok) {
        const data = await response.json();
        if (data.youtube_client_id) {
          setClientId(data.youtube_client_id);
          setHasCredentials(true);
        }
      }
    } catch (error) {
      console.error('Failed to load config:', error);
    }

    // Check auth status
    await youtubeStore.checkAuthStatus();

    // If authenticated, load channels
    if (youtubeStore.authenticated && youtubeStore.channels.length === 0) {
      await youtubeStore.fetchChannels();
    }
  });

  createEffect(() => {
    const range = dateRange();
    const today = new Date();
    const end = today.toISOString().split('T')[0];

    let start;
    switch (range) {
      case '7days':
        start = new Date(today.setDate(today.getDate() - 7)).toISOString().split('T')[0];
        break;
      case '30days':
        start = new Date(today.setDate(today.getDate() - 30)).toISOString().split('T')[0];
        break;
      case '90days':
        start = new Date(today.setDate(today.getDate() - 90)).toISOString().split('T')[0];
        break;
      case 'custom':
        return;
      default:
        start = new Date(today.setDate(today.getDate() - 30)).toISOString().split('T')[0];
    }

    setStartDate(start);
    setEndDate(end);
  });

  createEffect(() => {
    if (youtubeStore.selectedChannel && startDate() && endDate() && !showSettings()) {
      loadAnalytics();
    }
  });

  const loadAnalytics = async () => {
    if (!youtubeStore.selectedChannel) return;

    const data = await youtubeStore.fetchAnalytics(
      youtubeStore.selectedChannel,
      startDate(),
      endDate()
    );
    setAnalytics(data);
  };

  const saveCredentials = async () => {
    try {
      setSaveStatus('saving');

      const response = await fetch('http://localhost:3001/database/config', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          youtube_client_id: clientId(),
          youtube_client_secret: clientSecret(),
        }),
      });

      if (response.ok) {
        setSaveStatus('success');
        setHasCredentials(true);
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

  const formatNumber = (num) => {
    if (!num) return '0';
    return new Intl.NumberFormat().format(num);
  };

  const formatDuration = (seconds) => {
    if (!seconds) return '0:00';
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    const secs = seconds % 60;

    if (hours > 0) {
      return `${hours}:${minutes.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
    }
    return `${minutes}:${secs.toString().padStart(2, '0')}`;
  };

  const selectedChannel = () => {
    return youtubeStore.channels.find(ch => ch.id === youtubeStore.selectedChannel);
  };

  const needsSetup = () => !hasCredentials() || !youtubeStore.authenticated;

  return (
    <div class="h-full overflow-y-auto bg-base-200">
      <div class="max-w-6xl mx-auto p-6 space-y-6">
        {/* Header */}
        <div class="flex items-center gap-4 p-6 bg-gradient-to-r from-red-600 to-red-700 rounded-xl shadow-lg">
          <div class="p-4 bg-white/20 rounded-lg backdrop-blur-sm">
            <IconBrandYoutube size={40} class="text-white" />
          </div>
          <div class="flex-1">
            <h1 class="text-3xl font-bold text-white">YouTube</h1>
            <p class="text-red-100">
              <Show when={needsSetup()} fallback="View detailed metrics and channel performance">
                Connect your YouTube channel and view analytics
              </Show>
            </p>
          </div>
          <Show when={!needsSetup() && !showSettings()}>
            <button
              onClick={() => setShowSettings(true)}
              class="btn btn-ghost text-white gap-2"
            >
              <IconSettings size={20} />
              Settings
            </button>
          </Show>
          <Show when={showSettings()}>
            <button
              onClick={() => setShowSettings(false)}
              class="btn btn-ghost text-white gap-2"
            >
              <IconX size={20} />
              Close
            </button>
          </Show>
        </div>

        {/* Settings View */}
        <Show when={needsSetup() || showSettings()}>
          <div class="space-y-6">
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
                      <button onClick={handleAuthenticate} class="btn btn-primary gap-2" disabled={!hasCredentials()}>
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

            {/* Setup Instructions (only show if not authenticated) */}
            <Show when={!youtubeStore.authenticated}>
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
            </Show>
          </div>
        </Show>

        {/* Analytics View */}
        <Show when={!needsSetup() && !showSettings()}>
          <div class="space-y-6">
            {/* Channel Selector */}
            <div class="card bg-base-100 shadow-xl">
              <div class="card-body">
                <h2 class="card-title">Channel</h2>
                <select
                  class="select select-bordered w-full"
                  value={youtubeStore.selectedChannel || ''}
                  onChange={(e) => youtubeStore.selectChannel(e.target.value)}
                >
                  <option value="" disabled>Select a channel</option>
                  <For each={youtubeStore.channels}>
                    {(channel) => (
                      <option value={channel.id}>{channel.title}</option>
                    )}
                  </For>
                </select>

                <Show when={selectedChannel()}>
                  <div class="mt-4 flex items-center gap-3">
                    <Show when={selectedChannel().thumbnail_url}>
                      <img
                        src={selectedChannel().thumbnail_url}
                        alt={selectedChannel().title}
                        class="w-16 h-16 rounded-full"
                      />
                    </Show>
                    <div>
                      <h3 class="font-semibold">{selectedChannel().title}</h3>
                      <div class="flex gap-4 text-sm opacity-70 mt-1">
                        <span>{formatNumber(selectedChannel().subscriber_count)} subscribers</span>
                        <span>{formatNumber(selectedChannel().video_count)} videos</span>
                        <span>{formatNumber(selectedChannel().view_count)} total views</span>
                      </div>
                    </div>
                  </div>
                </Show>
              </div>
            </div>

            {/* Date Range Selector */}
            <div class="card bg-base-100 shadow-xl">
              <div class="card-body">
                <div class="flex items-center justify-between">
                  <h2 class="card-title">Date Range</h2>
                  <button
                    onClick={loadAnalytics}
                    disabled={youtubeStore.loading || !youtubeStore.selectedChannel}
                    class={`btn btn-sm btn-primary gap-2 ${youtubeStore.loading ? 'loading' : ''}`}
                  >
                    <Show when={!youtubeStore.loading}>
                      <IconRefresh size={16} />
                    </Show>
                    Refresh
                  </button>
                </div>
                <div class="flex gap-2 flex-wrap">
                  <button
                    class={`btn btn-sm ${dateRange() === '7days' ? 'btn-primary' : 'btn-outline'}`}
                    onClick={() => setDateRange('7days')}
                  >
                    Last 7 Days
                  </button>
                  <button
                    class={`btn btn-sm ${dateRange() === '30days' ? 'btn-primary' : 'btn-outline'}`}
                    onClick={() => setDateRange('30days')}
                  >
                    Last 30 Days
                  </button>
                  <button
                    class={`btn btn-sm ${dateRange() === '90days' ? 'btn-primary' : 'btn-outline'}`}
                    onClick={() => setDateRange('90days')}
                  >
                    Last 90 Days
                  </button>
                  <button
                    class={`btn btn-sm ${dateRange() === 'custom' ? 'btn-primary' : 'btn-outline'}`}
                    onClick={() => setDateRange('custom')}
                  >
                    Custom
                  </button>
                </div>

                <Show when={dateRange() === 'custom'}>
                  <div class="flex gap-4 mt-4">
                    <div class="form-control flex-1">
                      <label class="label">
                        <span class="label-text">Start Date</span>
                      </label>
                      <input
                        type="date"
                        value={startDate()}
                        onInput={(e) => setStartDate(e.target.value)}
                        class="input input-bordered"
                      />
                    </div>
                    <div class="form-control flex-1">
                      <label class="label">
                        <span class="label-text">End Date</span>
                      </label>
                      <input
                        type="date"
                        value={endDate()}
                        onInput={(e) => setEndDate(e.target.value)}
                        class="input input-bordered"
                      />
                    </div>
                  </div>
                </Show>
              </div>
            </div>

            {/* Analytics Dashboard */}
            <Show when={analytics()}>
              <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
                {/* Views */}
                <div class="card bg-base-100 shadow-xl">
                  <div class="card-body">
                    <div class="flex items-center justify-between">
                      <div>
                        <p class="text-sm opacity-70">Views</p>
                        <p class="text-2xl font-bold">{formatNumber(analytics().views)}</p>
                      </div>
                      <IconEye size={32} class="text-blue-500" />
                    </div>
                  </div>
                </div>

                {/* Watch Time */}
                <div class="card bg-base-100 shadow-xl">
                  <div class="card-body">
                    <div class="flex items-center justify-between">
                      <div>
                        <p class="text-sm opacity-70">Watch Time</p>
                        <p class="text-2xl font-bold">
                          {formatNumber(Math.floor((analytics().watch_time || 0) / 60))} min
                        </p>
                      </div>
                      <IconClock size={32} class="text-purple-500" />
                    </div>
                  </div>
                </div>

                {/* Subscriber Change */}
                <div class="card bg-base-100 shadow-xl">
                  <div class="card-body">
                    <div class="flex items-center justify-between">
                      <div>
                        <p class="text-sm opacity-70">Subscribers</p>
                        <p class="text-2xl font-bold flex items-center gap-1">
                          {analytics().subscriber_change > 0 ? '+' : ''}
                          {formatNumber(analytics().subscriber_change)}
                          <Show when={analytics().subscriber_change > 0}>
                            <IconTrendingUp size={20} class="text-green-500" />
                          </Show>
                          <Show when={analytics().subscriber_change < 0}>
                            <IconTrendingDown size={20} class="text-red-500" />
                          </Show>
                        </p>
                      </div>
                      <IconUsers size={32} class="text-green-500" />
                    </div>
                  </div>
                </div>

                {/* Average View Duration */}
                <div class="card bg-base-100 shadow-xl">
                  <div class="card-body">
                    <div class="flex items-center justify-between">
                      <div>
                        <p class="text-sm opacity-70">Avg View Duration</p>
                        <p class="text-2xl font-bold">
                          {formatDuration(analytics().average_view_duration)}
                        </p>
                      </div>
                      <IconClock size={32} class="text-orange-500" />
                    </div>
                  </div>
                </div>

                {/* Likes */}
                <div class="card bg-base-100 shadow-xl">
                  <div class="card-body">
                    <div class="flex items-center justify-between">
                      <div>
                        <p class="text-sm opacity-70">Likes</p>
                        <p class="text-2xl font-bold">{formatNumber(analytics().likes)}</p>
                      </div>
                      <IconThumbUp size={32} class="text-red-500" />
                    </div>
                  </div>
                </div>

                {/* Comments */}
                <div class="card bg-base-100 shadow-xl">
                  <div class="card-body">
                    <div class="flex items-center justify-between">
                      <div>
                        <p class="text-sm opacity-70">Comments</p>
                        <p class="text-2xl font-bold">{formatNumber(analytics().comments)}</p>
                      </div>
                      <IconMessage size={32} class="text-indigo-500" />
                    </div>
                  </div>
                </div>

                {/* Shares */}
                <div class="card bg-base-100 shadow-xl">
                  <div class="card-body">
                    <div class="flex items-center justify-between">
                      <div>
                        <p class="text-sm opacity-70">Shares</p>
                        <p class="text-2xl font-bold">{formatNumber(analytics().shares)}</p>
                      </div>
                      <IconShare size={32} class="text-teal-500" />
                    </div>
                  </div>
                </div>
              </div>
            </Show>
          </div>
        </Show>

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
