import { createSignal, createEffect, onCleanup, Show } from 'solid-js';
import { IconCpu, IconDatabase, IconActivity, IconBrandTwitch, IconPlayerPlay, IconPlayerStop, IconCheck, IconX, IconCalendar, IconClock } from '@tabler/icons-solidjs';
import twitchStore from '../twitch/TwitchStore.jsx';

export default function DashboardViewport() {
  const [systemStats, setSystemStats] = createSignal(null);
  const [statsHistory, setStatsHistory] = createSignal([]);
  const [twitchConfig, setTwitchConfig] = createSignal(null);
  const [twitchStatus, setTwitchStatus] = createSignal({ status: 'disconnected' });
  const [currentSchedule, setCurrentSchedule] = createSignal(null);
  const [startingBot, setStartingBot] = createSignal(false);
  const [stoppingBot, setStoppingBot] = createSignal(false);
  const [autoStartAttempted, setAutoStartAttempted] = createSignal(false);
  const MAX_HISTORY = 60; // Keep last 60 data points

  // Fetch Twitch config and status
  createEffect(() => {
    const fetchTwitchData = async () => {
      try {
        const config = await twitchStore.fetchConfig();
        setTwitchConfig(config);

        const status = await twitchStore.fetchStatus();
        setTwitchStatus(status);
      } catch (error) {
        // Silent fail - don't spam console
      }
    };

    fetchTwitchData();
    const interval = setInterval(fetchTwitchData, 2000); // Update every 2 seconds

    onCleanup(() => clearInterval(interval));
  });

  // Fetch system stats from bridge server
  createEffect(() => {
    const fetchSystemStats = async () => {
      try {
        const response = await fetch('/system/stats');
        if (response.ok) {
          const stats = await response.json();
          setSystemStats(stats);

          // Add to history
          setStatsHistory(prev => {
            const newHistory = [...prev, {
              timestamp: Date.now(),
              cpu: Math.round(stats.cpu_usage),
              memory: Math.round(stats.memory_usage),
              gpu: stats.gpu_usage ? Math.round(stats.gpu_usage) : null
            }];
            // Keep only last MAX_HISTORY points
            return newHistory.slice(-MAX_HISTORY);
          });
        }
      } catch (error) {
        // Silent fail - don't spam console
      }
    };

    fetchSystemStats();
    const interval = setInterval(fetchSystemStats, 1000); // Update every second

    onCleanup(() => clearInterval(interval));
  });

  // Auto-start bot if configured and not connected
  createEffect(() => {
    const config = twitchConfig();
    const status = twitchStatus();

    if (!autoStartAttempted() && config?.has_token && status.status !== 'connected') {
      setAutoStartAttempted(true);
      console.log('[Dashboard] Auto-starting Twitch bot...');
      handleStartBot();
    }
  });

  // Fetch current/next scheduled stream
  createEffect(() => {
    const fetchScheduleData = async () => {
      try {
        const response = await fetch('/twitch/schedule');
        if (response.ok) {
          const data = await response.json();
          // Find current or next segment from database
          if (data?.data?.segments?.length > 0) {
            const now = new Date();
            // Find current segment
            let current = data.data.segments.find(seg => {
              const start = new Date(seg.start_time);
              const end = new Date(seg.end_time);
              return start <= now && end >= now && !seg.canceled_until;
            });

            // If no current, find next upcoming
            if (!current) {
              current = data.data.segments.find(seg => {
                const start = new Date(seg.start_time);
                return start > now && !seg.canceled_until;
              });
            }

            if (current) {
              const start = new Date(current.start_time);
              const end = new Date(current.end_time);
              setCurrentSchedule({
                ...current,
                is_current: start <= now && end >= now,
                formatted_start: start.toLocaleString(),
                formatted_end: end.toLocaleString(),
              });
            }
          }
        }
      } catch (error) {
        // Endpoint not yet implemented - silent fail
      }
    };

    // Skip schedule fetching for now - endpoint not implemented
    // fetchScheduleData();
    // const interval = setInterval(fetchScheduleData, 30000);
    // onCleanup(() => clearInterval(interval));
  });

  const handleStartBot = async () => {
    try {
      setStartingBot(true);
      await twitchStore.startBot();
      const status = await twitchStore.fetchStatus();
      setTwitchStatus(status);
    } catch (error) {
      console.error('Failed to start bot:', error);
      alert(`Failed to start bot: ${error.message}`);
    } finally {
      setStartingBot(false);
    }
  };

  const handleStopBot = async () => {
    try {
      setStoppingBot(true);
      await twitchStore.stopBot();
      const status = await twitchStore.fetchStatus();
      setTwitchStatus(status);
    } catch (error) {
      console.error('Failed to stop bot:', error);
      alert(`Failed to stop bot: ${error.message}`);
    } finally {
      setStoppingBot(false);
    }
  };

  const getUsageColor = (usage) => {
    if (usage >= 85) return 'text-error';
    if (usage >= 70) return 'text-warning';
    return 'text-success';
  };

  const getProgressColor = (usage) => {
    if (usage >= 85) return 'progress-error';
    if (usage >= 70) return 'progress-warning';
    return 'progress-success';
  };

  const renderMiniChart = (data, key) => {
    if (!data || data.length === 0) return null;

    const height = 60;
    const width = 200;
    const points = data.map((point, index) => {
      const x = (index / (MAX_HISTORY - 1)) * width;
      const y = height - (point[key] / 100) * height;
      return `${x},${y}`;
    }).join(' ');

    return (
      <svg width={width} height={height} class="mt-2">
        <polyline
          points={points}
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          opacity="0.7"
        />
      </svg>
    );
  };

  return (
    <div class="h-full overflow-y-auto bg-gradient-to-br from-base-300 to-base-200">
      <div class="max-w-6xl mx-auto p-8 space-y-6">
        {/* Twitch Bot Control */}
        <Show when={twitchConfig()?.has_token}>
          <div class="card bg-base-100 shadow-xl border-2 border-purple-500/20">
            <div class="card-body">
              <div class="flex items-center justify-between mb-4">
                <div class="flex items-center gap-3">
                  <div class="p-3 bg-purple-500/20 rounded-lg">
                    <IconBrandTwitch size={32} class="text-purple-500" />
                  </div>
                  <div>
                    <h2 class="card-title text-2xl">Twitch Bot</h2>
                    <p class="text-sm text-base-content/60">
                      {twitchConfig()?.bot_username || 'Not configured'}
                    </p>
                  </div>
                </div>
                <div class={`badge badge-lg gap-2 ${twitchStatus().status === 'connected' ? 'badge-success' : 'badge-error'}`}>
                  {twitchStatus().status === 'connected' ? <IconCheck size={16} /> : <IconX size={16} />}
                  {twitchStatus().status === 'connected' ? 'Connected' : 'Disconnected'}
                </div>
              </div>

              <div class="divider my-2"></div>

              <div class="grid grid-cols-1 md:grid-cols-2 gap-4 mb-4">
                <div>
                  <p class="text-xs text-base-content/60 mb-1">Status</p>
                  <p class="font-semibold capitalize">{twitchStatus().status || 'Unknown'}</p>
                </div>
                <div>
                  <p class="text-xs text-base-content/60 mb-1">Channels</p>
                  <p class="font-semibold">
                    {twitchConfig()?.channels?.length || 0} connected
                  </p>
                </div>
                <Show when={twitchStatus().connected_channels?.length > 0}>
                  <div class="md:col-span-2">
                    <p class="text-xs text-base-content/60 mb-1">Connected To</p>
                    <div class="flex flex-wrap gap-2 mt-2">
                      {twitchStatus().connected_channels?.map(channel => (
                        <div class="badge badge-primary gap-1">
                          # {channel}
                        </div>
                      ))}
                    </div>
                  </div>
                </Show>
              </div>

              <div class="flex gap-3">
                <Show
                  when={twitchStatus().status === 'connected'}
                  fallback={
                    <button
                      class={`btn btn-success flex-1 gap-2 ${startingBot() ? 'loading' : ''}`}
                      onClick={handleStartBot}
                      disabled={startingBot()}
                    >
                      {!startingBot() && <IconPlayerPlay size={20} />}
                      {startingBot() ? 'Starting...' : 'Start Bot'}
                    </button>
                  }
                >
                  <button
                    class={`btn btn-error flex-1 gap-2 ${stoppingBot() ? 'loading' : ''}`}
                    onClick={handleStopBot}
                    disabled={stoppingBot()}
                  >
                    {!stoppingBot() && <IconPlayerStop size={20} />}
                    {stoppingBot() ? 'Stopping...' : 'Stop Bot'}
                  </button>
                </Show>
              </div>
            </div>
          </div>
        </Show>

        {/* Schedule Card */}
        <Show when={currentSchedule()}>
          <div class="card bg-base-100 shadow-xl border-2 border-blue-500/20">
            <div class="card-body">
              <div class="flex items-center gap-3 mb-4">
                <div class="p-3 bg-blue-500/20 rounded-lg">
                  <IconCalendar size={32} class="text-blue-500" />
                </div>
                <div class="flex-1">
                  <h2 class="card-title text-2xl">
                    {currentSchedule().is_current ? 'Currently Scheduled' : 'Next Stream'}
                  </h2>
                  <p class="text-sm text-base-content/60">
                    {currentSchedule().is_current ? 'Live now' : 'Upcoming'}
                  </p>
                </div>
                <div class={`badge badge-lg ${currentSchedule().is_current ? 'badge-success' : 'badge-info'}`}>
                  {currentSchedule().is_current ? 'Live' : 'Scheduled'}
                </div>
              </div>

              <div class="divider my-2"></div>

              <div class="space-y-4">
                <div>
                  <h3 class="text-2xl font-bold text-primary mb-2">{currentSchedule().title}</h3>
                  <Show when={currentSchedule().category}>
                    <div class="badge badge-primary">{currentSchedule().category.name}</div>
                  </Show>
                </div>

                <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                  <div class="flex items-start gap-3">
                    <IconClock size={20} class="text-base-content/60 mt-1" />
                    <div>
                      <p class="text-xs text-base-content/60 mb-1">Start Time</p>
                      <p class="font-semibold">{currentSchedule().formatted_start}</p>
                    </div>
                  </div>
                  <div class="flex items-start gap-3">
                    <IconClock size={20} class="text-base-content/60 mt-1" />
                    <div>
                      <p class="text-xs text-base-content/60 mb-1">End Time</p>
                      <p class="font-semibold">{currentSchedule().formatted_end}</p>
                    </div>
                  </div>
                </div>

                <Show when={currentSchedule().is_recurring}>
                  <div class="alert alert-info">
                    <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" class="stroke-current shrink-0 w-6 h-6"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path></svg>
                    <span>This is a recurring stream</span>
                  </div>
                </Show>
              </div>
            </div>
          </div>
        </Show>

        {/* Performance Tips */}
        <div class="card bg-primary text-primary-content shadow-xl">
          <div class="card-body">
            <h2 class="card-title text-2xl">Performance Tips</h2>
            <ul class="list-disc list-inside space-y-2">
              <li>Monitor CPU usage to identify intensive operations</li>
              <li>Keep memory usage below 80% for optimal performance</li>
              <li>High GPU usage is normal when rendering stream overlays</li>
              <li>Close unused viewports to free up resources</li>
            </ul>
          </div>
        </div>
      </div>
    </div>
  );
}
