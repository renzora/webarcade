import { render } from 'solid-js/web';
import { createSignal, createEffect, onCleanup, For, Show } from 'solid-js';
import '@/index.css';
import { WEBARCADE_WS } from '@/api/bridge';
const MAX_MESSAGES = 50;
const MESSAGE_LIFETIME = 15000; // 15 seconds

// Typewriter component for animated text
function TypewriterText(props) {
  const text = props.text || '';
  const speed = props.speed || 30;
  const [displayedText, setDisplayedText] = createSignal('');
  let timerId = null;
  let currentIndex = 0;

  // Start typing animation
  const typeNextChar = () => {
    if (currentIndex < text.length) {
      setDisplayedText(text.substring(0, currentIndex + 1));
      currentIndex++;
      timerId = setTimeout(typeNextChar, speed);
    }
  };

  // Start typing immediately
  createEffect(() => {
    typeNextChar();
    // Cleanup timeout on unmount
    onCleanup(() => {
      if (timerId) clearTimeout(timerId);
    });
  });

  return <span class="break-all">{displayedText()}</span>;
}

function ChatOverlay() {
  const [isConnected, setIsConnected] = createSignal(false);
  const [messages, setMessages] = createSignal([]);
  let ws;
  let messageId = 0;

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
        // New plugin-based event format
        if (data.event_type === 'twitch.chat_message' && data.payload) {
          handleChatMessage(data.payload);
        }
      } catch (error) {
        console.error('Error parsing message:', error);
      }
    };
  };

  // Parse emotes from IRC tags format
  // Format: "emoteid:start-end,start-end/emoteid2:start-end"
  const parseEmotesFromTags = (emotesTag) => {
    if (!emotesTag) return [];

    try {
      const emotes = [];
      const emoteParts = emotesTag.split('/');

      for (const part of emoteParts) {
        const [emoteId, positions] = part.split(':');
        if (emoteId && positions) {
          emotes.push({
            id: emoteId,
            positions: positions.split(',')
          });
        }
      }

      return emotes;
    } catch (error) {
      console.error('Error parsing emotes:', error);
      return [];
    }
  };

  // Detect if message is a command and extract command name
  const detectCommand = (message) => {
    const trimmed = message.trim();
    if (trimmed.startsWith('!')) {
      const match = trimmed.match(/^!(\w+)/);
      if (match) {
        return match[1].toLowerCase();
      }
    }
    return null;
  };

  const handleChatMessage = (payload) => {
    const command = detectCommand(payload.message);

    // Extract data from IRC tags
    const tags = payload.tags || {};
    const color = tags.color || '#9146FF';

    // Parse emotes from tags (format: "emoteid:start-end,start-end/emoteid:start-end")
    const emotes = parseEmotesFromTags(tags.emotes || '');

    // Parse badges (can be from badges string or tags)
    const badgesStr = payload.badges || tags.badges || '';
    const isBroadcaster = payload.is_broadcaster || badgesStr.includes('broadcaster');
    const isModerator = payload.is_mod || badgesStr.includes('moderator');
    const isVIP = badgesStr.includes('vip');
    const isSubscriber = payload.is_subscriber || badgesStr.includes('subscriber');

    // Debug logging
    console.log('Chat message event:', {
      username: payload.username,
      display_name: payload.display_name,
      message: payload.message,
      tags: tags
    });

    const msg = {
      id: messageId++,
      username: payload.username,
      displayName: payload.display_name || payload.username,
      message: payload.message,
      color: color,
      profilePicture: tags['profile-image-url'] || `https://static-cdn.jtvnw.net/user-default-pictures-uv/ce57700a-def9-11e9-842d-784f43822e80-profile_image-70x70.png`,
      isBroadcaster: isBroadcaster,
      isModerator: isModerator,
      isVIP: isVIP,
      isSubscriber: isSubscriber,
      emotes: emotes,
      messageParts: parseMessageWithEmotes(payload.message, emotes),
      timestamp: Date.now(),
      isCommand: !!command,
      commandName: command,
      locationFlag: null, // TODO: Add location flag support
      isBirthday: false, // TODO: Add birthday support
      level: null, // TODO: Add level support from levels plugin
      current_xp: null,
      xp_for_next_level: null
    };

    setMessages(prev => {
      const newMessages = [msg, ...prev].slice(0, MAX_MESSAGES);
      return newMessages;
    });

    // Add slide-off class before removing message
    setTimeout(() => {
      setMessages(prev => prev.map(m =>
        m.id === msg.id ? { ...m, isRemoving: true } : m
      ));

      // Actually remove after animation completes (0.5s)
      setTimeout(() => {
        setMessages(prev => prev.filter(m => m.id !== msg.id));
      }, 500);
    }, MESSAGE_LIFETIME);
  };

  const parseMessageWithEmotes = (message, emotes) => {
    if (!emotes || emotes.length === 0) {
      return [{ type: 'text', content: message }];
    }

    const parts = [];
    let lastIndex = 0;

    // Sort emotes by position
    const sortedEmotes = [...emotes].sort((a, b) => {
      const aStart = parseInt(a.positions[0].split('-')[0]);
      const bStart = parseInt(b.positions[0].split('-')[0]);
      return aStart - bStart;
    });

    sortedEmotes.forEach(emote => {
      emote.positions.forEach(position => {
        const [start, end] = position.split('-').map(Number);

        // Add text before emote
        if (start > lastIndex) {
          parts.push({ type: 'text', content: message.substring(lastIndex, start) });
        }

        // Add emote
        parts.push({
          type: 'emote',
          content: message.substring(start, end + 1),
          url: `https://static-cdn.jtvnw.net/emoticons/v2/${emote.id}/default/dark/1.0`
        });

        lastIndex = end + 1;
      });
    });

    // Add remaining text
    if (lastIndex < message.length) {
      parts.push({ type: 'text', content: message.substring(lastIndex) });
    }

    return parts;
  };

  const getBadgeUrl = (badge) => {
    const badgeMap = {
      'broadcaster': 'https://static-cdn.jtvnw.net/badges/v1/5527c58c-fb7d-422d-b71b-f309dcb85cc1/1',
      'moderator': 'https://static-cdn.jtvnw.net/badges/v1/3267646d-33f0-4b17-b3df-f923a41db1d0/1',
      'vip': 'https://static-cdn.jtvnw.net/badges/v1/b817aba4-fad8-49e2-b88a-7cc744dfa6ec/1',
      'subscriber': 'https://static-cdn.jtvnw.net/badges/v1/5d9f2208-5dd8-11e7-8513-2ff4adfae661/1',
    };
    return badgeMap[badge] || null;
  };

  // Get emoji/icon for specific commands
  const getCommandIcon = (commandName) => {
    const iconMap = {
      'dice': '🎲',
      'task': '✅',
      'tasks': '✅',
      'todo': '✅',
      'todos': '✅',
      'counter': '📊',
      'count': '📊',
      'tts': '🔊',
      'hue': '💡',
      'watchtime': '⏱️',
      'uptime': '📺',
      'timer': '⏲️',
      'joke': '😂',
      'dadjoke': '👨',
      '8ball': '🔮',
      'eightball': '🔮',
      'quote': '💭',
      'roast': '🔥',
      'yomomma': '🤣',
      'ymj': '🤣',
    };
    return iconMap[commandName] || '⚡';
  };

  createEffect(() => {
    connectWebSocket();
    onCleanup(() => ws?.close());
  });

  return (
    <div class="fixed inset-0 pointer-events-none overflow-hidden font-sans bg-transparent">
      {/* Chat Messages - Twitch Style */}
      <div class="absolute bottom-0 left-0 w-[420px] max-h-screen flex flex-col-reverse gap-1 p-2">
        <For each={messages()}>
          {(msg) => (
            <div
              class={`flex gap-2 px-3 py-2 rounded-md backdrop-blur-sm border-l-2 ${
                msg.isRemoving
                  ? 'animate-[slideOutRight_0.5s_ease-in_forwards]'
                  : 'animate-[slideDownBounce_0.5s_ease-out]'
              } ${
                msg.isCommand
                  ? 'command-glow bg-gradient-to-r from-purple-900/90 to-black/90'
                  : 'bg-black/90'
              } ${
                msg.commandName === 'dice' ? 'dice-command' : ''
              }`}
              style={{ 'border-left-color': msg.color || '#9146FF' }}
            >
              {/* Profile Picture with XP Ring */}
              <div class="flex-shrink-0 relative">
                {/* XP Progress Ring */}
                <Show when={msg.level && msg.current_xp !== null && msg.xp_for_next_level !== null}>
                  {(() => {
                    const progress = Math.max(0.02, Math.min(1, (msg.current_xp || 0) / (msg.xp_for_next_level || 1)));
                    const circumference = 2 * Math.PI * 22;
                    const dashOffset = circumference * (1 - progress);
                    const gradientId = `xp-gradient-${msg.id}`;

                    return (
                      <svg class="absolute -inset-1 w-12 h-12" style="transform: rotate(-90deg)">
                        {/* Background ring */}
                        <circle
                          cx="24"
                          cy="24"
                          r="22"
                          fill="none"
                          stroke="rgba(0, 0, 0, 0.5)"
                          stroke-width="2.5"
                        />
                        {/* Progress ring with gradient */}
                        <circle
                          cx="24"
                          cy="24"
                          r="22"
                          fill="none"
                          stroke={`url(#${gradientId})`}
                          stroke-width="2.5"
                          stroke-linecap="round"
                          stroke-dasharray={circumference}
                          stroke-dashoffset={dashOffset}
                          class="transition-all duration-500 ease-out animate-[pulseRing_2s_ease-in-out_infinite]"
                        />
                        {/* Define gradient */}
                        <defs>
                          <linearGradient id={gradientId} x1="0%" y1="0%" x2="100%" y2="100%">
                            <stop offset="0%" style="stop-color: #3b82f6; stop-opacity: 1" />
                            <stop offset="50%" style="stop-color: #a855f7; stop-opacity: 1" />
                            <stop offset="100%" style="stop-color: #ec4899; stop-opacity: 1" />
                          </linearGradient>
                        </defs>
                      </svg>
                    );
                  })()}
                </Show>

                {/* Profile Image */}
                <img
                  src={msg.profilePicture}
                  alt={msg.displayName}
                  class={`w-10 h-10 rounded-full border-2 border-black/50 relative z-10 ${
                    msg.isCommand ? 'animate-[commandBounce_0.6s_ease-out]' : 'animate-[bounceIn_0.3s_ease-out]'
                  }`}
                />

                {/* Command Icon Badge */}
                <Show when={msg.isCommand}>
                  <div class={`absolute -bottom-1 -right-1 text-lg z-20 ${
                    msg.commandName === 'dice' ? 'animate-[diceRoll_1s_ease-in-out]' : 'animate-[iconPop_0.5s_ease-out]'
                  }`}>
                    {getCommandIcon(msg.commandName)}
                  </div>
                </Show>
              </div>

              {/* Message Content */}
              <div class="flex-1 min-w-0">
                {/* Badges and Username */}
                <div class="flex items-center justify-between gap-1 mb-0.5">
                  <div class="flex items-center gap-1">
                    <Show when={msg.isBroadcaster}>
                      <img src={getBadgeUrl('broadcaster')} alt="Broadcaster" class="w-4 h-4" />
                    </Show>
                    <Show when={msg.isModerator}>
                      <img src={getBadgeUrl('moderator')} alt="Moderator" class="w-4 h-4" />
                    </Show>
                    <Show when={msg.isVIP}>
                      <img src={getBadgeUrl('vip')} alt="VIP" class="w-4 h-4" />
                    </Show>
                    <Show when={msg.isSubscriber}>
                      <img src={getBadgeUrl('subscriber')} alt="Subscriber" class="w-4 h-4" />
                    </Show>
                    <span
                      class="font-bold text-sm"
                      style={{ color: msg.color }}
                    >
                      {msg.displayName}
                    </span>
                    {/* Level Badge */}
                    <Show when={msg.level}>
                      <span class="text-[9px] bg-gradient-to-r from-yellow-400 to-yellow-600 text-black font-bold px-1.5 py-0.5 rounded leading-none shadow-lg border border-yellow-700">
                        Lvl {msg.level}
                      </span>
                    </Show>
                    {/* Location Flag */}
                    <Show when={msg.locationFlag}>
                      <Show
                        when={msg.locationFlag.startsWith('http')}
                        fallback={<span class="text-xs">{msg.locationFlag}</span>}
                      >
                        <img
                          src={msg.locationFlag}
                          alt="Flag"
                          class="w-4 h-3 inline-block rounded-sm"
                          title="Location"
                          onError={(e) => {
                            console.error('Flag image failed to load:', msg.locationFlag);
                            e.target.style.display = 'none';
                          }}
                        />
                      </Show>
                    </Show>
                    {/* Birthday Cake */}
                    <Show when={msg.isBirthday}>
                      <span class="text-sm animate-bounce" title="It's their birthday!">🎂</span>
                    </Show>
                  </div>
                </div>

                {/* Message with Emotes and Typewriter Effect */}
                <div class="text-white text-sm flex items-center gap-1 flex-wrap break-words">
                    <For each={msg.messageParts}>
                      {(part) => (
                        <Show
                          when={part.type === 'emote'}
                          fallback={<TypewriterText text={part.content} speed={50} />}
                        >
                          <img
                            src={part.url}
                            alt={part.content}
                            class="inline-block h-7 align-middle animate-[bounceIn_0.3s_ease-out]"
                            title={part.content}
                          />
                        </Show>
                      )}
                    </For>
                  </div>
              </div>
            </div>
          )}
        </For>
      </div>

      <style>{`
        @keyframes slideDownBounce {
          0% {
            opacity: 0;
            transform: translateY(-30px) scale(0.95);
          }
          50% {
            transform: translateY(5px) scale(1.02);
          }
          70% {
            transform: translateY(-2px) scale(0.99);
          }
          100% {
            opacity: 1;
            transform: translateY(0) scale(1);
          }
        }

        @keyframes slideOutRight {
          0% {
            opacity: 1;
            transform: translateX(0);
          }
          100% {
            opacity: 0;
            transform: translateX(120%);
          }
        }

        @keyframes bounceIn {
          0% {
            opacity: 0;
            transform: scale(0);
          }
          50% {
            transform: scale(1.2);
          }
          100% {
            opacity: 1;
            transform: scale(1);
          }
        }

        /* Command-specific animations */
        @keyframes commandBounce {
          0% {
            opacity: 0;
            transform: scale(0) rotate(0deg);
          }
          40% {
            transform: scale(1.3) rotate(10deg);
          }
          60% {
            transform: scale(0.9) rotate(-5deg);
          }
          80% {
            transform: scale(1.1) rotate(3deg);
          }
          100% {
            opacity: 1;
            transform: scale(1) rotate(0deg);
          }
        }

        @keyframes iconPop {
          0% {
            opacity: 0;
            transform: scale(0) rotate(-180deg);
          }
          50% {
            transform: scale(1.4) rotate(10deg);
          }
          70% {
            transform: scale(0.8) rotate(-5deg);
          }
          100% {
            opacity: 1;
            transform: scale(1) rotate(0deg);
          }
        }

        @keyframes diceRoll {
          0% {
            transform: scale(0) rotate(0deg);
            opacity: 0;
          }
          20% {
            transform: scale(1.2) rotate(180deg);
            opacity: 1;
          }
          40% {
            transform: scale(0.9) rotate(360deg);
          }
          60% {
            transform: scale(1.1) rotate(540deg);
          }
          80% {
            transform: scale(0.95) rotate(720deg);
          }
          100% {
            transform: scale(1) rotate(720deg);
            opacity: 1;
          }
        }

        @keyframes commandGlow {
          0%, 100% {
            box-shadow: 0 0 5px rgba(147, 51, 234, 0.5),
                        0 0 10px rgba(147, 51, 234, 0.3),
                        0 0 15px rgba(147, 51, 234, 0.2);
          }
          50% {
            box-shadow: 0 0 10px rgba(147, 51, 234, 0.8),
                        0 0 20px rgba(147, 51, 234, 0.5),
                        0 0 30px rgba(147, 51, 234, 0.3);
          }
        }

        /* Command glow effect */
        .command-glow {
          animation: commandGlow 2s ease-in-out infinite;
          position: relative;
          overflow: visible;
        }

        /* Dice-specific shimmer effect */
        .dice-command::before {
          content: '🎲';
          position: absolute;
          top: 50%;
          right: -30px;
          font-size: 2rem;
          animation: diceFloat 2s ease-in-out infinite;
          opacity: 0.3;
          pointer-events: none;
        }

        @keyframes diceFloat {
          0%, 100% {
            transform: translateY(-50%) rotate(0deg) scale(1);
            opacity: 0.3;
          }
          50% {
            transform: translateY(-70%) rotate(180deg) scale(1.2);
            opacity: 0.6;
          }
        }

        /* Add subtle particle effect for commands */
        .command-glow::after {
          content: '✨';
          position: absolute;
          top: -10px;
          right: -10px;
          font-size: 1rem;
          animation: sparkle 1.5s ease-in-out infinite;
          pointer-events: none;
        }

        @keyframes sparkle {
          0%, 100% {
            opacity: 0;
            transform: scale(0.5) rotate(0deg);
          }
          50% {
            opacity: 1;
            transform: scale(1.2) rotate(180deg);
          }
        }

        /* XP Progress Ring Animations */
        @keyframes pulseRing {
          0%, 100% {
            opacity: 1;
            filter: drop-shadow(0 0 2px rgba(59, 130, 246, 0.5));
          }
          50% {
            opacity: 0.9;
            filter: drop-shadow(0 0 6px rgba(168, 85, 247, 0.8));
          }
        }
      `}</style>
    </div>
  );
}

// Only render when used as standalone (for OBS browser sources)
if (document.getElementById('root')) {
  render(() => <ChatOverlay />, document.getElementById('root'));
}
