import { createSignal, onMount, For, Show } from 'solid-js';
import { bridgeFetch } from '@/api/bridge.js';
import { IconMusic, IconPlayerPlay, IconPlayerSkipForward, IconTrash, IconRefresh, IconClearAll } from '@tabler/icons-solidjs';

export default function SongRequestsViewport() {
  const [pendingRequests, setPendingRequests] = createSignal([]);
  const [allRequests, setAllRequests] = createSignal([]);
  const [loading, setLoading] = createSignal(true);
  const [showHistory, setShowHistory] = createSignal(false);

  onMount(async () => {
    await loadRequests();
    setLoading(false);

    // Auto-refresh every 5 seconds
    setInterval(loadRequests, 5000);
  });

  const loadRequests = async () => {
    try {
      const pendingResponse = await bridgeFetch('/song_requests/pending');
      const pendingData = await pendingResponse.json();
      if (pendingData.success) {
        setPendingRequests(pendingData.data || []);
      }

      if (showHistory()) {
        const allResponse = await bridgeFetch('/song_requests/all?limit=50');
        const allData = await allResponse.json();
        if (allData.success) {
          setAllRequests(allData.data || []);
        }
      }
    } catch (e) {
      console.error('Failed to load song requests:', e);
    }
  };

  const updateStatus = async (id, status) => {
    try {
      const response = await bridgeFetch('/song_requests/status', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ id, status }),
      });

      const data = await response.json();
      if (data.success) {
        await loadRequests();
      } else {
        alert(data.error || 'Failed to update status');
      }
    } catch (e) {
      console.error('Failed to update status:', e);
      alert(`Failed to update status: ${e.message}`);
    }
  };

  const deleteRequest = async (id) => {
    if (!confirm('Delete this song request?')) return;

    try {
      const response = await bridgeFetch('/song_requests/:id', {
        method: 'DELETE',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ id }),
      });

      const data = await response.json();
      if (data.success) {
        await loadRequests();
      } else {
        alert(data.error || 'Failed to delete request');
      }
    } catch (e) {
      console.error('Failed to delete request:', e);
      alert(`Failed to delete request: ${e.message}`);
    }
  };

  const clearCompleted = async () => {
    if (!confirm('Clear all completed song requests?')) return;

    try {
      const response = await bridgeFetch('/song_requests/clear', {
        method: 'DELETE',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ status: 'completed' }),
      });

      const data = await response.json();
      if (data.success) {
        await loadRequests();
        alert('Cleared completed requests');
      } else {
        alert(data.error || 'Failed to clear requests');
      }
    } catch (e) {
      console.error('Failed to clear requests:', e);
      alert(`Failed to clear requests: ${e.message}`);
    }
  };

  const formatDate = (timestamp) => {
    const date = new Date(timestamp * 1000);
    return date.toLocaleString();
  };

  const getStatusBadge = (status) => {
    switch (status) {
      case 'pending':
        return <span class="px-2 py-1 bg-yellow-600/30 text-yellow-400 rounded text-xs">Pending</span>;
      case 'playing':
        return <span class="px-2 py-1 bg-green-600/30 text-green-400 rounded text-xs">Playing</span>;
      case 'completed':
        return <span class="px-2 py-1 bg-blue-600/30 text-blue-400 rounded text-xs">Completed</span>;
      case 'skipped':
        return <span class="px-2 py-1 bg-gray-600/30 text-gray-400 rounded text-xs">Skipped</span>;
      case 'error':
        return <span class="px-2 py-1 bg-red-600/30 text-red-400 rounded text-xs">Error</span>;
      default:
        return <span class="px-2 py-1 bg-gray-600/30 text-gray-400 rounded text-xs">{status}</span>;
    }
  };

  return (
    <div class="p-6 max-w-6xl mx-auto">
      <div class="flex items-center justify-between mb-6">
        <div class="flex items-center gap-3">
          <IconMusic size={32} class="text-purple-500" />
          <h1 class="text-2xl font-bold">Song Requests Queue</h1>
        </div>
        <div class="flex gap-2">
          <button
            onClick={loadRequests}
            class="px-4 py-2 bg-gray-700 hover:bg-gray-600 rounded-lg flex items-center gap-2 transition-colors"
          >
            <IconRefresh size={20} />
            Refresh
          </button>
          <button
            onClick={() => setShowHistory(!showHistory())}
            class="px-4 py-2 bg-gray-700 hover:bg-gray-600 rounded-lg transition-colors"
          >
            {showHistory() ? 'Hide' : 'Show'} History
          </button>
          <button
            onClick={clearCompleted}
            class="px-4 py-2 bg-red-700 hover:bg-red-600 rounded-lg flex items-center gap-2 transition-colors"
          >
            <IconClearAll size={20} />
            Clear Completed
          </button>
        </div>
      </div>

      <Show when={loading()}>
        <div class="text-center py-8">
          <div class="animate-spin w-8 h-8 border-4 border-purple-500 border-t-transparent rounded-full mx-auto"></div>
          <p class="mt-4 text-gray-400">Loading song requests...</p>
        </div>
      </Show>

      <Show when={!loading()}>
        {/* Pending Queue */}
        <div class="bg-gray-800 rounded-lg p-6 mb-6">
          <h2 class="text-lg font-semibold mb-4">
            Pending Queue ({pendingRequests().length})
          </h2>

          <Show
            when={pendingRequests().length > 0}
            fallback={
              <div class="text-center py-8 text-gray-400">
                <IconMusic size={48} class="mx-auto mb-2 opacity-50" />
                <p>No pending song requests</p>
              </div>
            }
          >
            <div class="space-y-2">
              <For each={pendingRequests()}>
                {(request, index) => (
                  <div class="bg-gray-700/50 rounded-lg p-4 flex items-center justify-between hover:bg-gray-700 transition-colors">
                    <div class="flex-1">
                      <div class="flex items-center gap-3 mb-1">
                        <span class="text-gray-400 font-mono text-sm">#{index() + 1}</span>
                        <span class="font-semibold text-lg">{request.song_query}</span>
                        {getStatusBadge(request.status)}
                      </div>
                      <div class="text-sm text-gray-400">
                        Requested by <span class="text-purple-400">{request.requester_name}</span>
                        {' • '}
                        {formatDate(request.requested_at)}
                      </div>
                    </div>

                    <div class="flex gap-2">
                      <button
                        onClick={() => updateStatus(request.id, 'playing')}
                        class="px-3 py-2 bg-green-600 hover:bg-green-500 rounded-lg flex items-center gap-1 transition-colors"
                        title="Mark as Playing"
                      >
                        <IconPlayerPlay size={18} />
                        Play
                      </button>
                      <button
                        onClick={() => updateStatus(request.id, 'skipped')}
                        class="px-3 py-2 bg-yellow-600 hover:bg-yellow-500 rounded-lg flex items-center gap-1 transition-colors"
                        title="Skip"
                      >
                        <IconPlayerSkipForward size={18} />
                        Skip
                      </button>
                      <button
                        onClick={() => deleteRequest(request.id)}
                        class="px-3 py-2 bg-red-600 hover:bg-red-500 rounded-lg transition-colors"
                        title="Delete"
                      >
                        <IconTrash size={18} />
                      </button>
                    </div>
                  </div>
                )}
              </For>
            </div>
          </Show>
        </div>

        {/* History */}
        <Show when={showHistory()}>
          <div class="bg-gray-800 rounded-lg p-6">
            <h2 class="text-lg font-semibold mb-4">
              Request History (Last 50)
            </h2>

            <Show
              when={allRequests().length > 0}
              fallback={
                <div class="text-center py-8 text-gray-400">
                  No request history
                </div>
              }
            >
              <div class="space-y-2">
                <For each={allRequests()}>
                  {(request) => (
                    <div class="bg-gray-700/30 rounded-lg p-3 flex items-center justify-between">
                      <div class="flex-1">
                        <div class="flex items-center gap-3 mb-1">
                          <span class="font-medium">{request.song_query}</span>
                          {getStatusBadge(request.status)}
                        </div>
                        <div class="text-sm text-gray-400">
                          by {request.requester_name} • {formatDate(request.requested_at)}
                          <Show when={request.played_at}>
                            {' • '}Played {formatDate(request.played_at)}
                          </Show>
                        </div>
                      </div>

                      <button
                        onClick={() => deleteRequest(request.id)}
                        class="px-2 py-1 text-red-400 hover:text-red-300 transition-colors"
                        title="Delete"
                      >
                        <IconTrash size={16} />
                      </button>
                    </div>
                  )}
                </For>
              </div>
            </Show>
          </div>
        </Show>

        {/* Instructions */}
        <div class="bg-blue-900/20 border border-blue-700 rounded-lg p-6 mt-6">
          <h3 class="font-semibold mb-2">How It Works</h3>
          <ul class="list-disc list-inside space-y-1 text-sm text-gray-300">
            <li>Users send song requests via Discord using the configured command prefix</li>
            <li>Requests appear here in the pending queue</li>
            <li>Click "Play" when you start playing a song in Pear Desktop (YouTube Music)</li>
            <li>Use "Skip" to remove a song from queue without playing it</li>
            <li>The queue auto-refreshes every 5 seconds</li>
          </ul>
        </div>
      </Show>
    </div>
  );
}
