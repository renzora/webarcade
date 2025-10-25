import { render } from 'solid-js/web';
import { createSignal, createEffect, onCleanup, For, Show } from 'solid-js';
import '@/index.css';

const WEBARCADE_API = 'http://localhost:3001';

function TickerOverlay() {
  const [messages, setMessages] = createSignal([]);
  const [tickerText, setTickerText] = createSignal('');
  const [currentTime, setCurrentTime] = createSignal('');
  const [streamDays, setStreamDays] = createSignal(0);

  // Load enabled ticker messages
  const loadMessages = async () => {
    try {
      const response = await fetch(`${WEBARCADE_API}/api/ticker/messages/enabled`);
      const data = await response.json();
      setMessages(data);

      // Create scrolling text from messages
      if (data.length > 0) {
        const text = data.map(m => m.message).join('   •   ');
        setTickerText(text);
      } else {
        setTickerText('');
      }
    } catch (error) {
      console.error('Failed to load ticker messages:', error);
    }
  };

  // Load stream start days
  const loadStreamDays = async () => {
    try {
      const response = await fetch(`${WEBARCADE_API}/api/status/config`);
      const data = await response.json();
      setStreamDays(data.stream_start_days || 0);
    } catch (error) {
      console.error('Failed to load stream days:', error);
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

  createEffect(() => {
    loadMessages();
    loadStreamDays();
    updateTime();

    // Reload messages every 30 seconds
    const messagesInterval = setInterval(loadMessages, 30000);

    // Update time every second
    const timeInterval = setInterval(updateTime, 1000);

    // Reload stream days every 5 minutes
    const daysInterval = setInterval(loadStreamDays, 300000);

    onCleanup(() => {
      clearInterval(messagesInterval);
      clearInterval(timeInterval);
      clearInterval(daysInterval);
    });
  });

  return (
    <div class="fixed inset-0 pointer-events-none overflow-hidden">
      {/* Ticker Bar at Bottom */}
      <div class="absolute bottom-0 left-0 right-0 bg-gradient-to-r from-purple-900/95 via-blue-900/95 to-purple-900/95 backdrop-blur-sm border-t-2 border-purple-500 shadow-lg h-16 flex items-center overflow-hidden">
        {/* Status Elements - Left Side */}
        <div class="flex items-center gap-3 px-4 flex-shrink-0">
          {/* LIVE 24/7 Indicator */}
          <div class="bg-gradient-to-r from-red-600 to-red-700 px-3 py-1.5 rounded-lg shadow-2xl border border-red-400 flex items-center gap-2">
            <div class="relative">
              <div class="w-2.5 h-2.5 bg-red-500 rounded-full animate-pulse shadow-lg"></div>
              <div class="absolute inset-0 w-2.5 h-2.5 bg-red-400 rounded-full animate-ping"></div>
            </div>
            <span class="text-white font-black text-base tracking-wider drop-shadow-lg">
              LIVE 24/7
            </span>
          </div>

          {/* Days Since Stream Started */}
          <div class="bg-gradient-to-r from-cyan-900/95 to-teal-900/95 px-3 py-1.5 rounded-lg shadow-xl border border-cyan-500">
            <div class="text-lg font-bold text-white drop-shadow-lg">
              Day {streamDays()}
            </div>
          </div>

          {/* Current Time with UK Flag */}
          <div class="bg-gradient-to-r from-indigo-900/95 to-blue-900/95 px-3 py-1.5 rounded-lg shadow-xl border border-indigo-500 flex items-center gap-2">
            <svg width="24" height="18" viewBox="0 0 60 30" xmlns="http://www.w3.org/2000/svg">
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
            <span class="text-lg font-mono font-bold text-white tracking-wider drop-shadow-lg">
              {currentTime()}
            </span>
          </div>

          {/* Separator */}
          <div class="w-px h-8 bg-purple-400/50"></div>
        </div>

        {/* Scrolling Ticker Text - Right Side */}
        <Show when={tickerText()}>
          <div class="flex-1 overflow-hidden">
            <div class="ticker-scroll flex items-center gap-8">
              <span class="ticker-text text-white font-bold text-xl whitespace-nowrap px-4">
                {tickerText()}
              </span>
              <span class="ticker-text text-white font-bold text-xl whitespace-nowrap px-4">
                {tickerText()}
              </span>
              <span class="ticker-text text-white font-bold text-xl whitespace-nowrap px-4">
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
            transform: translateX(-33.333%);
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

        .ticker-scroll {
          animation: ticker-scroll 30s linear infinite;
          display: flex;
        }

        .ticker-text {
          text-shadow: 2px 2px 4px rgba(0, 0, 0, 0.8);
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

render(() => <TickerOverlay />, document.getElementById('root'));
