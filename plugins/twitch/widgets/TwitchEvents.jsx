import { createSignal, createEffect, Show, For } from 'solid-js';
import { bridgeFetch } from '@/api/bridge';
import { IconRefresh, IconBolt } from '@tabler/icons-solidjs';

export default function TwitchEvents() {
  const [events, setEvents] = createSignal([]);
  const [subscriptions, setSubscriptions] = createSignal([]);
  const [loading, setLoading] = createSignal(false);

  const loadData = async () => {
    setLoading(true);

    try {
      const [eventsResponse, subsResponse] = await Promise.all([
        bridgeFetch('/twitch/eventsub/events'),
        bridgeFetch('/twitch/eventsub/subscriptions')
      ]);

      const eventsData = await eventsResponse.json();
      const subsData = await subsResponse.json();

      setEvents(eventsData);
      setSubscriptions(subsData);
    } catch (e) {
      console.error('Failed to load EventSub data:', e);
    } finally {
      setLoading(false);
    }
  };

  createEffect(() => {
    loadData();

    // Auto-refresh every 10 seconds
    const interval = setInterval(loadData, 10000);
    return () => clearInterval(interval);
  });

  const formatTimestamp = (timestamp) => {
    return new Date(timestamp * 1000).toLocaleString();
  };

  return (
    <div class="p-4 space-y-4">
      <div class="flex items-center justify-between">
        <h2 class="text-lg font-bold">EventSub Events</h2>
        <button
          class="btn btn-sm btn-ghost"
          onClick={loadData}
          disabled={loading()}
        >
          <IconRefresh class="w-4 h-4" />
        </button>
      </div>

      <Show
        when={!loading()}
        fallback={
          <div class="flex justify-center p-8">
            <span class="loading loading-spinner loading-lg"></span>
          </div>
        }
      >
        <div class="space-y-4">
          {/* Subscriptions */}
          <div class="card bg-base-100 shadow-md">
            <div class="card-body">
              <h3 class="card-title text-sm">Active Subscriptions</h3>
              <Show
                when={subscriptions().length > 0}
                fallback={
                  <p class="text-sm text-base-content/60">
                    No active subscriptions
                  </p>
                }
              >
                <div class="space-y-2">
                  <For each={subscriptions()}>
                    {(sub) => (
                      <div class="flex items-center justify-between bg-base-200 p-2 rounded">
                        <div>
                          <div class="text-sm font-medium">
                            {sub.subscription_type}
                          </div>
                          <div class="text-xs text-base-content/60">
                            Status: {sub.status} | Cost: {sub.cost}
                          </div>
                        </div>
                      </div>
                    )}
                  </For>
                </div>
              </Show>
            </div>
          </div>

          {/* Recent Events */}
          <div class="card bg-base-100 shadow-md">
            <div class="card-body">
              <h3 class="card-title text-sm">Recent Events</h3>
              <Show
                when={events().length > 0}
                fallback={
                  <p class="text-sm text-base-content/60">
                    No events received yet
                  </p>
                }
              >
                <div class="space-y-2 max-h-96 overflow-y-auto">
                  <For each={events()}>
                    {(event) => (
                      <div class="bg-base-200 p-3 rounded space-y-1">
                        <div class="flex items-center gap-2">
                          <IconBolt class="w-4 h-4 text-primary" />
                          <span class="text-sm font-medium">
                            {event.event_type}
                          </span>
                        </div>
                        <div class="text-xs text-base-content/60">
                          {formatTimestamp(event.timestamp)}
                        </div>
                        <pre class="text-xs bg-base-300 p-2 rounded overflow-x-auto">
                          {JSON.stringify(event.event_data, null, 2)}
                        </pre>
                      </div>
                    )}
                  </For>
                </div>
              </Show>
            </div>
          </div>
        </div>
      </Show>
    </div>
  );
}
