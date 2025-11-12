import { createSignal, onCleanup, onMount } from 'solid-js';
import { fetchStats } from '../../api.js';

export default function CpuWidget(props) {
  const [stats, setStats] = createSignal({ cpu_usage: 0, cpu_count: 0 });
  let interval;

  const updateStats = async () => {
    try {
      const data = await fetchStats();
      setStats(data);
    } catch (error) {
      console.error('[sysmonitor] Failed to fetch CPU stats:', error);
    }
  };

  onMount(() => {
    updateStats();
    interval = setInterval(updateStats, 2000);
  });

  onCleanup(() => {
    if (interval) clearInterval(interval);
  });

  return (
    <div class="card bg-base-100 shadow-xl h-full">
      <div class="card-body">
        <h2 class="card-title text-sm">CPU Usage</h2>
        <div class="flex items-center justify-center flex-1">
          <div class="radial-progress text-primary" style={`--value:${stats().cpu_usage}`} role="progressbar">
            {stats().cpu_usage.toFixed(1)}%
          </div>
        </div>
        <div class="text-xs text-center opacity-70">
          {stats().cpu_count} cores
        </div>
      </div>
    </div>
  );
}
