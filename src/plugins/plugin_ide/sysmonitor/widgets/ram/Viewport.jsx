import { createSignal, onCleanup, onMount } from 'solid-js';
import { fetchStats } from '../../api.js';

export default function RamWidget(props) {
  const [stats, setStats] = createSignal({ ram_used: 0, ram_total: 0, ram_percent: 0 });
  let interval;

  const updateStats = async () => {
    try {
      const data = await fetchStats();
      setStats(data);
    } catch (error) {
      console.error('[sysmonitor] Failed to fetch RAM stats:', error);
    }
  };

  onMount(() => {
    updateStats();
    interval = setInterval(updateStats, 2000);
  });

  onCleanup(() => {
    if (interval) clearInterval(interval);
  });

  const formatBytes = (bytes) => {
    return (bytes / 1024 / 1024 / 1024).toFixed(2);
  };

  return (
    <div class="card bg-base-100 shadow-xl h-full">
      <div class="card-body">
        <h2 class="card-title text-sm">RAM Usage</h2>
        <div class="flex items-center justify-center flex-1">
          <div class="radial-progress text-secondary" style={`--value:${stats().ram_percent}`} role="progressbar">
            {stats().ram_percent.toFixed(1)}%
          </div>
        </div>
        <div class="text-xs text-center opacity-70">
          {formatBytes(stats().ram_used)} / {formatBytes(stats().ram_total)} GB
        </div>
      </div>
    </div>
  );
}
