import { createSignal, createEffect, onCleanup, Show } from 'solid-js';
import { withingsAPI } from './api';
import { Activity, RefreshCw, TrendingUp, Droplet, Dumbbell } from 'lucide-solid';

export default function WithingsViewport() {
  const [authStatus, setAuthStatus] = createSignal(null);
  const [stats, setStats] = createSignal(null);
  const [latestMeasurement, setLatestMeasurement] = createSignal(null);
  const [syncing, setSyncing] = createSignal(false);
  const [loading, setLoading] = createSignal(true);

  const fetchData = async () => {
    try {
      const [statusData, statsData, latestData] = await Promise.all([
        withingsAPI.getAuthStatus(),
        withingsAPI.getMeasurementStats(),
        withingsAPI.getLatestMeasurements(),
      ]);

      setAuthStatus(statusData);
      setStats(statsData);
      setLatestMeasurement(latestData);
    } catch (error) {
      console.error('Failed to fetch Withings data:', error);
    } finally {
      setLoading(false);
    }
  };

  const handleSync = async () => {
    setSyncing(true);
    try {
      await withingsAPI.syncMeasurements();
      await fetchData();
    } catch (error) {
      console.error('Failed to sync measurements:', error);
    } finally {
      setSyncing(false);
    }
  };

  createEffect(() => {
    fetchData();

    // Poll for updates every 30 seconds
    const interval = setInterval(fetchData, 30000);
    onCleanup(() => clearInterval(interval));
  });

  const formatDate = (timestamp) => {
    if (!timestamp) return 'N/A';
    return new Date(timestamp * 1000).toLocaleString();
  };

  const formatValue = (value, unit = '') => {
    if (value === null || value === undefined) return 'N/A';
    return `${value.toFixed(1)}${unit}`;
  };

  return (
    <div class="p-6 space-y-6">
      {/* Header */}
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-3">
          <Activity class="w-8 h-8 text-primary" />
          <div>
            <h1 class="text-3xl font-bold">Withings Dashboard</h1>
            <p class="text-base-content/60">Body composition tracking and analytics</p>
          </div>
        </div>

        <button
          class="btn btn-primary gap-2"
          classList={{ 'loading': syncing() }}
          onClick={handleSync}
          disabled={syncing()}
        >
          <Show when={!syncing()}>
            <RefreshCw class="w-4 h-4" />
          </Show>
          {syncing() ? 'Syncing...' : 'Sync Data'}
        </button>
      </div>

      <Show when={loading()}>
        <div class="flex justify-center items-center h-64">
          <span class="loading loading-spinner loading-lg"></span>
        </div>
      </Show>

      <Show when={!loading()}>
        {/* Auth Status */}
        <div class="alert" classList={{
          'alert-success': authStatus()?.authenticated,
          'alert-warning': !authStatus()?.authenticated,
        }}>
          <div>
            <h3 class="font-bold">
              {authStatus()?.authenticated ? 'Connected' : 'Not Connected'}
            </h3>
            <div class="text-sm">
              {authStatus()?.authenticated
                ? `Connected as: ${authStatus()?.user_id}`
                : 'Please connect your Withings account to view your data'}
            </div>
          </div>
        </div>

        {/* Latest Measurement Card */}
        <Show when={latestMeasurement()}>
          <div class="card bg-base-200 shadow-xl">
            <div class="card-body">
              <h2 class="card-title">Latest Measurement</h2>
              <p class="text-sm text-base-content/60">
                {formatDate(latestMeasurement()?.timestamp)}
              </p>

              <div class="grid grid-cols-1 md:grid-cols-3 gap-4 mt-4">
                <div class="stat bg-base-100 rounded-lg">
                  <div class="stat-figure text-primary">
                    <TrendingUp class="w-8 h-8" />
                  </div>
                  <div class="stat-title">Weight</div>
                  <div class="stat-value text-primary">
                    {formatValue(latestMeasurement()?.weight)}
                  </div>
                  <div class="stat-desc">kg</div>
                </div>

                <div class="stat bg-base-100 rounded-lg">
                  <div class="stat-figure text-secondary">
                    <Dumbbell class="w-8 h-8" />
                  </div>
                  <div class="stat-title">Muscle Mass</div>
                  <div class="stat-value text-secondary">
                    {formatValue(latestMeasurement()?.muscle_mass)}
                  </div>
                  <div class="stat-desc">kg</div>
                </div>

                <div class="stat bg-base-100 rounded-lg">
                  <div class="stat-figure text-accent">
                    <Droplet class="w-8 h-8" />
                  </div>
                  <div class="stat-title">Body Water</div>
                  <div class="stat-value text-accent">
                    {formatValue(latestMeasurement()?.hydration)}
                  </div>
                  <div class="stat-desc">kg</div>
                </div>
              </div>

              <div class="grid grid-cols-2 md:grid-cols-4 gap-4 mt-4">
                <div class="text-center p-3 bg-base-100 rounded-lg">
                  <div class="text-xs text-base-content/60">Fat Mass</div>
                  <div class="text-lg font-bold">{formatValue(latestMeasurement()?.fat_mass)} kg</div>
                </div>

                <div class="text-center p-3 bg-base-100 rounded-lg">
                  <div class="text-xs text-base-content/60">Fat Ratio</div>
                  <div class="text-lg font-bold">{formatValue(latestMeasurement()?.fat_ratio)} %</div>
                </div>

                <div class="text-center p-3 bg-base-100 rounded-lg">
                  <div class="text-xs text-base-content/60">Bone Mass</div>
                  <div class="text-lg font-bold">{formatValue(latestMeasurement()?.bone_mass)} kg</div>
                </div>

                <div class="text-center p-3 bg-base-100 rounded-lg">
                  <div class="text-xs text-base-content/60">Fat Free Mass</div>
                  <div class="text-lg font-bold">{formatValue(latestMeasurement()?.fat_free_mass)} kg</div>
                </div>
              </div>
            </div>
          </div>
        </Show>

        {/* Statistics */}
        <Show when={stats()?.total > 0}>
          <div class="card bg-base-200 shadow-xl">
            <div class="card-body">
              <h2 class="card-title">Overall Statistics</h2>

              <div class="grid grid-cols-2 md:grid-cols-3 gap-4 mt-4">
                <div class="stat bg-base-100 rounded-lg">
                  <div class="stat-title">Total Measurements</div>
                  <div class="stat-value">{stats()?.total || 0}</div>
                </div>

                <div class="stat bg-base-100 rounded-lg">
                  <div class="stat-title">Avg Weight</div>
                  <div class="stat-value text-sm">{formatValue(stats()?.avg_weight)} kg</div>
                </div>

                <div class="stat bg-base-100 rounded-lg">
                  <div class="stat-title">Avg Muscle</div>
                  <div class="stat-value text-sm">{formatValue(stats()?.avg_muscle)} kg</div>
                </div>

                <div class="stat bg-base-100 rounded-lg">
                  <div class="stat-title">Avg Hydration</div>
                  <div class="stat-value text-sm">{formatValue(stats()?.avg_hydration)} kg</div>
                </div>

                <div class="stat bg-base-100 rounded-lg">
                  <div class="stat-title">First Measurement</div>
                  <div class="stat-value text-xs">{formatDate(stats()?.first_measurement)}</div>
                </div>

                <div class="stat bg-base-100 rounded-lg">
                  <div class="stat-title">Last Measurement</div>
                  <div class="stat-value text-xs">{formatDate(stats()?.last_measurement)}</div>
                </div>
              </div>
            </div>
          </div>
        </Show>

        {/* Info Card */}
        <Show when={!latestMeasurement()}>
          <div class="card bg-base-200 shadow-xl">
            <div class="card-body text-center">
              <Activity class="w-16 h-16 mx-auto text-base-content/40" />
              <h2 class="card-title justify-center">No Measurements Yet</h2>
              <p class="text-base-content/60">
                Click "Sync Data" to fetch your latest measurements from Withings.
              </p>
            </div>
          </div>
        </Show>
      </Show>
    </div>
  );
}
