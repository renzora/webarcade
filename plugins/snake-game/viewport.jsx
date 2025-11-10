import { createSignal, onMount, onCleanup } from 'solid-js';

export default function SnakeGameViewport() {
  const [score, setScore] = createSignal(0);
  const [highScore, setHighScore] = createSignal(0);
  const [gameOver, setGameOver] = createSignal(false);
  const [isPaused, setIsPaused] = createSignal(false);
  const [gameStarted, setGameStarted] = createSignal(false);

  let canvas;
  let gameLoop;
  let resizeObserver;

  const GRID_SIZE = 30;
  const INITIAL_SPEED = 150;

  let snake = [{ x: 15, y: 15 }];
  let direction = { x: 1, y: 0 };
  let nextDirection = { x: 1, y: 0 };
  let food = { x: 20, y: 15 };
  let speed = INITIAL_SPEED;
  let cellSize = 20;

  const generateFood = () => {
    let newFood;
    do {
      newFood = {
        x: Math.floor(Math.random() * GRID_SIZE),
        y: Math.floor(Math.random() * GRID_SIZE)
      };
    } while (snake.some(segment => segment.x === newFood.x && segment.y === newFood.y));
    food = newFood;
  };

  const resizeCanvas = () => {
    if (!canvas) return;

    const parent = canvas.parentElement;
    const width = parent.clientWidth;
    const height = parent.clientHeight;

    canvas.width = width;
    canvas.height = height;

    // Calculate cell size based on viewport
    cellSize = Math.min(width / GRID_SIZE, height / GRID_SIZE);

    draw();
  };

  const draw = () => {
    if (!canvas) return;
    const ctx = canvas.getContext('2d');

    // Clear canvas
    ctx.fillStyle = '#000000';
    ctx.fillRect(0, 0, canvas.width, canvas.height);

    // Calculate offset to center the grid
    const gridWidth = GRID_SIZE * cellSize;
    const gridHeight = GRID_SIZE * cellSize;
    const offsetX = (canvas.width - gridWidth) / 2;
    const offsetY = (canvas.height - gridHeight) / 2;

    // Draw grid
    ctx.strokeStyle = '#1a1a1a';
    ctx.lineWidth = 1;
    for (let i = 0; i <= GRID_SIZE; i++) {
      ctx.beginPath();
      ctx.moveTo(offsetX + i * cellSize, offsetY);
      ctx.lineTo(offsetX + i * cellSize, offsetY + gridHeight);
      ctx.stroke();
      ctx.beginPath();
      ctx.moveTo(offsetX, offsetY + i * cellSize);
      ctx.lineTo(offsetX + gridWidth, offsetY + i * cellSize);
      ctx.stroke();
    }

    // Draw snake with gradient
    snake.forEach((segment, index) => {
      const alpha = 1 - (index / snake.length) * 0.3;
      ctx.fillStyle = index === 0 ? '#4ade80' : `rgba(34, 197, 94, ${alpha})`;
      ctx.fillRect(
        offsetX + segment.x * cellSize + 1,
        offsetY + segment.y * cellSize + 1,
        cellSize - 2,
        cellSize - 2
      );

      // Draw eyes on head
      if (index === 0) {
        ctx.fillStyle = '#000000';
        const eyeSize = Math.max(3, cellSize * 0.15);
        const eyeOffset = cellSize * 0.25;

        if (direction.x === 1) { // Moving right
          ctx.fillRect(offsetX + segment.x * cellSize + cellSize - eyeOffset, offsetY + segment.y * cellSize + eyeOffset - 2, eyeSize, eyeSize);
          ctx.fillRect(offsetX + segment.x * cellSize + cellSize - eyeOffset, offsetY + segment.y * cellSize + cellSize - eyeOffset - 2, eyeSize, eyeSize);
        } else if (direction.x === -1) { // Moving left
          ctx.fillRect(offsetX + segment.x * cellSize + eyeOffset - 2, offsetY + segment.y * cellSize + eyeOffset - 2, eyeSize, eyeSize);
          ctx.fillRect(offsetX + segment.x * cellSize + eyeOffset - 2, offsetY + segment.y * cellSize + cellSize - eyeOffset - 2, eyeSize, eyeSize);
        } else if (direction.y === -1) { // Moving up
          ctx.fillRect(offsetX + segment.x * cellSize + eyeOffset - 2, offsetY + segment.y * cellSize + eyeOffset - 2, eyeSize, eyeSize);
          ctx.fillRect(offsetX + segment.x * cellSize + cellSize - eyeOffset - 2, offsetY + segment.y * cellSize + eyeOffset - 2, eyeSize, eyeSize);
        } else { // Moving down
          ctx.fillRect(offsetX + segment.x * cellSize + eyeOffset - 2, offsetY + segment.y * cellSize + cellSize - eyeOffset, eyeSize, eyeSize);
          ctx.fillRect(offsetX + segment.x * cellSize + cellSize - eyeOffset - 2, offsetY + segment.y * cellSize + cellSize - eyeOffset, eyeSize, eyeSize);
        }
      }
    });

    // Draw food with glow effect
    const foodRadius = Math.max(1, cellSize / 2 - 2);
    const gradient = ctx.createRadialGradient(
      offsetX + food.x * cellSize + cellSize / 2,
      offsetY + food.y * cellSize + cellSize / 2,
      0,
      offsetX + food.x * cellSize + cellSize / 2,
      offsetY + food.y * cellSize + cellSize / 2,
      Math.max(1, cellSize / 2)
    );
    gradient.addColorStop(0, '#ef4444');
    gradient.addColorStop(1, '#991b1b');

    ctx.fillStyle = gradient;
    ctx.beginPath();
    ctx.arc(
      offsetX + food.x * cellSize + cellSize / 2,
      offsetY + food.y * cellSize + cellSize / 2,
      foodRadius,
      0,
      Math.PI * 2
    );
    ctx.fill();
  };

  const update = () => {
    if (gameOver() || isPaused() || !gameStarted()) return;

    direction = nextDirection;

    const head = {
      x: snake[0].x + direction.x,
      y: snake[0].y + direction.y
    };

    // Check wall collision
    if (head.x < 0 || head.x >= GRID_SIZE || head.y < 0 || head.y >= GRID_SIZE) {
      endGame();
      return;
    }

    // Check self collision
    if (snake.some(segment => segment.x === head.x && segment.y === head.y)) {
      endGame();
      return;
    }

    snake.unshift(head);

    // Check food collision
    if (head.x === food.x && head.y === food.y) {
      setScore(score() + 10);
      localStorage.setItem('snakeCurrentScore', (score() + 10).toString());
      generateFood();
      speed = Math.max(50, INITIAL_SPEED - Math.floor(score() / 50) * 10);
    } else {
      snake.pop();
    }

    // Update current length in localStorage
    localStorage.setItem('snakeCurrentLength', snake.length.toString());
  };

  const endGame = () => {
    setGameOver(true);
    if (score() > highScore()) {
      setHighScore(score());
      localStorage.setItem('snakeHighScore', score().toString());
    }
    clearInterval(gameLoop);
  };

  const startGame = () => {
    snake = [{ x: 15, y: 15 }];
    direction = { x: 1, y: 0 };
    nextDirection = { x: 1, y: 0 };
    speed = INITIAL_SPEED;
    setScore(0);
    setGameOver(false);
    setIsPaused(false);
    setGameStarted(true);
    generateFood();

    // Reset current score and length in localStorage
    localStorage.setItem('snakeCurrentScore', '0');
    localStorage.setItem('snakeCurrentLength', '1');

    clearInterval(gameLoop);
    gameLoop = setInterval(() => {
      update();
      draw();
    }, speed);
  };

  const togglePause = () => {
    if (!gameStarted() || gameOver()) return;
    setIsPaused(!isPaused());
  };

  const handleKeyPress = (e) => {
    if (!gameStarted() || gameOver()) {
      if (e.key === 'Enter' || e.key === ' ') {
        startGame();
      }
      return;
    }

    if (e.key === 'p' || e.key === 'P' || e.key === ' ') {
      togglePause();
      return;
    }

    const keyDirections = {
      'ArrowUp': { x: 0, y: -1 },
      'ArrowDown': { x: 0, y: 1 },
      'ArrowLeft': { x: -1, y: 0 },
      'ArrowRight': { x: 1, y: 0 },
      'w': { x: 0, y: -1 },
      's': { x: 0, y: 1 },
      'a': { x: -1, y: 0 },
      'd': { x: 1, y: 0 }
    };

    const newDirection = keyDirections[e.key];
    if (newDirection) {
      e.preventDefault();
      // Prevent reversing direction
      if (newDirection.x !== -direction.x || newDirection.y !== -direction.y) {
        nextDirection = newDirection;
      }
    }
  };

  onMount(() => {
    const savedHighScore = localStorage.getItem('snakeHighScore');
    if (savedHighScore) {
      setHighScore(parseInt(savedHighScore));
    }

    window.addEventListener('keydown', handleKeyPress);

    // Set up resize observer
    resizeCanvas();
    resizeObserver = new ResizeObserver(resizeCanvas);
    resizeObserver.observe(canvas.parentElement);

    draw();
  });

  onCleanup(() => {
    clearInterval(gameLoop);
    window.removeEventListener('keydown', handleKeyPress);
    if (resizeObserver) {
      resizeObserver.disconnect();
    }
  });

  return (
    <div class="w-full h-full bg-black relative">
      {/* Game Canvas Container */}
      <canvas
        ref={canvas}
        class="w-full h-full"
        style="image-rendering: pixelated;"
      />

      {/* Overlay Messages */}
      {!gameStarted() && (
        <div class="absolute inset-0 flex items-center justify-center bg-black/90">
          <div class="text-center p-8">
            <div class="text-6xl mb-4">🐍</div>
            <h2 class="text-2xl font-bold mb-4">Press <kbd class="kbd kbd-sm">Space</kbd> to Start</h2>
          </div>
        </div>
      )}

      {gameOver() && (
        <div class="absolute inset-0 flex items-center justify-center bg-black/90">
          <div class="text-center p-8">
            <div class="text-6xl mb-4">💀</div>
            <h2 class="text-3xl font-bold text-error mb-4">Game Over!</h2>
            <p class="text-xl mb-2">Score: <span class="text-success font-bold">{score()}</span></p>
            {score() === highScore() && score() > 0 && (
              <p class="text-lg text-warning mb-4">🏆 New High Score!</p>
            )}
            <p class="text-sm opacity-50 mt-4">Press <kbd class="kbd kbd-sm">Space</kbd> to play again</p>
          </div>
        </div>
      )}

      {isPaused() && !gameOver() && (
        <div class="absolute inset-0 flex items-center justify-center bg-black/90">
          <div class="text-center p-8">
            <div class="text-6xl mb-4">⏸️</div>
            <h2 class="text-3xl font-bold text-warning mb-4">Paused</h2>
          </div>
        </div>
      )}
    </div>
  );
}
