import { createSignal, createEffect, onCleanup, For, Show } from 'solid-js';
import { IconMessage, IconTrash, IconRefresh, IconUser, IconClock } from '@tabler/icons-solidjs';

const ConfessionsViewport = () => {
  const [confessions, setConfessions] = createSignal([]);
  const [loading, setLoading] = createSignal(true);
  const [selectedConfession, setSelectedConfession] = createSignal(null);

  // Fetch confessions from the backend
  const fetchConfessions = async () => {
    try {
      const response = await fetch('http://localhost:3001/confessions');
      if (response.ok) {
        const data = await response.json();
        setConfessions(data);
      }
    } catch (error) {
      console.error('[Confessions] Failed to fetch:', error);
    } finally {
      setLoading(false);
    }
  };

  // Delete a confession
  const deleteConfession = async (id) => {
    if (!confirm('Are you sure you want to delete this confession?')) {
      return;
    }

    try {
      const response = await fetch(`http://localhost:3001/confessions/${id}`, {
        method: 'DELETE'
      });

      if (response.ok) {
        await fetchConfessions();
        setSelectedConfession(null);
      }
    } catch (error) {
      console.error('[Confessions] Failed to delete:', error);
    }
  };

  // Format timestamp
  const formatDate = (timestamp) => {
    const date = new Date(timestamp);
    return new Intl.DateTimeFormat('en-US', {
      month: 'short',
      day: 'numeric',
      year: 'numeric',
      hour: '2-digit',
      minute: '2-digit'
    }).format(date);
  };

  // Auto-refresh every 10 seconds
  createEffect(() => {
    fetchConfessions();
    const interval = setInterval(fetchConfessions, 10000);
    onCleanup(() => clearInterval(interval));
  });

  return (
    <div className="h-full bg-base-200 flex flex-col">
      {/* Header */}
      <div className="flex-shrink-0 bg-base-300 border-b border-base-content/10 p-4">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-3">
            <IconMessage className="w-6 h-6 text-primary" />
            <h2 className="text-lg font-semibold">Anonymous Confessions</h2>
          </div>
          <div className="flex items-center gap-2">
            <span className="text-sm text-base-content/60">
              {confessions().length} {confessions().length === 1 ? 'confession' : 'confessions'}
            </span>
            <button
              onClick={() => fetchConfessions()}
              className="btn btn-sm btn-ghost"
              title="Refresh"
            >
              <IconRefresh className="w-4 h-4" />
            </button>
          </div>
        </div>
      </div>

      {/* Content */}
      <div className="flex-1 overflow-hidden flex">
        {/* Confessions List */}
        <div className="w-1/3 border-r border-base-content/10 overflow-y-auto">
          <Show when={loading()}>
            <div className="flex items-center justify-center h-full">
              <span className="loading loading-spinner loading-lg text-primary"></span>
            </div>
          </Show>

          <Show when={!loading() && confessions().length === 0}>
            <div className="flex flex-col items-center justify-center h-full text-center p-8">
              <IconMessage className="w-16 h-16 text-base-content/20 mb-4" />
              <p className="text-base-content/60">No confessions yet</p>
              <p className="text-sm text-base-content/40 mt-2">
                Users can whisper the bot with !confession [message]
              </p>
            </div>
          </Show>

          <Show when={!loading() && confessions().length > 0}>
            <div className="divide-y divide-base-content/10">
              <For each={confessions()}>
                {(confession) => (
                  <button
                    onClick={() => setSelectedConfession(confession)}
                    className={`w-full text-left p-4 hover:bg-base-300 transition-colors ${
                      selectedConfession()?.id === confession.id ? 'bg-base-300' : ''
                    }`}
                  >
                    <div className="flex items-start gap-3">
                      <IconUser className="w-5 h-5 text-base-content/40 flex-shrink-0 mt-0.5" />
                      <div className="flex-1 min-w-0">
                        <p className="text-sm font-medium text-base-content/80 mb-1">
                          Anonymous
                        </p>
                        <p className="text-sm text-base-content/60 line-clamp-2">
                          {confession.message}
                        </p>
                        <div className="flex items-center gap-2 mt-2 text-xs text-base-content/40">
                          <IconClock className="w-3 h-3" />
                          <span>{formatDate(confession.created_at)}</span>
                        </div>
                      </div>
                    </div>
                  </button>
                )}
              </For>
            </div>
          </Show>
        </div>

        {/* Confession Detail */}
        <div className="flex-1 overflow-y-auto">
          <Show
            when={selectedConfession()}
            fallback={
              <div className="flex items-center justify-center h-full text-center p-8">
                <div>
                  <IconMessage className="w-16 h-16 text-base-content/20 mb-4 mx-auto" />
                  <p className="text-base-content/60">Select a confession to view details</p>
                </div>
              </div>
            }
          >
            <div className="p-6">
              {/* Header */}
              <div className="flex items-start justify-between mb-6">
                <div>
                  <div className="flex items-center gap-2 mb-2">
                    <IconUser className="w-6 h-6 text-base-content/60" />
                    <h3 className="text-lg font-semibold">Anonymous Confession</h3>
                  </div>
                  <div className="flex items-center gap-2 text-sm text-base-content/40">
                    <IconClock className="w-4 h-4" />
                    <span>{formatDate(selectedConfession().created_at)}</span>
                  </div>
                </div>
                <button
                  onClick={() => deleteConfession(selectedConfession().id)}
                  className="btn btn-sm btn-error btn-outline"
                  title="Delete confession"
                >
                  <IconTrash className="w-4 h-4" />
                  Delete
                </button>
              </div>

              {/* Message */}
              <div className="bg-base-300 rounded-lg p-6">
                <p className="text-base leading-relaxed whitespace-pre-wrap">
                  {selectedConfession().message}
                </p>
              </div>

              {/* Metadata */}
              <div className="mt-6 grid grid-cols-2 gap-4">
                <div className="bg-base-300 rounded-lg p-4">
                  <p className="text-xs text-base-content/40 mb-1">Username</p>
                  <p className="text-sm font-medium">{selectedConfession().username}</p>
                </div>
                <div className="bg-base-300 rounded-lg p-4">
                  <p className="text-xs text-base-content/40 mb-1">Confession ID</p>
                  <p className="text-sm font-mono">{selectedConfession().id}</p>
                </div>
              </div>
            </div>
          </Show>
        </div>
      </div>
    </div>
  );
};

export default ConfessionsViewport;
