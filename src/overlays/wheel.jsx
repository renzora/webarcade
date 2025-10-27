import { render } from 'solid-js/web';
import { createSignal, createEffect, onCleanup, Show, For } from 'solid-js';
import confetti from 'canvas-confetti';
import '@/index.css';
import { WEBARCADE_WS } from '@/api/bridge';

function WheelOverlay() {
  const [isConnected, setIsConnected] = createSignal(false);
  const [isSpinning, setIsSpinning] = createSignal(false);
  const [rotation, setRotation] = createSignal(0);
  const [options, setOptions] = createSignal([]);
  const [winner, setWinner] = createSignal(null);
  const [showWinner, setShowWinner] = createSignal(false);
  const [wheelSize, setWheelSize] = createSignal(700);
  const [wheelPosition, setWheelPosition] = createSignal('hidden'); // 'hidden', 'entering', 'center', 'exiting'

  let ws;
  let confettiIntervals = [];

  // Launch confetti celebration
  const launchConfetti = () => {
    const colors = [
      '#FF6B9D', // Hot pink
      '#FFC75F', // Golden yellow
      '#845EC2', // Purple
      '#00D2FC', // Cyan
      '#FF5E78', // Red
      '#4FFBDF', // Turquoise
      '#FFD93D', // Bright yellow
      '#A8E6CF', // Mint green
      '#FF8066', // Coral
      '#B39CD0', // Lavender
      '#F38181', // Light red
      '#95E1D3', // Aqua
    ];

    const duration = 5000; // 5 seconds
    const animationEnd = Date.now() + duration;

    // Random wind drift that applies to all confetti
    const drift = (Math.random() - 0.5) * 0.3; // -0.15 to 0.15

    const interval = setInterval(() => {
      const timeLeft = animationEnd - Date.now();

      if (timeLeft <= 0) {
        clearInterval(interval);
        return;
      }

      // Launch confetti from random positions at the top
      confetti({
        particleCount: 10,
        startVelocity: 40,
        spread: 60,
        origin: { x: Math.random(), y: 0 },
        colors: colors,
        ticks: 350, // Enough lifetime to reach bottom of screen
        gravity: 1.8, // Fast falling
        drift: drift, // Apply wind
        scalar: 2.5, // Much larger particles
        shapes: ['square', 'circle'], // Mix of shapes
        flat: false, // 3D rotation for more visibility
        disableForReducedMotion: false // Always show confetti
      });
    }, 50); // Launch bursts every 50ms

    confettiIntervals.push(interval);
  };

  // Stop all confetti
  const stopConfetti = () => {
    confettiIntervals.forEach(interval => clearInterval(interval));
    confettiIntervals = [];
  };

  // Connect to WebSocket
  const connectWebSocket = () => {
    ws = new WebSocket(WEBARCADE_WS);

    ws.onopen = () => {
      console.log('✅ Connected to WebArcade (Wheel Overlay)');
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
        console.log('🎡 Wheel overlay received event:', data.type, data);

        // Handle Twitch events - check for wheel_spin type (serde tagged enum)
        if (data.type === 'twitch_event' && data.event?.type === 'wheel_spin') {
          console.log('🎡 Processing wheel spin event:', data.event);
          handleWheelSpin(data.event);
        }
      } catch (error) {
        console.error('Error parsing event:', error);
      }
    };
  };

  const handleWheelSpin = (data) => {
    console.log('Wheel spin event:', data);
    setOptions(data.options || []);
    setWinner(data.winner);
    spinWheel(data.winner, data.options || []);
  };

  const spinWheel = (winnerText, wheelOptions) => {
    if (isSpinning()) return;

    // Start entrance animation - wheel floats up from bottom
    setWheelPosition('entering');
    setIsSpinning(true);
    setShowWinner(false);

    // Calculate winner index
    const winnerIndex = wheelOptions.findIndex(opt => opt.text === winnerText);
    if (winnerIndex === -1) {
      console.error('Winner not found in options');
      setIsSpinning(false);
      setWheelPosition('hidden');
      return;
    }

    // Calculate rotation
    const optionCount = wheelOptions.length;
    const degreesPerOption = 360 / optionCount;
    const targetDegree = (optionCount - winnerIndex) * degreesPerOption - (degreesPerOption / 2);

    // Add multiple full rotations for effect (8-12 spins)
    const fullRotations = 8 + Math.floor(Math.random() * 4);
    const finalRotation = fullRotations * 360 + targetDegree;

    // After entrance animation, wheel is at center and starts spinning
    setTimeout(() => {
      setWheelPosition('center');
      setRotation(finalRotation);
    }, 800); // Entrance animation duration

    // Show winner after spin completes (7 seconds from start of spin + entrance time)
    setTimeout(() => {
      setIsSpinning(false);
      setShowWinner(true);
      launchConfetti();

      // After showing winner for 5 seconds, start exit animation
      setTimeout(() => {
        setShowWinner(false);
        setWheelPosition('exiting');
        stopConfetti();

        // Clean up after exit animation completes
        setTimeout(() => {
          setRotation(0);
          setWheelPosition('hidden');
        }, 1000); // Exit animation duration
      }, 5000);
    }, 7800); // 800ms entrance + 7000ms spin
  };

  createEffect(() => {
    connectWebSocket();
    onCleanup(() => ws?.close());
  });

  return (
    <div class="fixed inset-0 pointer-events-none overflow-hidden font-sans">
      {/* Size Controls */}
      <Show when={options().length > 0 && wheelPosition() === 'center' && !isSpinning()}>
        <div class="fixed bottom-4 left-1/2 -translate-x-1/2 bg-black/80 px-4 py-2 rounded-lg pointer-events-auto flex items-center gap-3 z-50">
          <button
            onClick={() => setWheelSize(Math.max(200, wheelSize() - 50))}
            class="bg-white/20 hover:bg-white/30 px-3 py-1 rounded text-white font-bold"
          >
            -
          </button>
          <span class="text-white text-sm">{wheelSize()}px</span>
          <button
            onClick={() => setWheelSize(Math.min(800, wheelSize() + 50))}
            class="bg-white/20 hover:bg-white/30 px-3 py-1 rounded text-white font-bold"
          >
            +
          </button>
        </div>
      </Show>

      {/* Wheel Container */}
      <Show when={options().length > 0 && wheelPosition() !== 'hidden'}>
        <div
          class="absolute"
          style={{
            width: `${wheelSize()}px`,
            height: `${wheelSize()}px`,
            left: '50%',
            top: '50%',
            'margin-left': `-${wheelSize() / 2}px`,
            'margin-top': `-${wheelSize() / 2}px`,
            perspective: '1500px',
            transform: wheelPosition() === 'center' ? 'translateY(0)' :
                      wheelPosition() === 'exiting' ? 'translateY(calc(50vh + 50%)) rotateX(90deg) rotateZ(360deg)' :
                      'translateY(calc(50vh + 50%))',
            transition: wheelPosition() === 'hidden' ? 'none' :
                       wheelPosition() === 'exiting' ? 'transform 1s cubic-bezier(0.55, 0.055, 0.675, 0.19)' :
                       'transform 0.8s cubic-bezier(0.34, 1.56, 0.64, 1)'
          }}
        >
          {/* Spinning Wheel */}
          <div
            class="absolute inset-0 rounded-full shadow-2xl"
            style={{
              transform: `rotate(${rotation()}deg)`,
              transition: isSpinning() ? 'transform 7s cubic-bezier(0.05, 0.5, 0.1, 1)' : 'none'
            }}
          >
            <svg viewBox="0 0 100 100" class="w-full h-full">
              <For each={options()}>
                {(option, index) => {
                  const optionCount = options().length;
                  const degreesPerOption = 360 / optionCount;
                  const startAngle = index() * degreesPerOption;
                  const endAngle = startAngle + degreesPerOption;

                  // Calculate path for pie slice
                  const startRad = (startAngle - 90) * (Math.PI / 180);
                  const endRad = (endAngle - 90) * (Math.PI / 180);

                  const x1 = 50 + 50 * Math.cos(startRad);
                  const y1 = 50 + 50 * Math.sin(startRad);
                  const x2 = 50 + 50 * Math.cos(endRad);
                  const y2 = 50 + 50 * Math.sin(endRad);

                  const largeArc = degreesPerOption > 180 ? 1 : 0;

                  return (
                    <>
                      <path
                        d={`M 50 50 L ${x1} ${y1} A 50 50 0 ${largeArc} 1 ${x2} ${y2} Z`}
                        fill={option.color}
                        stroke="white"
                        stroke-width="0.5"
                      />
                      <g transform={`rotate(${startAngle + degreesPerOption / 2} 50 50)`}>
                        <text
                          x="50"
                          y="15"
                          fill="white"
                          font-size="3.5"
                          font-weight="bold"
                          text-anchor="middle"
                          transform="rotate(-90 50 15)"
                        >
                          {option.text}
                        </text>
                      </g>
                    </>
                  );
                }}
              </For>
            </svg>
          </div>

          {/* Arcade Lights - Outside the wheel */}
          <For each={Array.from({ length: 32 }, (_, i) => i)}>
            {(i) => {
              const angle = (i * 360) / 32;
              const rad = (angle - 90) * (Math.PI / 180);
              const distance = wheelSize() / 2 + 20; // 20px outside the wheel edge
              const x = wheelSize() / 2 + distance * Math.cos(rad);
              const y = wheelSize() / 2 + distance * Math.sin(rad);
              const colors = ['#FFD700', '#FF1493', '#00FFFF', '#FF6B00', '#FF00FF', '#00FF00'];
              const color = colors[i % colors.length];
              const delay = i * 0.05;

              return (
                <div
                  class="absolute rounded-full"
                  style={{
                    left: `${x}px`,
                    top: `${y}px`,
                    width: '16px',
                    height: '16px',
                    'background-color': color,
                    'box-shadow': `0 0 20px ${color}, 0 0 10px ${color}`,
                    transform: 'translate(-50%, -50%)',
                    animation: `bulbPulse 1s ease-in-out ${delay}s infinite`
                  }}
                />
              );
            }}
          </For>

          {/* Center Circle */}
          <div
            class="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 bg-white rounded-full shadow-lg border-4 border-yellow-400 flex items-center justify-center"
            style={{
              width: `${wheelSize() * 0.16}px`,
              height: `${wheelSize() * 0.16}px`
            }}
          >
            <span class="font-bold text-gray-800" style={{ 'font-size': `${wheelSize() * 0.048}px` }}>SPIN</span>
          </div>

          {/* Pointer */}
          <div
            class="absolute top-0 left-1/2 -translate-x-1/2 drop-shadow-lg z-10"
            style={{
              width: '0',
              height: '0',
              'border-left': `${wheelSize() * 0.04}px solid transparent`,
              'border-right': `${wheelSize() * 0.04}px solid transparent`,
              'border-top': `${wheelSize() * 0.08}px solid #ef4444`,
              transform: 'translateX(-50%) translateY(-8px)'
            }}
          />
        </div>
      </Show>

      {/* Winner Announcement */}
      <Show when={showWinner()}>
        <div class="absolute top-1/4 left-1/2 -translate-x-1/2 flex items-center justify-center animate-[fadeIn_0.5s_ease-out]">
          <div class="bg-gradient-to-br from-yellow-400 via-yellow-500 to-yellow-600 p-2 rounded-3xl shadow-2xl">
            <div class="bg-black/90 rounded-3xl px-12 py-8">
              <div class="text-center">
                <h1 class="text-6xl font-black text-white mb-4 drop-shadow-2xl animate-bounce">
                  🎉 WINNER! 🎉
                </h1>
                <div class="text-5xl font-bold text-transparent bg-clip-text bg-gradient-to-r from-yellow-300 via-yellow-400 to-yellow-500 drop-shadow-xl">
                  {winner()}
                </div>
              </div>
            </div>
          </div>
        </div>
      </Show>

      {/* Arcade Light Animation Styles */}
      <style>{`
        @keyframes bulbPulse {
          0%, 100% {
            opacity: 1;
            transform: scale(1);
          }
          50% {
            opacity: 0.3;
            transform: scale(0.8);
          }
        }
      `}</style>
    </div>
  );
}

// Only render when used as standalone (for OBS browser sources)
if (document.getElementById('root')) {
  render(() => <WheelOverlay />, document.getElementById('root'));
}
