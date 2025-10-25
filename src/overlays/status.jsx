import { render } from 'solid-js/web';
import { createSignal, createEffect, onCleanup } from 'solid-js';
import '@/index.css';

const WEBARCADE_API = 'http://localhost:3001';

function StatusOverlay() {
  const [currentTime, setCurrentTime] = createSignal('');
  const [streamDays, setStreamDays] = createSignal(0);

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
    loadStreamDays();
    updateTime();

    // Update time every second
    const timeInterval = setInterval(updateTime, 1000);

    // Reload stream days every 5 minutes
    const daysInterval = setInterval(loadStreamDays, 300000);

    onCleanup(() => {
      clearInterval(timeInterval);
      clearInterval(daysInterval);
    });
  });

  return (
    <div class="fixed inset-0 pointer-events-none overflow-hidden font-sans">
      {/* Status Bar - Top Left - Inline */}
      <div class="absolute top-4 left-4 flex items-center gap-3">
        {/* LIVE 24/7 Indicator */}
        <div class="flex items-center gap-2">
          {/* Animated Red Dot */}
          <div class="relative">
            <div class="w-3 h-3 bg-red-500 rounded-full animate-pulse shadow-lg"></div>
            <div class="absolute inset-0 w-3 h-3 bg-red-400 rounded-full animate-ping"></div>
          </div>
          <span class="text-white font-black text-lg tracking-wider drop-shadow-lg">
            LIVE 24/7
          </span>
        </div>

        {/* Current Time and Day Combined */}
        <div class="flex items-center gap-3">
          <svg width="32" height="24" viewBox="0 0 60 30" xmlns="http://www.w3.org/2000/svg">
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
          <span class="text-2xl font-mono font-bold text-white tracking-wider drop-shadow-lg">
            {currentTime()} • Day {streamDays()}
          </span>
        </div>
      </div>

      <style>{`
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

        .animate-pulse {
          animation: pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite;
        }

        .animate-ping {
          animation: ping 1s cubic-bezier(0, 0, 0.2, 1) infinite;
        }
      `}</style>
    </div>
  );
}

render(() => <StatusOverlay />, document.getElementById('root'));
