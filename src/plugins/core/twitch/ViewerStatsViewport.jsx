import { createSignal, onMount, For, Show } from 'solid-js';
import twitchStore from './TwitchStore.jsx';
import { bridgeFetch } from '@/api/bridge.js';
import { IconUsers, IconAlertCircle, IconChevronLeft, IconChevronRight, IconCalendar } from '@tabler/icons-solidjs';

export default function ViewerStatsViewport() {
  const [viewers, setViewers] = createSignal([]);
  const [loading, setLoading] = createSignal(true);
  const [selectedChannel, setSelectedChannel] = createSignal('');
  const [selectedPeriod, setSelectedPeriod] = createSignal('day');
  const [status, setStatus] = createSignal({ status: 'disconnected', connected_channels: [] });

  // Pagination state
  const [currentPage, setCurrentPage] = createSignal(1);
  const [totalViewers, setTotalViewers] = createSignal(0);
  const ITEMS_PER_PAGE = 50;

  const periods = [
    { value: 'day', label: 'Last 24 Hours' },
    { value: 'week', label: 'Last 7 Days' },
    { value: 'month', label: 'Last 30 Days' },
  ];

  onMount(async () => {
    const currentStatus = await twitchStore.fetchStatus();
    if (currentStatus) {
      setStatus(currentStatus);
      if (currentStatus.connected_channels && currentStatus.connected_channels.length > 0) {
        setSelectedChannel(currentStatus.connected_channels[0]);
        await loadViewers(currentStatus.connected_channels[0], 'day');
      }
    }
    setLoading(false);
  });

  const loadViewers = async (channel, period, page = 1) => {
    if (!channel) return;

    try {
      setLoading(true);
      const offset = (page - 1) * ITEMS_PER_PAGE;
      const response = await bridgeFetch(`/database/watchtime/by-period?channel=${channel}&period=${period}&limit=${ITEMS_PER_PAGE}&offset=${offset}`);
      const data = await response.json();
      setViewers(data.viewers || []);
      setTotalViewers(data.total || 0);
      setCurrentPage(page);
    } catch (e) {
      console.error('Failed to load viewers:', e);
    } finally {
      setLoading(false);
    }
  };

  const handleChannelChange = async (channel) => {
    setSelectedChannel(channel);
    setCurrentPage(1);
    await loadViewers(channel, selectedPeriod());
  };

  const handlePeriodChange = async (period) => {
    setSelectedPeriod(period);
    setCurrentPage(1);
    await loadViewers(selectedChannel(), period);
  };

  const handlePageChange = (newPage) => {
    loadViewers(selectedChannel(), selectedPeriod(), newPage);
  };

  const formatWatchtime = (minutes) => {
    const hours = Math.floor(minutes / 60);
    const mins = minutes % 60;
    if (hours > 0) {
      return `${hours}h ${mins}m`;
    }
    return `${mins}m`;
  };

  const formatLastSeen = (timestamp) => {
    const date = new Date(timestamp * 1000);
    const now = new Date();
    const diffMs = now - date;
    const diffMins = Math.floor(diffMs / 60000);
    const diffHours = Math.floor(diffMins / 60);
    const diffDays = Math.floor(diffHours / 24);

    if (diffDays > 0) {
      return `${diffDays}d ago`;
    } else if (diffHours > 0) {
      return `${diffHours}h ago`;
    } else if (diffMins > 0) {
      return `${diffMins}m ago`;
    } else {
      return 'Just now';
    }
  };

  const getPeriodLabel = () => {
    return periods.find(p => p.value === selectedPeriod())?.label || 'Unknown';
  };

  const totalPages = () => Math.ceil(totalViewers() / ITEMS_PER_PAGE);

  return (
    <div class="h-full flex flex-col bg-base-200">
      {/* Header */}
      <div class="flex items-center justify-between bg-base-100 border-b border-base-300 px-4 py-3">
        <div class="flex items-center gap-3 flex-1">
          <IconUsers size={20} class="text-primary" />
          <h2 class="text-lg font-semibold">Viewer Statistics</h2>
        </div>

        <Show when={status().connected_channels.length > 0}>
          <select
            class="select select-bordered select-sm"
            value={selectedChannel()}
            onChange={(e) => handleChannelChange(e.target.value)}
          >
            {status().connected_channels.map((channel) => (
              <option value={channel}>#{channel}</option>
            ))}
          </select>
        </Show>
      </div>

      {/* Period Filter */}
      <div class="p-4 bg-base-100 border-b border-base-300">
        <div class="flex items-center gap-2">
          <IconCalendar size={18} class="opacity-60" />
          <div class="flex gap-2 flex-1">
            <For each={periods}>
              {(period) => (
                <button
                  class={`btn btn-sm ${selectedPeriod() === period.value ? 'btn-primary' : 'btn-outline'}`}
                  onClick={() => handlePeriodChange(period.value)}
                >
                  {period.label}
                </button>
              )}
            </For>
          </div>
        </div>
      </div>

      {/* Stats Summary */}
      <div class="px-4 py-3 bg-base-100 border-b border-base-300">
        <div class="grid grid-cols-2 gap-4">
          <div class="stats shadow-sm">
            <div class="stat py-3 px-4">
              <div class="stat-title text-xs">Active Viewers</div>
              <div class="stat-value text-2xl text-primary">{totalViewers()}</div>
              <div class="stat-desc text-xs">{getPeriodLabel()}</div>
            </div>
          </div>
          <div class="stats shadow-sm">
            <div class="stat py-3 px-4">
              <div class="stat-title text-xs">Showing</div>
              <div class="stat-value text-2xl">{viewers().length}</div>
              <div class="stat-desc text-xs">on page {currentPage()}</div>
            </div>
          </div>
        </div>
      </div>

      {/* Viewers List */}
      <div class="flex-1 overflow-y-auto">
        <Show
          when={!loading() && selectedChannel()}
          fallback={
            <div class="flex items-center justify-center h-full">
              <div class="text-center">
                <IconAlertCircle size={48} class="mx-auto mb-4 opacity-30" />
                <p class="text-sm text-base-content/60">
                  {loading() ? 'Loading viewer statistics...' : 'Select a channel to view statistics'}
                </p>
              </div>
            </div>
          }
        >
          <Show
            when={viewers().length > 0}
            fallback={
              <div class="flex items-center justify-center h-full">
                <div class="text-center">
                  <IconUsers size={48} class="mx-auto mb-4 opacity-30" />
                  <p class="text-sm font-semibold mb-2">No viewers in this period</p>
                  <p class="text-xs text-base-content/60">
                    No one has chatted in {getPeriodLabel().toLowerCase()}
                  </p>
                </div>
              </div>
            }
          >
            <div class="overflow-x-auto">
              <table class="table table-zebra table-sm">
                <thead class="sticky top-0 bg-base-200 z-10">
                  <tr>
                    <th class="w-12">#</th>
                    <th>Username</th>
                    <th>Total Watchtime</th>
                    <th>Last Active</th>
                  </tr>
                </thead>
                <tbody>
                  <For each={viewers()}>
                    {(viewer, index) => (
                      <tr>
                        <td class="text-base-content/60">
                          {(currentPage() - 1) * ITEMS_PER_PAGE + index() + 1}
                        </td>
                        <td class="font-semibold">{viewer.username}</td>
                        <td class="text-primary font-bold">{formatWatchtime(viewer.total_minutes)}</td>
                        <td class="text-base-content/60 text-sm">{formatLastSeen(viewer.last_seen)}</td>
                      </tr>
                    )}
                  </For>
                </tbody>
              </table>
            </div>
          </Show>
        </Show>
      </div>

      {/* Pagination */}
      <Show when={totalPages() > 1}>
        <div class="flex items-center justify-between bg-base-100 border-t border-base-300 px-4 py-3">
          <div class="text-sm text-base-content/60">
            Page {currentPage()} of {totalPages()}
          </div>
          <div class="flex gap-2">
            <button
              class="btn btn-sm btn-outline"
              onClick={() => handlePageChange(currentPage() - 1)}
              disabled={currentPage() === 1}
            >
              <IconChevronLeft size={16} />
              Previous
            </button>
            <button
              class="btn btn-sm btn-outline"
              onClick={() => handlePageChange(currentPage() + 1)}
              disabled={currentPage() === totalPages()}
            >
              Next
              <IconChevronRight size={16} />
            </button>
          </div>
        </div>
      </Show>
    </div>
  );
}
