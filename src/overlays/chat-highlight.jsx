import { render } from 'solid-js/web';
import { createSignal, createEffect, onCleanup, For } from 'solid-js';
import '@/index.css';
import { WEBARCADE_WS } from '@/api/bridge';

function ChatHighlightOverlay() {
  const [highlights, setHighlights] = createSignal([]);
  const [settings, setSettings] = createSignal({
    enabled: true,
    duration: 8000, // milliseconds to show highlight
    maxHighlights: 5, // max concurrent highlights
    position: 'top', // 'top', 'center', 'bottom'
    animation: 'slide', // 'slide', 'fade', 'zoom'
  });

  let ws;
  let nextId = 0;

  // Connect to WebSocket for highlight events
  const connectWebSocket = () => {
    ws = new WebSocket(WEBARCADE_WS);

    ws.onopen = () => {
      console.log('✅ Connected to WebArcade WebSocket (Chat Highlight)');
    };

    ws.onclose = () => {
      console.log('❌ Disconnected from WebSocket (Chat Highlight)');
      setTimeout(connectWebSocket, 3000);
    };

    ws.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data);

        // Listen for highlight events
        if (data.type === 'chat_highlight' && settings().enabled) {
          console.log('[Chat Highlight] Received highlight:', data);
          addHighlight(data);
        }

        // Settings update
        if (data.type === 'chat_highlight_settings') {
          setSettings(data.settings);
        }

        // Clear all highlights
        if (data.type === 'chat_highlight_clear') {
          setHighlights([]);
        }
      } catch (error) {
        console.error('Error parsing WebSocket message:', error);
      }
    };
  };

  // Add a new highlight to display
  const addHighlight = (data) => {
    const newHighlight = {
      id: nextId++,
      username: data.username || 'Unknown',
      displayName: data.display_name || data.username || 'Unknown',
      message: data.message || '',
      color: data.color || '#9147ff',
      profileImage: data.profile_image_url || null,
      badges: data.badges || [],
      timestamp: Date.now(),
    };

    console.log('[Chat Highlight] Adding highlight:', newHighlight);

    setHighlights(prev => {
      const updated = [...prev, newHighlight];
      // Limit max highlights
      if (updated.length > settings().maxHighlights) {
        return updated.slice(-settings().maxHighlights);
      }
      return updated;
    });

    // Auto-remove after duration
    setTimeout(() => {
      removeHighlight(newHighlight.id);
    }, settings().duration);
  };

  // Remove a highlight by ID
  const removeHighlight = (id) => {
    setHighlights(prev => prev.filter(h => h.id !== id));
  };

  // Connect WebSocket on mount
  createEffect(() => {
    connectWebSocket();

    onCleanup(() => {
      if (ws) {
        ws.close();
      }
    });
  });

  // Get position classes
  const getPositionClasses = () => {
    const pos = settings().position;
    switch (pos) {
      case 'top':
        return 'top-8 left-1/2 -translate-x-1/2';
      case 'center':
        return 'top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2';
      case 'bottom':
        return 'bottom-8 left-1/2 -translate-x-1/2';
      default:
        return 'top-8 left-1/2 -translate-x-1/2';
    }
  };

  // Get animation classes
  const getAnimationClass = () => {
    const anim = settings().animation;
    switch (anim) {
      case 'slide':
        return 'highlight-slide-in';
      case 'fade':
        return 'highlight-fade-in';
      case 'zoom':
        return 'highlight-zoom-in';
      default:
        return 'highlight-slide-in';
    }
  };

  return (
    <div class="fixed inset-0 pointer-events-none overflow-hidden bg-transparent">
      <div class={`absolute ${getPositionClasses()} w-full max-w-4xl px-4`}>
        <div class="flex flex-col gap-3">
          <For each={highlights()}>
            {(highlight, index) => (
              <div
                class={`highlight-card ${getAnimationClass()} pointer-events-auto`}
                style={{
                  'animation-delay': `${index() * 100}ms`,
                }}
              >
                {/* Highlight Container */}
                <div class="relative bg-gradient-to-br from-purple-900/95 via-violet-900/95 to-fuchsia-900/95 backdrop-blur-md rounded-2xl shadow-2xl border-4 border-purple-400/50 overflow-hidden">
                  {/* Animated background effect */}
                  <div class="absolute inset-0 bg-gradient-to-r from-transparent via-white/10 to-transparent animate-shimmer"></div>

                  {/* Glowing border effect */}
                  <div class="absolute inset-0 rounded-2xl animate-pulse-glow"></div>

                  {/* Content */}
                  <div class="relative p-6">
                    {/* Header with user info */}
                    <div class="flex items-center gap-4 mb-4">
                      {/* Profile Image */}
                      {highlight.profileImage ? (
                        <img
                          src={highlight.profileImage}
                          alt={highlight.displayName}
                          class="w-16 h-16 rounded-full border-4 border-purple-400 shadow-lg"
                        />
                      ) : (
                        <div class="w-16 h-16 rounded-full bg-gradient-to-br from-purple-500 to-pink-500 flex items-center justify-center border-4 border-purple-400 shadow-lg">
                          <span class="text-2xl font-bold text-white">
                            {highlight.displayName.charAt(0).toUpperCase()}
                          </span>
                        </div>
                      )}

                      {/* Username and badges */}
                      <div class="flex-1">
                        <div class="flex items-center gap-2 mb-1">
                          <For each={highlight.badges}>
                            {(badge) => (
                              <span class="text-xs px-2 py-0.5 bg-purple-600/80 rounded-full text-white font-semibold">
                                {badge}
                              </span>
                            )}
                          </For>
                        </div>
                        <div
                          class="text-2xl font-bold drop-shadow-lg"
                          style={{ color: highlight.color }}
                        >
                          {highlight.displayName}
                        </div>
                      </div>

                      {/* Highlight Icon */}
                      <div class="text-5xl animate-bounce-slow">
                        ⭐
                      </div>
                    </div>

                    {/* Message */}
                    <div class="bg-black/30 rounded-xl p-4 backdrop-blur-sm border border-purple-400/30">
                      <p class="text-white text-2xl font-semibold leading-relaxed break-words">
                        {highlight.message}
                      </p>
                    </div>
                  </div>

                  {/* Bottom accent bar */}
                  <div class="h-2 bg-gradient-to-r from-purple-500 via-pink-500 to-purple-500 animate-gradient-x"></div>
                </div>
              </div>
            )}
          </For>
        </div>
      </div>

      <style>{`
        /* Slide in animation */
        @keyframes highlight-slide-in {
          from {
            opacity: 0;
            transform: translateY(-30px);
          }
          to {
            opacity: 1;
            transform: translateY(0);
          }
        }

        .highlight-slide-in {
          animation: highlight-slide-in 0.5s ease-out forwards;
        }

        /* Fade in animation */
        @keyframes highlight-fade-in {
          from {
            opacity: 0;
          }
          to {
            opacity: 1;
          }
        }

        .highlight-fade-in {
          animation: highlight-fade-in 0.5s ease-out forwards;
        }

        /* Zoom in animation */
        @keyframes highlight-zoom-in {
          from {
            opacity: 0;
            transform: scale(0.8);
          }
          to {
            opacity: 1;
            transform: scale(1);
          }
        }

        .highlight-zoom-in {
          animation: highlight-zoom-in 0.5s ease-out forwards;
        }

        /* Shimmer effect */
        @keyframes shimmer {
          0% {
            transform: translateX(-100%);
          }
          100% {
            transform: translateX(100%);
          }
        }

        .animate-shimmer {
          animation: shimmer 3s infinite;
        }

        /* Pulse glow effect */
        @keyframes pulse-glow {
          0%, 100% {
            box-shadow: 0 0 20px rgba(168, 85, 247, 0.4),
                        0 0 40px rgba(168, 85, 247, 0.2),
                        inset 0 0 20px rgba(168, 85, 247, 0.1);
          }
          50% {
            box-shadow: 0 0 30px rgba(168, 85, 247, 0.6),
                        0 0 60px rgba(168, 85, 247, 0.3),
                        inset 0 0 30px rgba(168, 85, 247, 0.2);
          }
        }

        .animate-pulse-glow {
          animation: pulse-glow 2s ease-in-out infinite;
        }

        /* Gradient animation */
        @keyframes gradient-x {
          0%, 100% {
            background-position: 0% 50%;
          }
          50% {
            background-position: 100% 50%;
          }
        }

        .animate-gradient-x {
          background-size: 200% 200%;
          animation: gradient-x 3s ease infinite;
        }

        /* Slow bounce */
        @keyframes bounce-slow {
          0%, 100% {
            transform: translateY(0);
          }
          50% {
            transform: translateY(-10px);
          }
        }

        .animate-bounce-slow {
          animation: bounce-slow 2s ease-in-out infinite;
        }

        .highlight-card {
          opacity: 0;
        }
      `}</style>
    </div>
  );
}

// Only render when used as standalone (for OBS browser sources)
if (document.getElementById('root')) {
  render(() => <ChatHighlightOverlay />, document.getElementById('root'));
}
