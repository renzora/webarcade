import { render } from 'solid-js/web';
import { createSignal, createEffect, onCleanup } from 'solid-js';
import '@/index.css';
import { WEBARCADE_WS, BRIDGE_API } from '@/api/bridge';

function ViewerCounterOverlay() {
  const [viewerCount, setViewerCount] = createSignal(0);
  const [isLive, setIsLive] = createSignal(false);

  let ws = null;

  // Fetch viewer count from API
  const fetchViewerCount = async () => {
    try {
      console.log('🔍 Fetching viewer count from API...');
      const response = await fetch(`${BRIDGE_API}/twitch/stream-info`);

      if (!response.ok) {
        console.error('❌ API response not OK:', response.status, response.statusText);
        setViewerCount(0);
        setIsLive(false);
        return;
      }

      const data = await response.json();
      console.log('📊 Stream info received:', data);

      if (data.stream && data.stream.viewer_count !== undefined) {
        console.log('✅ Stream is LIVE with', data.stream.viewer_count, 'viewers');
        setViewerCount(data.stream.viewer_count);
        setIsLive(true);
      } else {
        console.log('⚫ Stream is offline or no viewer data');
        // Stream is offline
        setViewerCount(0);
        setIsLive(false);
      }
    } catch (error) {
      console.error('❌ Failed to fetch viewer count:', error);
      setIsLive(false);
    }
  };

  // Connect to WebSocket for real-time updates
  const connectWebSocket = () => {
    ws = new WebSocket(WEBARCADE_WS);

    ws.onopen = () => {
      console.log('✅ Viewer Counter WebSocket connected');
    };

    ws.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data);

        // Listen for any Twitch events that might indicate changes
        if (data.type === 'twitch_event') {
          // Refresh viewer count when chat activity happens
          fetchViewerCount();
        }
      } catch (err) {
        console.error('Failed to parse WebSocket message:', err);
      }
    };

    ws.onerror = (error) => {
      console.error('❌ Viewer Counter WebSocket error:', error);
    };

    ws.onclose = () => {
      console.log('🔌 Viewer Counter WebSocket disconnected, reconnecting in 3 seconds...');
      setTimeout(connectWebSocket, 3000);
    };
  };

  createEffect(() => {
    // Initial fetch
    fetchViewerCount();
    connectWebSocket();

    // Poll for viewer count every 30 seconds
    const pollInterval = setInterval(fetchViewerCount, 30000);

    onCleanup(() => {
      clearInterval(pollInterval);
      if (ws) {
        ws.close();
      }
    });
  });

  return (
    <div class="fixed inset-0 pointer-events-none overflow-hidden font-sans bg-transparent">
      {/* Viewer Counter - Top Right */}
      <div class="absolute top-4 right-4">
        <div class="bg-gradient-to-r from-purple-900/95 via-indigo-900/95 to-purple-900/95 backdrop-blur-sm px-5 py-3 rounded-lg shadow-2xl border-2 border-purple-500/50 flex items-center gap-3">
          {/* Eye Icon */}
          <svg
            class="w-8 h-8 text-purple-300"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"
            />
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z"
            />
          </svg>

          {/* Viewer Count Display */}
          <div class="flex flex-col">
            <div class="text-xs font-semibold text-purple-300 uppercase tracking-wider">
              {isLive() ? 'Live Viewers' : 'Offline'}
            </div>
            <div class="text-3xl font-bold text-white tracking-tight tabular-nums">
              {viewerCount().toLocaleString()}
            </div>
          </div>

          {/* Live Indicator */}
          {isLive() && (
            <div class="relative ml-1">
              <div class="w-3 h-3 bg-red-500 rounded-full animate-pulse shadow-lg"></div>
              <div class="absolute inset-0 w-3 h-3 bg-red-400 rounded-full animate-ping"></div>
            </div>
          )}
        </div>
      </div>

      <style>{`
        @keyframes pulse {
          0%, 100% {
            opacity: 1;
          }
          50% {
            opacity: 0.5;
          }
        }

        @keyframes ping {
          75%, 100% {
            transform: scale(2);
            opacity: 0;
          }
        }

        .animate-pulse {
          animation: pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite;
        }

        .animate-ping {
          animation: ping 1s cubic-bezier(0, 0, 0.2, 1) infinite;
        }

        .tabular-nums {
          font-variant-numeric: tabular-nums;
        }
      `}</style>
    </div>
  );
}

// Only render when used as standalone (for OBS browser sources)
if (document.getElementById('root')) {
  render(() => <ViewerCounterOverlay />, document.getElementById('root'));
}
