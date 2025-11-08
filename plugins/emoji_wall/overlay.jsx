import { render } from 'solid-js/web';
import { createSignal, createEffect, onCleanup, For } from 'solid-js';
import '@/index.css';
import { WEBARCADE_WS } from '@/api/bridge';

function EmojiWallOverlay() {
  const [emotes, setEmotes] = createSignal([]);
  const [settings, setSettings] = createSignal({
    enabled: true,
    gravity: 0.5,
    bounce: 0.7,
    size: 64,
    lifetime: 5000, // milliseconds before vanishing
    maxEmotes: 50
  });

  let ws;
  let nextId = 0;

  // Connect to WebSocket for chat messages
  const connectWebSocket = () => {
    ws = new WebSocket(WEBARCADE_WS);

    ws.onopen = () => {
      console.log('✅ Connected to WebArcade WebSocket (Emoji Wall)');
    };

    ws.onclose = () => {
      console.log('❌ Disconnected from WebSocket (Emoji Wall)');
      setTimeout(connectWebSocket, 3000);
    };

    ws.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data);

        // Listen for Twitch chat messages
        if (data.type === 'twitch_event' && data.event?.type === 'chat_message') {
          if (settings().enabled) {
            const emotes = data.event.emotes || [];
            console.log('[Emote Wall] Chat message received, emotes:', emotes);
            extractAndSpawnEmotes(emotes);
          }
        }

        // Manual emote spawn
        if (data.type === 'emote_wall_spawn') {
          spawnEmote(data.emoteId, data.emoteName);
        }

        // Settings update
        if (data.type === 'emote_wall_update') {
          setSettings(data.settings);
        }

        // Clear all
        if (data.type === 'emote_wall_clear') {
          setEmotes([]);
        }
      } catch (error) {
        console.error('Error parsing WebSocket message:', error);
      }
    };
  };

  // Extract Twitch emotes from chat message and spawn them
  const extractAndSpawnEmotes = (emotes) => {
    if (emotes && emotes.length > 0) {
      console.log('[Emote Wall] Found emotes:', emotes);
      emotes.forEach(emote => {
        // Spawn one for each position the emote appears in the message
        emote.positions.forEach(() => {
          spawnEmote(emote.id, emote.name || 'emote');
        });
      });
    }
  };

  // Spawn a single Twitch emote with bouncing physics
  const spawnEmote = (emoteId, emoteName = 'emote') => {
    const currentSettings = settings();

    // Random starting position near top
    const x = Math.random() * (window.innerWidth - 100) + 50;
    const y = Math.random() * 100;

    // Random initial velocity
    const vx = (Math.random() - 0.5) * 10;
    const vy = Math.random() * 5 + 2;

    const newEmote = {
      id: nextId++,
      emoteId: emoteId,
      emoteName: emoteName,
      emoteUrl: `https://static-cdn.jtvnw.net/emoticons/v2/${emoteId}/default/dark/2.0`,
      x: x,
      y: y,
      vx: vx, // velocity x
      vy: vy, // velocity y
      rotation: Math.random() * 360,
      rotationSpeed: (Math.random() - 0.5) * 10,
      size: currentSettings.size + (Math.random() - 0.5) * 20,
      opacity: 1,
      spawnTime: Date.now()
    };

    console.log('[Emote Wall] Spawning emote:', emoteName, emoteId, 'at', x, y);

    setEmotes(prev => {
      const updated = [...prev, newEmote];
      // Limit max emotes
      if (updated.length > currentSettings.maxEmotes) {
        return updated.slice(-currentSettings.maxEmotes);
      }
      return updated;
    });
  };

  // Physics animation loop
  const animate = () => {
    const currentSettings = settings();
    const now = Date.now();

    setEmotes(prev => {
      return prev
        .map(emote => {
          // Apply gravity
          let newVy = emote.vy + currentSettings.gravity;
          let newVx = emote.vx;
          let newX = emote.x + newVx;
          let newY = emote.y + newVy;
          let newRotation = emote.rotation + emote.rotationSpeed;

          // Bounce off bottom
          if (newY > window.innerHeight - emote.size) {
            newY = window.innerHeight - emote.size;
            newVy = -newVy * currentSettings.bounce;
            // Reduce horizontal velocity on bounce
            newVx *= 0.9;
          }

          // Bounce off top
          if (newY < 0) {
            newY = 0;
            newVy = -newVy * currentSettings.bounce;
          }

          // Bounce off sides
          if (newX < 0) {
            newX = 0;
            newVx = -newVx * currentSettings.bounce;
          }
          if (newX > window.innerWidth - emote.size) {
            newX = window.innerWidth - emote.size;
            newVx = -newVx * currentSettings.bounce;
          }

          // Calculate opacity based on lifetime
          const age = now - emote.spawnTime;
          const fadeStart = currentSettings.lifetime * 0.7;
          let newOpacity = 1;

          if (age > fadeStart) {
            newOpacity = 1 - ((age - fadeStart) / (currentSettings.lifetime - fadeStart));
          }

          return {
            ...emote,
            x: newX,
            y: newY,
            vx: newVx,
            vy: newVy,
            rotation: newRotation,
            opacity: Math.max(0, newOpacity)
          };
        })
        .filter(emote => {
          // Remove emotes that have exceeded lifetime
          const age = now - emote.spawnTime;
          return age < currentSettings.lifetime;
        });
    });
  };

  // Start animation loop
  let animationFrameId;
  createEffect(() => {
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

  return (
    <div class="fixed inset-0 pointer-events-none overflow-hidden bg-transparent">
      <For each={emotes()}>
        {(emote) => (
          <img
            src={emote.emoteUrl}
            alt={emote.emoteName}
            class="absolute"
            style={{
              left: `${emote.x}px`,
              top: `${emote.y}px`,
              transform: `rotate(${emote.rotation}deg)`,
              width: `${emote.size}px`,
              height: `${emote.size}px`,
              opacity: emote.opacity,
              'user-select': 'none',
              'image-rendering': 'pixelated',
              transition: 'opacity 0.3s ease-out'
            }}
          />
        )}
      </For>
    </div>
  );
}

function EmojiWallOverlay_Legacy() {
  return <EmojiWallOverlay />;
}

// Only render when used as standalone (for OBS browser sources)
if (document.getElementById('root')) {
  render(() => <EmojiWallOverlay />, document.getElementById('root'));
}
