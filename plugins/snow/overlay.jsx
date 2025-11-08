import { render } from 'solid-js/web';
import { createSignal, createEffect, onCleanup, For } from 'solid-js';
import '@/index.css';
import { WEBARCADE_WS } from '@/api/bridge';

function SnowOverlay() {
  const [snowflakes, setSnowflakes] = createSignal([]);
  const [settings, setSettings] = createSignal({
    enabled: true,
    intensity: 50, // number of snowflakes
    speed: 1, // fall speed multiplier
    windSpeed: 0.5, // wind effect
    size: { min: 2, max: 8 }, // snowflake size range
    opacity: { min: 0.3, max: 0.8 }
  });

  let ws;
  let nextId = 0;

  // Connect to WebSocket for settings updates
  const connectWebSocket = () => {
    ws = new WebSocket(WEBARCADE_WS);

    ws.onopen = () => {
      console.log('✅ Connected to WebArcade WebSocket (Snow Overlay)');
    };

    ws.onclose = () => {
      console.log('❌ Disconnected from WebSocket (Snow Overlay)');
      setTimeout(connectWebSocket, 3000);
    };

    ws.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data);

        // Settings update
        if (data.type === 'snow_overlay_update') {
          setSettings(data.settings);
        }

        // Clear all
        if (data.type === 'snow_overlay_clear') {
          setSnowflakes([]);
        }
      } catch (error) {
        console.error('Error parsing WebSocket message:', error);
      }
    };
  };

  // Snowflake emoji variations
  const snowflakeEmojis = ['❄️', '❅', '❆', '✻', '✼', '❋'];

  // Create a new snowflake
  const createSnowflake = () => {
    const currentSettings = settings();
    const size = currentSettings.size.min + Math.random() * (currentSettings.size.max - currentSettings.size.min);
    const opacity = currentSettings.opacity.min + Math.random() * (currentSettings.opacity.max - currentSettings.opacity.min);

    // Depth layers - smaller/faster flakes in front, larger/slower in back
    const depth = Math.random();
    const depthFactor = 0.3 + depth * 0.7; // 0.3 to 1.0

    // Randomly choose between emoji or circle (70% emoji, 30% circle)
    const useEmoji = Math.random() > 0.3;
    const emojiType = snowflakeEmojis[Math.floor(Math.random() * snowflakeEmojis.length)];

    return {
      id: nextId++,
      x: Math.random() * window.innerWidth,
      y: -20,
      size: size * depthFactor,
      opacity: opacity * (0.6 + depth * 0.4),
      speed: (0.3 + Math.random() * 0.8) * currentSettings.speed * depthFactor,
      drift: (Math.random() * 1.5 - 0.75) * depthFactor, // horizontal drift
      wobble: Math.random() * Math.PI * 2, // for sine wave effect
      wobbleSpeed: (0.005 + Math.random() * 0.01) * depthFactor,
      wobbleAmplitude: 20 + Math.random() * 40, // width of wobble
      rotation: Math.random() * 360,
      rotationSpeed: (Math.random() - 0.5) * 2,
      blur: depth < 0.3 ? 0.8 : depth < 0.6 ? 0.5 : 0.3,
      depth: depth,
      useEmoji: useEmoji,
      emoji: emojiType
    };
  };

  // Initialize snowflakes
  const initializeSnowflakes = () => {
    const currentSettings = settings();
    if (!currentSettings.enabled) return;

    const flakes = [];
    for (let i = 0; i < currentSettings.intensity; i++) {
      const flake = createSnowflake();
      // Spread them across the screen initially
      flake.y = Math.random() * window.innerHeight;
      flakes.push(flake);
    }
    setSnowflakes(flakes);
  };

  // Animation loop
  const animate = () => {
    const currentSettings = settings();
    if (!currentSettings.enabled) {
      setSnowflakes([]);
      return;
    }

    setSnowflakes(prev => {
      const updated = prev.map(flake => {
        // Update wobble for horizontal sine wave motion
        const newWobble = flake.wobble + flake.wobbleSpeed;
        const wobbleOffset = Math.sin(newWobble) * flake.wobbleAmplitude * 0.05;

        // Calculate new position with more natural drift
        let newX = flake.x + (flake.drift * currentSettings.windSpeed * 0.5) + wobbleOffset;
        let newY = flake.y + flake.speed;

        // Update rotation
        const newRotation = flake.rotation + flake.rotationSpeed;

        // Wrap around horizontally
        if (newX < -20) newX = window.innerWidth + 20;
        if (newX > window.innerWidth + 20) newX = -20;

        // Reset to top when it falls off screen
        if (newY > window.innerHeight + 20) {
          return createSnowflake();
        }

        return {
          ...flake,
          x: newX,
          y: newY,
          wobble: newWobble,
          rotation: newRotation
        };
      });

      // Maintain the desired number of snowflakes
      while (updated.length < currentSettings.intensity) {
        updated.push(createSnowflake());
      }
      while (updated.length > currentSettings.intensity) {
        updated.pop();
      }

      return updated;
    });
  };

  // Start animation loop
  let animationFrameId;
  createEffect(() => {
    initializeSnowflakes();

    const loop = () => {
      animate();
      animationFrameId = requestAnimationFrame(loop);
    };
    loop();

    onCleanup(() => {
      if (animationFrameId) {
        cancelAnimationFrame(animationFrameId);
      }
    });
  });

  // Connect WebSocket on mount
  createEffect(() => {
    connectWebSocket();

    onCleanup(() => {
      if (ws) {
        ws.close();
      }
    });
  });

  // Handle window resize
  createEffect(() => {
    const handleResize = () => {
      initializeSnowflakes();
    };

    window.addEventListener('resize', handleResize);

    onCleanup(() => {
      window.removeEventListener('resize', handleResize);
    });
  });

  return (
    <div class="fixed inset-0 pointer-events-none overflow-hidden bg-transparent">
      <For each={snowflakes()}>
        {(flake) => (
          <div
            class="absolute"
            style={{
              left: `${flake.x}px`,
              top: `${flake.y}px`,
              opacity: flake.opacity,
              transform: `rotate(${flake.rotation}deg)`,
              'will-change': 'transform'
            }}
          >
            {flake.useEmoji ? (
              /* Emoji Snowflake */
              <div
                style={{
                  'font-size': `${flake.size * 3}px`,
                  'line-height': '1',
                  filter: `drop-shadow(0 0 ${flake.size * 0.5}px rgba(255, 255, 255, ${flake.opacity * 0.8}))`,
                  'text-shadow': `0 0 ${flake.size}px rgba(200, 230, 255, ${flake.opacity})`
                }}
              >
                {flake.emoji}
              </div>
            ) : (
              /* Circle Snowflake */
              <div
                class="snowflake-shape"
                style={{
                  width: `${flake.size}px`,
                  height: `${flake.size}px`,
                  background: 'radial-gradient(circle, white 0%, rgba(255, 255, 255, 0.9) 50%, rgba(255, 255, 255, 0.4) 100%)',
                  'border-radius': '50%',
                  filter: `blur(${flake.blur}px)`,
                  'box-shadow': `0 0 ${flake.size * 0.5}px rgba(255, 255, 255, ${flake.opacity * 0.6})`
                }}
              />
            )}
          </div>
        )}
      </For>
    </div>
  );
}

// Only render when used as standalone (for OBS browser sources)
if (document.getElementById('root')) {
  render(() => <SnowOverlay />, document.getElementById('root'));
}

export default SnowOverlay;
