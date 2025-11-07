import { render } from 'solid-js/web';
import { createSignal, createEffect, onCleanup } from 'solid-js';
import '@/index.css';

const WEBARCADE_API = 'http://localhost:3001';
const WEBSOCKET_URL = 'ws://localhost:3002';

// Mood emoji mappings based on 1-10 scale
const getMoodEmoji = (mood) => {
  if (mood >= 9) return '😄'; // 9-10: Great
  if (mood >= 7) return '🙂'; // 7-8: Good
  if (mood >= 5) return '😐'; // 5-6: Okay
  if (mood >= 3) return '😔'; // 3-4: Bad
  return '😫'; // 1-2: Awful
};

const getMoodColor = (mood) => {
  if (mood >= 9) return '#10b981'; // green
  if (mood >= 7) return '#3b82f6'; // blue
  if (mood >= 5) return '#f59e0b'; // amber
  if (mood >= 3) return '#f97316'; // orange
  return '#ef4444'; // red
};

function MoodTickerOverlay() {
  const [moodData, setMoodData] = createSignal({
    mood: 5, // 1-10 scale
    weight: null,
    sleep: null,
    water: 0,
    show_background: true
  });

  let ws = null;

  // Load mood data
  const loadMoodData = async () => {
    try {
      console.log('🔄 Loading mood ticker data...');
      const response = await fetch(`${WEBARCADE_API}/mood-tracker/data`);
      const data = await response.json();
      console.log('💫 Mood data loaded:', data);
      setMoodData(data);
    } catch (error) {
      console.error('❌ Failed to load mood data:', error);
    }
  };

  // Connect to WebSocket
  const connectWebSocket = () => {
    ws = new WebSocket(WEBSOCKET_URL);

    ws.onopen = () => {
      console.log('✅ WebSocket connected (mood ticker)');
    };

    ws.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data);
        console.log('📨 WebSocket message received (mood ticker):', data);

        // Handle mood ticker updates
        if (data.type === 'mood_ticker_update') {
          console.log('📡 Received mood ticker update:', data.data);
          setMoodData(data.data);
        }
      } catch (err) {
        console.error('❌ Failed to parse WebSocket message:', err);
      }
    };

    ws.onerror = (error) => {
      console.error('❌ WebSocket error:', error);
    };

    ws.onclose = () => {
      console.log('🔌 WebSocket disconnected, reconnecting in 3 seconds...');
      setTimeout(connectWebSocket, 3000);
    };
  };

  createEffect(() => {
    loadMoodData();
    connectWebSocket();

    onCleanup(() => {
      if (ws) {
        ws.close();
      }
    });
  });

  return (
    <div class="fixed inset-0 pointer-events-none overflow-hidden bg-transparent">
      {/* Mood Ticker Bar */}
      <div class={`mood-ticker-bar absolute bottom-0 left-0 right-0 ${moodData().show_background ? 'bg-gradient-to-r from-indigo-900/95 via-purple-900/95 to-pink-900/95 backdrop-blur-sm border-t-4 border-b-4 border-black/20 shadow-lg' : ''} h-16 flex items-center overflow-hidden`}>
        {/* Stats - Horizontal Layout */}
        <div class="flex items-center gap-2 px-4 w-full">

          {/* Mood */}
          <div
            class="px-3 py-1.5 rounded shadow-lg border flex items-center gap-3 min-w-[180px]"
            style={{
              background: moodData().show_background
                ? 'rgba(0, 0, 0, 0.2)'
                : 'linear-gradient(135deg, rgba(139, 92, 246, 0.7) 0%, rgba(168, 85, 247, 0.7) 100%)',
              'border-color': moodData().show_background ? 'rgba(192, 132, 252, 0.2)' : 'rgba(192, 132, 252, 0.5)'
            }}
          >
            <span class="text-2xl">{getMoodEmoji(moodData().mood)}</span>
            <div class="flex flex-col gap-1 flex-1">
              <div class="flex items-center justify-between">
                <span class="text-white font-semibold text-xs uppercase tracking-wide">Mood</span>
                <span class="text-purple-200 font-bold text-sm">
                  {moodData().mood}/10
                </span>
              </div>
              {/* Progress bar */}
              <div class="w-full h-1.5 bg-black/40 rounded-full overflow-hidden">
                <div
                  class="h-full rounded-full transition-all duration-300"
                  style={{
                    width: `${(moodData().mood / 10) * 100}%`,
                    'background-color': getMoodColor(moodData().mood)
                  }}
                ></div>
              </div>
            </div>
          </div>

          {/* Sleep */}
          <div
            class="px-3 py-1.5 rounded shadow-lg border flex items-center gap-3 min-w-[180px]"
            style={{
              background: moodData().show_background
                ? 'rgba(0, 0, 0, 0.2)'
                : 'linear-gradient(135deg, rgba(99, 102, 241, 0.7) 0%, rgba(129, 140, 248, 0.7) 100%)',
              'border-color': moodData().show_background ? 'rgba(129, 140, 248, 0.2)' : 'rgba(129, 140, 248, 0.5)'
            }}
          >
            <span class="text-2xl">😴</span>
            <div class="flex flex-col gap-1 flex-1">
              <div class="flex items-center justify-between">
                <span class="text-white font-semibold text-xs uppercase tracking-wide">Sleep</span>
                <span class="text-indigo-200 font-bold text-sm">
                  {moodData().sleep ? `${moodData().sleep}h` : 'No data'}
                </span>
              </div>
              {/* Progress bar - target 8 hours */}
              <div class="w-full h-1.5 bg-black/40 rounded-full overflow-hidden">
                <div
                  class="h-full rounded-full transition-all duration-300"
                  style={{
                    width: moodData().sleep ? `${Math.min((moodData().sleep / 8) * 100, 100)}%` : '0%',
                    'background-color': moodData().sleep >= 7 ? '#818cf8' : moodData().sleep >= 5 ? '#f59e0b' : '#ef4444'
                  }}
                ></div>
              </div>
            </div>
          </div>

          {/* Water Intake */}
          <div
            class="px-3 py-1.5 rounded shadow-lg border flex items-center gap-3 min-w-[180px]"
            style={{
              background: moodData().show_background
                ? 'rgba(0, 0, 0, 0.2)'
                : 'linear-gradient(135deg, rgba(6, 182, 212, 0.7) 0%, rgba(20, 184, 166, 0.7) 100%)',
              'border-color': moodData().show_background ? 'rgba(34, 211, 238, 0.2)' : 'rgba(34, 211, 238, 0.5)'
            }}
          >
            <span class="text-2xl">💧</span>
            <div class="flex flex-col gap-1 flex-1">
              <div class="flex items-center justify-between">
                <span class="text-white font-semibold text-xs uppercase tracking-wide">Water</span>
                <span class="text-cyan-200 font-bold text-sm">
                  {moodData().water || 0}/8
                </span>
              </div>
              {/* Progress bar - target 8 glasses */}
              <div class="w-full h-1.5 bg-black/40 rounded-full overflow-hidden">
                <div
                  class="h-full rounded-full transition-all duration-300"
                  style={{
                    width: `${Math.min((moodData().water / 8) * 100, 100)}%`,
                    'background-color': moodData().water >= 8 ? '#06b6d4' : moodData().water >= 4 ? '#14b8a6' : '#3b82f6'
                  }}
                ></div>
              </div>
            </div>
          </div>

          {/* Weight (no progress bar) */}
          <div
            class="px-3 py-1.5 rounded shadow-lg border flex items-center gap-2 flex-shrink-0"
            style={{
              background: moodData().show_background
                ? 'rgba(0, 0, 0, 0.2)'
                : 'linear-gradient(135deg, rgba(59, 130, 246, 0.7) 0%, rgba(96, 165, 250, 0.7) 100%)',
              'border-color': moodData().show_background ? 'rgba(96, 165, 250, 0.2)' : 'rgba(96, 165, 250, 0.5)'
            }}
          >
            <span class="text-2xl">⚖️</span>
            <div class="flex flex-col">
              <span class="text-white font-semibold text-xs uppercase tracking-wide">Weight</span>
              <span class="text-blue-200 font-bold text-sm">
                {moodData().weight ? `${moodData().weight} kg` : 'No data'}
              </span>
            </div>
          </div>
        </div>
      </div>

      <style>{`
        @keyframes shine {
          0% {
            left: -150%;
          }
          100% {
            left: 250%;
          }
        }

        .mood-ticker-bar {
          position: relative;
        }

        .mood-ticker-bar::before {
          content: '';
          position: absolute;
          top: 0;
          left: -150%;
          width: 150%;
          height: 100%;
          background: linear-gradient(
            90deg,
            transparent,
            rgba(255, 255, 255, 0.15) 50%,
            transparent
          );
          animation: shine 6s infinite;
          pointer-events: none;
        }
      `}</style>
    </div>
  );
}

// Only render when used as standalone (for OBS browser sources)
if (document.getElementById('root')) {
  render(() => <MoodTickerOverlay />, document.getElementById('root'));
}
