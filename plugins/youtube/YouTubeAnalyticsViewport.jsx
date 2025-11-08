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
} from '@tabler/icons-solidjs';
import youtubeStore from './YouTubeStore.jsx';

export default function YouTubeAnalyticsViewport() {
  const [dateRange, setDateRange] = createSignal('30days');
  const [startDate, setStartDate] = createSignal('');
  const [endDate, setEndDate] = createSignal('');
  const [analytics, setAnalytics] = createSignal(null);

  onMount(() => {
    if (!youtubeStore.authenticated) {
      console.log('Not authenticated');
      return;
    }

    if (youtubeStore.channels.length === 0) {
      youtubeStore.fetchChannels();
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
    if (youtubeStore.selectedChannel && startDate() && endDate()) {
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

  return (
    <div class="h-full overflow-y-auto bg-base-200">
      <div class="max-w-6xl mx-auto p-6 space-y-6">
        {/* Header */}
        <div class="flex items-center gap-4 p-6 bg-gradient-to-r from-red-600 to-red-700 rounded-xl shadow-lg">
          <div class="p-4 bg-white/20 rounded-lg backdrop-blur-sm">
            <IconBrandYoutube size={40} class="text-white" />
          </div>
          <div class="flex-1">
            <h1 class="text-3xl font-bold text-white">YouTube Analytics</h1>
            <p class="text-red-100">View detailed metrics and channel performance</p>
          </div>
          <button
            onClick={loadAnalytics}
            disabled={youtubeStore.loading || !youtubeStore.selectedChannel}
            class={`btn btn-ghost text-white gap-2 ${youtubeStore.loading ? 'loading' : ''}`}
          >
            <Show when={!youtubeStore.loading}>
              <IconRefresh size={16} />
            </Show>
            Refresh
          </button>
        </div>

        <Show when={!youtubeStore.authenticated}>
          <div class="alert alert-warning shadow-lg">
            <IconAlertCircle size={16} />
            <span>
              Please connect your YouTube account in the settings to view analytics.
            </span>
          </div>
        </Show>

        <Show when={youtubeStore.authenticated}>
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
              <h2 class="card-title">Date Range</h2>
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

          <Show when={youtubeStore.error}>
            <div class="alert alert-error shadow-lg">
              <IconAlertCircle size={16} />
              <span>{youtubeStore.error}</span>
            </div>
          </Show>
        </Show>
      </div>
    </div>
  );
}
