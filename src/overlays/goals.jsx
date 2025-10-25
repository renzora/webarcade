import { render } from 'solid-js/web';
import { createSignal, createEffect, onCleanup, Show, For } from 'solid-js';
import '@/index.css';
import { WEBARCADE_WS } from '@/api/bridge';

function GoalsOverlay() {
  const [isConnected, setIsConnected] = createSignal(false);
  const [goals, setGoals] = createSignal([]);
  const [channel, setChannel] = createSignal('');

  let ws;

  // Fetch initial goals data
  const fetchGoals = async (channelName) => {
    if (!channelName) return;

    try {
      const response = await fetch(`http://localhost:3001/database/goals?channel=${channelName}`);
      if (response.ok) {
        const data = await response.json();
        setGoals(data || []);
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
          setGoals(data.goals || []);
          console.log('📊 Goals updated via WebSocket:', data.goals);
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

  const getGoalColor = (goalType) => {
    switch (goalType) {
      case 'subscriber':
        return '#9146ff'; // Twitch purple
      case 'follower':
        return '#00ff7f'; // Green
      default:
        return '#3b82f6'; // Blue
    }
  };

  const getGoalBorderColor = (goalType) => {
    switch (goalType) {
      case 'subscriber':
        return '#4a1f7a'; // Much darker purple
      case 'follower':
        return '#009944'; // Much darker green
      default:
        return '#1d4ed8'; // Much darker blue
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
    <div class="fixed inset-0 pointer-events-none overflow-hidden font-sans">
      {/* Goals Display */}
      <Show when={goals().length > 0}>
        <div class="absolute top-0 left-0 right-0 pointer-events-none">
          <div class="space-y-2">
            <For each={goals()}>
              {(goal) => (
                <div
                  class="relative overflow-hidden bg-black/80 backdrop-blur-sm border shadow-xl"
                  style={{
                    "border-color": getGoalBorderColor(goal.type)
                  }}
                >
                  {/* Background Progress Bar */}
                  <div
                    class="absolute inset-0 transition-all duration-1000 ease-out"
                    style={{
                      background: getGoalColor(goal.type),
                      width: `${getPercentage(goal.current, goal.target)}%`,
                      opacity: 0.5
                    }}
                  />

                  {/* Content */}
                  <div class="relative px-4 py-2">
                    {/* Header and Progress - Single Row */}
                    <div class="flex items-center gap-3">
                      {/* Icon */}
                      <span class="text-xl">{getGoalIcon(goal.type)}</span>

                      {/* Title */}
                      <div class="flex items-center gap-2 min-w-[200px]">
                        <h3 class="text-base font-bold text-white">{goal.title}</h3>
                        <Show when={goal.is_sub_goal}>
                          <div class="badge badge-xs badge-warning">Sub</div>
                        </Show>
                      </div>

                      {/* Progress Numbers */}
                      <div class="flex items-baseline gap-2 min-w-[180px]">
                        <span class="text-2xl font-bold text-white font-mono">
                          {goal.current.toLocaleString()}
                        </span>
                        <span class="text-base text-white/60 font-mono">
                          / {goal.target.toLocaleString()}
                        </span>
                      </div>

                      {/* Percentage */}
                      <div class="flex-1 flex justify-end">
                        <span class="text-sm font-semibold text-white/80">
                          {getPercentage(goal.current, goal.target).toFixed(1)}%
                        </span>
                      </div>
                    </div>

                    {/* Description (if exists) */}
                    <Show when={goal.description}>
                      <p class="text-xs text-white/60 mt-1 ml-9">{goal.description}</p>
                    </Show>
                  </div>
                </div>
              )}
            </For>
          </div>
        </div>
      </Show>
    </div>
  );
}

render(() => <GoalsOverlay />, document.getElementById('root'));
