import { createSignal, onMount, Show, For, onCleanup } from 'solid-js';
import { BRIDGE_API, WEBARCADE_WS } from '@/api/bridge.js';

const RouletteViewport = () => {
  const [currentGame, setCurrentGame] = createSignal(null);
  const [bets, setBets] = createSignal([]);
  const [history, setHistory] = createSignal([]);
  const [loading, setLoading] = createSignal(false);
  const [channel, setChannel] = createSignal('pianofire'); // Default channel
  const [swapLayout, setSwapLayout] = createSignal(false);
  const [isEnabled, setIsEnabled] = createSignal(true);

  // Load layout preference and enabled state from database
  onMount(async () => {
    try {
      const response = await fetch(`${BRIDGE_API}/system/settings?key=roulette-swap-layout`);
      const data = await response.json();
      if (data.value === 'true') {
        setSwapLayout(true);
      }
    } catch (error) {
      console.error('Failed to load layout preference:', error);
    }

    // Load enabled state
    try {
      const response = await fetch(`${BRIDGE_API}/roulette/enabled?channel=${channel()}`);
      const data = await response.json();
      setIsEnabled(data.enabled);
    } catch (error) {
            <p><strong>Enable/Disable:</strong> !roulette on | !roulette off</p>
      console.error('Failed to load enabled state:', error);
    }
  });

  // Roulette number colors
  const getNumberColor = (num) => {
    if (num === 0) return 'green';
    const redNumbers = [1, 3, 5, 7, 9, 12, 14, 16, 18, 19, 21, 23, 25, 27, 30, 32, 34, 36];
    return redNumbers.includes(num) ? 'red' : 'black';
  };

  // Load current game
  const loadCurrentGame = async () => {
    try {
      const response = await fetch(`${BRIDGE_API}/roulette/game?channel=${channel()}`);
      const data = await response.json();
      setCurrentGame(data.game);
      setBets(data.bets || []);
    } catch (error) {
      console.error('Failed to load current game:', error);
    }
  };

  // Load game history
  const loadHistory = async () => {
    try {
      const response = await fetch(`${BRIDGE_API}/roulette/history?channel=${channel()}`);
      const data = await response.json();
      setHistory(data);
    } catch (error) {
      console.error('Failed to load history:', error);
    }
  };

  // WebSocket connection for real-time updates
  onMount(() => {
    loadCurrentGame();
    loadHistory();

    // Set up WebSocket for real-time updates
    const ws = new WebSocket(WEBARCADE_WS);

    ws.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data);

        if (data.type === 'roulette_bet_placed' || data.type === 'roulette_result') {
          // Reload current game when bets are placed or wheel is spun
          loadCurrentGame();
          if (data.type === 'roulette_result') {
            loadHistory();
          }
        }
      } catch (error) {
        console.error('WebSocket message error:', error);
      }
    };

    onCleanup(() => {
      ws.close();
    });

    // Poll for updates every 5 seconds as backup
    const interval = setInterval(() => {
      loadCurrentGame();
    }, 5000);

    onCleanup(() => {
      clearInterval(interval);
    });
  });

  // Calculate total wagered and potential payouts
  const getTotalWagered = () => {
    return bets().reduce((sum, bet) => sum + bet.amount, 0);
  };

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

  // Toggle layout swap and save to database
  const toggleLayoutSwap = async () => {
    const newValue = !swapLayout();
    setSwapLayout(newValue);

    try {
      await fetch(`${BRIDGE_API}/system/settings`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          key: 'roulette-swap-layout',
          value: newValue.toString()
        })
      });
    } catch (error) {
      console.error('Failed to save layout preference:', error);
    }
  };

  // Toggle enabled state
  const toggleEnabled = async () => {
    const newValue = !isEnabled();
    setIsEnabled(newValue);

    try {
      await fetch(`${BRIDGE_API}/roulette/toggle`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          channel: channel(),
          enabled: newValue
        })
      });
    } catch (error) {
      console.error('Failed to toggle enabled state:', error);
      // Revert on error
      setIsEnabled(!newValue);
    }
  };

  return (
    <div class="p-6 h-full overflow-y-auto bg-gray-900 text-white">
      <div class="max-w-6xl mx-auto space-y-6">
        {/* Header */}
        <div class="flex items-center justify-between">
          <h1 class="text-3xl font-bold">Roulette Manager</h1>
          <div class="flex gap-3">
            <button
              onClick={toggleEnabled}
              class={`px-4 py-2 rounded-lg transition font-semibold ${
                isEnabled()
                  ? 'bg-green-600 hover:bg-green-700 text-white'
                  : 'bg-red-600 hover:bg-red-700 text-white'
              }`}
              title="Toggle roulette overlay visibility"
            >
              {isEnabled() ? '✓ Enabled' : '✗ Disabled'}
            </button>
            <button
              onClick={toggleLayoutSwap}
              class={`px-4 py-2 rounded-lg transition font-semibold ${
                swapLayout()
                  ? 'bg-yellow-600 hover:bg-yellow-700 text-white'
                  : 'bg-gray-700 hover:bg-gray-600 text-gray-300'
              }`}
              title="Swap wheel and table positions in overlay"
            >
              ⇄ {swapLayout() ? 'Wheel Left' : 'Wheel Right'}
            </button>
            <button
              onClick={() => { loadCurrentGame(); loadHistory(); }}
              class="px-4 py-2 bg-blue-600 hover:bg-blue-700 rounded-lg transition"
            >
              Refresh
            </button>
          </div>
        </div>

        {/* Instructions */}
        <div class="bg-gray-800 rounded-lg p-4 border border-gray-700">
          <h2 class="text-lg font-semibold mb-2">How to Use</h2>
          <div class="text-sm text-gray-300 space-y-1">
            <p><strong>Start Game:</strong> !roulette start</p>
            <p><strong>Place Bet:</strong> !bet {'<amount>'} {'<type>'} (e.g., !bet 100 red or !bet 50 17)</p>
            <p><strong>Spin Wheel:</strong> !roulette spin</p>
            <p><strong>Cancel Bets:</strong> !roulette cancel</p>
          </div>
        </div>

        {/* Current Game */}
        <div class="bg-gray-800 rounded-lg p-6 border border-gray-700">
          <h2 class="text-xl font-bold mb-4">Current Game</h2>
          <Show
            when={currentGame()}
            fallback={
              <div class="text-center py-8 text-gray-400">
                <p>No active game</p>
                <p class="text-sm mt-2">Start a game with <code class="bg-gray-700 px-2 py-1 rounded">!roulette start</code></p>
              </div>
            }
          >
            <div class="space-y-4">
              <div class="flex items-center justify-between">
                <div>
                  <span class="text-gray-400">Status:</span>
                  <span class={`ml-2 px-3 py-1 rounded-full text-sm font-semibold ${
                    currentGame().status === 'betting' ? 'bg-green-600' :
                    currentGame().status === 'spinning' ? 'bg-yellow-600' :
                    'bg-gray-600'
                  }`}>
                    {currentGame().status.toUpperCase()}
                  </span>
                </div>
                <div class="text-right">
                  <div class="text-gray-400 text-sm">Total Wagered</div>
                  <div class="text-2xl font-bold text-yellow-400">{getTotalWagered()} coins</div>
                </div>
              </div>

              <Show when={currentGame().winning_number !== null}>
                <div class="bg-gray-900 rounded-lg p-4 text-center">
                  <div class="text-gray-400 text-sm mb-2">Winning Number</div>
                  <div class="flex items-center justify-center gap-3">
                    <div class={`text-6xl font-bold ${
                      getNumberColor(currentGame().winning_number) === 'red' ? 'text-red-500' :
                      getNumberColor(currentGame().winning_number) === 'black' ? 'text-white' :
                      'text-green-500'
                    }`}>
                      {currentGame().winning_number}
                    </div>
                    <div class="text-2xl">
                      {getNumberColor(currentGame().winning_number) === 'red' ? '🔴' :
                       getNumberColor(currentGame().winning_number) === 'black' ? '⚫' :
                       '🟢'}
                    </div>
                  </div>
                </div>
              </Show>

              {/* Bets Breakdown */}
              <div>
                <h3 class="text-lg font-semibold mb-3">Active Bets ({bets().length})</h3>
                <Show
                  when={bets().length > 0}
                  fallback={<p class="text-gray-400 text-center py-4">No bets placed yet</p>}
                >
                  <div class="space-y-2">
                    <For each={getBetsByType()}>
                      {(group) => (
                        <div class="bg-gray-900 rounded p-3">
                          <div class="flex items-center justify-between mb-2">
                            <div class="font-semibold">
                              {group.value.toUpperCase()}
                              <span class="text-gray-400 text-sm ml-2">({group.bets.length} bets)</span>
                            </div>
                            <div class="text-yellow-400 font-bold">{group.total} coins</div>
                          </div>
                          <div class="flex flex-wrap gap-2">
                            <For each={group.bets}>
                              {(bet) => (
                                <div class="text-xs bg-gray-800 px-2 py-1 rounded">
                                  {bet.username}: {bet.amount}
                                </div>
                              )}
                            </For>
                          </div>
                        </div>
                      )}
                    </For>
                  </div>
                </Show>
              </div>
            </div>
          </Show>
        </div>

        {/* Game History */}
        <div class="bg-gray-800 rounded-lg p-6 border border-gray-700">
          <h2 class="text-xl font-bold mb-4">Recent Games</h2>
          <Show
            when={history().length > 0}
            fallback={<p class="text-gray-400 text-center py-4">No completed games yet</p>}
          >
            <div class="space-y-2">
              <For each={history()}>
                {(game) => (
                  <div class="bg-gray-900 rounded p-4 flex items-center justify-between">
                    <div class="flex items-center gap-4">
                      <div class={`text-3xl font-bold ${
                        getNumberColor(game.winning_number) === 'red' ? 'text-red-500' :
                        getNumberColor(game.winning_number) === 'black' ? 'text-white' :
                        'text-green-500'
                      }`}>
                        {game.winning_number}
                      </div>
                      <div>
                        <div class="text-sm text-gray-400">
                          {new Date(game.completed_at * 1000).toLocaleString()}
                        </div>
                        <div class="text-xs text-gray-500">Game #{game.id}</div>
                      </div>
                    </div>
                    <div class="text-right">
                      <div class="text-gray-400 text-sm">Channel</div>
                      <div class="font-semibold">{game.channel}</div>
                    </div>
                  </div>
                )}
              </For>
            </div>
          </Show>
        </div>
      </div>
    </div>
  );
};

export default RouletteViewport;
