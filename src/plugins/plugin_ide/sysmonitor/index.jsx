import { createPlugin } from '@/api/plugin';
import { IconCpu, IconDeviceDesktop } from '@tabler/icons-solidjs';
import { createSignal } from 'solid-js';
import MyViewport from './viewport.jsx';
import { fetchStats } from './api.js';

export default createPlugin({
  id: 'sysmonitor',
  name: 'System Monitor',
  version: '1.0.0',
  description: 'System monitoring plugin for CPU and RAM usage',
  author: 'WebArcade',

  async onStart(api) {
    console.log('[sysmonitor] Starting...');

    const [stats, setStats] = createSignal({ cpu_usage: 0, ram_percent: 0 });

    // Update stats every 2 seconds
    const updateStats = async () => {
      try {
        const data = await fetchStats();
        setStats(data);
      } catch (error) {
        console.error('[sysmonitor] Failed to fetch stats:', error);
      }
    };

    // Initial fetch
    updateStats();
    const interval = setInterval(updateStats, 2000);

    // Register viewport
    api.viewport('sysmonitor-viewport', {
      label: 'System Monitor',
      component: MyViewport,
      icon: IconDeviceDesktop,
      description: 'Monitor CPU and RAM usage'
    });

    // Register menu item
    api.menu('sysmonitor-menu', {
      label: 'System Monitor',
      icon: IconDeviceDesktop,
      onClick: () => {
        api.open('sysmonitor-viewport', { label: 'System Monitor' });
      }
    });

    // Register status bar item
    api.status('sysmonitor-status', {
      label: () => `CPU: ${stats().cpu_usage?.toFixed(1) || 0}% | RAM: ${stats().ram_percent?.toFixed(1) || 0}%`,
      icon: IconCpu,
      onClick: () => {
        api.open('sysmonitor-viewport', { label: 'System Monitor' });
      }
    });

    // Register right panel tab
    api.rightPanel('sysmonitor-panel', {
      label: 'System',
      icon: IconCpu,
      component: () => (
        <div class="p-4 space-y-4">
          <h3 class="text-lg font-bold">System Status</h3>
          <div class="stats stats-vertical shadow w-full">
            <div class="stat">
              <div class="stat-title">CPU Usage</div>
              <div class="stat-value text-2xl">{stats().cpu_usage?.toFixed(1) || 0}%</div>
              <div class="stat-desc">{stats().cpu_count || 0} cores</div>
            </div>
            <div class="stat">
              <div class="stat-title">RAM Usage</div>
              <div class="stat-value text-2xl">{stats().ram_percent?.toFixed(1) || 0}%</div>
              <div class="stat-desc">
                {((stats().ram_used || 0) / 1024 / 1024 / 1024).toFixed(2)} GB /
                {((stats().ram_total || 0) / 1024 / 1024 / 1024).toFixed(2)} GB
              </div>
            </div>
          </div>
        </div>
      )
    });

    // Cleanup on stop
    return () => {
      clearInterval(interval);
    };
  }
});
