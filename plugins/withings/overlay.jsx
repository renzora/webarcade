import { render } from 'solid-js/web';
import { createSignal, createEffect, onCleanup, Show } from 'solid-js';
import '@/index.css';
import { WEBARCADE_WS, BRIDGE_API } from '@/api/bridge';

function WeightOverlay() {
  const [isConnected, setIsConnected] = createSignal(false);
  const [currentWeight, setCurrentWeight] = createSignal(null);
  const [weightChange, setWeightChange] = createSignal(null);
  const [visible, setVisible] = createSignal(true);
  const [heightCm] = createSignal(175); // Default height

  const calculateBMI = (weightKg, heightCm) => {
    const heightM = heightCm / 100;
    return (weightKg / (heightM * heightM)).toFixed(1);
  };

  let ws;

  // Fetch weight data from API
  const fetchWeightData = async () => {
    try {
      // Get latest weight
      const latestResponse = await fetch(`${BRIDGE_API}/withings/latest`);
      const latestData = await latestResponse.json();
      if (latestData.success && latestData.data) {
        setCurrentWeight(latestData.data);
      }

      // Get weight history for trend
      const startDate = Math.floor(Date.now() / 1000) - (30 * 86400); // Last 30 days
      const historyResponse = await fetch(`${BRIDGE_API}/withings/history?start_date=${startDate}&limit=100`);
      const historyData = await historyResponse.json();
      if (historyData.success && historyData.data && historyData.data.length >= 2) {
        const latest = historyData.data[0].weight;
        const oldest = historyData.data[historyData.data.length - 1].weight;
        const change = latest - oldest;
        setWeightChange(change);
      }
    } catch (error) {
      console.error('Failed to fetch weight data:', error);
    }
  };

  // Connect to WebSocket
  const connectWebSocket = () => {
    ws = new WebSocket(WEBARCADE_WS);

    ws.onopen = () => {
      console.log('✅ Connected to WebArcade');
      setIsConnected(true);
      fetchWeightData();
    };

    ws.onclose = () => {
      console.log('❌ Disconnected');
      setIsConnected(false);
      setTimeout(connectWebSocket, 3000);
    };

    ws.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data);
        // Handle weight update events
        if (data.type === 'weight_update') {
          fetchWeightData();
        }
        // Handle overlay visibility toggle
        if (data.type === 'overlay_toggle' && data.overlay === 'weight') {
          setVisible(data.visible);
        }
      } catch (error) {
        console.error('Error parsing event:', error);
      }
    };
  };

  const formatWeight = (kg) => {
    const lbs = kg * 2.20462;
    return {
      kg: kg.toFixed(1),
      lbs: lbs.toFixed(1)
    };
  };

  const formatDate = (timestamp) => {
    const date = new Date(timestamp * 1000);
    return date.toLocaleDateString('en-US', { month: 'short', day: 'numeric' });
  };

  // Initialize WebSocket and fetch data
  createEffect(() => {
    connectWebSocket();
    onCleanup(() => ws?.close());
  });

  return (
    <div class="fixed inset-0 pointer-events-none overflow-hidden font-sans bg-transparent">
      <Show when={visible() && currentWeight()}>
        <div class="absolute bottom-8 right-8 pointer-events-none">
          <div class="bg-gradient-to-br from-black/90 to-black/70 backdrop-blur-xl shadow-2xl rounded-2xl overflow-hidden min-w-[320px] border border-white/10">
            {/* Header */}
            <div class="bg-gradient-to-r from-primary/20 to-accent/20 px-6 py-3 border-b border-white/10">
              <div class="flex items-center justify-between">
                <div class="flex items-center gap-2">
                  <div class="text-2xl">⚖️</div>
                  <h2 class="text-xl font-bold text-white">Weight Tracker</h2>
                </div>
                <div class="text-xs text-white/60">
                  {formatDate(currentWeight().date)}
                </div>
              </div>
            </div>

            {/* Current Weight */}
            <div class="px-6 py-6">
              <div class="text-center">
                <div class="text-sm text-white/60 font-semibold mb-2">Current Weight</div>
                <div class="text-5xl font-bold text-white mb-2">
                  {formatWeight(currentWeight().weight).lbs}
                  <span class="text-2xl text-white/60 ml-2">lbs</span>
                </div>
                <div class="text-lg text-white/60 mb-3">
                  {formatWeight(currentWeight().weight).kg} kg
                </div>

                {/* BMI */}
                <div class="inline-flex items-center gap-2 px-4 py-2 bg-purple-500/20 rounded-full">
                  <span class="text-sm text-white/60">BMI:</span>
                  <span class="text-xl font-bold text-purple-400">
                    {calculateBMI(currentWeight().weight, heightCm())}
                  </span>
                </div>

                {/* Weight Change */}
                <Show when={weightChange() !== null}>
                  <div class={`mt-4 inline-flex items-center gap-1 px-3 py-1.5 rounded-full text-sm font-bold ${
                    weightChange() > 0 ? 'bg-error/20 text-error' : 'bg-success/20 text-success'
                  }`}>
                    <span class="text-lg">
                      {weightChange() > 0 ? '↑' : '↓'}
                    </span>
                    <span>
                      {Math.abs(weightChange() * 2.20462).toFixed(1)} lbs
                    </span>
                    <span class="text-xs opacity-60">(30 days)</span>
                  </div>
                </Show>
              </div>

              {/* Body Composition */}
              <Show when={currentWeight().fat_mass || currentWeight().muscle_mass}>
                <div class="mt-6 pt-6 border-t border-white/10">
                  <div class="grid grid-cols-2 gap-4">
                    <Show when={currentWeight().fat_mass}>
                      <div class="bg-white/5 rounded-lg p-3">
                        <div class="text-xs text-white/60 font-semibold mb-1">Fat Mass</div>
                        <div class="text-xl font-bold text-orange-400">
                          {(currentWeight().fat_mass * 2.20462).toFixed(1)}
                          <span class="text-sm text-white/60 ml-1">lbs</span>
                        </div>
                        <div class="text-xs text-white/60">
                          {currentWeight().fat_mass.toFixed(1)} kg
                        </div>
                      </div>
                    </Show>
                    <Show when={currentWeight().muscle_mass}>
                      <div class="bg-white/5 rounded-lg p-3">
                        <div class="text-xs text-white/60 font-semibold mb-1">Muscle Mass</div>
                        <div class="text-xl font-bold text-green-400">
                          {(currentWeight().muscle_mass * 2.20462).toFixed(1)}
                          <span class="text-sm text-white/60 ml-1">lbs</span>
                        </div>
                        <div class="text-xs text-white/60">
                          {currentWeight().muscle_mass.toFixed(1)} kg
                        </div>
                      </div>
                    </Show>
                    <Show when={currentWeight().bone_mass}>
                      <div class="bg-white/5 rounded-lg p-3">
                        <div class="text-xs text-white/60 font-semibold mb-1">Bone Mass</div>
                        <div class="text-xl font-bold text-blue-400">
                          {(currentWeight().bone_mass * 2.20462).toFixed(1)}
                          <span class="text-sm text-white/60 ml-1">lbs</span>
                        </div>
                        <div class="text-xs text-white/60">
                          {currentWeight().bone_mass.toFixed(1)} kg
                        </div>
                      </div>
                    </Show>
                    <Show when={currentWeight().hydration}>
                      <div class="bg-white/5 rounded-lg p-3">
                        <div class="text-xs text-white/60 font-semibold mb-1">Hydration</div>
                        <div class="text-xl font-bold text-cyan-400">
                          {(currentWeight().hydration * 2.20462).toFixed(1)}
                          <span class="text-sm text-white/60 ml-1">lbs</span>
                        </div>
                        <div class="text-xs text-white/60">
                          {currentWeight().hydration.toFixed(1)} kg
                        </div>
                      </div>
                    </Show>
                  </div>
                </div>
              </Show>
            </div>

            {/* Connection Status */}
            <div class="px-4 py-2 bg-black/30 border-t border-white/10">
              <div class="flex items-center justify-center gap-2 text-xs">
                <div class={`w-2 h-2 rounded-full ${isConnected() ? 'bg-success' : 'bg-error'} animate-pulse`}></div>
                <span class="text-white/60">
                  {isConnected() ? 'Connected' : 'Disconnected'}
                </span>
              </div>
            </div>
          </div>
        </div>
      </Show>
    </div>
  );
}

// Only render when used as standalone (for OBS browser sources)
if (document.getElementById('root')) {
  render(() => <WeightOverlay />, document.getElementById('root'));
}
