import { render } from 'solid-js/web';
import { createSignal, createEffect, onCleanup, Show, For } from 'solid-js';
import '@/index.css';
import { WEBARCADE_WS } from '@/api/bridge';

function GoalsOverlay() {
  const [isConnected, setIsConnected] = createSignal(false);
  const [goals, setGoals] = createSignal([]);
  const [channel, setChannel] = createSignal('');
  const [animatingGoals, setAnimatingGoals] = createSignal(new Set());
  const [previousGoals, setPreviousGoals] = createSignal(new Map());

  let ws;

  // Fetch initial goals data
  const fetchGoals = async (channelName) => {
    if (!channelName) return;

    try {
      const response = await fetch(`http://localhost:3001/goals/list?channel=${channelName}`);
      if (response.ok) {
        const data = await response.json();
        setGoals(data || []);

        // Initialize previous goals map
        const prevGoals = new Map();
        (data || []).forEach(goal => {
          prevGoals.set(goal.id, goal.current);
        });
        setPreviousGoals(prevGoals);

        console.log('📊 Loaded initial goals:', data);
      }
    } catch (error) {
      console.error('Failed to fetch initial goals:', error);
    }
  };

  // Connect to WebSocket
  const connectWebSocket = () => {
    ws = new WebSocket(WEBARCADE_WS);

    ws.onopen = () => {
      console.log('✅ Connected to WebArcade');
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

        // Get channel from connection message
        if (data.type === 'connected' && data.channel) {
          const channelName = data.channel;
          setChannel(channelName);
          console.log('📡 Connected to channel:', channelName);
          // Fetch initial goals
          fetchGoals(channelName);
        }

        // Update goals from WebSocket
        if (data.type === 'goals_update') {
          const newGoals = data.goals || [];
          const prevGoals = previousGoals();

          // Check for goal value increases
          newGoals.forEach(goal => {
            const prevValue = prevGoals.get(goal.id);
            if (prevValue !== undefined && goal.current > prevValue) {
              // Goal increased! Trigger animation
              setAnimatingGoals(prev => {
                const newSet = new Set(prev);
                newSet.add(goal.id);
                return newSet;
              });

              // Remove animation after 3 seconds
              setTimeout(() => {
                setAnimatingGoals(prev => {
                  const newSet = new Set(prev);
                  newSet.delete(goal.id);
                  return newSet;
                });
              }, 3000);
            }
          });

          // Update previous goals map
          const newPrevGoals = new Map();
          newGoals.forEach(goal => {
            newPrevGoals.set(goal.id, goal.current);
          });
          setPreviousGoals(newPrevGoals);

          setGoals(newGoals);
          console.log('📊 Goals updated via WebSocket:', newGoals);
        }
      } catch (error) {
        console.error('Error parsing event:', error);
      }
    };
  };

  const getPercentage = (current, target) => {
    if (target === 0) return 0;
    return Math.min(100, Math.max(0, (current / target) * 100));
  };

  const getGoalGradient = (goalType) => {
    switch (goalType) {
      case 'subscriber':
        return 'linear-gradient(90deg, #7c3aed 0%, #a78bfa 100%)'; // Deep purple to light purple
      case 'follower':
        return 'linear-gradient(90deg, #059669 0%, #34d399 100%)'; // Deep green to light green
      default:
        return 'linear-gradient(90deg, #dc2626 0%, #f87171 100%)'; // Deep red to light red
    }
  };

  const getGoalAccentColor = (goalType) => {
    switch (goalType) {
      case 'subscriber':
        return '#7c3aed';
      case 'follower':
        return '#059669';
      default:
        return '#dc2626';
    }
  };

  const getGoalIcon = (goalType) => {
    switch (goalType) {
      case 'subscriber':
        return '⭐';
      case 'follower':
        return '❤️';
      default:
        return '🎯';
    }
  };

  // Initialize WebSocket
  createEffect(() => {
    connectWebSocket();
    onCleanup(() => ws?.close());
  });

  return (
    <div class="fixed inset-0 pointer-events-none overflow-hidden font-sans bg-transparent">
      {/* Goals Display */}
      <Show when={goals().length > 0}>
        <div class="absolute top-0 left-0 right-0 pointer-events-none">
          <div class="space-y-2">
            <For each={goals()}>
              {(goal) => {
                const percentage = getPercentage(goal.current, goal.target);
                return (
                  <div
                    class="relative overflow-hidden bg-gradient-to-r from-black/95 via-black/90 to-black/95 backdrop-blur-md border border-black/50"
                  >
                    {/* Background Progress Bar with Gradient */}
                    <div
                      class="absolute inset-0 transition-all duration-1000 ease-out"
                      style={{
                        background: getGoalGradient(goal.type),
                        width: `${percentage}%`,
                        opacity: 0.5
                      }}
                    />

                    {/* Pulsing glow effect near the end of progress */}
                    <Show when={percentage < 100 && percentage > 10}>
                      <div
                        class="absolute top-0 bottom-0 w-24 goal-edge-glow"
                        style={{
                          left: `${percentage}%`,
                          background: `linear-gradient(to right, transparent, ${getGoalAccentColor(goal.type)}40, transparent)`,
                        }}
                      />
                    </Show>

                    {/* Floating Emoji Animation */}
                    <Show when={animatingGoals().has(goal.id)}>
                      <div class="absolute inset-0 overflow-visible pointer-events-none">
                        <div class="floating-emoji emoji-1">{getGoalIcon(goal.type)}</div>
                        <div class="floating-emoji emoji-2">{getGoalIcon(goal.type)}</div>
                        <div class="floating-emoji emoji-3">{getGoalIcon(goal.type)}</div>
                        <div class="floating-emoji emoji-4">{getGoalIcon(goal.type)}</div>
                        <div class="floating-emoji emoji-5">{getGoalIcon(goal.type)}</div>
                      </div>
                    </Show>

                    {/* Content */}
                    <div class="relative flex items-center">
                      {/* Icon - Full Height */}
                      <div
                        class="flex items-center justify-center px-2 py-1.5"
                        style={{
                          background: `${getGoalAccentColor(goal.type)}20`
                        }}
                      >
                        <span class="text-3xl">{getGoalIcon(goal.type)}</span>
                      </div>

                      {/* Main Content */}
                      <div class="flex-1 px-2 py-1.5">
                        <div class="flex items-center justify-between">
                          {/* Left side - Title and Description */}
                          <div class="flex-1 min-w-[200px]">
                            <div class="flex items-center gap-1.5">
                              <h3 class="text-sm font-bold text-white drop-shadow-lg">{goal.title}</h3>
                              <Show when={goal.is_sub_goal}>
                                <div class="text-xs px-1.5 py-0.5 bg-amber-500/30 border border-amber-400/50 rounded text-amber-300 font-semibold">Sub</div>
                              </Show>
                            </div>
                            {/* Description - Right below title */}
                            <Show when={goal.description}>
                              <p class="text-xs text-white/60 mt-0.5 drop-shadow-md">{goal.description}</p>
                            </Show>
                          </div>

                          {/* Right side - Progress Numbers */}
                          <div class="flex items-baseline gap-1.5">
                            <span class="text-lg font-bold text-white font-mono drop-shadow-lg">
                              {goal.current.toLocaleString()}
                            </span>
                            <span class="text-sm text-white/60 font-mono">
                              / {goal.target.toLocaleString()}
                            </span>
                          </div>
                        </div>
                      </div>
                    </div>

                    {/* Shimmer effect */}
                    <div class="goal-shimmer"></div>
                  </div>
                );
              }}
            </For>
          </div>
        </div>
      </Show>

      <style>{`
        @keyframes shimmer {
          0% {
            transform: translateX(-100%);
          }
          100% {
            transform: translateX(100%);
          }
        }

        @keyframes edge-pulse {
          0%, 100% {
            opacity: 0.4;
            transform: scaleX(1);
          }
          50% {
            opacity: 0.8;
            transform: scaleX(1.1);
          }
        }

        @keyframes float-up {
          0% {
            transform: translateY(0) scale(1) rotate(0deg);
            opacity: 0;
          }
          10% {
            opacity: 1;
          }
          80% {
            opacity: 1;
          }
          100% {
            transform: translateY(-150px) scale(1.5) rotate(20deg);
            opacity: 0;
          }
        }

        .goal-edge-glow {
          animation: edge-pulse 2s ease-in-out infinite;
          pointer-events: none;
        }

        .goal-shimmer {
          position: absolute;
          top: 0;
          left: -100%;
          width: 100%;
          height: 100%;
          background: linear-gradient(
            90deg,
            transparent,
            rgba(255, 255, 255, 0.1) 50%,
            transparent
          );
          animation: shimmer 8s infinite;
          pointer-events: none;
        }

        .floating-emoji {
          position: absolute;
          bottom: 50%;
          font-size: 2rem;
          animation: float-up 3s ease-out forwards;
          filter: drop-shadow(0 0 8px rgba(255, 255, 255, 0.5));
        }

        .emoji-1 {
          left: 20%;
          animation-delay: 0s;
        }

        .emoji-2 {
          left: 35%;
          animation-delay: 0.2s;
        }

        .emoji-3 {
          left: 50%;
          animation-delay: 0.4s;
        }

        .emoji-4 {
          left: 65%;
          animation-delay: 0.6s;
        }

        .emoji-5 {
          left: 80%;
          animation-delay: 0.8s;
        }
      `}</style>
    </div>
  );
}

// Only render when used as standalone (for OBS browser sources)
if (document.getElementById('root')) {
  render(() => <GoalsOverlay />, document.getElementById('root'));
}
