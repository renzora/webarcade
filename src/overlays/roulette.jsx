import { render } from 'solid-js/web';
import { createSignal, createEffect, onCleanup, Show, For } from 'solid-js';
import '@/index.css';
import { WEBARCADE_WS } from '@/api/bridge';

const RESULT_DISPLAY_DURATION = 8000;
const API_URL = 'http://localhost:3001';

function RouletteOverlay() {
  const [isConnected, setIsConnected] = createSignal(false);
  const [currentGame, setCurrentGame] = createSignal(null);
  const [bets, setBets] = createSignal([]);
  const [isSpinning, setIsSpinning] = createSignal(false);
  const [winningNumber, setWinningNumber] = createSignal(null);
  const [showResult, setShowResult] = createSignal(false);
  const [wheelRotation, setWheelRotation] = createSignal(0);
  const [ballAngle, setBallAngle] = createSignal(0);
  const [ballRadiusRatio, setBallRadiusRatio] = createSignal(0.775); // Ratio of wheel radius (0-1), starts at rim
  const [ballVerticalPos, setBallVerticalPos] = createSignal(0);
  const [channel, setChannel] = createSignal(null);
  const [wheelSize, setWheelSize] = createSignal(600);
  const [showWheel, setShowWheel] = createSignal(false);
  const [betTimer, setBetTimer] = createSignal(null);
  const [winners, setWinners] = createSignal([]);
  const [totalPayout, setTotalPayout] = createSignal(0);
  const [totalWagered, setTotalWagered] = createSignal(0);
  const [ballTrail, setBallTrail] = createSignal([]);
  const [history, setHistory] = createSignal([]);
  const [tableScale, setTableScale] = createSignal(1);
  const [swapLayout, setSwapLayout] = createSignal(false);

  let ws;
  let hideTimeout;
  let animationFrame;
  let idleSpinFrame;
  let timerInterval;

  // Roulette wheel numbers in European layout order (clockwise)
  const wheelNumbers = [
    0, 32, 15, 19, 4, 21, 2, 25, 17, 34, 6, 27, 13, 36, 11, 30, 8, 23, 10,
    5, 24, 16, 33, 1, 20, 14, 31, 9, 22, 18, 29, 7, 28, 12, 35, 3, 26
  ];

  // Get number color
  const getNumberColor = (num) => {
    if (num === 0) return 'green';
    const redNumbers = [1, 3, 5, 7, 9, 12, 14, 16, 18, 19, 21, 23, 25, 27, 30, 32, 34, 36];
    return redNumbers.includes(num) ? 'red' : 'black';
  };

  // Calculate responsive wheel size - RIGHT SIDE ONLY
  const updateWheelSize = () => {
    const width = window.innerWidth;
    const height = window.innerHeight;
    // Use right side (50% width) for wheel, make it fill the height
    const availableWidth = width * 0.5;
    const size = Math.min(availableWidth * 0.95, height * 0.95);
    setWheelSize(size);

    // Calculate betting table scale based on available width
    const leftSideWidth = width * 0.5;
    const minTableWidth = 760; // Minimum width needed for table (includes 2:1 columns)
    const scale = Math.min(1, (leftSideWidth - 48) / minTableWidth); // 48px for padding
    setTableScale(scale);
  };

  // Idle wheel animation (slow spin when not betting)
  const startIdleSpin = () => {
    let rotation = wheelRotation();
    const animate = () => {
      if (!isSpinning() && showWheel() && currentGame()?.status === 'betting') {
        rotation += 0.3; // Slow rotation during betting
        setWheelRotation(rotation % 360);
        idleSpinFrame = requestAnimationFrame(animate);
      }
    };
    animate();
  };

  // Start countdown timer
  const startBetTimer = (seconds = 30) => {
    if (timerInterval) clearInterval(timerInterval);
    setBetTimer(seconds);
    timerInterval = setInterval(() => {
      setBetTimer(prev => {
        if (prev <= 1) {
          clearInterval(timerInterval);
          return 0;
        }
        return prev - 1;
      });
    }, 1000);
  };

  // Stop countdown timer
  const stopBetTimer = () => {
    if (timerInterval) clearInterval(timerInterval);
    setBetTimer(null);
  };

  // Connect to WebSocket
  const connectWebSocket = () => {
    ws = new WebSocket(WEBARCADE_WS);

    ws.onopen = () => {
      console.log('[Roulette] WebSocket connected');
      setIsConnected(true);
    };

    ws.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data);
        console.log('[Roulette] WebSocket message:', data);

        // Handle connection event (old format)
        if (data.type === 'connected' && data.channel) {
          const channelName = data.channel;
          setChannel(channelName);
          console.log('[Roulette] Connected to channel:', channelName);
          loadCurrentGame();
          loadHistory();
          loadLayoutPreference();
          return;
        }

        // Handle new plugin-based event format
        if (data.event_type && data.payload) {
          const eventType = data.event_type;
          const payload = data.payload;

          if (eventType === 'roulette.game_started') {
            console.log('[Roulette] Game started! Timer:', payload.timer_seconds, 'seconds');
            setShowWheel(true);
            setShowResult(false);
            setWinningNumber(null);
            loadCurrentGame();
            startIdleSpin();
            startBetTimer(payload.timer_seconds || 30);
          } else if (eventType === 'roulette.game_stopped') {
            console.log('[Roulette] Game stopped!');
            setShowWheel(false);
            setCurrentGame(null);
            stopBetTimer();
            setBets([]);
            if (idleSpinFrame) cancelAnimationFrame(idleSpinFrame);
          } else if (eventType === 'roulette.bet_placed') {
            loadCurrentGame();
          } else if (eventType === 'roulette.spin_started') {
            startPhysicsSpin(payload.game_id);
          } else if (eventType === 'roulette.result') {
            handleRouletteResult(payload);
            loadHistory();
          }
        }
      } catch (error) {
        console.error('[Roulette] WebSocket message error:', error);
      }
    };

    ws.onclose = () => {
      console.log('[Roulette] WebSocket disconnected');
      setIsConnected(false);
      setTimeout(connectWebSocket, 3000);
    };

    ws.onerror = (error) => {
      console.error('[Roulette] WebSocket error:', error);
    };
  };

  // Load current game
  const loadCurrentGame = async () => {
    if (!channel()) return;

    try {
      const response = await fetch(`${API_URL}/roulette/game?channel=${channel()}`);
      const data = await response.json();
      setCurrentGame(data.game);
      setBets(data.bets || []);

      // If there's an active game (betting or spinning), show the wheel
      if (data.game && (data.game.status === 'betting' || data.game.status === 'spinning')) {
        console.log('[Roulette] Active game detected, showing wheel. Status:', data.game.status);
        setShowWheel(true);

        // If betting, start the idle spin
        if (data.game.status === 'betting') {
          startIdleSpin();
          // Try to sync the timer if we have creation time
          if (data.game.created_at) {
            const now = Math.floor(Date.now() / 1000);
            const elapsed = now - data.game.created_at;
            const remaining = Math.max(0, 30 - elapsed);
            if (remaining > 0) {
              console.log('[Roulette] Starting bet timer with', remaining, 'seconds remaining');
              startBetTimer(remaining);
            }
          }
        } else if (data.game.status === 'spinning') {
          // User refreshed during spin - show ball spinning on rim
          console.log('[Roulette] Joined mid-spin, showing ball animation');
          setIsSpinning(true);
          setBallRadiusRatio(0.775); // Ball on rim
          setBallAngle(0);

          // Start a continuous spin animation until result comes
          const continuousSpin = () => {
            if (isSpinning()) {
              setBallAngle(prev => prev - 5); // Spin counter-clockwise
              setWheelRotation(prev => (prev + 0.5) % 360); // Wheel spins slowly
              animationFrame = requestAnimationFrame(continuousSpin);
            }
          };
          continuousSpin();
        }
      }
    } catch (error) {
      console.error('[Roulette] Failed to load game:', error);
    }
  };

  // Load game history
  const loadHistory = async () => {
    if (!channel()) return;

    try {
      const response = await fetch(`${API_URL}/roulette/history?channel=${channel()}&limit=5`);
      const data = await response.json();
      setHistory(data || []);
    } catch (error) {
      console.error('[Roulette] Failed to load history:', error);
    }
  };

  // Load layout preference from database
  const loadLayoutPreference = async () => {
    try {
      const response = await fetch(`${API_URL}/system/settings?key=roulette-swap-layout`);
      const data = await response.json();
      if (data.value === 'true') {
        console.log('[Roulette] Setting swapped layout from database');
        setSwapLayout(true);
      } else {
        setSwapLayout(false);
      }
    } catch (error) {
      console.error('[Roulette] Failed to load layout preference:', error);
    }
  };

  // Start physics-based spin with NO predetermined result
  const startPhysicsSpin = (gameId) => {
    console.log('[Roulette] Starting physics-based spin...');
    setIsSpinning(true);
    stopBetTimer();
    if (idleSpinFrame) cancelAnimationFrame(idleSpinFrame);

    const startTime = Date.now();
    const duration = 10000; // 10 seconds total

    // Wheel spins clockwise - slightly faster than idle
    const wheelAngularVelocity = 0.5; // degrees per frame
    let currentWheelRotation = wheelRotation();

    // Ball radius as RATIO of max wheel radius (0-1)
    // SVG viewBox max radius is 200, so:
    // - Rim at radius 155 = 155/200 = 0.775 ratio
    // - Pocket at exact middle of colored area (100-180): 140/200 = 0.70 ratio
    const rimRatio = 0.775;
    const pocketRatio = 0.70; // (100 + 180) / 2 / 200 = 140/200

    let ballAngle = 0;
    let ballAngularVelocity = -12; // Counter-clockwise, moderate speed
    let ballRadiusRatio = rimRatio; // Start at rim as ratio

    // Set initial ball position immediately
    setBallAngle(ballAngle);
    setBallRadiusRatio(ballRadiusRatio);
    setBallVerticalPos(0);

    console.log('[Roulette] Ball starting at ratio:', ballRadiusRatio, '(rim)');
    console.log('[Roulette] Ball will settle at ratio:', pocketRatio, '(pocket)');

    const animate = () => {
      const now = Date.now();
      const elapsed = now - startTime;
      const progress = Math.min(elapsed / duration, 1);

      // Spin the wheel continuously
      currentWheelRotation += wheelAngularVelocity;
      setWheelRotation(currentWheelRotation);

      // Ball physics with smooth transitions
      if (progress < 0.5) {
        // PHASE 1: Ball spins fast along rim (0-50%)
        ballAngle += ballAngularVelocity;
        ballAngularVelocity *= 0.9985; // Very gradual slowdown
        ballRadiusRatio = rimRatio; // Stay at rim
      } else if (progress < 0.85) {
        // PHASE 2: Ball gradually drops inward (50-85%)
        ballAngle += ballAngularVelocity;
        ballAngularVelocity *= 0.992; // Faster slowdown

        // Smoothly drop from rim to pocket
        const dropProgress = (progress - 0.5) / 0.35; // 0 to 1
        ballRadiusRatio = rimRatio - (rimRatio - pocketRatio) * Math.pow(dropProgress, 2);
      } else {
        // PHASE 3: Ball settling into pocket (85-100%)
        ballAngle += ballAngularVelocity;
        ballAngularVelocity *= 0.96; // Rapid slowdown
        ballRadiusRatio = pocketRatio; // Locked into pocket depth
      }

      setBallAngle(ballAngle);
      setBallRadiusRatio(ballRadiusRatio);
      setBallVerticalPos(0);

      if (progress < 1) {
        animationFrame = requestAnimationFrame(animate);
      } else {
        // Animation complete - calculate where ball landed
        setIsSpinning(false);
        calculateAndSubmitResult(gameId, ballAngle, currentWheelRotation);
      }
    };

    animate();
  };

  // Calculate which pocket the ball landed in
  const calculateAndSubmitResult = async (gameId, finalBallAngle, finalWheelRotation) => {
    // Ball's position relative to wheel
    // Add 90 degrees to account for wheel segments starting at -90 degrees (top)
    let relativeAngle = finalBallAngle - finalWheelRotation + 90;

    // Ensure positive angle using proper modulo for negative numbers
    relativeAngle = ((relativeAngle % 360) + 360) % 360;

    // Find which segment
    const degreesPerSegment = 360 / wheelNumbers.length;
    const segmentIndex = Math.floor(relativeAngle / degreesPerSegment);
    const winningNumber = wheelNumbers[segmentIndex];

    console.log('[Roulette] 🎯 BALL LANDED!');
    console.log('[Roulette]   - Final ball angle:', finalBallAngle.toFixed(2));
    console.log('[Roulette]   - Final wheel rotation:', finalWheelRotation.toFixed(2));
    console.log('[Roulette]   - Relative angle (adjusted):', relativeAngle.toFixed(2));
    console.log('[Roulette]   - Degrees per segment:', degreesPerSegment.toFixed(2));
    console.log('[Roulette]   - Segment index:', segmentIndex, '/', wheelNumbers.length);
    console.log('[Roulette]   - Winning number:', winningNumber, '(' + getNumberColor(winningNumber) + ')');
    console.log('[Roulette]   - Ball radius ratio:', ballRadiusRatio(), '(', (ballRadiusRatio() * wheelSize() / 2).toFixed(1), 'px )');

    // Safety check - ensure winning number is valid
    if (winningNumber === undefined || winningNumber === null) {
      console.error('[Roulette] ERROR: Invalid winning number calculated!');
      console.error('[Roulette] Segment index:', segmentIndex, 'Wheel numbers length:', wheelNumbers.length);
      return;
    }

    // Set winning number but DON'T show result yet
    // Result will be shown when backend sends roulette_result event
    setWinningNumber(winningNumber);
    console.log('[Roulette] Winning number set, waiting for backend to process bets...');

    // Send result to backend
    try {
      const response = await fetch(`${API_URL}/roulette/result`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          channel: channel(),
          game_id: gameId,
          winning_number: winningNumber
        })
      });
      console.log('[Roulette] Result submitted to backend:', await response.json());
    } catch (error) {
      console.error('[Roulette] Failed to submit result:', error);
    }
  };

  // Handle roulette result from backend (after physics determined winner)
  const handleRouletteResult = (data) => {
    console.log('[Roulette] Backend result received:', data);
    console.log('[Roulette] Current winning number signal:', winningNumber());
    console.log('[Roulette] Current showResult signal:', showResult());

    // Store winners info
    setTotalWagered(data.total_wagered || 0);
    setTotalPayout(data.total_payout || 0);

    // Calculate winners from bets
    console.log('[Roulette] Checking bets for winners...');
    console.log('[Roulette] Winning number:', data.winning_number, 'Color:', data.winning_color);
    console.log('[Roulette] All bets:', bets());

    const winningBets = bets().filter(bet => {
      let isWinner = false;

      if (bet.bet_type === 'number') {
        isWinner = parseInt(bet.bet_value) === data.winning_number;
      } else if (bet.bet_type === 'red' || bet.bet_type === 'black') {
        isWinner = bet.bet_value === data.winning_color;
        console.log('[Roulette] Color bet check:', bet.username, 'bet', bet.bet_value, 'vs winning', data.winning_color, '=', isWinner);
      } else if (bet.bet_type === 'odd') {
        isWinner = data.winning_number > 0 && data.winning_number % 2 === 1;
      } else if (bet.bet_type === 'even') {
        isWinner = data.winning_number > 0 && data.winning_number % 2 === 0;
      } else if (bet.bet_type === 'low') {
        isWinner = data.winning_number >= 1 && data.winning_number <= 18;
      } else if (bet.bet_type === 'high') {
        isWinner = data.winning_number >= 19 && data.winning_number <= 36;
      } else if (bet.bet_type === 'dozen1') {
        isWinner = data.winning_number >= 1 && data.winning_number <= 12;
      } else if (bet.bet_type === 'dozen2') {
        isWinner = data.winning_number >= 13 && data.winning_number <= 24;
      } else if (bet.bet_type === 'dozen3') {
        isWinner = data.winning_number >= 25 && data.winning_number <= 36;
      }

      if (isWinner) {
        console.log('[Roulette] ✓ Winner found:', bet.username, bet.bet_type, bet.bet_value, 'amount:', bet.amount);
      }

      return isWinner;
    });

    console.log('[Roulette] Total winning bets:', winningBets.length);

    const getMultiplier = (betType) => {
      if (betType === 'number') return 35;
      if (['dozen1', 'dozen2', 'dozen3', 'column1', 'column2', 'column3'].includes(betType)) return 2;
      return 1;
    };

    const winnersData = winningBets.map(bet => ({
      username: bet.username,
      amount: bet.amount,
      payout: bet.amount * (getMultiplier(bet.bet_type) + 1)
    }));

    setWinners(winnersData);

    // Display result for a few seconds, then auto-restart
    setShowResult(true);
    console.log('[Roulette] Result display should now be visible');

    if (hideTimeout) clearTimeout(hideTimeout);
    hideTimeout = setTimeout(async () => {
      console.log('[Roulette] ========== AUTO-RESTART TRIGGERED ==========');
      setShowResult(false);
      setWinningNumber(null);
      setBallAngle(0);
      setBallRadiusRatio(0.775); // Reset to rim ratio
      setBallVerticalPos(0);
      setWinners([]);
      setTotalPayout(0);
      setTotalWagered(0);

      // Give backend a moment to finish processing completed game
      console.log('[Roulette] Waiting 500ms for backend to complete game...');
      await new Promise(resolve => setTimeout(resolve, 500));

      // Auto-start new game after result display
      try {
        console.log('[Roulette] Calling auto-start API for channel:', channel());
        const response = await fetch(`${API_URL}/roulette/start?channel=${channel()}`);

        if (!response.ok) {
          console.error('[Roulette] HTTP error:', response.status, response.statusText);
          return;
        }

        const result = await response.json();
        console.log('[Roulette] Auto-start API response:', JSON.stringify(result));

        if (!result.success) {
          console.warn('[Roulette] ⚠️ Auto-start failed:', result.message);
          console.log('[Roulette] Checking for existing active game...');
          // If game already exists, just reload
          await loadCurrentGame();
          console.log('[Roulette] Current game after reload:', currentGame());
          if (currentGame()?.status === 'betting') {
            setShowWheel(true);
            startIdleSpin();
            startBetTimer(30);
          }
        } else {
          console.log('[Roulette] ✅ Auto-start successful! Game ID:', result.game_id);
          console.log('[Roulette] Waiting for WebSocket roulette_game_started event...');
          // WebSocket will receive roulette_game_started and handle everything
        }
      } catch (error) {
        console.error('[Roulette] ❌ Failed to auto-start new game:', error);
      }
    }, RESULT_DISPLAY_DURATION);
  };

  // Get bets for a specific bet type/value
  const getBetsForPosition = (betType, betValue) => {
    return bets().filter(bet =>
      bet.bet_type === betType && bet.bet_value === betValue
    );
  };

  // Group bets by type for display
  const getBetsByType = () => {
    const grouped = {};
    bets().forEach(bet => {
      const key = `${bet.bet_type}:${bet.bet_value}`;
      if (!grouped[key]) {
        grouped[key] = {
          type: bet.bet_type,
          value: bet.bet_value,
          bets: [],
          total: 0
        };
      }
      grouped[key].bets.push(bet);
      grouped[key].total += bet.amount;
    });
    return Object.values(grouped);
  };

  createEffect(() => {
    console.log('[Roulette] Overlay initialized');
    connectWebSocket();
    updateWheelSize();

    window.addEventListener('resize', updateWheelSize);

    onCleanup(() => {
      if (ws) ws.close();
      if (hideTimeout) clearTimeout(hideTimeout);
      if (animationFrame) cancelAnimationFrame(animationFrame);
      if (idleSpinFrame) cancelAnimationFrame(idleSpinFrame);
      if (timerInterval) clearInterval(timerInterval);
      window.removeEventListener('resize', updateWheelSize);
    });
  });

  return (
    <div class="fixed inset-0 pointer-events-none overflow-hidden bg-transparent">
      <Show when={showWheel()}>
        <div class={`flex h-full ${swapLayout() ? 'flex-row-reverse' : ''}`}>
          {/* LEFT/RIGHT SIDE: Betting Table */}
          <div class="w-1/2 h-full flex flex-col p-6 overflow-hidden">
            {/* Header with Timer and History */}
            <div class="bg-black/90 backdrop-blur-sm rounded-2xl px-6 py-4 border-2 border-yellow-500/50 mb-4">
              <div class="text-center">
                <div class="flex items-center justify-center gap-4 mb-3">
                  <div class="text-yellow-400 text-xl font-semibold uppercase tracking-wider">
                    🎰 Roulette {currentGame()?.status === 'betting' ? '- Place Your Bets!' : ''}
                  </div>
                  <Show when={betTimer() !== null && betTimer() > 0}>
                    <div class={`text-3xl font-bold px-6 py-2 rounded-lg ${
                      betTimer() <= 5 ? 'bg-red-600 text-white animate-pulse' :
                      betTimer() <= 10 ? 'bg-orange-600 text-white' :
                      'bg-green-600 text-white'
                    }`}>
                      {betTimer()}s
                    </div>
                  </Show>
                </div>

                {/* Recent History */}
                <Show when={history().length > 0}>
                  <div class="flex items-center justify-center gap-2">
                    <span class="text-gray-400 text-sm mr-2">Recent:</span>
                    <For each={history().slice(0, 5)}>
                      {(game) => (
                        <div class={`w-10 h-10 rounded-full flex items-center justify-center text-white font-bold text-lg border-2 ${
                          getNumberColor(game.winning_number) === 'red' ? 'bg-red-600 border-red-400' :
                          getNumberColor(game.winning_number) === 'black' ? 'bg-black border-white' :
                          'bg-green-600 border-green-400'
                        }`}>
                          {game.winning_number}
                        </div>
                      )}
                    </For>
                  </div>
                </Show>
              </div>
            </div>

            {/* Betting Table - Compact */}
            <div class="flex-1 bg-black/90 backdrop-blur-sm rounded-2xl border-2 border-yellow-500/50 overflow-hidden flex flex-col">
              <div class="bg-yellow-600 px-4 py-2 border-b-2 border-yellow-500">
                <h2 class="text-white text-lg font-bold uppercase">Active Bets ({bets().length})</h2>
              </div>

              <div class="flex-1 overflow-y-auto">
                <Show
                  when={bets().length > 0}
                  fallback={
                    <div class="text-center py-8 text-gray-400">
                      <div class="text-4xl mb-2">🎲</div>
                      <p class="text-sm">Use !bet &lt;amount&gt; &lt;type&gt; in chat</p>
                    </div>
                  }
                >
                  <table class="w-full text-sm">
                    <thead class="sticky top-0 bg-gray-900 border-b border-yellow-600">
                      <tr>
                        <th class="text-left px-3 py-2 text-yellow-400 font-bold uppercase text-xs">Player</th>
                        <th class="text-center px-2 py-2 text-yellow-400 font-bold uppercase text-xs">Bet</th>
                        <th class="text-right px-3 py-2 text-yellow-400 font-bold uppercase text-xs">Amount</th>
                        <th class="text-right px-3 py-2 text-yellow-400 font-bold uppercase text-xs">Win</th>
                      </tr>
                    </thead>
                    <tbody>
                      <For each={bets()}>
                        {(bet, index) => {
                          const multiplier =
                            bet.bet_type === 'number' ? 35 :
                            ['dozen1', 'dozen2', 'dozen3', 'column1', 'column2', 'column3'].includes(bet.bet_type) ? 2 :
                            1;
                          const potentialWin = bet.amount * (multiplier + 1);

                          return (
                            <tr class={`border-b border-gray-800/50 hover:bg-gray-800/50 transition-colors ${
                              index() % 2 === 0 ? 'bg-gray-900/30' : 'bg-black/30'
                            }`}>
                              <td class="px-3 py-2">
                                <div class="flex items-center gap-2">
                                  <img
                                    src={`https://decapi.me/twitch/avatar/${bet.username}`}
                                    alt={bet.username}
                                    class="w-7 h-7 rounded-full border border-yellow-500/50"
                                    onError={(e) => {
                                      e.target.src = `https://ui-avatars.com/api/?name=${bet.username}&background=random&size=64`;
                                    }}
                                  />
                                  <span class="text-white font-medium text-sm">{bet.username}</span>
                                </div>
                              </td>
                              <td class="px-2 py-2 text-center">
                                <div class={`inline-block px-2 py-1 rounded text-xs font-bold text-white ${
                                  bet.bet_type === 'number' ? 'bg-purple-600' :
                                  bet.bet_type === 'red' ? 'bg-red-600' :
                                  bet.bet_type === 'black' ? 'bg-gray-900 border border-white' :
                                  'bg-green-600'
                                }`}>
                                  {bet.bet_value.toUpperCase()}
                                </div>
                              </td>
                              <td class="px-3 py-2 text-right">
                                <span class="text-yellow-400 font-bold">{bet.amount}</span>
                              </td>
                              <td class="px-3 py-2 text-right">
                                <span class="text-green-400 font-bold">{potentialWin}</span>
                                <span class="text-gray-500 text-xs ml-1">({multiplier}:1)</span>
                              </td>
                            </tr>
                          );
                        }}
                      </For>
                    </tbody>
                  </table>
                </Show>
              </div>

              {/* Total Summary - Compact */}
              <Show when={bets().length > 0}>
                <div class="bg-gray-900 border-t border-yellow-600 px-4 py-2">
                  <div class="flex items-center justify-between text-xs">
                    <div>
                      <span class="text-gray-400 uppercase">{new Set(bets().map(b => b.username)).size} Players</span>
                    </div>
                    <div class="text-right">
                      <span class="text-gray-400 uppercase mr-2">Total:</span>
                      <span class="text-yellow-400 font-bold text-sm">{bets().reduce((sum, b) => sum + b.amount, 0)} coins</span>
                    </div>
                  </div>
                </div>
              </Show>
            </div>

            {/* Visual Betting Table Layout */}
            <div class="mt-4 bg-green-800 rounded-2xl border-2 border-yellow-500/50 p-4 overflow-hidden">
              <div class="text-white text-lg font-bold mb-3 text-center">Betting Table</div>

              {/* Wrapper with adjusted dimensions based on scale */}
              <div
                class="transition-all duration-300"
                style={{
                  'width': `${760 * tableScale()}px`, // Full table width including 2:1 columns
                  'height': `${280 * tableScale()}px` // Full table height
                }}
              >
                <div
                  class="flex gap-2 origin-top-left transition-transform duration-300"
                  style={{
                    'min-width': '760px', // Actual table width
                    'transform': `scale(${tableScale()})`
                  }}
                >
                {/* Zero column */}
                <div class="flex flex-col">
                  <div class="bg-green-600 border-2 border-white w-12 h-[168px] flex items-center justify-center text-white text-xl font-bold relative">
                    <div class="transform -rotate-90">0</div>
                    <Show when={getBetsForPosition('number', '0').length > 0}>
                      <div class="absolute top-1 right-1 flex flex-col gap-0.5">
                        <For each={getBetsForPosition('number', '0').slice(0, 3)}>
                          {(bet) => (
                            <img
                              src={`https://decapi.me/twitch/avatar/${bet.username}`}
                              alt={bet.username}
                              class="w-4 h-4 rounded-full border border-yellow-400"
                              onError={(e) => {
                                e.target.src = `https://ui-avatars.com/api/?name=${bet.username}&background=random&size=32`;
                              }}
                            />
                          )}
                        </For>
                        <Show when={getBetsForPosition('number', '0').length > 3}>
                          <div class="bg-yellow-500 text-black text-xs w-4 h-4 flex items-center justify-center rounded-full font-bold">
                            +{getBetsForPosition('number', '0').length - 3}
                          </div>
                        </Show>
                      </div>
                    </Show>
                  </div>
                </div>

                <div class="flex flex-col gap-1">
                  {/* Numbers grid */}
                  <div class="flex flex-col gap-1">
                    {/* Row 3 */}
                    <div class="flex gap-1">
                      <For each={[3, 6, 9, 12, 15, 18, 21, 24, 27, 30, 33, 36]}>
                        {(num) => (
                          <div class={`${getNumberColor(num) === 'red' ? 'bg-red-600' : 'bg-black'} border-2 border-white w-12 h-12 flex items-center justify-center text-white text-sm font-bold relative`}>
                            {num}
                            <Show when={getBetsForPosition('number', num.toString()).length > 0}>
                              <div class="absolute top-0.5 right-0.5 flex flex-wrap gap-0.5 max-w-[20px]">
                                <For each={getBetsForPosition('number', num.toString()).slice(0, 2)}>
                                  {(bet) => (
                                    <img
                                      src={`https://decapi.me/twitch/avatar/${bet.username}`}
                                      alt={bet.username}
                                      class="w-4 h-4 rounded-full border border-yellow-400"
                                      onError={(e) => {
                                        e.target.src = `https://ui-avatars.com/api/?name=${bet.username}&background=random&size=32`;
                                      }}
                                    />
                                  )}
                                </For>
                                <Show when={getBetsForPosition('number', num.toString()).length > 2}>
                                  <div class="bg-yellow-500 text-black text-[8px] w-4 h-4 flex items-center justify-center rounded-full font-bold">
                                    +{getBetsForPosition('number', num.toString()).length - 2}
                                  </div>
                                </Show>
                              </div>
                            </Show>
                          </div>
                        )}
                      </For>
                      <div class="bg-green-700 border-2 border-white w-12 h-12 flex items-center justify-center text-white text-xs font-bold relative">
                        2:1
                        <Show when={getBetsForPosition('column3', 'column3').length > 0}>
                          <div class="absolute top-0 right-0 bg-yellow-500 text-black text-xs w-4 h-4 flex items-center justify-center rounded-full">
                            {getBetsForPosition('column3', 'column3').length}
                          </div>
                        </Show>
                      </div>
                    </div>

                    {/* Row 2 */}
                    <div class="flex gap-1">
                      <For each={[2, 5, 8, 11, 14, 17, 20, 23, 26, 29, 32, 35]}>
                        {(num) => (
                          <div class={`${getNumberColor(num) === 'red' ? 'bg-red-600' : 'bg-black'} border-2 border-white w-12 h-12 flex items-center justify-center text-white text-sm font-bold relative`}>
                            {num}
                            <Show when={getBetsForPosition('number', num.toString()).length > 0}>
                              <div class="absolute top-0.5 right-0.5 flex flex-wrap gap-0.5 max-w-[20px]">
                                <For each={getBetsForPosition('number', num.toString()).slice(0, 2)}>
                                  {(bet) => (
                                    <img
                                      src={`https://decapi.me/twitch/avatar/${bet.username}`}
                                      alt={bet.username}
                                      class="w-4 h-4 rounded-full border border-yellow-400"
                                      onError={(e) => {
                                        e.target.src = `https://ui-avatars.com/api/?name=${bet.username}&background=random&size=32`;
                                      }}
                                    />
                                  )}
                                </For>
                                <Show when={getBetsForPosition('number', num.toString()).length > 2}>
                                  <div class="bg-yellow-500 text-black text-[8px] w-4 h-4 flex items-center justify-center rounded-full font-bold">
                                    +{getBetsForPosition('number', num.toString()).length - 2}
                                  </div>
                                </Show>
                              </div>
                            </Show>
                          </div>
                        )}
                      </For>
                      <div class="bg-green-700 border-2 border-white w-12 h-12 flex items-center justify-center text-white text-xs font-bold relative">
                        2:1
                        <Show when={getBetsForPosition('column2', 'column2').length > 0}>
                          <div class="absolute top-0 right-0 bg-yellow-500 text-black text-xs w-4 h-4 flex items-center justify-center rounded-full">
                            {getBetsForPosition('column2', 'column2').length}
                          </div>
                        </Show>
                      </div>
                    </div>

                    {/* Row 1 */}
                    <div class="flex gap-1">
                      <For each={[1, 4, 7, 10, 13, 16, 19, 22, 25, 28, 31, 34]}>
                        {(num) => (
                          <div class={`${getNumberColor(num) === 'red' ? 'bg-red-600' : 'bg-black'} border-2 border-white w-12 h-12 flex items-center justify-center text-white text-sm font-bold relative`}>
                            {num}
                            <Show when={getBetsForPosition('number', num.toString()).length > 0}>
                              <div class="absolute top-0.5 right-0.5 flex flex-wrap gap-0.5 max-w-[20px]">
                                <For each={getBetsForPosition('number', num.toString()).slice(0, 2)}>
                                  {(bet) => (
                                    <img
                                      src={`https://decapi.me/twitch/avatar/${bet.username}`}
                                      alt={bet.username}
                                      class="w-4 h-4 rounded-full border border-yellow-400"
                                      onError={(e) => {
                                        e.target.src = `https://ui-avatars.com/api/?name=${bet.username}&background=random&size=32`;
                                      }}
                                    />
                                  )}
                                </For>
                                <Show when={getBetsForPosition('number', num.toString()).length > 2}>
                                  <div class="bg-yellow-500 text-black text-[8px] w-4 h-4 flex items-center justify-center rounded-full font-bold">
                                    +{getBetsForPosition('number', num.toString()).length - 2}
                                  </div>
                                </Show>
                              </div>
                            </Show>
                          </div>
                        )}
                      </For>
                      <div class="bg-green-700 border-2 border-white w-12 h-12 flex items-center justify-center text-white text-xs font-bold relative">
                        2:1
                        <Show when={getBetsForPosition('column1', 'column1').length > 0}>
                          <div class="absolute top-0 right-0 bg-yellow-500 text-black text-xs w-4 h-4 flex items-center justify-center rounded-full">
                            {getBetsForPosition('column1', 'column1').length}
                          </div>
                        </Show>
                      </div>
                    </div>
                  </div>

                  {/* Outside bets row */}
                  <div class="flex gap-1">
                    <For each={[
                      { type: 'low', label: '1-18' },
                      { type: 'even', label: 'EVEN' },
                      { type: 'red', label: 'RED' },
                      { type: 'black', label: 'BLACK' },
                      { type: 'odd', label: 'ODD' },
                      { type: 'high', label: '19-36' }
                    ]}>
                      {(bet) => {
                        const betList = getBetsForPosition(bet.type, bet.type);
                        return (
                          <div class={`${
                            bet.type === 'red' ? 'bg-red-600' :
                            bet.type === 'black' ? 'bg-black' :
                            'bg-green-700'
                          } border-2 border-white w-24 h-12 flex items-center justify-center text-white text-xs font-bold relative`}>
                            {bet.label}
                            <Show when={betList.length > 0}>
                              <div class="absolute top-0 right-0 bg-yellow-500 text-black text-xs w-4 h-4 flex items-center justify-center rounded-full">
                                {betList.length}
                              </div>
                            </Show>
                          </div>
                        );
                      }}
                    </For>
                  </div>

                  {/* Dozens row */}
                  <div class="flex gap-1">
                    <For each={[
                      { type: 'dozen1', label: '1st 12' },
                      { type: 'dozen2', label: '2nd 12' },
                      { type: 'dozen3', label: '3rd 12' }
                    ]}>
                      {(bet) => {
                        const betList = getBetsForPosition(bet.type, bet.type);
                        return (
                          <div class="bg-green-700 border-2 border-white flex-1 h-12 flex items-center justify-center text-white text-xs font-bold relative">
                            {bet.label}
                            <Show when={betList.length > 0}>
                              <div class="absolute top-0 right-0 bg-yellow-500 text-black text-xs w-4 h-4 flex items-center justify-center rounded-full">
                                {betList.length}
                              </div>
                            </Show>
                          </div>
                        );
                      }}
                    </For>
                  </div>
                </div>
              </div>
              </div> {/* Close wrapper div */}
            </div>
          </div>

          {/* LEFT/RIGHT SIDE: Roulette Wheel */}
          <div class="w-1/2 h-full flex items-center justify-center relative">
            <div class="absolute top-1/2 left-1/2 transform -translate-x-1/2 -translate-y-1/2"
                 style={{ width: `${wheelSize()}px`, height: `${wheelSize()}px` }}>

          {/* SVG Wheel */}
          <svg
            width={wheelSize()}
            height={wheelSize()}
            viewBox="0 0 400 400"
            class="absolute inset-0"
            style={{ transform: `rotate(${wheelRotation()}deg)`, transition: 'none' }}
          >
            {/* Background gradient */}
            <defs>
              <radialGradient id="centerGold">
                <stop offset="0%" stop-color="#ffd700" />
                <stop offset="100%" stop-color="#b8860b" />
              </radialGradient>
              <linearGradient id="metalRim" x1="0%" y1="0%" x2="100%" y2="100%">
                <stop offset="0%" stop-color="#ffd700" />
                <stop offset="50%" stop-color="#ffed4e" />
                <stop offset="100%" stop-color="#b8860b" />
              </linearGradient>
            </defs>

            {/* Outer rim with metallic gradient */}
            <circle cx="200" cy="200" r="195" fill="url(#metalRim)" stroke="#8b6914" stroke-width="3"/>

            {/* Inner rim shadow */}
            <circle cx="200" cy="200" r="185" fill="none" stroke="#000000" stroke-width="2" opacity="0.5"/>

            {/* Main wheel background - SOLID BLACK */}
            <circle cx="200" cy="200" r="180" fill="#000000" />

            {/* Number pockets */}
            <For each={wheelNumbers}>
              {(number, index) => {
                const angle = (360 / wheelNumbers.length) * index() - 90;
                const angleRad = (angle * Math.PI) / 180;
                const nextAngleRad = ((angle + 360 / wheelNumbers.length) * Math.PI) / 180;

                const outerR = 180;
                const innerR = 100;

                const x1 = 200 + outerR * Math.cos(angleRad);
                const y1 = 200 + outerR * Math.sin(angleRad);
                const x2 = 200 + outerR * Math.cos(nextAngleRad);
                const y2 = 200 + outerR * Math.sin(nextAngleRad);
                const x3 = 200 + innerR * Math.cos(nextAngleRad);
                const y3 = 200 + innerR * Math.sin(nextAngleRad);
                const x4 = 200 + innerR * Math.cos(angleRad);
                const y4 = 200 + innerR * Math.sin(angleRad);

                const color = getNumberColor(number);
                const fillColor =
                  color === 'red' ? '#dc2626' :
                  color === 'green' ? '#16a34a' :
                  '#1f2937';

                const midAngle = angle + (360 / wheelNumbers.length) / 2;
                const midAngleRad = (midAngle * Math.PI) / 180;

                // Position numbers at top of pocket (near outer edge)
                const textR = 165;
                const textX = 200 + textR * Math.cos(midAngleRad);
                const textY = 200 + textR * Math.sin(midAngleRad);

                // Border below numbers
                const borderR = 155;
                const b1 = 200 + borderR * Math.cos(angleRad);
                const b2 = 200 + borderR * Math.sin(angleRad);
                const b3 = 200 + borderR * Math.cos(nextAngleRad);
                const b4 = 200 + borderR * Math.sin(nextAngleRad);

                const isWinning = showResult() && number === winningNumber();

                return (
                  <g>
                    {/* Pocket */}
                    <path
                      d={`M ${x1} ${y1} A ${outerR} ${outerR} 0 0 1 ${x2} ${y2} L ${x3} ${y3} A ${innerR} ${innerR} 0 0 0 ${x4} ${y4} Z`}
                      fill={fillColor}
                      stroke="none"
                      opacity={isWinning ? 1 : 0.95}
                      filter={isWinning ? 'brightness(1.5) drop-shadow(0 0 10px currentColor)' : 'none'}
                    />

                    {/* Side separator line - bright gold */}
                    <line
                      x1={x1}
                      y1={y1}
                      x2={x4}
                      y2={y4}
                      stroke="#ffd700"
                      stroke-width="2.5"
                      opacity="0.9"
                    />

                    {/* Border below number - white/gold arc */}
                    <path
                      d={`M ${b1} ${b2} A ${borderR} ${borderR} 0 0 1 ${b3} ${b4}`}
                      fill="none"
                      stroke="#ffffff"
                      stroke-width="2"
                      opacity="0.8"
                    />

                    {/* Number - at top of pocket */}
                    <text
                      x={textX}
                      y={textY}
                      fill="white"
                      font-size="15"
                      font-weight="bold"
                      text-anchor="middle"
                      dominant-baseline="middle"
                      transform={`rotate(${midAngle + 90}, ${textX}, ${textY})`}
                      style={{ 'text-shadow': '0 2px 4px rgba(0,0,0,1)' }}
                    >
                      {number}
                    </text>
                  </g>
                );
              }}
            </For>

            {/* Center circle */}
            <circle cx="200" cy="200" r="90" fill="url(#centerGold)" stroke="#ffd700" stroke-width="4"/>
          </svg>

          {/* Ball - NO GLOW */}
          <Show when={isSpinning() || showResult()}>
            <div class="absolute inset-0 pointer-events-none">
              {/* Main ball */}
              <div
                class="absolute rounded-full"
                style={{
                  left: '50%',
                  top: '50%',
                  width: `${wheelSize() * 0.03}px`,
                  height: `${wheelSize() * 0.03}px`,
                  // Convert ratio to pixels based on current wheel size
                  transform: `translate(-50%, -50%) rotate(${ballAngle()}deg) translateX(${ballRadiusRatio() * (wheelSize() / 2)}px) translateY(${ballVerticalPos()}px)`,
                  'box-shadow': `0 ${Math.max(2, -ballVerticalPos() * 2)}px ${Math.max(4, -ballVerticalPos() * 3)}px rgba(0,0,0,0.4)`,
                  transition: 'none',
                  'z-index': 100,
                  background: 'radial-gradient(circle at 35% 35%, #ffffff, #f0f0f0, #d0d0d0)'
                }}
              />
            </div>
          </Show>
            </div>

            {/* Result Display - Overlaid on wheel */}
            <Show when={showResult() && winningNumber() !== null}>
              <div class="absolute top-1/2 left-1/2 transform -translate-x-1/2 -translate-y-1/2 animate-[bounce_1s_ease-in-out] z-50">
          <div class="bg-black/90 backdrop-blur-md rounded-3xl px-12 py-6 border-4 border-yellow-500 shadow-2xl">
            <div class="text-center space-y-2">
              <div class="text-yellow-400 text-xl font-bold uppercase tracking-wider">
                {winners().length > 0 ? 'Winners!' : 'House Wins!'}
              </div>
              <div class="flex items-center justify-center gap-4">
                <div
                  class={`text-7xl font-black ${
                    getNumberColor(winningNumber()) === 'red' ? 'text-red-500' :
                    getNumberColor(winningNumber()) === 'black' ? 'text-white' :
                    'text-green-500'
                  }`}
                  style={{
                    'text-shadow': '0 0 40px currentColor, 0 0 80px currentColor'
                  }}
                >
                  {winningNumber()}
                </div>
                <div class="text-5xl">
                  {getNumberColor(winningNumber()) === 'red' ? '🔴' :
                   getNumberColor(winningNumber()) === 'black' ? '⚫' :
                   '🟢'}
                </div>
              </div>
              <div class="text-white text-lg uppercase tracking-wide">
                {getNumberColor(winningNumber())}
              </div>

              {/* Winners List */}
              <Show when={winners().length > 0}>
                <div class="mt-4 pt-4 border-t border-yellow-500/50">
                  <div class="text-green-400 text-lg font-bold mb-2">
                    {winners().length} Winner{winners().length > 1 ? 's' : ''}!
                  </div>
                  <div class="space-y-2 max-h-32 overflow-y-auto">
                    <For each={winners()}>
                      {(winner) => (
                        <div class="bg-green-600/20 rounded-lg px-4 py-2 flex items-center justify-between gap-4 border border-green-500/30">
                          <div class="text-white font-semibold">{winner.username}</div>
                          <div class="text-yellow-400 font-bold text-lg">
                            +{winner.payout} coins
                          </div>
                        </div>
                      )}
                    </For>
                  </div>
                  <div class="mt-3 text-gray-300 text-sm">
                    Total Payout: <span class="text-yellow-400 font-bold">{totalPayout()} coins</span>
                  </div>
                </div>
              </Show>

              {/* House Wins Display */}
              <Show when={winners().length === 0 && totalWagered() > 0}>
                <div class="mt-4 pt-4 border-t border-yellow-500/50">
                  <div class="text-red-400 text-lg font-bold">
                    House wins {totalWagered()} coins!
                  </div>
                </div>
              </Show>
            </div>
          </div>
              </div>
            </Show>
          </div>
        </div>
      </Show>

      {/* OLD Betting Table - REMOVED */}
      <Show when={false}>
        <div class="absolute bottom-4 left-1/2 transform -translate-x-1/2 max-w-[95vw]">
          <div class="bg-green-800 backdrop-blur-sm rounded-2xl p-4 border-4 border-yellow-600 shadow-2xl">
            <div class="flex gap-2">
              {/* Zero Section */}
              <div class="flex flex-col gap-1">
                <div class="bg-green-600 border-2 border-white rounded w-12 h-full flex items-center justify-center relative">
                  <div class="text-white text-2xl font-bold">0</div>
                  <Show when={getBetsForPosition('number', '0').length > 0}>
                    <div class="absolute inset-0 flex items-center justify-center pointer-events-none">
                      <For each={getBetsForPosition('number', '0')}>
                        {(bet) => (
                          <div class="absolute bg-yellow-500 text-black text-xs px-2 py-1 rounded-full font-bold shadow-lg border-2 border-white animate-[bounce_0.5s_ease-in-out]">
                            {bet.amount}
                          </div>
                        )}
                      </For>
                    </div>
                  </Show>
                </div>
              </div>

              <div class="flex flex-col gap-1">
                {/* Number Grid - 3 rows x 12 columns */}
                <div class="flex flex-col gap-1">
                  {/* Row 3 */}
                  <div class="flex gap-1">
                    <For each={[3, 6, 9, 12, 15, 18, 21, 24, 27, 30, 33, 36]}>
                      {(num) => (
                        <div class={`${getNumberColor(num) === 'red' ? 'bg-red-600' : 'bg-black'} border-2 border-white rounded w-10 h-10 flex items-center justify-center relative`}>
                          <div class="text-white text-sm font-bold">{num}</div>
                          <Show when={getBetsForPosition('number', num.toString()).length > 0}>
                            <div class="absolute inset-0 flex items-center justify-center pointer-events-none">
                              <For each={getBetsForPosition('number', num.toString())}>
                                {(bet) => (
                                  <div class="absolute bg-yellow-500 text-black text-xs px-1.5 py-0.5 rounded-full font-bold shadow-lg border border-white animate-[bounce_0.5s_ease-in-out]">
                                    {bet.amount}
                                  </div>
                                )}
                              </For>
                            </div>
                          </Show>
                        </div>
                      )}
                    </For>
                    <div class="bg-green-700 border-2 border-white rounded w-12 h-10 flex items-center justify-center text-white text-xs font-bold relative">
                      2:1
                    </div>
                  </div>

                  {/* Row 2 */}
                  <div class="flex gap-1">
                    <For each={[2, 5, 8, 11, 14, 17, 20, 23, 26, 29, 32, 35]}>
                      {(num) => (
                        <div class={`${getNumberColor(num) === 'red' ? 'bg-red-600' : 'bg-black'} border-2 border-white rounded w-10 h-10 flex items-center justify-center relative`}>
                          <div class="text-white text-sm font-bold">{num}</div>
                          <Show when={getBetsForPosition('number', num.toString()).length > 0}>
                            <div class="absolute inset-0 flex items-center justify-center pointer-events-none">
                              <For each={getBetsForPosition('number', num.toString())}>
                                {(bet) => (
                                  <div class="absolute bg-yellow-500 text-black text-xs px-1.5 py-0.5 rounded-full font-bold shadow-lg border border-white animate-[bounce_0.5s_ease-in-out]">
                                    {bet.amount}
                                  </div>
                                )}
                              </For>
                            </div>
                          </Show>
                        </div>
                      )}
                    </For>
                    <div class="bg-green-700 border-2 border-white rounded w-12 h-10 flex items-center justify-center text-white text-xs font-bold relative">
                      2:1
                    </div>
                  </div>

                  {/* Row 1 */}
                  <div class="flex gap-1">
                    <For each={[1, 4, 7, 10, 13, 16, 19, 22, 25, 28, 31, 34]}>
                      {(num) => (
                        <div class={`${getNumberColor(num) === 'red' ? 'bg-red-600' : 'bg-black'} border-2 border-white rounded w-10 h-10 flex items-center justify-center relative`}>
                          <div class="text-white text-sm font-bold">{num}</div>
                          <Show when={getBetsForPosition('number', num.toString()).length > 0}>
                            <div class="absolute inset-0 flex items-center justify-center pointer-events-none">
                              <For each={getBetsForPosition('number', num.toString())}>
                                {(bet) => (
                                  <div class="absolute bg-yellow-500 text-black text-xs px-1.5 py-0.5 rounded-full font-bold shadow-lg border border-white animate-[bounce_0.5s_ease-in-out]">
                                    {bet.amount}
                                  </div>
                                )}
                              </For>
                            </div>
                          </Show>
                        </div>
                      )}
                    </For>
                    <div class="bg-green-700 border-2 border-white rounded w-12 h-10 flex items-center justify-center text-white text-xs font-bold relative">
                      2:1
                    </div>
                  </div>
                </div>

                {/* Outside Bets Row */}
                <div class="flex gap-1 mt-1">
                  <For each={[
                    { type: 'low', label: '1-18', span: 2 },
                    { type: 'even', label: 'EVEN', span: 2 },
                    { type: 'red', label: 'RED', span: 2 },
                    { type: 'black', label: 'BLACK', span: 2 },
                    { type: 'odd', label: 'ODD', span: 2 },
                    { type: 'high', label: '19-36', span: 2 }
                  ]}>
                    {(bet) => {
                      const betList = getBetsForPosition(bet.type, bet.type);
                      return (
                        <div class={`${
                          bet.type === 'red' ? 'bg-red-600' :
                          bet.type === 'black' ? 'bg-black' :
                          'bg-green-700'
                        } border-2 border-white rounded h-10 flex items-center justify-center relative`}
                        style={{ width: `${bet.span * 2.75}rem` }}>
                          <div class="text-white text-xs font-bold">{bet.label}</div>
                          <Show when={betList.length > 0}>
                            <div class="absolute inset-0 flex items-center justify-center pointer-events-none">
                              <For each={betList}>
                                {(b) => (
                                  <div class="absolute bg-yellow-500 text-black text-xs px-1.5 py-0.5 rounded-full font-bold shadow-lg border border-white animate-[bounce_0.5s_ease-in-out]">
                                    {b.amount}
                                  </div>
                                )}
                              </For>
                            </div>
                          </Show>
                        </div>
                      );
                    }}
                  </For>
                </div>

                {/* Dozen Bets Row */}
                <div class="flex gap-1">
                  <For each={[
                    { type: 'dozen1', label: '1st 12' },
                    { type: 'dozen2', label: '2nd 12' },
                    { type: 'dozen3', label: '3rd 12' }
                  ]}>
                    {(bet) => {
                      const betList = getBetsForPosition(bet.type, bet.type);
                      return (
                        <div class="bg-green-700 border-2 border-white rounded h-10 flex-1 flex items-center justify-center relative">
                          <div class="text-white text-xs font-bold">{bet.label}</div>
                          <Show when={betList.length > 0}>
                            <div class="absolute inset-0 flex items-center justify-center pointer-events-none">
                              <For each={betList}>
                                {(b) => (
                                  <div class="absolute bg-yellow-500 text-black text-xs px-1.5 py-0.5 rounded-full font-bold shadow-lg border border-white animate-[bounce_0.5s_ease-in-out]">
                                    {b.amount}
                                  </div>
                                )}
                              </For>
                            </div>
                          </Show>
                        </div>
                      );
                    }}
                  </For>
                </div>
              </div>
            </div>
          </div>
        </div>
      </Show>
    </div>
  );
}

render(() => <RouletteOverlay />, document.getElementById('root'));
