import { createSignal, onMount, onCleanup } from 'solid-js';
import { IconDeviceGamepad2, IconTrophy, IconChartLine } from '@tabler/icons-solidjs';

export default function SnakeGamePanel() {
  const [score, setScore] = createSignal(0);
  const [highScore, setHighScore] = createSignal(0);
  const [snakeLength, setSnakeLength] = createSignal(1);
  const [gamesPlayed, setGamesPlayed] = createSignal(0);
  const [totalScore, setTotalScore] = createSignal(0);

  onMount(() => {
    const savedHighScore = localStorage.getItem('snakeHighScore');
    const savedGamesPlayed = localStorage.getItem('snakeGamesPlayed');
    const savedTotalScore = localStorage.getItem('snakeTotalScore');

    if (savedHighScore) setHighScore(parseInt(savedHighScore));
    if (savedGamesPlayed) setGamesPlayed(parseInt(savedGamesPlayed));
    if (savedTotalScore) setTotalScore(parseInt(savedTotalScore));

    // Listen for storage changes to update live score
    const handleStorageUpdate = (e) => {
      if (e.key === 'snakeCurrentScore') {
        setScore(parseInt(e.newValue || '0'));
      } else if (e.key === 'snakeCurrentLength') {
        setSnakeLength(parseInt(e.newValue || '1'));
      } else if (e.key === 'snakeHighScore') {
        setHighScore(parseInt(e.newValue || '0'));
      }
    };

    window.addEventListener('storage', handleStorageUpdate);

    // Poll for updates since storage events don't fire in same window
    const interval = setInterval(() => {
      const currentScore = localStorage.getItem('snakeCurrentScore');
      const currentLength = localStorage.getItem('snakeCurrentLength');
      const currentHighScore = localStorage.getItem('snakeHighScore');

      if (currentScore) setScore(parseInt(currentScore));
      if (currentLength) setSnakeLength(parseInt(currentLength));
      if (currentHighScore) setHighScore(parseInt(currentHighScore));
    }, 100);

    onCleanup(() => {
      window.removeEventListener('storage', handleStorageUpdate);
      clearInterval(interval);
    });
  });

  const averageScore = () => {
    if (gamesPlayed() === 0) return 0;
    return Math.round(totalScore() / gamesPlayed());
  };

  return (
    <div class="p-4">
      <div class="flex items-center gap-2 mb-4">
        <IconDeviceGamepad2 size={20} class="text-success" />
        <h2 class="text-lg font-bold">Snake Game</h2>
      </div>

      <div class="space-y-3">
        {/* Current Score */}
        <div class="bg-gradient-to-br from-success/20 to-success/5 rounded-lg p-4">
          <div class="flex items-center gap-2 mb-2">
            <IconDeviceGamepad2 size={18} class="text-success" />
            <span class="text-sm font-medium opacity-70">Current Score</span>
          </div>
          <div class="text-3xl font-bold text-success">{score()}</div>
        </div>

        {/* High Score */}
        <div class="bg-gradient-to-br from-warning/20 to-warning/5 rounded-lg p-4">
          <div class="flex items-center gap-2 mb-2">
            <IconTrophy size={18} class="text-warning" />
            <span class="text-sm font-medium opacity-70">High Score</span>
          </div>
          <div class="text-3xl font-bold text-warning">{highScore()}</div>
        </div>

        {/* Snake Length */}
        <div class="bg-gradient-to-br from-info/20 to-info/5 rounded-lg p-4">
          <div class="flex items-center gap-2 mb-2">
            <span class="text-sm font-medium opacity-70">Snake Length</span>
          </div>
          <div class="text-3xl font-bold text-info">{snakeLength()}</div>
        </div>

        {/* Stats */}
        <div class="bg-base-200 rounded-lg p-4">
          <div class="flex items-center gap-2 mb-3">
            <IconChartLine size={18} class="text-info" />
            <span class="text-sm font-medium opacity-70">Statistics</span>
          </div>

          <div class="space-y-2">
            <div class="flex justify-between items-center">
              <span class="text-xs opacity-60">Games Played</span>
              <span class="text-sm font-semibold">{gamesPlayed()}</span>
            </div>
            <div class="flex justify-between items-center">
              <span class="text-xs opacity-60">Average Score</span>
              <span class="text-sm font-semibold text-info">{averageScore()}</span>
            </div>
            <div class="flex justify-between items-center">
              <span class="text-xs opacity-60">Total Score</span>
              <span class="text-sm font-semibold text-success">{totalScore()}</span>
            </div>
          </div>
        </div>

        {/* Quick Start Button */}
        <button
          class="btn btn-success btn-block gap-2"
          onClick={() => {
            // This would trigger navigation to the snake game viewport
            // The actual implementation depends on your routing system
            const event = new CustomEvent('navigate-to-viewport', {
              detail: { viewport: 'webarcade-snake-game' }
            });
            window.dispatchEvent(event);
          }}
        >
          <IconDeviceGamepad2 size={18} />
          Play Snake
        </button>

        {/* Tips */}
        <div class="bg-base-200/50 rounded-lg p-3">
          <div class="text-xs font-medium mb-2 opacity-70">Pro Tips</div>
          <ul class="text-xs opacity-60 space-y-1">
            <li>• Plan your path ahead</li>
            <li>• Use the edges strategically</li>
            <li>• Don't trap yourself in corners</li>
            <li>• The game speeds up as you score!</li>
          </ul>
        </div>
      </div>
    </div>
  );
}
