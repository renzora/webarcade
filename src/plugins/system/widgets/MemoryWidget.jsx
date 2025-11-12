import { createSignal, createEffect, onCleanup } from 'solid-js';
import { IconActivity } from '@tabler/icons-solidjs';
import { bridge } from '@/api/bridge'

export default function MemoryWidget() {
  const [memoryUsage, setMemoryUsage] = createSignal(0);

  createEffect(() => {
    const fetchSystemStats = async () => {
      try {
        const response = await bridge('/system/stats');
        if (response.ok) {
          const stats = await response.json();
          setMemoryUsage(stats.memory_usage || 0);
        }
      } catch (error) {
        // Handle error silently
      }
    };

    fetchSystemStats();
    const interval = setInterval(fetchSystemStats, 2000);
    onCleanup(() => clearInterval(interval));
  });

  const getUsageColor = (usage) => {
    if (usage >= 85) return 'text-error';
    if (usage >= 70) return 'text-warning';
    return 'text-success';
  };

  const getProgressColor = (usage) => {
    if (usage >= 85) return 'progress-error';
    if (usage >= 70) return 'progress-warning';
    return 'progress-success';
  };

  const getBgGradient = (usage) => {
    if (usage >= 85) return 'from-error/20 to-error/5';
    if (usage >= 70) return 'from-warning/20 to-warning/5';
    return 'from-success/20 to-success/5';
  };

  return (
    <div class={`card bg-gradient-to-br ${getBgGradient(memoryUsage())} bg-base-100 shadow-lg h-full flex flex-col justify-between p-4`}>
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-2">
          <IconActivity size={20} class="opacity-60" />
          <span class="text-sm font-medium opacity-70">Memory</span>
        </div>
      </div>

      <div class={`text-4xl font-bold ${getUsageColor(memoryUsage())}`}>
        {memoryUsage().toFixed(1)}%
      </div>

      <progress
        class={`progress ${getProgressColor(memoryUsage())} w-full h-2`}
        value={memoryUsage()}
        max="100"
      />
    </div>
  );
}
