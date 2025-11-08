import { createSignal, createEffect, onCleanup, Show, For } from 'solid-js';
import { IconCalendar, IconClock, IconPlus, IconEdit, IconTrash, IconRefresh, IconAlertCircle } from '@tabler/icons-solidjs';

export default function TwitchScheduleViewport() {
  const [schedule, setSchedule] = createSignal(null);
  const [loading, setLoading] = createSignal(true);
  const [error, setError] = createSignal(null);
  const [syncing, setSyncing] = createSignal(false);
  const [showCreateModal, setShowCreateModal] = createSignal(false);
  const [editingSegment, setEditingSegment] = createSignal(null);

  // Fetch schedule data
  const fetchSchedule = async () => {
    try {
      setError(null);
      const response = await fetch('/twitch/schedule');
      if (response.ok) {
        const data = await response.json();
        setSchedule(data);
      } else {
        const errorText = await response.text();
        setError(`Failed to fetch schedule: ${errorText}`);
      }
    } catch (err) {
      setError(`Error fetching schedule: ${err.message}`);
    } finally {
      setLoading(false);
    }
  };

  // Sync schedule from Twitch
  const handleSync = async () => {
    try {
      setSyncing(true);
      setError(null);
      const response = await fetch('/twitch/schedule/sync', {
        method: 'POST',
      });
      if (response.ok) {
        await fetchSchedule();
      } else {
        const errorText = await response.text();
        setError(`Failed to sync schedule: ${errorText}`);
      }
    } catch (err) {
      setError(`Error syncing schedule: ${err.message}`);
    } finally {
      setSyncing(false);
    }
  };

  // Delete segment
  const handleDelete = async (segmentId) => {
    if (!confirm('Are you sure you want to delete this schedule segment?')) {
      return;
    }

    try {
      const response = await fetch(`/twitch/schedule/segment?id=${segmentId}`, {
        method: 'DELETE',
      });
      if (response.ok) {
        await fetchSchedule();
      } else {
        const errorText = await response.text();
        setError(`Failed to delete segment: ${errorText}`);
      }
    } catch (err) {
      setError(`Error deleting segment: ${err.message}`);
    }
  };

  // Format date/time
  const formatDateTime = (dateString) => {
    const date = new Date(dateString);
    return date.toLocaleString('en-US', {
      weekday: 'short',
      month: 'short',
      day: 'numeric',
      hour: 'numeric',
      minute: '2-digit',
    });
  };

  // Format duration
  const formatDuration = (start, end) => {
    const startDate = new Date(start);
    const endDate = new Date(end);
    const durationMs = endDate - startDate;
    const hours = Math.floor(durationMs / (1000 * 60 * 60));
    const minutes = Math.floor((durationMs % (1000 * 60 * 60)) / (1000 * 60));
    if (hours > 0) {
      return `${hours}h ${minutes}m`;
    }
    return `${minutes}m`;
  };

  // Check if segment is in progress
  const isInProgress = (start, end) => {
    const now = new Date();
    return new Date(start) <= now && now <= new Date(end);
  };

  // Check if segment is upcoming
  const isUpcoming = (start) => {
    return new Date(start) > new Date();
  };

  // Initialize
  createEffect(() => {
    fetchSchedule();
    const interval = setInterval(fetchSchedule, 60000); // Refresh every minute

    onCleanup(() => clearInterval(interval));
  });

  return (
    <div class="h-full overflow-y-auto bg-gradient-to-br from-base-300 to-base-200">
      <div class="max-w-7xl mx-auto p-8">
        {/* Header */}
        <div class="flex items-center justify-between mb-6">
          <div class="flex items-center gap-4">
            <div class="p-3 bg-blue-500/20 rounded-lg">
              <IconCalendar size={40} class="text-blue-500" />
            </div>
            <div>
              <h1 class="text-3xl font-bold">Stream Schedule</h1>
              <p class="text-base-content/60">Manage your upcoming streams</p>
            </div>
          </div>

          <div class="flex gap-2">
            <button
              class={`btn btn-primary gap-2 ${syncing() ? 'loading' : ''}`}
              onClick={handleSync}
              disabled={syncing()}
            >
              {!syncing() && <IconRefresh size={20} />}
              {syncing() ? 'Syncing...' : 'Sync from Twitch'}
            </button>
          </div>
        </div>

        {/* Error Alert */}
        <Show when={error()}>
          <div class="alert alert-error mb-6">
            <IconAlertCircle size={24} />
            <span>{error()}</span>
          </div>
        </Show>

        {/* Loading State */}
        <Show when={loading()}>
          <div class="flex items-center justify-center h-64">
            <div class="text-center">
              <div class="loading loading-spinner loading-lg mb-4"></div>
              <p class="text-base-content/70">Loading schedule...</p>
            </div>
          </div>
        </Show>

        {/* Schedule Content */}
        <Show when={!loading() && schedule()}>
          <Show
            when={schedule()?.data?.segments?.length > 0}
            fallback={
              <div class="card bg-base-100 shadow-xl">
                <div class="card-body text-center py-16">
                  <IconCalendar size={64} class="mx-auto text-base-content/30 mb-4" />
                  <h2 class="text-2xl font-bold mb-2">No Scheduled Streams</h2>
                  <p class="text-base-content/60 mb-6">
                    You haven't scheduled any streams yet. Create your first schedule on Twitch and sync here.
                  </p>
                  <button
                    class="btn btn-primary btn-wide mx-auto"
                    onClick={handleSync}
                  >
                    Sync from Twitch
                  </button>
                </div>
              </div>
            }
          >
            {/* Broadcaster Info */}
            <div class="card bg-base-100 shadow-xl mb-6">
              <div class="card-body">
                <div class="flex items-center gap-3">
                  <div class="avatar placeholder">
                    <div class="bg-primary text-primary-content rounded-full w-12">
                      <span class="text-xl">
                        {schedule()?.data?.broadcaster_name?.charAt(0).toUpperCase()}
                      </span>
                    </div>
                  </div>
                  <div>
                    <h2 class="text-xl font-bold">{schedule()?.data?.broadcaster_name}</h2>
                    <p class="text-sm text-base-content/60">
                      {schedule()?.data?.segments?.length} scheduled segment(s)
                    </p>
                  </div>
                </div>

                <Show when={schedule()?.data?.vacation}>
                  <div class="alert alert-warning mt-4">
                    <svg xmlns="http://www.w3.org/2000/svg" class="stroke-current shrink-0 h-6 w-6" fill="none" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
                    </svg>
                    <span>
                      Vacation Mode: {formatDateTime(schedule().data.vacation.start_time)} - {formatDateTime(schedule().data.vacation.end_time)}
                    </span>
                  </div>
                </Show>
              </div>
            </div>

            {/* Schedule Timeline */}
            <div class="space-y-4">
              <For each={schedule()?.data?.segments}>
                {(segment) => {
                  const inProgress = isInProgress(segment.start_time, segment.end_time);
                  const upcoming = isUpcoming(segment.start_time);
                  const canceled = segment.canceled_until !== null && segment.canceled_until !== undefined;

                  return (
                    <div
                      class={`card bg-base-100 shadow-xl border-2 ${
                        inProgress ? 'border-success' : upcoming ? 'border-primary' : 'border-base-300'
                      } ${canceled ? 'opacity-50' : ''}`}
                    >
                      <div class="card-body">
                        <div class="flex items-start justify-between gap-4">
                          <div class="flex-1">
                            <div class="flex items-center gap-3 mb-2">
                              <Show when={inProgress}>
                                <div class="badge badge-success gap-2">
                                  <span class="relative flex h-2 w-2">
                                    <span class="animate-ping absolute inline-flex h-full w-full rounded-full bg-success-content opacity-75"></span>
                                    <span class="relative inline-flex rounded-full h-2 w-2 bg-success-content"></span>
                                  </span>
                                  Live Now
                                </div>
                              </Show>
                              <Show when={upcoming && !inProgress}>
                                <div class="badge badge-primary">Upcoming</div>
                              </Show>
                              <Show when={canceled}>
                                <div class="badge badge-error">Canceled</div>
                              </Show>
                              <Show when={segment.is_recurring}>
                                <div class="badge badge-info">Recurring</div>
                              </Show>
                            </div>

                            <h3 class="text-2xl font-bold mb-2">{segment.title}</h3>

                            <Show when={segment.category}>
                              <div class="badge badge-lg badge-primary mb-3">
                                {segment.category.name}
                              </div>
                            </Show>

                            <div class="grid grid-cols-1 md:grid-cols-3 gap-4 mt-4">
                              <div class="flex items-start gap-2">
                                <IconClock size={20} class="text-base-content/60 mt-0.5" />
                                <div>
                                  <p class="text-xs text-base-content/60">Start Time</p>
                                  <p class="font-semibold">{formatDateTime(segment.start_time)}</p>
                                </div>
                              </div>
                              <div class="flex items-start gap-2">
                                <IconClock size={20} class="text-base-content/60 mt-0.5" />
                                <div>
                                  <p class="text-xs text-base-content/60">End Time</p>
                                  <p class="font-semibold">{formatDateTime(segment.end_time)}</p>
                                </div>
                              </div>
                              <div class="flex items-start gap-2">
                                <IconClock size={20} class="text-base-content/60 mt-0.5" />
                                <div>
                                  <p class="text-xs text-base-content/60">Duration</p>
                                  <p class="font-semibold">
                                    {formatDuration(segment.start_time, segment.end_time)}
                                  </p>
                                </div>
                              </div>
                            </div>
                          </div>

                          <div class="flex gap-2">
                            <button
                              class="btn btn-square btn-error btn-sm"
                              onClick={() => handleDelete(segment.id)}
                              title="Delete segment"
                            >
                              <IconTrash size={18} />
                            </button>
                          </div>
                        </div>
                      </div>
                    </div>
                  );
                }}
              </For>
            </div>
          </Show>
        </Show>
      </div>
    </div>
  );
}
