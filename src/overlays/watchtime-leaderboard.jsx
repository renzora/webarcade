import { render } from 'solid-js/web';
import { createSignal, onMount, onCleanup, For, Show } from 'solid-js';
import '@/index.css';
import { WEBARCADE_WS, BRIDGE_API } from '@/api/bridge';

// Users to exclude from the leaderboard
const EXCLUDED_USERS = ['streamelements', 'webarcade'];

export default function WatchtimeLeaderboard() {
  const [watchers, setWatchers] = createSignal([]);
  const [isConnected, setIsConnected] = createSignal(false);
  const [channel, setChannel] = createSignal('');
  const [limit, setLimit] = createSignal(3); // Default to top 3

  let ws;
  let refreshInterval;

  // Format minutes to readable format (e.g., "12h 34m")
  const formatWatchtime = (minutes) => {
    const hours = Math.floor(minutes / 60);
    const mins = minutes % 60;

    if (hours === 0) {
      return `${mins}m`;
    }
    return `${hours}h ${mins}m`;
  };

  // Fetch watchtime data from the API
  const fetchWatchtime = async () => {
    if (!channel()) return;

    try {
      const response = await fetch(
        `${BRIDGE_API}/database/watchtime/all?channel=${channel()}&limit=50&offset=0`
      );

      if (!response.ok) {
        console.error('Failed to fetch watchtime data');
        return;
      }

      const data = await response.json();

      if (data.watchers) {
        // Filter out excluded users (case-insensitive)
        const filtered = data.watchers.filter(watcher => {
          const username = watcher.username.toLowerCase();
          const channelName = channel().toLowerCase();

          // Exclude the streamer and other specified users
          return username !== channelName &&
                 !EXCLUDED_USERS.includes(username);
        });

        // Take only the top N watchers
        setWatchers(filtered.slice(0, limit()));
      }
    } catch (error) {
      console.error('Error fetching watchtime:', error);
    }
  };

  // Connect to WebSocket
  const connectWebSocket = () => {
    ws = new WebSocket(WEBARCADE_WS);

    ws.onopen = () => {
      console.log('Connected to WebArcade WebSocket');
      setIsConnected(true);
    };

    ws.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data);

        // Listen for connected event to get channel info
        if (data.type === 'connected' && data.channel) {
          setChannel(data.channel);
          fetchWatchtime();
        }

        // Refresh on chat messages (optional - for more real-time updates)
        if (data.type === 'twitch_event' && data.event?.type === 'chat_message') {
          // You could update more frequently on chat activity if desired
        }
      } catch (error) {
        console.error('Error parsing WebSocket message:', error);
      }
    };

    ws.onerror = (error) => {
      console.error('WebSocket error:', error);
      setIsConnected(false);
    };

    ws.onclose = () => {
      console.log('WebSocket disconnected');
      setIsConnected(false);

      // Reconnect after 5 seconds
      setTimeout(connectWebSocket, 5000);
    };
  };

  onMount(() => {
    connectWebSocket();

    // Refresh watchtime data every 30 seconds
    refreshInterval = setInterval(() => {
      fetchWatchtime();
    }, 30000);
  });

  onCleanup(() => {
    if (ws) {
      ws.close();
    }
    if (refreshInterval) {
      clearInterval(refreshInterval);
    }
  });

  // Get medal emoji for top 3
  const getMedal = (index) => {
    switch (index) {
      case 0: return '🥇';
      case 1: return '🥈';
      case 2: return '🥉';
      default: return `${index + 1}.`;
    }
  };

  function App() {
    return (
    <>
    <style>{`
      @keyframes sparkle {
        0%, 100% {
          filter: brightness(1) drop-shadow(0 0 2px rgba(255, 215, 0, 0.5));
        }
        50% {
          filter: brightness(1.3) drop-shadow(0 0 8px rgba(255, 215, 0, 0.8));
        }
      }
      .sparkle-gold {
        animation: sparkle 2s ease-in-out infinite;
      }
    `}</style>
    <div class="min-h-screen font-sans p-4">
      <div class="w-full">
        {/* Leaderboard */}
        <div>
          <Show when={watchers().length > 0}>
            <div class="space-y-3">
              <For each={watchers()}>
                {(watcher, index) => (
                  <div
                    class="group relative bg-gradient-to-r from-purple-800 to-indigo-800 hover:from-purple-700 hover:to-indigo-700 rounded-xl p-4 border border-purple-500 hover:border-purple-400 transition-all duration-300 transform hover:scale-[1.02]"
                    classList={{
                      'ring-2 ring-yellow-400': index() === 0,
                      'ring-2 ring-gray-300': index() === 1,
                      'ring-2 ring-amber-600': index() === 2,
                    }}
                  >
                    <div class="flex items-center justify-between">
                      {/* Rank and Username */}
                      <div class="flex items-center space-x-4 flex-1">
                        <span
                          class="text-2xl font-bold w-12 text-center"
                          classList={{
                            'text-yellow-300 sparkle-gold': index() === 0,
                            'text-gray-300': index() === 1,
                            'text-amber-600': index() === 2,
                            'text-purple-300': index() >= 3,
                          }}
                        >
                          {getMedal(index())}
                        </span>

                        <div class="flex-1">
                          <p
                            class="text-xl font-semibold truncate"
                            classList={{
                              'text-yellow-200': index() === 0,
                              'text-gray-200': index() === 1,
                              'text-amber-400': index() === 2,
                              'text-white': index() >= 3,
                            }}
                          >
                            {watcher.username}
                          </p>
                        </div>
                      </div>

                      {/* Watchtime */}
                      <div class="text-right ml-4">
                        <p
                          class="text-2xl font-bold tracking-wide"
                          classList={{
                            'text-yellow-300': index() === 0,
                            'text-gray-300': index() === 1,
                            'text-amber-500': index() === 2,
                            'text-purple-300': index() >= 3,
                          }}
                        >
                          {formatWatchtime(watcher.total_minutes)}
                        </p>
                        <p class="text-xs text-purple-300 mt-1">
                          {watcher.total_minutes.toLocaleString()} minutes
                        </p>
                      </div>
                    </div>

                    {/* Decorative gradient bar based on watchtime percentage */}
                    <div class="mt-3 h-1.5 bg-purple-900 rounded-full overflow-hidden">
                      <div
                        class="h-full rounded-full transition-all duration-500"
                        classList={{
                          'bg-gradient-to-r from-yellow-400 to-yellow-300': index() === 0,
                          'bg-gradient-to-r from-gray-300 to-gray-200': index() === 1,
                          'bg-gradient-to-r from-amber-600 to-amber-500': index() === 2,
                          'bg-gradient-to-r from-purple-500 to-indigo-500': index() >= 3,
                        }}
                        style={{
                          width: `${(watcher.total_minutes / (watchers()[0]?.total_minutes || 1)) * 100}%`
                        }}
                      />
                    </div>
                  </div>
                )}
              </For>
            </div>
          </Show>
        </div>
      </div>
    </div>
    </>
  );
  }

  return <App />;
}

render(() => <WatchtimeLeaderboard />, document.getElementById('root'));
