import { createSignal, onMount, For, Show, createEffect, onCleanup } from 'solid-js';
import { bridgeFetch } from '@/api/bridge.js';
import { IconScale, IconRefresh, IconAlertCircle, IconActivity, IconTrendingUp, IconTrendingDown } from '@tabler/icons-solidjs';
import { Chart, registerables } from 'chart.js';
import 'chartjs-adapter-date-fns';

Chart.register(...registerables);

export default function WithingsViewport() {
  const [latestWeight, setLatestWeight] = createSignal(null);
  const [history, setHistory] = createSignal([]);
  const [loading, setLoading] = createSignal(true);
  const [historyDays, setHistoryDays] = createSignal(30);
  const [showConfigModal, setShowConfigModal] = createSignal(false);
  const [clientId, setClientId] = createSignal('');
  const [clientSecret, setClientSecret] = createSignal('');
  const [isConfigured, setIsConfigured] = createSignal(false);
  const [heightCm, setHeightCm] = createSignal(175); // Default height in cm

  let weightChartRef;
  let bmiChartRef;
  let fatMassChartRef;
  let muscleMassChartRef;
  let weightChartInstance = null;
  let bmiChartInstance = null;
  let fatMassChartInstance = null;
  let muscleMassChartInstance = null;

  onMount(async () => {
    await loadData();
    await checkConfig();
  });

  const checkConfig = async () => {
    try {
      const response = await bridgeFetch('/withings/config');
      const data = await response.json();
      if (data.success && data.data) {
        setIsConfigured(data.data.configured || false);
        if (data.data.client_id) {
          setClientId(data.data.client_id);
        }
      }
    } catch (e) {
      console.error('Failed to check config:', e);
    }
  };

  const loadData = async () => {
    setLoading(true);
    try {
      // Load latest weight
      const latestResponse = await bridgeFetch('/withings/latest');
      const latestData = await latestResponse.json();
      if (latestData.success && latestData.data) {
        setLatestWeight(latestData.data);
      }

      // Load weight history
      const daysAgo = historyDays();
      const startDate = Math.floor(Date.now() / 1000) - (daysAgo * 86400);
      const historyResponse = await bridgeFetch(`/withings/history?start_date=${startDate}&limit=100`);
      const historyData = await historyResponse.json();
      if (historyData.success && historyData.data) {
        setHistory(historyData.data);
      }
    } catch (e) {
      console.error('Failed to load Withings data:', e);
    } finally {
      setLoading(false);
    }
  };

  const saveConfig = async () => {
    try {
      const response = await bridgeFetch('/withings/config', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          client_id: clientId(),
          client_secret: clientSecret()
        })
      });
      const data = await response.json();
      if (data.success) {
        alert('Credentials saved! Now click "Connect Withings" to authorize.');
        setShowConfigModal(false);
        setClientSecret('');
        await checkConfig();
      } else {
        alert(data.error || 'Failed to save credentials');
      }
    } catch (e) {
      console.error('Failed to save config:', e);
      alert(`Failed to save config: ${e.message}`);
    }
  };

  const connectWithings = async () => {
    try {
      const response = await bridgeFetch('/withings/auth-url');
      const data = await response.json();
      if (data.success && data.auth_url) {
        window.open(data.auth_url, '_blank', 'width=600,height=800');
      } else {
        alert(data.error || 'Failed to get authorization URL');
      }
    } catch (e) {
      console.error('Failed to get auth URL:', e);
      alert(`Failed to get auth URL: ${e.message}`);
    }
  };

  const syncWithings = async () => {
    try {
      setLoading(true);
      const response = await bridgeFetch('/withings/sync', { method: 'POST' });
      const data = await response.json();
      if (data.success) {
        alert(data.content || 'Synced successfully!');
        await loadData();
      } else {
        alert(data.error || 'Sync failed');
      }
    } catch (e) {
      console.error('Failed to sync:', e);
      alert(`Sync failed: ${e.message}`);
    } finally {
      setLoading(false);
    }
  };

  const formatDate = (timestamp) => {
    const date = new Date(timestamp * 1000);
    return date.toLocaleDateString('en-US', { month: 'short', day: 'numeric', year: 'numeric' });
  };

  const formatDateTime = (timestamp) => {
    const date = new Date(timestamp * 1000);
    return date.toLocaleString('en-US', {
      month: 'short',
      day: 'numeric',
      year: 'numeric',
      hour: 'numeric',
      minute: '2-digit'
    });
  };

  const formatWeight = (kg) => {
    const lbs = kg * 2.20462;
    return `${kg.toFixed(1)} kg / ${lbs.toFixed(1)} lbs`;
  };

  const calculateBMI = (weightKg, heightCm) => {
    const heightM = heightCm / 100;
    return weightKg / (heightM * heightM);
  };

  const getMetricChange = (metricName) => {
    const hist = history();
    if (hist.length < 2) return null;

    let getMetric;
    switch(metricName) {
      case 'weight':
        getMetric = (entry) => entry.weight;
        break;
      case 'bmi':
        getMetric = (entry) => calculateBMI(entry.weight, heightCm());
        break;
      case 'fat_mass':
        getMetric = (entry) => entry.fat_mass;
        break;
      case 'muscle_mass':
        getMetric = (entry) => entry.muscle_mass;
        break;
      default:
        return null;
    }

    const validEntries = hist.filter(e => getMetric(e) != null);
    if (validEntries.length < 2) return null;

    const latest = getMetric(validEntries[0]);
    const oldest = getMetric(validEntries[validEntries.length - 1]);
    const change = latest - oldest;
    const percent = ((change / oldest) * 100);

    return {
      value: change,
      percent: percent,
      isPositive: change > 0
    };
  };

  const getWeightChange = () => {
    const hist = history();
    if (hist.length < 2) return null;
    const latest = hist[0].weight;
    const oldest = hist[hist.length - 1].weight;
    const change = latest - oldest;
    return {
      kg: change,
      lbs: change * 2.20462,
      percent: ((change / oldest) * 100).toFixed(1)
    };
  };

  // Update charts when history changes
  createEffect(() => {
    const hist = history();
    if (hist.length === 0) return;

    // Small delay to ensure canvas elements are mounted
    setTimeout(() => {
      // Prepare data
      const sortedHistory = [...hist].reverse(); // Oldest to newest
      const labels = sortedHistory.map(e => new Date(e.date * 1000));

      console.log('Creating charts with', sortedHistory.length, 'data points');

      // Weight chart
      if (weightChartRef) {
        if (weightChartInstance) {
          weightChartInstance.destroy();
        }
        try {
            weightChartInstance = new Chart(weightChartRef, {
              type: 'line',
              data: {
                labels: labels,
                datasets: [{
                  label: 'Weight (kg)',
                  data: sortedHistory.map(e => e.weight),
                  borderColor: '#3b82f6',
                  backgroundColor: 'rgba(59, 130, 246, 0.1)',
                  tension: 0.4,
                  fill: true
                }]
              },
              options: {
                responsive: true,
                maintainAspectRatio: false,
                plugins: {
                  legend: { display: false }
                },
                scales: {
                  x: { type: 'time', time: { unit: 'day' } },
                  y: { beginAtZero: false }
                }
              }
            });
            console.log('Weight chart created');
          } catch (e) {
            console.error('Failed to create weight chart:', e);
          }
        }

        // BMI chart
        if (bmiChartRef) {
          if (bmiChartInstance) {
            bmiChartInstance.destroy();
          }
          try {
            bmiChartInstance = new Chart(bmiChartRef, {
              type: 'line',
              data: {
                labels: labels,
                datasets: [{
                  label: 'BMI',
                  data: sortedHistory.map(e => calculateBMI(e.weight, heightCm())),
                  borderColor: '#8b5cf6',
                  backgroundColor: 'rgba(139, 92, 246, 0.1)',
                  tension: 0.4,
                  fill: true
                }]
              },
              options: {
                responsive: true,
                maintainAspectRatio: false,
                plugins: {
                  legend: { display: false }
                },
                scales: {
                  x: { type: 'time', time: { unit: 'day' } },
                  y: { beginAtZero: false }
                }
              }
            });
            console.log('BMI chart created');
          } catch (e) {
            console.error('Failed to create BMI chart:', e);
          }
        }

        // Fat Mass chart
        if (fatMassChartRef && sortedHistory.some(e => e.fat_mass)) {
          if (fatMassChartInstance) {
            fatMassChartInstance.destroy();
          }
          try {
            fatMassChartInstance = new Chart(fatMassChartRef, {
              type: 'line',
              data: {
                labels: labels,
                datasets: [{
                  label: 'Fat Mass (kg)',
                  data: sortedHistory.map(e => e.fat_mass),
                  borderColor: '#f59e0b',
                  backgroundColor: 'rgba(245, 158, 11, 0.1)',
                  tension: 0.4,
                  fill: true,
                  spanGaps: true
                }]
              },
              options: {
                responsive: true,
                maintainAspectRatio: false,
                plugins: {
                  legend: { display: false }
                },
                scales: {
                  x: { type: 'time', time: { unit: 'day' } },
                  y: { beginAtZero: false }
                }
              }
            });
            console.log('Fat mass chart created');
          } catch (e) {
            console.error('Failed to create fat mass chart:', e);
          }
        }

        // Muscle Mass chart
        if (muscleMassChartRef && sortedHistory.some(e => e.muscle_mass)) {
          if (muscleMassChartInstance) {
            muscleMassChartInstance.destroy();
          }
          try {
            muscleMassChartInstance = new Chart(muscleMassChartRef, {
              type: 'line',
              data: {
                labels: labels,
                datasets: [{
                  label: 'Muscle Mass (kg)',
                  data: sortedHistory.map(e => e.muscle_mass),
                  borderColor: '#10b981',
                  backgroundColor: 'rgba(16, 185, 129, 0.1)',
                  tension: 0.4,
                  fill: true,
                  spanGaps: true
                }]
              },
              options: {
                responsive: true,
                maintainAspectRatio: false,
                plugins: {
                  legend: { display: false }
                },
                scales: {
                  x: { type: 'time', time: { unit: 'day' } },
                  y: { beginAtZero: false }
                }
              }
            });
            console.log('Muscle mass chart created');
          } catch (e) {
            console.error('Failed to create muscle mass chart:', e);
          }
        }
      }, 100); // 100ms delay
    });

  // Cleanup charts on unmount
  onCleanup(() => {
    if (weightChartInstance) weightChartInstance.destroy();
    if (bmiChartInstance) bmiChartInstance.destroy();
    if (fatMassChartInstance) fatMassChartInstance.destroy();
    if (muscleMassChartInstance) muscleMassChartInstance.destroy();
  });

  return (
    <div class="h-full flex flex-col bg-base-200">
      {/* Header */}
      <div class="flex items-center justify-between bg-base-100 border-b border-base-300 px-4 py-3">
        <div class="flex items-center gap-3 flex-1">
          <IconScale size={20} class="text-primary" />
          <h2 class="text-lg font-semibold">Withings Health Data</h2>
        </div>

        <div class="flex gap-2">
          <div class="flex items-center gap-2">
            <label class="text-xs">Height:</label>
            <input
              type="number"
              class="input input-bordered input-sm w-20"
              value={heightCm()}
              onInput={(e) => setHeightCm(parseInt(e.target.value) || 175)}
              min="100"
              max="250"
            />
            <span class="text-xs">cm</span>
          </div>
          <select
            class="select select-bordered select-sm"
            value={historyDays()}
            onChange={(e) => {
              setHistoryDays(parseInt(e.target.value));
              loadData();
            }}
          >
            <option value={7}>Last 7 days</option>
            <option value={30}>Last 30 days</option>
            <option value={90}>Last 90 days</option>
            <option value={365}>Last year</option>
          </select>
          <button
            class="btn btn-sm btn-ghost"
            onClick={loadData}
            disabled={loading()}
          >
            <IconRefresh size={16} class={loading() ? 'animate-spin' : ''} />
          </button>
          <Show when={!isConfigured()}>
            <button
              class="btn btn-sm btn-ghost"
              onClick={() => setShowConfigModal(true)}
            >
              Configure
            </button>
          </Show>
          <Show when={isConfigured()}>
            <button
              class="btn btn-sm btn-ghost"
              onClick={connectWithings}
            >
              Connect Withings
            </button>
            <button
              class="btn btn-sm btn-primary"
              onClick={syncWithings}
              disabled={loading()}
            >
              Sync
            </button>
          </Show>
        </div>
      </div>

      <div class="flex-1 overflow-y-auto p-4">
        <Show
          when={!loading()}
          fallback={
            <div class="flex items-center justify-center h-full">
              <div class="text-center">
                <IconRefresh size={48} class="mx-auto mb-4 opacity-30 animate-spin" />
                <p class="text-sm text-base-content/60">Loading health data...</p>
              </div>
            </div>
          }
        >
          <div class="grid gap-4">
            {/* Metrics Grid with Charts */}
            <Show
              when={latestWeight() && history().length > 0}
              fallback={
                <div class="card bg-base-100 shadow-sm">
                  <div class="card-body">
                    <div class="text-center py-12">
                      <IconAlertCircle size={64} class="mx-auto mb-4 opacity-30" />
                      <h3 class="text-lg font-semibold mb-2">No Weight Data</h3>
                      <p class="text-sm text-base-content/60 mb-4">
                        {isConfigured()
                          ? 'Connect to Withings and sync your data to see charts'
                          : 'Configure your Withings API credentials to get started'}
                      </p>
                      <Show when={!isConfigured()}>
                        <button
                          class="btn btn-primary btn-sm"
                          onClick={() => setShowConfigModal(true)}
                        >
                          Configure Now
                        </button>
                      </Show>
                      <Show when={isConfigured()}>
                        <div class="flex gap-2 justify-center">
                          <button
                            class="btn btn-ghost btn-sm"
                            onClick={connectWithings}
                          >
                            Connect Withings
                          </button>
                          <button
                            class="btn btn-primary btn-sm"
                            onClick={syncWithings}
                          >
                            Sync Data
                          </button>
                        </div>
                      </Show>
                    </div>
                  </div>
                </div>
              }
            >
              <div class="grid grid-cols-1 lg:grid-cols-2 gap-4">
                {/* Weight Chart */}
                <div class="card bg-base-100 shadow-sm">
                  <div class="card-body p-4">
                    <div class="flex items-center justify-between mb-3">
                      <div>
                        <h3 class="text-sm font-semibold">Weight</h3>
                        <div class="text-2xl font-bold text-blue-500">
                          {latestWeight().weight.toFixed(1)} kg
                        </div>
                        <div class="text-xs text-base-content/60">
                          {(latestWeight().weight * 2.20462).toFixed(1)} lbs
                        </div>
                      </div>
                      <Show when={getMetricChange('weight')}>
                        {(change) => (
                          <div class={`flex items-center gap-1 text-sm font-semibold ${change().isPositive ? 'text-error' : 'text-success'}`}>
                            {change().isPositive ? <IconTrendingUp size={16} /> : <IconTrendingDown size={16} />}
                            <span>{change().percent > 0 ? '+' : ''}{change().percent.toFixed(1)}%</span>
                          </div>
                        )}
                      </Show>
                    </div>
                    <div style={{ height: '200px' }}>
                      <canvas ref={weightChartRef}></canvas>
                    </div>
                  </div>
                </div>

                {/* BMI Chart */}
                <div class="card bg-base-100 shadow-sm">
                  <div class="card-body p-4">
                    <div class="flex items-center justify-between mb-3">
                      <div>
                        <h3 class="text-sm font-semibold">BMI</h3>
                        <div class="text-2xl font-bold text-purple-500">
                          {calculateBMI(latestWeight().weight, heightCm()).toFixed(1)}
                        </div>
                        <div class="text-xs text-base-content/60">
                          Height: {heightCm()} cm
                        </div>
                      </div>
                      <Show when={getMetricChange('bmi')}>
                        {(change) => (
                          <div class={`flex items-center gap-1 text-sm font-semibold ${change().isPositive ? 'text-error' : 'text-success'}`}>
                            {change().isPositive ? <IconTrendingUp size={16} /> : <IconTrendingDown size={16} />}
                            <span>{change().percent > 0 ? '+' : ''}{change().percent.toFixed(1)}%</span>
                          </div>
                        )}
                      </Show>
                    </div>
                    <div style={{ height: '200px' }}>
                      <canvas ref={bmiChartRef}></canvas>
                    </div>
                  </div>
                </div>

                {/* Fat Mass Chart */}
                <Show when={latestWeight().fat_mass}>
                  <div class="card bg-base-100 shadow-sm">
                    <div class="card-body p-4">
                      <div class="flex items-center justify-between mb-3">
                        <div>
                          <h3 class="text-sm font-semibold">Fat Mass</h3>
                          <div class="text-2xl font-bold text-orange-500">
                            {latestWeight().fat_mass.toFixed(1)} kg
                          </div>
                          <div class="text-xs text-base-content/60">
                            {(latestWeight().fat_mass * 2.20462).toFixed(1)} lbs
                          </div>
                        </div>
                        <Show when={getMetricChange('fat_mass')}>
                          {(change) => (
                            <div class={`flex items-center gap-1 text-sm font-semibold ${change().isPositive ? 'text-error' : 'text-success'}`}>
                              {change().isPositive ? <IconTrendingUp size={16} /> : <IconTrendingDown size={16} />}
                              <span>{change().percent > 0 ? '+' : ''}{change().percent.toFixed(1)}%</span>
                            </div>
                          )}
                        </Show>
                      </div>
                      <div style={{ height: '200px' }}>
                        <canvas ref={fatMassChartRef}></canvas>
                      </div>
                    </div>
                  </div>
                </Show>

                {/* Muscle Mass Chart */}
                <Show when={latestWeight().muscle_mass}>
                  <div class="card bg-base-100 shadow-sm">
                    <div class="card-body p-4">
                      <div class="flex items-center justify-between mb-3">
                        <div>
                          <h3 class="text-sm font-semibold">Muscle Mass</h3>
                          <div class="text-2xl font-bold text-green-500">
                            {latestWeight().muscle_mass.toFixed(1)} kg
                          </div>
                          <div class="text-xs text-base-content/60">
                            {(latestWeight().muscle_mass * 2.20462).toFixed(1)} lbs
                          </div>
                        </div>
                        <Show when={getMetricChange('muscle_mass')}>
                          {(change) => (
                            <div class={`flex items-center gap-1 text-sm font-semibold ${change().isPositive ? 'text-success' : 'text-error'}`}>
                              {change().isPositive ? <IconTrendingUp size={16} /> : <IconTrendingDown size={16} />}
                              <span>{change().percent > 0 ? '+' : ''}{change().percent.toFixed(1)}%</span>
                            </div>
                          )}
                        </Show>
                      </div>
                      <div style={{ height: '200px' }}>
                        <canvas ref={muscleMassChartRef}></canvas>
                      </div>
                    </div>
                  </div>
                </Show>
              </div>
            </Show>

            {/* Weight History Table */}
            <div class="card bg-base-100 shadow-sm">
              <div class="card-body">
                <h3 class="card-title text-sm">Weight History</h3>
                <Show
                  when={history().length > 0}
                  fallback={
                    <div class="text-center py-8">
                      <IconAlertCircle size={48} class="mx-auto mb-4 opacity-30" />
                      <p class="text-sm font-semibold mb-2">No weight data</p>
                      <p class="text-xs text-base-content/60">Sync with Withings to import your data</p>
                    </div>
                  }
                >
                  <div class="overflow-x-auto">
                    <table class="table table-sm">
                      <thead>
                        <tr>
                          <th>Date</th>
                          <th>Weight</th>
                          <th>Fat Mass</th>
                          <th>Muscle Mass</th>
                        </tr>
                      </thead>
                      <tbody>
                        <For each={history()}>
                          {(entry) => (
                            <tr class="hover">
                              <td class="text-xs">{formatDate(entry.date)}</td>
                              <td class="font-semibold">{formatWeight(entry.weight)}</td>
                              <td class="text-sm">
                                {entry.fat_mass ? `${entry.fat_mass.toFixed(1)} kg` : '-'}
                              </td>
                              <td class="text-sm">
                                {entry.muscle_mass ? `${entry.muscle_mass.toFixed(1)} kg` : '-'}
                              </td>
                            </tr>
                          )}
                        </For>
                      </tbody>
                    </table>
                  </div>
                </Show>
              </div>
            </div>
          </div>
        </Show>
      </div>

      {/* Configuration Modal */}
      <Show when={showConfigModal()}>
        <div class="modal modal-open">
          <div class="modal-box">
            <h3 class="font-bold text-lg mb-4">Configure Withings API</h3>

            <div class="space-y-4">
              <div>
                <label class="label">
                  <span class="label-text">Client ID</span>
                </label>
                <input
                  type="text"
                  placeholder="Paste your Withings Client ID"
                  class="input input-bordered w-full"
                  value={clientId()}
                  onInput={(e) => setClientId(e.target.value)}
                />
              </div>

              <div>
                <label class="label">
                  <span class="label-text">Client Secret</span>
                </label>
                <input
                  type="password"
                  placeholder="Paste your Withings Client Secret"
                  class="input input-bordered w-full"
                  value={clientSecret()}
                  onInput={(e) => setClientSecret(e.target.value)}
                />
              </div>

              <div class="alert alert-info text-sm">
                <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" class="stroke-current shrink-0 w-6 h-6"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path></svg>
                <div>
                  <div class="font-semibold">Setup Instructions:</div>
                  <ol class="list-decimal list-inside mt-1 space-y-1">
                    <li>Create an app at <a href="https://developer.withings.com" target="_blank" class="link">developer.withings.com</a></li>
                    <li>Set callback URL to: <code class="bg-base-300 px-1 rounded">http://localhost:3001/withings/callback</code></li>
                    <li>Copy the Client ID and Client Secret</li>
                    <li>Paste them here and click "Connect Withings"</li>
                  </ol>
                </div>
              </div>
            </div>

            <div class="modal-action">
              <button
                class="btn"
                onClick={() => {
                  setShowConfigModal(false);
                  setClientSecret('');
                }}
              >
                Cancel
              </button>
              <button
                class="btn btn-primary"
                onClick={saveConfig}
                disabled={!clientId() || !clientSecret()}
              >
                Save & Continue
              </button>
            </div>
          </div>
        </div>
      </Show>
    </div>
  );
}
