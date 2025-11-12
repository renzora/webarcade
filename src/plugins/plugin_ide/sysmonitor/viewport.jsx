import { createSignal, onCleanup, onMount } from 'solid-js';
import { fetchStats, fetchHistory } from './api.js';

export default function MyViewport() {
  const [stats, setStats] = createSignal({
    cpu_usage: 0,
    cpu_count: 0,
    ram_used: 0,
    ram_total: 0,
    ram_percent: 0
  });
  const [history, setHistory] = createSignal([]);

  let interval;

  const updateStats = async () => {
    try {
      const data = await fetchStats();
      setStats(data);
    } catch (error) {
      console.error('[sysmonitor] Failed to fetch stats:', error);
    }
  };

  const loadHistory = async () => {
    try {
      const data = await fetchHistory();
      setHistory(data);
    } catch (error) {
      console.error('[sysmonitor] Failed to fetch history:', error);
    }
  };

  onMount(() => {
    updateStats();
    loadHistory();
    interval = setInterval(() => {
      updateStats();
      loadHistory();
    }, 2000);
  });

  onCleanup(() => {
    if (interval) clearInterval(interval);
  });

  const formatBytes = (bytes) => {
    return (bytes / 1024 / 1024 / 1024).toFixed(2);
  };

  return (
    <div class="h-full w-full flex flex-col bg-base-200 p-6 overflow-y-auto">
      <h1 class="text-3xl font-bold mb-6">System Monitor</h1>

      {/* Current Stats */}
      <div class="grid grid-cols-1 md:grid-cols-2 gap-6 mb-6">
        {/* CPU Card */}
        <div class="card bg-base-100 shadow-xl">
          <div class="card-body">
            <h2 class="card-title">CPU Usage</h2>
            <div class="flex items-center justify-center py-4">
              <div class="radial-progress text-primary" style={`--value:${stats().cpu_usage}`} role="progressbar">
                {stats().cpu_usage.toFixed(1)}%
              </div>
            </div>
            <div class="divider"></div>
            <div class="stat-desc text-center">
              {stats().cpu_count} CPU Cores Available
            </div>
          </div>
        </div>

        {/* RAM Card */}
        <div class="card bg-base-100 shadow-xl">
          <div class="card-body">
            <h2 class="card-title">RAM Usage</h2>
            <div class="flex items-center justify-center py-4">
              <div class="radial-progress text-secondary" style={`--value:${stats().ram_percent}`} role="progressbar">
                {stats().ram_percent.toFixed(1)}%
              </div>
            </div>
            <div class="divider"></div>
            <div class="stat-desc text-center">
              {formatBytes(stats().ram_used)} GB / {formatBytes(stats().ram_total)} GB
            </div>
          </div>
        </div>
      </div>

      {/* Historical Data */}
      <div class="card bg-base-100 shadow-xl">
        <div class="card-body">
          <h2 class="card-title">Recent History</h2>
          <div class="overflow-x-auto">
            <table class="table table-zebra">
              <thead>
                <tr>
                  <th>Time</th>
                  <th>CPU Usage</th>
                  <th>RAM Used</th>
                </tr>
              </thead>
              <tbody>
                {history().slice(0, 10).map((stat) => (
                  <tr>
                    <td>{new Date(stat.timestamp * 1000).toLocaleTimeString()}</td>
                    <td>
                      <div class="badge badge-primary">{stat.cpu_usage.toFixed(1)}%</div>
                    </td>
                    <td>
                      {formatBytes(stat.ram_used)} GB / {formatBytes(stat.ram_total)} GB
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      </div>
    </div>
  );
}
