import { createSignal, createEffect, onCleanup } from 'solid-js';

const BRIDGE_URL = 'http://localhost:3001';

export default function TimerViewport() {
  const [timerName, setTimerName] = createSignal('Focus Timer');
  const [duration, setDuration] = createSignal(25); // minutes
  const [isPomodoro, setIsPomodoro] = createSignal(false);
  const [isRunning, setIsRunning] = createSignal(false);
  const [isPaused, setIsPaused] = createSignal(false);
  const [timeRemaining, setTimeRemaining] = createSignal(0); // seconds
  const [currentPhase, setCurrentPhase] = createSignal('work'); // 'work' or 'break'
  const [pomodoroCount, setPomodoroCount] = createSignal(0);

  let intervalId = null;

  // Pomodoro settings
  const WORK_DURATION = 25 * 60; // 25 minutes
  const SHORT_BREAK = 5 * 60; // 5 minutes
  const LONG_BREAK = 15 * 60; // 15 minutes
  const LONG_BREAK_INTERVAL = 4; // After 4 pomodoros

  const formatTime = (seconds) => {
    const mins = Math.floor(seconds / 60);
    const secs = seconds % 60;
    return `${mins.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
  };

  const getTimerPercentage = () => {
    if (!isRunning() && !isPaused()) return 100;

    let totalTime;
    if (isPomodoro()) {
      if (currentPhase() === 'work') {
        totalTime = WORK_DURATION;
      } else {
        // Determine if it's a long break or short break
        const isLongBreak = pomodoroCount() % LONG_BREAK_INTERVAL === 0;
        totalTime = isLongBreak ? LONG_BREAK : SHORT_BREAK;
      }
    } else {
      totalTime = duration() * 60;
    }

    return (timeRemaining() / totalTime) * 100;
  };

  const getTimerColor = () => {
    if (isPomodoro() && currentPhase() === 'break') {
      return '#3b82f6'; // Blue for rest/break
    }
    return '#ef4444'; // Red for work
  };

  const startTimer = () => {
    if (isRunning() && !isPaused()) return;

    if (!isRunning()) {
      // Starting fresh
      const initialTime = isPomodoro() ? WORK_DURATION : duration() * 60;
      setTimeRemaining(initialTime);
      setCurrentPhase('work');
      setIsRunning(true);
      setIsPaused(false);
    } else if (isPaused()) {
      // Resuming
      setIsPaused(false);
    }

    // Start countdown
    intervalId = setInterval(() => {
      setTimeRemaining(prev => {
        if (prev <= 1) {
          handleTimerComplete();
          return 0;
        }
        return prev - 1;
      });
    }, 1000);

    // Broadcast timer state
    broadcastTimerState();
  };

  const pauseTimer = () => {
    if (intervalId) {
      clearInterval(intervalId);
      intervalId = null;
    }
    setIsPaused(true);
    broadcastTimerState();
  };

  const stopTimer = () => {
    if (intervalId) {
      clearInterval(intervalId);
      intervalId = null;
    }
    setIsRunning(false);
    setIsPaused(false);
    setTimeRemaining(0);
    setCurrentPhase('work');
    broadcastTimerState();
  };

  const resetTimer = () => {
    stopTimer();
    setPomodoroCount(0);
  };

  const handleTimerComplete = () => {
    if (intervalId) {
      clearInterval(intervalId);
      intervalId = null;
    }

    // Play notification sound
    playNotificationSound();

    if (isPomodoro()) {
      // Pomodoro cycle logic
      if (currentPhase() === 'work') {
        const newCount = pomodoroCount() + 1;
        setPomodoroCount(newCount);

        // Determine break type
        const isLongBreak = newCount % LONG_BREAK_INTERVAL === 0;
        const breakDuration = isLongBreak ? LONG_BREAK : SHORT_BREAK;

        setCurrentPhase('break');
        setTimeRemaining(breakDuration);
        setIsRunning(true);
        setIsPaused(false);

        // Auto-start break
        startTimer();
      } else {
        // Break finished, ready for next work session
        setCurrentPhase('work');
        setIsRunning(false);
        setIsPaused(false);
      }
    } else {
      // Regular timer completed
      setIsRunning(false);
      setIsPaused(false);
    }

    broadcastTimerState();
  };

  const playNotificationSound = () => {
    // Simple beep sound
    const audioContext = new (window.AudioContext || window.webkitAudioContext)();
    const oscillator = audioContext.createOscillator();
    const gainNode = audioContext.createGain();

    oscillator.connect(gainNode);
    gainNode.connect(audioContext.destination);

    oscillator.frequency.value = 800;
    oscillator.type = 'sine';

    gainNode.gain.setValueAtTime(0.3, audioContext.currentTime);
    gainNode.gain.exponentialRampToValueAtTime(0.01, audioContext.currentTime + 0.5);

    oscillator.start(audioContext.currentTime);
    oscillator.stop(audioContext.currentTime + 0.5);
  };

  const broadcastTimerState = async () => {
    try {
      await fetch(`${BRIDGE_URL}/api/timer/broadcast`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          name: timerName(),
          isRunning: isRunning(),
          isPaused: isPaused(),
          timeRemaining: timeRemaining(),
          isPomodoro: isPomodoro(),
          currentPhase: currentPhase(),
          pomodoroCount: pomodoroCount()
        })
      });
    } catch (error) {
      console.error('Failed to broadcast timer state:', error);
    }
  };

  // Cleanup on unmount
  onCleanup(() => {
    if (intervalId) {
      clearInterval(intervalId);
    }
  });

  // Broadcast state changes
  createEffect(() => {
    if (isRunning() || isPaused()) {
      broadcastTimerState();
    }
  });

  return (
    <div class="h-full flex flex-col bg-base-200 p-6">
      <div class="flex-1 max-w-4xl mx-auto w-full">
        {/* Header */}
        <div class="mb-6">
          <h2 class="text-2xl font-bold mb-2">Timer</h2>
          <p class="text-base-content/60">Manage timers and Pomodoro sessions</p>
        </div>

        {/* Timer Configuration */}
        <div class="card bg-base-100 shadow-xl mb-4">
          <div class="card-body">
            <h3 class="card-title">Timer Settings</h3>

            {/* Timer Name */}
            <div class="form-control">
              <label class="label">
                <span class="label-text">Timer Name</span>
              </label>
              <input
                type="text"
                placeholder="Enter timer name"
                class="input input-bordered"
                value={timerName()}
                onInput={(e) => setTimerName(e.target.value)}
                disabled={isRunning()}
              />
            </div>

            {/* Pomodoro Toggle */}
            <div class="form-control">
              <label class="label cursor-pointer">
                <span class="label-text">Pomodoro Mode</span>
                <input
                  type="checkbox"
                  class="toggle toggle-primary"
                  checked={isPomodoro()}
                  onChange={(e) => setIsPomodoro(e.target.checked)}
                  disabled={isRunning()}
                />
              </label>
              <label class="label">
                <span class="label-text-alt">
                  {isPomodoro()
                    ? '25min work, 5min break (15min every 4th)'
                    : 'Custom duration timer'}
                </span>
              </label>
            </div>

            {/* Duration (only for non-Pomodoro) */}
            {!isPomodoro() && (
              <div class="form-control">
                <label class="label">
                  <span class="label-text">Duration (minutes)</span>
                </label>
                <input
                  type="number"
                  min="1"
                  max="180"
                  class="input input-bordered"
                  value={duration()}
                  onInput={(e) => setDuration(parseInt(e.target.value) || 1)}
                  disabled={isRunning()}
                />
              </div>
            )}
          </div>
        </div>

        {/* Timer Display */}
        <div class="card bg-base-100 shadow-xl overflow-hidden">
          <div class="card-body items-center text-center p-0">
            <div class="w-full p-6 pb-0">
              <h3 class="card-title text-lg mb-2 justify-center">
                {timerName()}
                {isPomodoro() && (
                  <div class="badge badge-primary">Pomodoro #{pomodoroCount() + 1}</div>
                )}
              </h3>

              {isPomodoro() && (
                <div class="mb-2">
                  <div class={`badge ${currentPhase() === 'work' ? 'badge-error' : 'badge-success'}`}>
                    {currentPhase() === 'work' ? 'Work Time' : 'Break Time'}
                  </div>
                </div>
              )}
            </div>

            {/* Time Display with Background Progress */}
            <div class="relative w-full" style={{ overflow: 'hidden' }}>
              {/* Background Progress Bar */}
              <div
                class="absolute inset-0 transition-all duration-1000 ease-linear"
                style={{
                  background: getTimerColor(),
                  width: `${getTimerPercentage()}%`,
                  opacity: 0.6
                }}
              />

              {/* Time Text */}
              <div class="relative text-7xl font-mono font-bold py-6 px-6">
                {isRunning() || isPaused()
                  ? formatTime(timeRemaining())
                  : formatTime(isPomodoro() ? WORK_DURATION : duration() * 60)
                }
              </div>
            </div>

            {/* Controls */}
            <div class="flex gap-2 w-full px-6 pb-6">
              {!isRunning() && !isPaused() && (
                <button class="btn btn-primary flex-1" onClick={startTimer}>
                  Start
                </button>
              )}
              {(isRunning() || isPaused()) && (
                <>
                  {!isPaused() ? (
                    <button class="btn btn-warning flex-1" onClick={pauseTimer}>
                      Pause
                    </button>
                  ) : (
                    <button class="btn btn-primary flex-1" onClick={startTimer}>
                      Resume
                    </button>
                  )}
                  <button class="btn btn-error flex-1" onClick={stopTimer}>
                    Stop
                  </button>
                </>
              )}
              {pomodoroCount() > 0 && (
                <button class="btn btn-ghost flex-1" onClick={resetTimer}>
                  Reset Count
                </button>
              )}
            </div>
          </div>
        </div>

        {/* Pomodoro Progress */}
        {isPomodoro() && pomodoroCount() > 0 && (
          <div class="card bg-base-100 shadow-xl mt-4">
            <div class="card-body">
              <h3 class="card-title text-sm">Pomodoro Progress</h3>
              <div class="flex gap-2 flex-wrap">
                {Array.from({ length: pomodoroCount() }).map((_, i) => (
                  <div class="w-12 h-12 rounded-full bg-primary flex items-center justify-center text-primary-content font-bold">
                    {i + 1}
                  </div>
                ))}
              </div>
            </div>
          </div>
        )}

        {/* Instructions */}
        <div class="alert alert-info mt-4">
          <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" class="stroke-current shrink-0 w-6 h-6"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path></svg>
          <div>
            <h3 class="font-bold">Timer Overlay</h3>
            <div class="text-xs">
              Add timer overlay in OBS: <code class="bg-base-300 px-2 py-0.5 rounded">http://localhost:3001/overlay/timer</code>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
