import { createSignal, createEffect, onCleanup } from 'solid-js';
import { IconAlertTriangle } from '@tabler/icons-solidjs';
import { api } from '@/api/bridge';

export default function SystemMonitorFooterButton() {
  const [systemStats, setSystemStats] = createSignal(null);

  createEffect(() => {
    const fetchSystemStats = async () => {
      try {
        const response = await api('systemMonitor/stats');
        if (response.ok) {
          const stats = await response.json();
          setSystemStats(stats);
        }
      } catch (error) {
        // Silently fail - system stats are not critical
      }
    };

    fetchSystemStats();
    const interval = setInterval(fetchSystemStats, 2000); // Poll every 2 seconds

    onCleanup(() => clearInterval(interval));
  });

  const getUsageColor = (usage, type = 'default') => {
    // Different thresholds for different resources
    let yellow, red;
    switch (type) {
      case 'cpu':
        yellow = 70;
        red = 85;
        break;
      case 'memory':
        yellow = 75;
        red = 90;
        break;
      case 'gpu':
        yellow = 80;
        red = 95;
        break;
      default:
        yellow = 70;
        red = 85;
    }

    if (usage >= red) return 'text-error'; // Red for critical
    if (usage >= yellow) return 'text-warning'; // Yellow for warning
    return 'text-success'; // Green for good
  };

  const needsWarningIcon = (usage, type = 'default') => {
    let red;
    switch (type) {
      case 'cpu':
        red = 85;
        break;
      case 'memory':
        red = 90;
        break;
      case 'gpu':
        red = 95;
        break;
      default:
        red = 85;
    }
    return usage >= red;
  };

  const stats = () => systemStats();

  return (
    <div class="flex items-center gap-2">
      {!stats() ? (
        <span class="text-base-content/90">Loading...</span>
      ) : (
        <>
          {/* CPU */}
          <div class="flex items-center gap-1">
            <span class={getUsageColor(Math.round(stats().cpu_usage), 'cpu')}>
              CPU: {Math.round(stats().cpu_usage)}%
            </span>
            {needsWarningIcon(Math.round(stats().cpu_usage), 'cpu') && (
              <IconAlertTriangle class="w-3 h-3 text-error animate-pulse" />
            )}
          </div>

          <span class="text-base-content/30">|</span>

          {/* RAM */}
          <div class="flex items-center gap-1">
            <span class={getUsageColor(Math.round(stats().memory_usage), 'memory')}>
              RAM: {Math.round(stats().memory_usage)}%
            </span>
            {needsWarningIcon(Math.round(stats().memory_usage), 'memory') && (
              <IconAlertTriangle class="w-3 h-3 text-error animate-pulse" />
            )}
          </div>

          {/* GPU */}
          {stats().gpu_usage !== null && stats().gpu_usage !== undefined && (
            <>
              <span class="text-base-content/30">|</span>
              <div class="flex items-center gap-1">
                <span class={getUsageColor(Math.round(stats().gpu_usage), 'gpu')}>
                  GPU: {Math.round(stats().gpu_usage)}%
                </span>
                {needsWarningIcon(Math.round(stats().gpu_usage), 'gpu') && (
                  <IconAlertTriangle class="w-3 h-3 text-error animate-pulse" />
                )}
              </div>
            </>
          )}
        </>
      )}
    </div>
  );
}
