import { render } from 'solid-js/web';
import { createSignal, createEffect, onCleanup, Show } from 'solid-js';
import '@/index.css';
import { WEBARCADE_WS } from '@/api/bridge';

function TimerOverlay() {
  const [isConnected, setIsConnected] = createSignal(false);
  const [timerName, setTimerName] = createSignal('Timer');
  const [isRunning, setIsRunning] = createSignal(false);
  const [isPaused, setIsPaused] = createSignal(false);
  const [timeRemaining, setTimeRemaining] = createSignal(0);
  const [totalTime, setTotalTime] = createSignal(0);
  const [isPomodoro, setIsPomodoro] = createSignal(false);
  const [currentPhase, setCurrentPhase] = createSignal('work');
  const [pomodoroCount, setPomodoroCount] = createSignal(0);

  let ws;
  let previousPhase = null;

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
        if (data.type === 'timer_state') {
          updateTimerState(data);
        }
      } catch (error) {
        console.error('Error parsing event:', error);
      }
    };
  };

  const updateTimerState = (data) => {
    setTimerName(data.name || 'Timer');
    setIsRunning(data.isRunning || false);
    setIsPaused(data.isPaused || false);
    setTimeRemaining(data.timeRemaining || 0);
    setIsPomodoro(data.isPomodoro || false);

    // Track total time when timer starts or phase changes
    const newPhase = data.currentPhase || 'work';
    if (previousPhase !== newPhase || totalTime() === 0) {
      setTotalTime(data.timeRemaining || 0);
      previousPhase = newPhase;
    }

    setCurrentPhase(newPhase);
    setPomodoroCount(data.pomodoroCount || 0);
  };

  const formatTime = (seconds) => {
    const mins = Math.floor(seconds / 60);
    const secs = seconds % 60;
    return `${mins.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
  };

  const getPercentageRemaining = () => {
    if (totalTime() === 0) return 100;
    return Math.max(0, Math.min(100, (timeRemaining() / totalTime()) * 100));
  };

  const getTimerColor = () => {
    if (isPomodoro() && currentPhase() === 'break') {
      return '#3b82f6'; // Blue for rest/break
    }
    return '#ef4444'; // Red for work
  };

  // Initialize WebSocket
  createEffect(() => {
    connectWebSocket();
    onCleanup(() => ws?.close());
  });

  return (
    <div class="fixed inset-0 pointer-events-none overflow-hidden font-sans">
      {/* Timer Display */}
      <Show when={isRunning() || isPaused()}>
        <div class="absolute top-0 left-1/2 -translate-x-1/2 pointer-events-none">
          <div class="relative bg-gradient-to-br from-black/90 to-black/70 backdrop-blur-xl shadow-2xl overflow-hidden min-w-[300px]">
            {/* Background Progress Bar - Full Overlay */}
            <div
              class="absolute inset-0 transition-all duration-1000 ease-linear"
              style={{
                background: getTimerColor(),
                width: `${getPercentageRemaining()}%`,
                opacity: 0.8
              }}
            />

            {/* Content */}
            <div class="relative">
              {/* Timer Name & Phase */}
              <div class="text-center px-4 pt-2">
                <div class="flex items-center justify-center gap-2">
                  <h2 class="text-xl font-bold text-white">{timerName()}</h2>
                  <Show when={isPomodoro()}>
                    <div class="badge badge-sm badge-primary">#{pomodoroCount() + 1}</div>
                  </Show>
                </div>

                <Show when={isPomodoro()}>
                  <div class="mt-0.5">
                    <div class={`badge badge-sm ${currentPhase() === 'work' ? 'badge-error' : 'badge-success'}`}>
                      {currentPhase() === 'work' ? '🍅 Work' : '☕ Break'}
                    </div>
                  </div>
                </Show>
              </div>

              {/* Time Display */}
              <div class="text-center pt-0 pb-2">
                <div class="text-6xl font-mono font-bold text-white tracking-wider">
                  {formatTime(timeRemaining())}
                </div>
              </div>

              {/* Status Indicator */}
              <Show when={isPaused()}>
                <div class="flex justify-center pb-4 px-8">
                  <div class="badge badge-warning gap-2">
                    ⏸ Paused
                  </div>
                </div>
              </Show>

              {/* Pomodoro Progress Dots */}
              <Show when={isPomodoro() && pomodoroCount() > 0}>
                <div class="flex justify-center gap-2 pb-4 pt-4 px-8 border-t border-white/10">
                  {Array.from({ length: Math.min(pomodoroCount(), 8) }).map((_, i) => (
                    <div class="w-3 h-3 rounded-full bg-primary"></div>
                  ))}
                  <Show when={pomodoroCount() > 8}>
                    <span class="text-xs text-white/60">+{pomodoroCount() - 8}</span>
                  </Show>
                </div>
              </Show>
            </div>
          </div>
        </div>
      </Show>
    </div>
  );
}

// Only render when used as standalone (for OBS browser sources)
if (document.getElementById('root')) {
  render(() => <TimerOverlay />, document.getElementById('root'));
}
