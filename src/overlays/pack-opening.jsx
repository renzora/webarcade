import { render } from 'solid-js/web';
import { createSignal, createEffect, onCleanup, Show, For } from 'solid-js';
import '@/index.css';
import { WEBARCADE_WS } from '@/api/bridge';

const DISPLAY_DURATION = 5000;
const SUSPENSE_DURATION = 2000; // 2 seconds of suspense for rare+ cards

// SVG Card Pack Component
function CardPack({ rarity, isOpening }) {
  const rarityColors = {
    common: { primary: '#6b7280', secondary: '#4b5563', accent: '#9ca3af' },
    uncommon: { primary: '#22c55e', secondary: '#16a34a', accent: '#86efac' },
    rare: { primary: '#3b82f6', secondary: '#2563eb', accent: '#93c5fd' },
    epic: { primary: '#a855f7', secondary: '#9333ea', accent: '#d8b4fe' },
    legendary: { primary: '#f97316', secondary: '#ea580c', accent: '#fdba74' },
    mythic: { primary: '#ef4444', secondary: '#dc2626', accent: '#fca5a5' }
  };

  const colors = rarityColors[rarity] || rarityColors.common;

  return (
    <svg width="300" height="400" viewBox="0 0 300 400" class="drop-shadow-2xl">
      <defs>
        <linearGradient id={`packGrad-${rarity}`} x1="0%" y1="0%" x2="100%" y2="100%">
          <stop offset="0%" style={`stop-color:${colors.primary};stop-opacity:1`} />
          <stop offset="100%" style={`stop-color:${colors.secondary};stop-opacity:1`} />
        </linearGradient>
        <linearGradient id={`shine-${rarity}`} x1="0%" y1="0%" x2="100%" y2="0%">
          <stop offset="0%" style="stop-color:rgba(255,255,255,0);stop-opacity:0" />
          <stop offset="50%" style="stop-color:rgba(255,255,255,0.8);stop-opacity:1" />
          <stop offset="100%" style="stop-color:rgba(255,255,255,0);stop-opacity:0" />
        </linearGradient>
        <filter id={`glow-${rarity}`}>
          <feGaussianBlur stdDeviation="5" result="coloredBlur"/>
          <feMerge>
            <feMergeNode in="coloredBlur"/>
            <feMergeNode in="SourceGraphic"/>
          </feMerge>
        </filter>
      </defs>

      {/* Pack body */}
      <rect x="30" y="60" width="240" height="320" rx="15"
            fill={`url(#packGrad-${rarity})`}
            stroke={colors.accent} stroke-width="3"
            class={isOpening ? 'pack-body-opening' : ''} />

      {/* Decorative pattern */}
      <circle cx="150" cy="200" r="60" fill="none" stroke={colors.accent} stroke-width="2" opacity="0.3" />
      <circle cx="150" cy="200" r="45" fill="none" stroke={colors.accent} stroke-width="2" opacity="0.5" />
      <circle cx="150" cy="200" r="30" fill="none" stroke="white" stroke-width="3" opacity="0.7" />

      {/* Star icon in center */}
      <path d="M 150 170 L 160 190 L 182 193 L 166 208 L 170 230 L 150 219 L 130 230 L 134 208 L 118 193 L 140 190 Z"
            fill="white" opacity="0.8" filter={`url(#glow-${rarity})`} />

      {/* Pack lid/flap - animated */}
      <g class={isOpening ? 'pack-lid-opening' : ''} style="transform-origin: 150px 60px">
        <rect x="30" y="20" width="240" height="80" rx="15"
              fill={colors.secondary}
              stroke={colors.accent} stroke-width="3" />
        <rect x="40" y="30" width="220" height="15" rx="5"
              fill={colors.accent} opacity="0.5" />
        <text x="150" y="70" text-anchor="middle"
              fill="white" font-size="24" font-weight="bold" font-family="Arial">
          CARDS
        </text>
      </g>

      {/* Shine effect */}
      <rect x="30" y="60" width="240" height="320" rx="15"
            fill={`url(#shine-${rarity})`}
            opacity="0.3"
            class="pack-shine" />
    </svg>
  );
}

// Particle Component
function Particle({ color, delay, index }) {
  const angle = (index * 360) / 30; // Distribute particles in circle
  const distance = 100 + Math.random() * 100;
  const duration = 1 + Math.random() * 0.5;

  return (
    <div
      class="absolute w-3 h-3 rounded-full"
      style={{
        background: color,
        left: '50%',
        top: '50%',
        animation: `particle-burst ${duration}s ease-out forwards`,
        'animation-delay': `${delay}s`,
        '--particle-angle': `${angle}deg`,
        '--particle-distance': `${distance}px`,
        'box-shadow': `0 0 10px ${color}`
      }}
    />
  );
}

// Suspense Screen Component
function SuspenseScreen({ rarity, onComplete }) {
  const rarityConfig = {
    rare: { color: '#3b82f6', glow: 'rgba(59, 130, 246, 0.6)', label: 'RARE!' },
    epic: { color: '#a855f7', glow: 'rgba(168, 85, 247, 0.6)', label: 'EPIC!!' },
    legendary: { color: '#f97316', glow: 'rgba(249, 115, 22, 0.8)', label: 'LEGENDARY!!!' },
    mythic: { color: '#ef4444', glow: 'rgba(239, 68, 68, 0.9)', label: 'MYTHIC!!!!' }
  };

  const config = rarityConfig[rarity] || rarityConfig.rare;

  createEffect(() => {
    const timer = setTimeout(onComplete, SUSPENSE_DURATION);
    onCleanup(() => clearTimeout(timer));
  });

  return (
    <div class="absolute inset-0 flex items-center justify-center z-50">
      {/* Particles */}
      <For each={Array.from({ length: 30 })}>
        {(_, i) => <Particle color={config.color} delay={0} index={i()} />}
      </For>

      {/* Suspense text */}
      <div class="suspense-text-reveal relative z-10">
        <h2
          class="text-8xl font-black tracking-wider"
          style={{
            color: config.color,
            'text-shadow': `0 0 40px ${config.glow}, 0 0 80px ${config.glow}`,
            animation: 'suspense-pulse 0.5s ease-in-out infinite'
          }}
        >
          {config.label}
        </h2>
      </div>

      {/* Flash overlay */}
      <div class="absolute inset-0 suspense-flash" style={{ background: config.color }} />
    </div>
  );
}

function PackOpeningOverlay() {
  const [isConnected, setIsConnected] = createSignal(false);
  const [packQueue, setPackQueue] = createSignal([]);
  const [currentPack, setCurrentPack] = createSignal(null);
  const [animationPhase, setAnimationPhase] = createSignal('hidden');
  const [revealedItems, setRevealedItems] = createSignal([]);
  const [showSuspense, setShowSuspense] = createSignal(false);
  const [suspenseRarity, setSuspenseRarity] = createSignal(null);
  const [packOpened, setPackOpened] = createSignal(false);

  let ws;
  let hideTimeout;

  const rarityConfig = {
    common: {
      color: 'from-gray-500 to-gray-700',
      glow: 'rgba(156, 163, 175, 0.6)',
      emoji: '⚪',
      textColor: 'text-gray-300'
    },
    uncommon: {
      color: 'from-green-500 to-green-700',
      glow: 'rgba(34, 197, 94, 0.6)',
      emoji: '🟢',
      textColor: 'text-green-300'
    },
    rare: {
      color: 'from-blue-500 to-blue-700',
      glow: 'rgba(59, 130, 246, 0.6)',
      emoji: '🔵',
      textColor: 'text-blue-300'
    },
    epic: {
      color: 'from-purple-500 to-purple-700',
      glow: 'rgba(168, 85, 247, 0.6)',
      emoji: '🟣',
      textColor: 'text-purple-300'
    },
    legendary: {
      color: 'from-orange-500 to-orange-700',
      glow: 'rgba(249, 115, 22, 0.6)',
      emoji: '🟠',
      textColor: 'text-orange-300'
    },
    mythic: {
      color: 'from-red-500 via-pink-500 to-red-700',
      glow: 'rgba(239, 68, 68, 0.8)',
      emoji: '🔴',
      textColor: 'text-red-300'
    }
  };

  const connectWebSocket = () => {
    ws = new WebSocket(WEBARCADE_WS);

    ws.onopen = () => {
      console.log('[Pack Opening] WebSocket connected');
      setIsConnected(true);
      ws.send(JSON.stringify({ type: 'subscribe', channels: ['pack_opening'] }));
    };

    ws.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data);
        console.log('[Pack Opening] WebSocket message:', data);

        if (data.type === 'pack_opening' && data.pack) {
          handlePackOpening(data.pack);
        }
      } catch (error) {
        console.error('[Pack Opening] WebSocket message error:', error);
      }
    };

    ws.onclose = () => {
      console.log('[Pack Opening] WebSocket disconnected');
      setIsConnected(false);
      setTimeout(connectWebSocket, 3000);
    };

    ws.onerror = (error) => {
      console.error('[Pack Opening] WebSocket error:', error);
    };
  };

  const handlePackOpening = (pack) => {
    console.log('[Pack Opening] New pack:', pack);
    setPackQueue([...packQueue(), pack]);

    if (animationPhase() === 'hidden') {
      showNextPack();
    }
  };

  const showNextPack = () => {
    const queue = packQueue();
    if (queue.length === 0) return;

    const [next, ...rest] = queue;
    setPackQueue(rest);
    setCurrentPack(next);
    setRevealedItems([]);
    setPackOpened(false);
    setShowSuspense(false);
    setAnimationPhase('opening');

    // Find the highest rarity in the pack
    const highestRarity = next.items.reduce((max, item) => {
      const rarityOrder = ['common', 'uncommon', 'rare', 'epic', 'legendary', 'mythic'];
      const maxIndex = rarityOrder.indexOf(max);
      const itemIndex = rarityOrder.indexOf(item.rarity);
      return itemIndex > maxIndex ? item.rarity : max;
    }, 'common');

    const isRareOrAbove = ['rare', 'epic', 'legendary', 'mythic'].includes(highestRarity);

    // Step 1: Show pack for 1 second
    setTimeout(() => {
      setPackOpened(true);
    }, 1000);

    // Step 2: If rare+, show suspense screen
    if (isRareOrAbove) {
      setTimeout(() => {
        setSuspenseRarity(highestRarity);
        setShowSuspense(true);
      }, 1500);

      // Step 3: After suspense, reveal items
      setTimeout(() => {
        setShowSuspense(false);
        revealItems(next.items);
      }, 1500 + SUSPENSE_DURATION);
    } else {
      // For common/uncommon, reveal immediately after pack opens
      setTimeout(() => {
        revealItems(next.items);
      }, 1500);
    }

    if (hideTimeout) clearTimeout(hideTimeout);

    const totalDelay = isRareOrAbove ? 1500 + SUSPENSE_DURATION + 800 : 1500 + 800;
    hideTimeout = setTimeout(() => {
      setAnimationPhase('exiting');

      setTimeout(() => {
        setAnimationPhase('hidden');
        setCurrentPack(null);
        setRevealedItems([]);
        setPackOpened(false);
        setShowSuspense(false);
        setTimeout(() => showNextPack(), 300);
      }, 600);
    }, totalDelay + DISPLAY_DURATION);
  };

  const revealItems = (items) => {
    items.forEach((item, index) => {
      setTimeout(() => {
        setRevealedItems([...revealedItems(), item]);
      }, index * 800); // Slower reveal - 800ms between cards
    });
  };

  createEffect(() => {
    console.log('[Pack Opening] Overlay initialized');
    connectWebSocket();

    // Expose test functions
    window.testPackOpening = () => {
      handlePackOpening({
        username: 'TestUser',
        pack_name: 'Starter Pack',
        items: [
          { name: 'Basic Sword', rarity: 'common', value: 10, drop_rate: 50.0 },
          { name: 'Magic Shield', rarity: 'rare', value: 50, drop_rate: 15.0 },
          { name: 'Dragon Scale', rarity: 'legendary', value: 500, drop_rate: 0.9 },
          { name: 'Health Potion', rarity: 'uncommon', value: 25, drop_rate: 30.0 },
          { name: 'Diamond Ring', rarity: 'epic', value: 250, drop_rate: 4.0 },
          { name: 'Ancient Artifact', rarity: 'mythic', value: 1000, drop_rate: 0.1 }
        ]
      });
    };

    window.testCommonPack = () => {
      handlePackOpening({
        username: 'TestUser',
        pack_name: 'Common Pack',
        items: [
          { name: 'Wooden Sword', rarity: 'common', value: 5 },
          { name: 'Leather Boots', rarity: 'common', value: 8 },
          { name: 'Iron Helmet', rarity: 'uncommon', value: 15 }
        ]
      });
    };

    window.testRarePack = () => {
      handlePackOpening({
        username: 'ProGamer',
        pack_name: 'Rare Pack',
        items: [
          { name: 'Enchanted Bow', rarity: 'uncommon', value: 20 },
          { name: 'Crystal Sword', rarity: 'rare', value: 100 },
          { name: 'Magic Ring', rarity: 'rare', value: 120 }
        ]
      });
    };

    window.testEpicPack = () => {
      handlePackOpening({
        username: 'EpicHunter',
        pack_name: 'Epic Pack',
        items: [
          { name: 'Rare Gem', rarity: 'rare', value: 80 },
          { name: 'Epic Armor', rarity: 'epic', value: 300 },
          { name: 'Epic Weapon', rarity: 'epic', value: 350 }
        ]
      });
    };

    window.testLegendaryPack = () => {
      handlePackOpening({
        username: 'Champion',
        pack_name: 'Legendary Pack',
        items: [
          { name: 'Phoenix Feather', rarity: 'epic', value: 250 },
          { name: 'Dragon Sword', rarity: 'legendary', value: 1000 },
          { name: 'Crown of Kings', rarity: 'legendary', value: 1200 }
        ]
      });
    };

    window.testMythicPack = () => {
      handlePackOpening({
        username: 'GodSlayer',
        pack_name: 'Mythic Pack',
        items: [
          { name: 'Ancient Relic', rarity: 'legendary', value: 800 },
          { name: 'Blade of Eternity', rarity: 'mythic', value: 5000 },
          { name: 'Staff of Cosmos', rarity: 'mythic', value: 6000 }
        ]
      });
    };

    onCleanup(() => {
      if (ws) ws.close();
      if (hideTimeout) clearTimeout(hideTimeout);
    });
  });

  const getRarityConfig = (rarity) => {
    return rarityConfig[rarity] || rarityConfig.common;
  };

  return (
    <div class="fixed inset-0 pointer-events-none bg-transparent">
      <Show when={currentPack()}>
        <style>
          {`
            @keyframes packShake {
              0%, 100% { transform: scale(1) rotate(0deg); }
              25% { transform: scale(1.05) rotate(-3deg); }
              50% { transform: scale(1.1) rotate(3deg); }
              75% { transform: scale(1.05) rotate(-3deg); }
            }

            @keyframes packExplode {
              0% { transform: scale(1); opacity: 1; }
              50% { transform: scale(1.3); opacity: 0.8; }
              100% { transform: scale(0); opacity: 0; }
            }

            /* Pack lid opening animation */
            .pack-lid-opening {
              animation: lidOpen 1s ease-out forwards;
              transform-origin: 150px 60px;
            }

            @keyframes lidOpen {
              0% { transform: rotateX(0deg); }
              100% { transform: rotateX(-120deg) translateY(-30px); }
            }

            /* Pack body reveal */
            .pack-body-opening {
              animation: bodyGlow 1s ease-out forwards;
            }

            @keyframes bodyGlow {
              0%, 100% { filter: brightness(1); }
              50% { filter: brightness(1.5) drop-shadow(0 0 30px currentColor); }
            }

            /* Pack shine animation */
            .pack-shine {
              animation: shine 2s ease-in-out infinite;
            }

            @keyframes shine {
              0% { transform: translateX(-100%); }
              100% { transform: translateX(100%); }
            }

            /* Particle burst */
            @keyframes particle-burst {
              0% {
                transform: translate(-50%, -50%) rotate(var(--particle-angle)) translateX(0) scale(1);
                opacity: 1;
              }
              100% {
                transform: translate(-50%, -50%) rotate(var(--particle-angle)) translateX(var(--particle-distance)) scale(0);
                opacity: 0;
              }
            }

            /* Suspense animations */
            .suspense-text-reveal {
              animation: suspenseZoom 0.5s cubic-bezier(0.34, 1.56, 0.64, 1) forwards;
            }

            @keyframes suspenseZoom {
              0% {
                transform: scale(0) rotate(-10deg);
                opacity: 0;
              }
              100% {
                transform: scale(1) rotate(0deg);
                opacity: 1;
              }
            }

            @keyframes suspense-pulse {
              0%, 100% {
                transform: scale(1);
              }
              50% {
                transform: scale(1.1);
              }
            }

            .suspense-flash {
              animation: flash 0.3s ease-out forwards;
            }

            @keyframes flash {
              0% { opacity: 0.8; }
              100% { opacity: 0; }
            }

            /* Card reveal - slow and dramatic */
            @keyframes itemReveal {
              0% {
                transform: scale(0) rotateY(180deg) translateY(50px);
                opacity: 0;
              }
              50% {
                transform: scale(1.15) rotateY(90deg) translateY(-20px);
                opacity: 0.5;
              }
              100% {
                transform: scale(1) rotateY(0deg) translateY(0);
                opacity: 1;
              }
            }

            @keyframes rarityGlow {
              0%, 100% {
                filter: drop-shadow(0 0 10px var(--glow-color)) brightness(1);
              }
              50% {
                filter: drop-shadow(0 0 30px var(--glow-color)) brightness(1.3);
              }
            }

            @keyframes slideUp {
              0% { transform: translateY(20px); opacity: 0; }
              100% { transform: translateY(0); opacity: 1; }
            }

            .pack-shake {
              animation: packShake 1s ease-in-out infinite;
            }

            .pack-explode {
              animation: packExplode 0.4s ease-out forwards;
            }

            .item-reveal {
              animation: itemReveal 1.2s cubic-bezier(0.68, -0.55, 0.265, 1.55) forwards;
            }

            .rarity-glow {
              animation: rarityGlow 2s ease-in-out infinite;
            }

            .slide-up {
              animation: slideUp 0.4s ease-out forwards;
            }

            .fade-exit {
              animation: fadeOut 0.6s ease-out forwards;
            }

            @keyframes fadeOut {
              0% { opacity: 1; }
              100% { opacity: 0; }
            }
          `}
        </style>

        {/* Background overlay */}
        <div
          class={`absolute inset-0 bg-black/60 backdrop-blur-sm z-40 ${
            animationPhase() === 'exiting' ? 'fade-exit' : ''
          }`}
        />

        {/* Pack opening container */}
        <div class="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 z-50 w-full max-w-4xl px-8">
          {/* Username header */}
          <div class="text-center mb-8 slide-up">
            <h1 class="text-4xl font-bold text-white mb-2" style="text-shadow: 0 0 20px rgba(255,255,255,0.5)">
              {currentPack().username} is opening...
            </h1>
            <h2 class="text-5xl font-bold bg-gradient-to-r from-yellow-400 via-orange-500 to-pink-500 bg-clip-text text-transparent"
                style="text-shadow: 0 0 30px rgba(255,200,0,0.6)">
              {currentPack().pack_name}
            </h2>
          </div>

          {/* SVG Pack Display - show before items are revealed */}
          <Show when={!showSuspense() && revealedItems().length === 0}>
            <div class="flex justify-center mb-12">
              <div class={packOpened() ? '' : 'pack-shake'}>
                <CardPack
                  rarity={currentPack().items.reduce((max, item) => {
                    const rarityOrder = ['common', 'uncommon', 'rare', 'epic', 'legendary', 'mythic'];
                    return rarityOrder.indexOf(item.rarity) > rarityOrder.indexOf(max) ? item.rarity : max;
                  }, 'common')}
                  isOpening={packOpened()}
                />
              </div>
            </div>
          </Show>

          {/* Suspense Screen */}
          <Show when={showSuspense()}>
            <SuspenseScreen
              rarity={suspenseRarity()}
              onComplete={() => {}}
            />
          </Show>

          {/* Items grid */}
          <div class="flex flex-wrap justify-center gap-6 mt-12">
            <Show when={revealedItems().length > 0}>
              {revealedItems().map((item, index) => {
                const config = getRarityConfig(item.rarity);
                return (
                  <div
                    class="item-reveal"
                    style={{
                      'animation-delay': `${index * 0.2}s`,
                      '--glow-color': config.glow
                    }}
                  >
                    <div class={`rarity-glow relative bg-gradient-to-br ${config.color} rounded-2xl p-6 shadow-2xl min-w-[250px]`}
                         style={{ '--glow-color': config.glow }}>
                      {/* Rarity badge */}
                      <div class="absolute -top-3 -right-3 bg-black/80 backdrop-blur-md rounded-full px-4 py-2 border-2 border-white/30">
                        <span class="text-2xl">{config.emoji}</span>
                      </div>

                      {/* Item content */}
                      <div class="text-center">
                        <div class="text-6xl mb-3">🎁</div>
                        <h3 class="text-2xl font-bold text-white mb-2" style="text-shadow: 2px 2px 4px rgba(0,0,0,0.8)">
                          {item.name}
                        </h3>
                        <div class={`text-lg font-semibold ${config.textColor} uppercase tracking-wider mb-2`}>
                          {item.rarity}
                        </div>
                        <Show when={item.drop_rate !== undefined}>
                          <div class="text-sm text-white/70 mb-2">
                            📊 {item.drop_rate}% chance
                          </div>
                        </Show>
                        <div class="text-xl text-white/90 font-bold">
                          💰 {item.value} coins
                        </div>
                      </div>

                      {/* Shine effect */}
                      <div class="absolute inset-0 rounded-2xl overflow-hidden pointer-events-none">
                        <div class="absolute inset-0 bg-gradient-to-tr from-transparent via-white/20 to-transparent transform -skew-x-12 animate-pulse" />
                      </div>
                    </div>
                  </div>
                );
              })}
            </Show>
          </div>

          {/* Total value */}
          <Show when={revealedItems().length === currentPack()?.items.length && revealedItems().length > 0}>
            <div class="text-center mt-8 slide-up" style="animation-delay: 0.5s">
              <div class="inline-block bg-black/80 backdrop-blur-md rounded-2xl px-8 py-4 border-2 border-yellow-500/50 shadow-2xl">
                <div class="text-lg text-gray-300 mb-1">Total Value</div>
                <div class="text-4xl font-bold text-yellow-400" style="text-shadow: 0 0 20px rgba(250,204,21,0.6)">
                  💰 {currentPack().items.reduce((sum, item) => sum + item.value, 0)} coins
                </div>
              </div>
            </div>
          </Show>
        </div>

        {/* Debug info */}
        <div class="absolute bottom-4 right-4 text-white text-sm font-mono bg-black/60 backdrop-blur-md px-4 py-3 rounded-xl border border-white/20 shadow-lg z-60">
          <div class="font-bold mb-1 text-cyan-300">Pack Opening</div>
          <div>Phase: <span class="text-yellow-300">{animationPhase()}</span></div>
          <div>Pack Opened: <span class="text-blue-300">{packOpened() ? 'Yes' : 'No'}</span></div>
          <div>Suspense: <span class="text-orange-300">{showSuspense() ? suspenseRarity() : 'No'}</span></div>
          <div>Items: <span class="text-green-300">{revealedItems().length}/{currentPack()?.items.length}</span></div>
          <div>Queue: <span class="text-purple-300">{packQueue().length}</span></div>
        </div>
      </Show>
    </div>
  );
}

const root = document.getElementById('root');
if (root) {
  render(() => <PackOpeningOverlay />, root);
}
