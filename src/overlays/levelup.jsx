import { render } from 'solid-js/web';
import { createSignal, createEffect, onCleanup, Show, For } from 'solid-js';
import '@/index.css';
import { WEBARCADE_WS } from '@/api/bridge';
const DISPLAY_DURATION = 8000; // 8 seconds

function LevelUpOverlay() {
  const [isConnected, setIsConnected] = createSignal(false);
  const [levelUpQueue, setLevelUpQueue] = createSignal([]);
  const [currentLevelUp, setCurrentLevelUp] = createSignal(null);
  const [isVisible, setIsVisible] = createSignal(false);

  let ws;
  let hideTimeout;

  // Connect to WebSocket
  const connectWebSocket = () => {
    ws = new WebSocket(WEBARCADE_WS);

    ws.onopen = () => {
      console.log('✅ Connected to WebArcade (Level Up Overlay)');
      setIsConnected(true);
    };

    ws.onclose = () => {
      console.log('❌ Disconnected');
      setIsConnected(false);
      setTimeout(connectWebSocket, 3000);
    };

    ws.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data);
        if (data.type === 'level_up') {
          handleLevelUp(data);
        }
      } catch (error) {
        console.error('Error parsing event:', error);
      }
    };
  };

  const handleLevelUp = (data) => {
    // Add to queue
    setLevelUpQueue([...levelUpQueue(), data]);

    // If not currently showing a level up, show it immediately
    if (!isVisible()) {
      showNextLevelUp();
    }
  };

  const showNextLevelUp = () => {
    const queue = levelUpQueue();
    if (queue.length === 0) return;

    // Get first item from queue
    const [next, ...rest] = queue;
    setLevelUpQueue(rest);
    setCurrentLevelUp(next);
    setIsVisible(true);

    // Clear any existing timeout
    if (hideTimeout) clearTimeout(hideTimeout);

    // Hide after duration
    hideTimeout = setTimeout(() => {
      setIsVisible(false);
      // Wait for fade out animation, then show next
      setTimeout(() => {
        setCurrentLevelUp(null);
        showNextLevelUp();
      }, 500);
    }, DISPLAY_DURATION);
  };

  const getLevelEmoji = (level) => {
    if (level <= 5) return '⭐';
    if (level <= 10) return '🌟';
    if (level <= 20) return '💫';
    if (level <= 50) return '✨';
    return '🏆';
  };

  const getGradientColors = (level) => {
    if (level <= 5) return 'from-blue-500 via-blue-600 to-purple-600';
    if (level <= 10) return 'from-purple-500 via-purple-600 to-pink-600';
    if (level <= 20) return 'from-pink-500 via-pink-600 to-red-600';
    if (level <= 50) return 'from-yellow-500 via-orange-500 to-red-600';
    return 'from-yellow-400 via-yellow-500 to-yellow-600';
  };

  // Initialize WebSocket
  createEffect(() => {
    connectWebSocket();
    onCleanup(() => {
      ws?.close();
      if (hideTimeout) clearTimeout(hideTimeout);
    });
  });

  return (
    <div class="fixed inset-0 pointer-events-none overflow-hidden font-sans bg-transparent">
      {/* Level Up Animation */}
      <Show when={isVisible() && currentLevelUp()}>
        <div
          class="absolute top-1/3 left-1/2 -translate-x-1/2 -translate-y-1/2 pointer-events-none transition-all duration-500"
          style={{
            opacity: isVisible() ? 1 : 0,
            transform: isVisible() ? 'translate(-50%, -50%) scale(1)' : 'translate(-50%, -50%) scale(0.8)'
          }}
        >
          {/* Main Container */}
          <div class="relative">
            {/* Glow Effect */}
            <div
              class={`absolute inset-0 blur-3xl opacity-60 animate-pulse bg-gradient-to-r ${getGradientColors(currentLevelUp()?.new_level || 1)}`}
              style={{
                'animation-duration': '2s'
              }}
            />

            {/* Content Card */}
            <div class={`relative bg-gradient-to-br ${getGradientColors(currentLevelUp()?.new_level || 1)} p-1 rounded-3xl shadow-2xl min-w-[500px]`}>
              <div class="bg-black/90 backdrop-blur-xl rounded-3xl p-8">
                {/* LEVEL UP Title */}
                <div class="text-center mb-4 animate-bounce" style={{ 'animation-duration': '1s' }}>
                  <h1 class="text-6xl font-black text-white drop-shadow-2xl tracking-wider">
                    🎊 LEVEL UP! 🎊
                  </h1>
                </div>

                {/* Username */}
                <div class="text-center mb-6">
                  <h2 class="text-4xl font-bold text-white drop-shadow-lg">
                    {currentLevelUp()?.username}
                  </h2>
                </div>

                {/* Level Display */}
                <div class="flex items-center justify-center gap-8 mb-6">
                  {/* Old Level */}
                  <div class="text-center opacity-50">
                    <div class="text-sm text-white/60 font-semibold mb-1">FROM</div>
                    <div class="text-5xl font-black text-white">
                      {currentLevelUp()?.old_level || 1}
                    </div>
                  </div>

                  {/* Arrow */}
                  <div class="text-5xl text-white animate-pulse">
                    →
                  </div>

                  {/* New Level */}
                  <div class="text-center">
                    <div class="text-sm text-white/60 font-semibold mb-1">TO</div>
                    <div class="flex items-center gap-3">
                      <div class="text-7xl font-black text-transparent bg-clip-text bg-gradient-to-r from-yellow-300 via-yellow-400 to-yellow-500 drop-shadow-xl animate-pulse">
                        {currentLevelUp()?.new_level || 2}
                      </div>
                      <div class="text-5xl animate-spin" style={{ 'animation-duration': '3s' }}>
                        {getLevelEmoji(currentLevelUp()?.new_level || 1)}
                      </div>
                    </div>
                  </div>
                </div>

                {/* Stats */}
                <div class="grid grid-cols-2 gap-4 mt-8">
                  <div class="bg-white/10 backdrop-blur rounded-xl p-4 text-center">
                    <div class="text-sm text-white/70 mb-1">Total XP</div>
                    <div class="text-2xl font-bold text-white">
                      {currentLevelUp()?.total_xp?.toLocaleString() || 0}
                    </div>
                  </div>
                  <div class="bg-white/10 backdrop-blur rounded-xl p-4 text-center">
                    <div class="text-sm text-white/70 mb-1">Next Level</div>
                    <div class="text-2xl font-bold text-white">
                      {currentLevelUp()?.xp_needed?.toLocaleString() || 0} XP
                    </div>
                  </div>
                </div>

                {/* Particles Effect (Decorative) */}
                <div class="absolute top-0 left-0 w-full h-full pointer-events-none overflow-hidden rounded-3xl">
                  <For each={Array(20).fill(0)}>
                    {(_, i) => (
                      <div
                        class="absolute w-2 h-2 bg-yellow-400 rounded-full opacity-60 animate-ping"
                        style={{
                          left: `${Math.random() * 100}%`,
                          top: `${Math.random() * 100}%`,
                          'animation-delay': `${i() * 0.1}s`,
                          'animation-duration': `${1 + Math.random()}s`
                        }}
                      />
                    )}
                  </For>
                </div>
              </div>
            </div>
          </div>
        </div>
      </Show>
    </div>
  );
}

// Only render when used as standalone (for OBS browser sources)
if (document.getElementById('root')) {
  render(() => <LevelUpOverlay />, document.getElementById('root'));
}
