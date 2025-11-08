import { render } from 'solid-js/web';
import { createSignal, createEffect, onCleanup, Show } from 'solid-js';
import '@/index.css';

const WEBARCADE_API = 'http://localhost:3001';
const WEBSOCKET_URL = 'ws://localhost:3002';

function TickerOverlay() {
  const [messages, setMessages] = createSignal([]);
  const [events, setEvents] = createSignal([]);
  const [tickerText, setTickerText] = createSignal('');
  const [currentTime, setCurrentTime] = createSignal('');
  const [streamDays, setStreamDays] = createSignal(0);
  const [tickerSpeed, setTickerSpeed] = createSignal(30);
  const [currentDate, setCurrentDate] = createSignal(new Date().toDateString());
  const [breakingNews, setBreakingNews] = createSignal({ active: false, message: '' });

  // Segment system
  const [segments, setSegments] = createSignal([]);
  const [currentSegmentIndex, setCurrentSegmentIndex] = createSignal(0);
  const [segmentTransitioning, setSegmentTransitioning] = createSignal(false);
  const [segmentDuration, setSegmentDuration] = createSignal(15); // seconds per segment

  let ws = null;
  let segmentInterval = null;

  // Load enabled ticker messages and events
  const loadMessages = async () => {
    try {
      console.log('🔄 Loading ticker data...');

      // Fetch both messages and events in parallel
      const [messagesResponse, eventsResponse] = await Promise.all([
        fetch(`${WEBARCADE_API}/ticker/messages/enabled`),
        fetch(`${WEBARCADE_API}/ticker/events`)
      ]);

      const messagesData = await messagesResponse.json();
      const eventsData = await eventsResponse.json();

      console.log('📝 Ticker messages:', messagesData.length, messagesData);
      console.log('🎉 Ticker events:', eventsData.length, eventsData);

      setMessages(messagesData);
      setEvents(eventsData);

      // Create scrolling text by combining messages and events
      const allItems = [];

      // Add enabled messages
      messagesData.forEach(m => {
        allItems.push(m.message);
      });

      // Add event display texts
      eventsData.forEach(e => {
        allItems.push(e.display_text);
      });

      console.log('✨ Total ticker items:', allItems.length, allItems);

      // Create scrolling text from combined items
      if (allItems.length > 0) {
        const text = '💎          ' + allItems.join('          💎          ');
        setTickerText(text);
        console.log('📺 Ticker text set:', text.substring(0, 100) + '...');
      } else {
        setTickerText('');
        console.log('⚠️ No ticker items to display');
      }
    } catch (error) {
      console.error('❌ Failed to load ticker data:', error);
    }
  };

  // Load stream status config
  const loadStatusConfig = async () => {
    try {
      const response = await fetch(`${WEBARCADE_API}/status/config`);
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
      setSegmentDuration(data.segment_duration || 15);

      // Load breaking news state
      setBreakingNews({
        active: data.breaking_news_active || false,
        message: data.breaking_news_message || ''
      });
    } catch (error) {
      console.error('Failed to load status config:', error);
    }
  };

  // Load ticker segments
  const loadSegments = async () => {
    try {
      const response = await fetch(`${WEBARCADE_API}/ticker/segments`);
      const data = await response.json();
      console.log('📺 Ticker segments loaded (all):', data);
      const enabledSegments = data.filter(s => s.enabled);
      console.log('📺 Ticker segments (enabled only):', enabledSegments);
      setSegments(enabledSegments);
    } catch (error) {
      console.error('❌ Failed to load ticker segments:', error);
    }
  };

  // Cycle to next segment with transition
  const nextSegment = () => {
    if (segments().length <= 1) return;

    const nextIndex = (currentSegmentIndex() + 1) % segments().length;
    console.log('🔄 Transitioning from segment', currentSegmentIndex(), 'to', nextIndex);

    setSegmentTransitioning(true);
    setTimeout(() => {
      setCurrentSegmentIndex(nextIndex);
      setTimeout(() => {
        setSegmentTransitioning(false);
      }, 50);
    }, 500);
  };

  // Handle animation complete (called when ticker finishes one loop or one pass)
  const handleAnimationComplete = (e) => {
    // Prevent event bubbling
    if (e) e.stopPropagation();

    if (segments().length > 1) {
      const currentSeg = currentSegment();
      const segmentType = currentSeg?.type || 'unknown';
      console.log(`✅ Animation complete for ${segmentType} segment, moving to next`);
      nextSegment();
    }
  };

  // Start segment cycling (fallback timer in case animation events don't fire)
  const startSegmentCycling = () => {
    if (segmentInterval) clearInterval(segmentInterval);
    if (segments().length <= 1) return;

    // Fallback timer - only triggers if animation doesn't complete within expected time
    const maxDuration = segmentDuration() * 1000;
    segmentInterval = setInterval(() => {
      console.log('⏰ Fallback timer triggered - animation may have stalled');
      nextSegment();
    }, maxDuration);
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

    // Check if the date has changed (crossed midnight)
    const newDate = now.toDateString();
    if (newDate !== currentDate()) {
      console.log('🌙 Midnight detected! Date changed from', currentDate(), 'to', newDate);
      setCurrentDate(newDate);
      // Reload status config to recalculate stream days
      loadStatusConfig();
    }
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
        console.log('📨 WebSocket message received:', data);

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

        // Handle ticker messages updates (includes events!)
        if (data.type === 'ticker_messages_update') {
          console.log('📡 Received ticker messages update - reloading all ticker data');
          loadMessages();
        }

        // Handle breaking news updates
        if (data.type === 'breaking_news_update') {
          console.log('📡 Received breaking news update:', data.breaking_news);
          setBreakingNews(data.breaking_news || { active: false, message: '' });
        }

        // Handle ticker segments updates
        if (data.type === 'ticker_segments_update') {
          console.log('📡 Received ticker segments update');
          loadSegments();
        }

        // Handle segment duration update
        if (data.type === 'segment_duration_update' && data.duration) {
          console.log('📡 Received segment duration update:', data.duration);
          setSegmentDuration(data.duration);
          startSegmentCycling();
        }
      } catch (err) {
        console.error('❌ Failed to parse WebSocket message:', err);
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
    loadSegments();
    updateTime();
    connectWebSocket();

    // Update time every second
    const timeInterval = setInterval(updateTime, 1000);

    onCleanup(() => {
      clearInterval(timeInterval);
      if (segmentInterval) clearInterval(segmentInterval);
      if (ws) {
        ws.close();
      }
    });
  });

  // Start segment cycling when segments change
  createEffect(() => {
    const segCount = segments().length;
    console.log('🔄 Segments changed, count:', segCount);

    // Clear any existing interval first
    if (segmentInterval) {
      clearInterval(segmentInterval);
      segmentInterval = null;
    }

    // Reset to first segment when segments change
    setCurrentSegmentIndex(0);

    // Only start cycling if there are multiple segments
    if (segCount > 1) {
      console.log('▶️ Starting segment cycling with', segCount, 'segments, duration:', segmentDuration(), 'seconds');
      startSegmentCycling();
    } else if (segCount === 1) {
      console.log('📍 Single segment mode - no cycling needed');
    } else {
      console.log('⚠️ No segments to display');
    }
  });

  // Generate ticker text for each segment type
  const getSegmentText = (segment) => {
    if (!segment) return '';

    switch (segment.type) {
      case 'messages':
        return tickerText();

      case 'schedule':
        const scheduleItems = segment.content?.schedule?.map(
          (item) => `${item.day}: ${item.time}`
        ).join('          ') || '24/7 Live Stream';
        return `📅 STREAM SCHEDULE          ${scheduleItems}`;

      case 'commands':
        const commandsList = segment.content?.commands?.map(
          (cmd) => `!${cmd.name} - ${cmd.description}`
        ).join('          ') || 'No commands configured';
        return `💬 CHAT COMMANDS          ${commandsList}`;

      case 'custom':
        return segment.content?.text || '';

      default:
        return '';
    }
  };

  // Calculate dynamic speed for custom messages based on text length
  const getCustomMessageSpeed = (text) => {
    const baseSpeed = 15; // Base duration in seconds
    const charCount = text.length;
    let speed;

    // Short messages (< 50 chars): faster
    if (charCount < 50) {
      speed = Math.max(8, baseSpeed * 0.5);
      console.log(`⚡ Short custom message (${charCount} chars) - speed: ${speed}s`);
    }
    // Medium messages (50-100 chars): normal
    else if (charCount < 100) {
      speed = baseSpeed;
      console.log(`➡️ Medium custom message (${charCount} chars) - speed: ${speed}s`);
    }
    // Long messages (100-200 chars): slower
    else if (charCount < 200) {
      speed = baseSpeed * 1.5;
      console.log(`🐌 Long custom message (${charCount} chars) - speed: ${speed}s`);
    }
    // Very long messages (200+ chars): much slower
    else {
      speed = Math.min(45, baseSpeed * 2);
      console.log(`🐢 Very long custom message (${charCount} chars) - speed: ${speed}s`);
    }

    return speed;
  };

  // Render segment content (always as scrolling ticker)
  const renderSegmentContent = (segment) => {
    if (!segment) return null;

    const text = getSegmentText(segment);
    const isCustom = segment.type === 'custom';

    if (isCustom) {
      // Custom messages: single pass, scroll completely off screen
      const customSpeed = getCustomMessageSpeed(text);

      return (
        <div
          class="ticker-scroll-once"
          style={`animation-duration: ${customSpeed}s`}
          onAnimationEnd={handleAnimationComplete}
        >
          <span class="ticker-text text-white font-bold text-xl whitespace-nowrap px-4">
            {text}
          </span>
        </div>
      );
    } else {
      // Other types: continuous loop with duplication
      return (
        <div class="ticker-scroll-continuous" onAnimationIteration={handleAnimationComplete}>
          <span class="ticker-text text-white font-bold text-xl whitespace-nowrap px-4">
            💎          {text}
          </span>
          <span class="ticker-text text-white font-bold text-xl whitespace-nowrap px-4">
            💎          {text}
          </span>
        </div>
      );
    }
  };

  // Get current segment to display
  const currentSegment = () => {
    const segs = segments();
    if (segs.length === 0) return null;
    return segs[currentSegmentIndex()];
  };

  return (
    <div class="fixed inset-0 pointer-events-none overflow-hidden bg-transparent">
      {/* Ticker Bar at Bottom */}
      <div class={`ticker-bar absolute bottom-0 left-0 right-0 ${breakingNews().active ? 'bg-gradient-to-r from-red-900/95 via-red-800/95 to-red-900/95' : 'bg-gradient-to-r from-purple-900/95 via-blue-900/95 to-purple-900/95'} backdrop-blur-sm border-t-4 border-black/20 shadow-lg h-16 flex items-center overflow-hidden`}>
        {/* Status Elements - Left Side */}
        <div class="flex items-center gap-2.5 px-4 flex-shrink-0">
          {/* LIVE 24/7 Indicator */}
          <div class="bg-gradient-to-r from-red-600 to-red-700 px-2.5 py-1.5 rounded shadow-xl border border-red-400 flex items-center gap-2">
            <div class="relative">
              <div class="w-2.5 h-2.5 bg-red-500 rounded-full animate-pulse shadow-lg"></div>
              <div class="absolute inset-0 w-2.5 h-2.5 bg-red-400 rounded-full animate-ping"></div>
            </div>
            <span class="text-white font-bold text-base tracking-wide drop-shadow-lg">
              LIVE 24/7
            </span>
          </div>

          {/* Days Since Stream Started */}
          <div class="bg-gradient-to-r from-cyan-900/95 to-teal-900/95 px-2.5 py-1.5 rounded shadow-lg border border-cyan-500">
            <div class="text-base font-bold text-white drop-shadow-lg">
              Day {streamDays()}
            </div>
          </div>

          {/* Current Time with UK Flag */}
          <div class="bg-gradient-to-r from-indigo-900/95 to-blue-900/95 px-2.5 py-1.5 rounded shadow-lg border border-indigo-500 flex items-center gap-2">
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
            <span class="text-base font-mono font-bold text-white tracking-wide drop-shadow-lg">
              {currentTime()}
            </span>
          </div>

          {/* Breaking News Badge */}
          <Show when={breakingNews().active}>
            <div class="bg-gradient-to-r from-red-600 to-red-700 px-3 py-1.5 rounded shadow-xl border border-red-400 animate-pulse-slow">
              <span class="text-white font-bold text-2xl tracking-wider drop-shadow-lg">
                🚨 BREAKING NEWS
              </span>
            </div>
          </Show>

          {/* Separator */}
          <div class="w-px h-8 bg-purple-400/50"></div>
        </div>

        {/* Breaking News or Segmented Ticker Content - Right Side */}
        <Show when={breakingNews().active} fallback={
          <Show when={segments().length > 0} fallback={
            <Show when={tickerText()}>
              <div class="flex-1 overflow-hidden relative">
                <div class="ticker-scroll-continuous">
                  <span class="ticker-text text-white font-bold text-xl whitespace-nowrap px-4">
                    {tickerText()}
                  </span>
                  <span class="ticker-text text-white font-bold text-xl whitespace-nowrap px-4">
                    {tickerText()}
                  </span>
                </div>
              </div>
            </Show>
          }>
            <div class={`flex-1 overflow-hidden relative transition-opacity duration-500 ${segmentTransitioning() ? 'opacity-0' : 'opacity-100'}`}>
              <div key={currentSegmentIndex()}>
                {renderSegmentContent(currentSegment())}
              </div>
            </div>
          </Show>
        }>
          <div class="flex-1 overflow-hidden relative flex items-center px-4">
            <span class="text-white font-bold text-2xl drop-shadow-lg animate-pulse-slow">
              {breakingNews().message}
            </span>
          </div>
        </Show>
      </div>

      <style>{`
        @keyframes ticker-scroll-continuous {
          0% {
            transform: translateX(0);
          }
          100% {
            transform: translateX(-50%);
          }
        }

        @keyframes ticker-scroll-once {
          0% {
            transform: translateX(100%);
          }
          100% {
            transform: translateX(-100%);
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

        @keyframes pulse-slow {
          0%, 100% {
            opacity: 1;
          }
          50% {
            opacity: 0.7;
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

        .ticker-scroll-continuous {
          animation: ticker-scroll-continuous ${tickerSpeed()}s linear infinite;
          display: inline-flex;
          white-space: nowrap;
        }

        .ticker-scroll-once {
          animation: ticker-scroll-once ${tickerSpeed()}s linear forwards;
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

        .animate-pulse-slow {
          animation: pulse-slow 3s cubic-bezier(0.4, 0, 0.6, 1) infinite;
        }

        .animate-ping {
          animation: ping 1s cubic-bezier(0, 0, 0.2, 1) infinite;
        }

        /* Pause animation on hover */
        .ticker-scroll-continuous:hover,
        .ticker-scroll-once:hover {
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
