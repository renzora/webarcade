import { createSignal, createEffect, onCleanup } from 'solid-js';
import { IconLayoutDashboard, IconCheck } from '@tabler/icons-solidjs';

export default function DemoFooterButton() {
  const [uptime, setUptime] = createSignal(0);
  const [isActive, setIsActive] = createSignal(true);

  createEffect(() => {
    const startTime = Date.now();

    const interval = setInterval(() => {
      setUptime(Math.floor((Date.now() - startTime) / 1000));
    }, 1000);

    onCleanup(() => clearInterval(interval));
  });

  const formatUptime = (seconds) => {
    const mins = Math.floor(seconds / 60);
    const secs = seconds % 60;
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  };

  return (
    <div class="flex items-center gap-2">
      <div
        class="flex items-center gap-1.5 cursor-pointer hover:opacity-80 transition-opacity"
        onClick={() => setIsActive(!isActive())}
        title="Click to toggle demo status"
      >
        <IconLayoutDashboard class="w-3.5 h-3.5 text-primary" />
        <span class="text-base-content/90">Demo</span>
        {isActive() ? (
          <span class="flex items-center gap-0.5 text-success">
            <IconCheck class="w-3 h-3" />
            Active
          </span>
        ) : (
          <span class="text-base-content/50">Inactive</span>
        )}
      </div>
      <span class="text-base-content/30">|</span>
      <span class="text-base-content/70 tabular-nums">
        {formatUptime(uptime())}
      </span>
    </div>
  );
}
