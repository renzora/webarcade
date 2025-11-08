import { createSignal, onMount, For, Show } from 'solid-js';
import twitchStore from '../twitch/TwitchStore.jsx';
import { bridgeFetch } from '@/api/bridge.js';
import { IconClock, IconSearch, IconAlertCircle, IconChevronLeft, IconChevronRight } from '@tabler/icons-solidjs';

export default function WatchtimeViewport() {
  const [watchers, setWatchers] = createSignal([]);
  const [loading, setLoading] = createSignal(true);
  const [selectedChannel, setSelectedChannel] = createSignal('');
  const [searchQuery, setSearchQuery] = createSignal('');
  const [status, setStatus] = createSignal({ status: 'disconnected', connected_channels: [] });
  const [orderBy, setOrderBy] = createSignal('watchtime');

  // Pagination state
  const [currentPage, setCurrentPage] = createSignal(1);
  const [totalWatchers, setTotalWatchers] = createSignal(0);
  const ITEMS_PER_PAGE = 50;

  onMount(async () => {
    const currentStatus = await twitchStore.fetchStatus();
    if (currentStatus) {
      setStatus({ ...currentStatus, connected_channels: currentStatus.connected_channels || [] });
      if (currentStatus.connected_channels && currentStatus.connected_channels.length > 0) {
        setSelectedChannel(currentStatus.connected_channels[0]);
        await loadWatchers(currentStatus.connected_channels[0]);
      }
    }
    setLoading(false);
  });

  const loadWatchers = async (channel, page = 1) => {
    if (!channel) return;

    try {
      setLoading(true);
      const offset = (page - 1) * ITEMS_PER_PAGE;
      const response = await bridgeFetch(`/watchtime/all?channel=${channel}&limit=${ITEMS_PER_PAGE}&offset=${offset}&order_by=${orderBy()}`);
      const data = await response.json();
      setWatchers(data.watchers || []);
      setTotalWatchers(data.total || 0);
      setCurrentPage(page);
    } catch (e) {
      console.error('Failed to load watchers:', e);
    } finally {
      setLoading(false);
    }
  };

  const searchWatchers = async (channel, query) => {
    if (!channel || !query.trim()) {
      await loadWatchers(channel);
      return;
    }

    try {
      setLoading(true);
      const response = await bridgeFetch(`/watchtime/search?channel=${channel}&search=${encodeURIComponent(query)}&order_by=${orderBy()}`);
      const data = await response.json();
      setWatchers(data || []);
      setTotalWatchers(data.length || 0);
      setCurrentPage(1);
    } catch (e) {
      console.error('Failed to search watchers:', e);
    } finally {
      setLoading(false);
    }
  };

  const handleChannelChange = async (channel) => {
    setSelectedChannel(channel);
    setSearchQuery('');
    setCurrentPage(1);
    await loadWatchers(channel);
  };

  const handleSearch = async (e) => {
    const query = e.target.value;
    setSearchQuery(query);

    if (query.trim() === '') {
      await loadWatchers(selectedChannel());
    } else {
      await searchWatchers(selectedChannel(), query);
    }
  };

  const handlePageChange = (newPage) => {
    loadWatchers(selectedChannel(), newPage);
  };

  const handleOrderChange = async (e) => {
    setOrderBy(e.target.value);
    setCurrentPage(1);
    if (searchQuery().trim() !== '') {
      await searchWatchers(selectedChannel(), searchQuery());
    } else {
      await loadWatchers(selectedChannel(), 1);
    }
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

  const totalPages = () => Math.ceil(totalWatchers() / ITEMS_PER_PAGE);
  const isSearching = () => searchQuery().trim() !== '';

  return (
    <div class="h-full flex flex-col bg-base-200">
      {/* Header */}
      <div class="flex items-center justify-between bg-base-100 border-b border-base-300 px-4 py-3">
        <div class="flex items-center gap-3 flex-1">
          <IconClock size={20} class="text-primary" />
          <h2 class="text-lg font-semibold">Watchtime</h2>
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

      {/* Search Bar and Sort */}
      <div class="p-4 bg-base-100 border-b border-base-300 space-y-3">
        <div class="relative">
          <IconSearch size={18} class="absolute left-3 top-1/2 -translate-y-1/2 opacity-50" />
          <input
            type="text"
            placeholder="Search by username..."
            class="input input-bordered input-sm w-full pl-10"
            value={searchQuery()}
            onInput={handleSearch}
          />
        </div>
        <div class="flex items-center gap-2">
          <label class="text-sm text-base-content/60 whitespace-nowrap">Sort by:</label>
          <select
            class="select select-bordered select-sm flex-1"
            value={orderBy()}
            onChange={handleOrderChange}
          >
            <option value="watchtime">Total Watchtime</option>
            <option value="last_seen">Last Seen</option>
            <option value="username">Username</option>
          </select>
        </div>
      </div>

      {/* Stats */}
      <div class="px-4 py-2 bg-base-100 border-b border-base-300 text-sm text-base-content/60">
        Showing {watchers().length} of {totalWatchers()} viewers
      </div>

      {/* Watchers List */}
      <div class="flex-1 overflow-y-auto">
        <Show
          when={!loading() && selectedChannel()}
          fallback={
            <div class="flex items-center justify-center h-full">
              <div class="text-center">
                <IconAlertCircle size={48} class="mx-auto mb-4 opacity-30" />
                <p class="text-sm text-base-content/60">
                  {loading() ? 'Loading watchers...' : 'Select a channel to view watchtime'}
                </p>
              </div>
            </div>
          }
        >
          <Show
            when={watchers().length > 0}
            fallback={
              <div class="flex items-center justify-center h-full">
                <div class="text-center">
                  <IconClock size={48} class="mx-auto mb-4 opacity-30" />
                  <p class="text-sm font-semibold mb-2">No watchtime data</p>
                  <p class="text-xs text-base-content/60">
                    {isSearching() ? 'No results found for your search' : 'Users will appear here as they watch and chat'}
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
                    <th>Last Seen</th>
                  </tr>
                </thead>
                <tbody>
                  <For each={watchers()}>
                    {(watcher, index) => (
                      <tr>
                        <td class="text-base-content/60">
                          {!isSearching() ? (currentPage() - 1) * ITEMS_PER_PAGE + index() + 1 : index() + 1}
                        </td>
                        <td class="font-semibold">{watcher.username}</td>
                        <td class="text-primary font-bold">{formatWatchtime(watcher.total_minutes)}</td>
                        <td class="text-base-content/60 text-sm">{formatLastSeen(watcher.last_seen)}</td>
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
      <Show when={!isSearching() && totalPages() > 1}>
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
