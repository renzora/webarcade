import { render } from 'solid-js/web';
import { createSignal, createEffect, onCleanup, Show } from 'solid-js';
import '@/index.css';

const WEBARCADE_API = 'http://localhost:3001';
const WEBSOCKET_URL = 'ws://localhost:3002';

function TickerOverlay() {
  const [messages, setMessages] = createSignal([]);
  const [tickerText, setTickerText] = createSignal('');
  const [currentTime, setCurrentTime] = createSignal('');
  const [streamDays, setStreamDays] = createSignal(0);
  const [tickerSpeed, setTickerSpeed] = createSignal(30);

  let ws = null;

  // Load enabled ticker messages
  const loadMessages = async () => {
    try {
      const response = await fetch(`${WEBARCADE_API}/api/ticker/messages/enabled`);
      const data = await response.json();
      setMessages(data);

      // Create scrolling text from messages
      if (data.length > 0) {
        const text = '💎          ' + data.map(m => m.message).join('          💎          ');
        setTickerText(text);
      } else {
        setTickerText('');
      }
    } catch (error) {
      console.error('Failed to load ticker messages:', error);
    }
  };

  // Load stream status config
  const loadStatusConfig = async () => {
    try {
      const response = await fetch(`${WEBARCADE_API}/api/status/config`);
      const data = await response.json();

      // Calculate days from start date if available
      if (data.stream_start_date) {
        const startDate = new Date(data.stream_start_date);
        const today = new Date();
        const daysSince = Math.floor((today - startDate) / (1000 * 60 * 60 * 24));
        setStreamDays(daysSince);
      } else {
        setStreamDays(0);
      }

      setTickerSpeed(data.ticker_speed || 30);
    } catch (error) {
      console.error('Failed to load status config:', error);
    }
  };

  // Update current time every second
  const updateTime = () => {
    const now = new Date();
    let hours = now.getHours();
    const ampm = hours >= 12 ? 'PM' : 'AM';
    hours = hours % 12;
    hours = hours ? hours : 12; // the hour '0' should be '12'
    const minutes = now.getMinutes().toString().padStart(2, '0');
    const seconds = now.getSeconds().toString().padStart(2, '0');
    setCurrentTime(`${hours}:${minutes}:${seconds} ${ampm}`);
  };

  // Connect to WebSocket
  const connectWebSocket = () => {
    ws = new WebSocket(WEBSOCKET_URL);

    ws.onopen = () => {
      console.log('✅ WebSocket connected');
    };

    ws.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data);

        // Handle status config updates
        if (data.type === 'status_config_update' && data.config) {
          console.log('📡 Received status config update:', data.config);

          // Update ticker speed
          if (data.config.ticker_speed) {
            setTickerSpeed(data.config.ticker_speed);
          }

          // Calculate days from start date if available
          if (data.config.stream_start_date) {
            const startDate = new Date(data.config.stream_start_date);
            const today = new Date();
            const daysSince = Math.floor((today - startDate) / (1000 * 60 * 60 * 24));
            setStreamDays(daysSince);
          } else {
            setStreamDays(0);
          }
        }

        // Handle ticker messages updates
        if (data.type === 'ticker_messages_update') {
          console.log('📡 Received ticker messages update - reloading messages');
          loadMessages();
        }
      } catch (err) {
        console.error('Failed to parse WebSocket message:', err);
      }
    };

    ws.onerror = (error) => {
      console.error('❌ WebSocket error:', error);
    };

    ws.onclose = () => {
      console.log('🔌 WebSocket disconnected, reconnecting in 3 seconds...');
      setTimeout(connectWebSocket, 3000);
    };
  };

  createEffect(() => {
    loadMessages();
    loadStatusConfig();
    updateTime();
    connectWebSocket();

    // Update time every second
    const timeInterval = setInterval(updateTime, 1000);

    onCleanup(() => {
      clearInterval(timeInterval);
      if (ws) {
        ws.close();
      }
    });
  });

  return (
    <div class="fixed inset-0 pointer-events-none overflow-hidden">
      {/* Ticker Bar at Bottom */}
      <div class="ticker-bar absolute bottom-0 left-0 right-0 bg-gradient-to-r from-purple-900/95 via-blue-900/95 to-purple-900/95 backdrop-blur-sm border-t-4 border-black/20 shadow-lg h-12 flex items-center overflow-hidden">
        {/* Status Elements - Left Side */}
        <div class="flex items-center gap-2 px-3 flex-shrink-0">
          {/* LIVE 24/7 Indicator */}
          <div class="bg-gradient-to-r from-red-600 to-red-700 px-2 py-1 rounded shadow-xl border border-red-400 flex items-center gap-1.5">
            <div class="relative">
              <div class="w-2 h-2 bg-red-500 rounded-full animate-pulse shadow-lg"></div>
              <div class="absolute inset-0 w-2 h-2 bg-red-400 rounded-full animate-ping"></div>
            </div>
            <span class="text-white font-bold text-sm tracking-wide drop-shadow-lg">
              LIVE 24/7
            </span>
          </div>

          {/* Days Since Stream Started */}
          <div class="bg-gradient-to-r from-cyan-900/95 to-teal-900/95 px-2 py-1 rounded shadow-lg border border-cyan-500">
            <div class="text-sm font-bold text-white drop-shadow-lg">
              Day {streamDays()}
            </div>
          </div>

          {/* Current Time with UK Flag */}
          <div class="bg-gradient-to-r from-indigo-900/95 to-blue-900/95 px-2 py-1 rounded shadow-lg border border-indigo-500 flex items-center gap-1.5">
            <svg width="20" height="15" viewBox="0 0 60 30" xmlns="http://www.w3.org/2000/svg">
              <clipPath id="s"><path d="M0,0 v30 h60 v-30 z"/></clipPath>
              <clipPath id="t"><path d="M30,15 h30 v15 z v-15 h-30 z h-30 v15 z v-15 h30 z"/></clipPath>
              <g clip-path="url(#s)">
                <path d="M0,0 v30 h60 v-30 z" fill="#012169"/>
                <path d="M0,0 L60,30 M60,0 L0,30" stroke="#fff" stroke-width="6"/>
                <path d="M0,0 L60,30 M60,0 L0,30" clip-path="url(#t)" stroke="#C8102E" stroke-width="4"/>
                <path d="M30,0 v30 M0,15 h60" stroke="#fff" stroke-width="10"/>
                <path d="M30,0 v30 M0,15 h60" stroke="#C8102E" stroke-width="6"/>
              </g>
            </svg>
            <span class="text-sm font-mono font-bold text-white tracking-wide drop-shadow-lg">
              {currentTime()}
            </span>
          </div>

          {/* Separator */}
          <div class="w-px h-6 bg-purple-400/50"></div>
        </div>

        {/* Scrolling Ticker Text - Right Side */}
        <Show when={tickerText()}>
          <div class="flex-1 overflow-hidden relative">
            <div class="ticker-scroll">
              <span class="ticker-text text-white font-bold text-lg whitespace-nowrap px-3">
                {tickerText()}
              </span>
              <span class="ticker-text text-white font-bold text-lg whitespace-nowrap px-3">
                {tickerText()}
              </span>
            </div>
          </div>
        </Show>
      </div>

      <style>{`
        @keyframes ticker-scroll {
          0% {
            transform: translateX(0);
          }
          100% {
            transform: translateX(-50%);
          }
        }

        @keyframes shine {
          0% {
            left: -150%;
          }
          100% {
            left: 250%;
          }
        }

        @keyframes pulse {
          0%, 100% {
            opacity: 1;
          }
          50% {
            opacity: 0.5;
          }
        }

        @keyframes ping {
          75%, 100% {
            transform: scale(2);
            opacity: 0;
          }
        }

        .ticker-bar {
          position: relative;
        }

        .ticker-bar::before {
          content: '';
          position: absolute;
          top: 0;
          left: -150%;
          width: 150%;
          height: 100%;
          background: linear-gradient(
            90deg,
            transparent,
            rgba(255, 255, 255, 0.15) 50%,
            transparent
          );
          animation: shine 6s infinite;
          pointer-events: none;
        }

        .ticker-scroll {
          animation: ticker-scroll ${tickerSpeed()}s linear infinite;
          display: inline-flex;
          white-space: nowrap;
        }

        .ticker-text {
          text-shadow: 2px 2px 4px rgba(0, 0, 0, 0.8);
          display: inline-block;
        }

        .animate-pulse {
          animation: pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite;
        }

        .animate-ping {
          animation: ping 1s cubic-bezier(0, 0, 0.2, 1) infinite;
        }

        /* Pause animation on hover */
        .ticker-scroll:hover {
          animation-play-state: paused;
        }
      `}</style>
    </div>
  );
}

// Only render when used as standalone (for OBS browser sources)
if (document.getElementById('root')) {
  render(() => <TickerOverlay />, document.getElementById('root'));
}
