import { createSignal, createEffect, onCleanup, Show } from 'solid-js';
import { IconHammer, IconCheck, IconX, IconRefresh } from '@tabler/icons-solidjs';
import { bridge } from '@/api/bridge';

const BuildProgress = () => {
  const [buildStatus, setBuildStatus] = createSignal({
    state: 'idle',
    progress: 0,
    message: 'Ready',
    timestamp: 0,
  });

  // Poll build status
  createEffect(() => {
    const fetchBuildStatus = async () => {
      try {
        const response = await bridge('/system/build-progress');
        if (response.ok) {
          const status = await response.json();
          setBuildStatus(status);
        }
      } catch (error) {
        // Silently fail
      }
    };

    fetchBuildStatus();
    const interval = setInterval(fetchBuildStatus, 1000); // Poll every 1 second

    onCleanup(() => clearInterval(interval));
  });

  const handleRebuild = async () => {
    try {
      const response = await bridge('/system/trigger-rebuild', {
        method: 'POST',
      });

      if (response.ok) {
        console.log('Rebuild triggered successfully');
      } else {
        console.error('Failed to trigger rebuild');
      }
    } catch (error) {
      console.error('Error triggering rebuild:', error);
    }
  };

  const getStatusColor = () => {
    const status = buildStatus();
    if (status.state === 'compiling') return 'text-warning';
    if (status.state === 'success') return 'text-success';
    if (status.state === 'error') return 'text-error';
    if (status.state === 'warning') return 'text-warning';
    return 'text-base-content/90';
  };

  const getStatusIcon = () => {
    const status = buildStatus();
    if (status.state === 'compiling') {
      return <IconHammer class="w-3 h-3 animate-pulse" />;
    }
    if (status.state === 'success') {
      return <IconCheck class="w-3 h-3" />;
    }
    if (status.state === 'error') {
      return <IconX class="w-3 h-3" />;
    }
    if (status.state === 'warning') {
      return <IconCheck class="w-3 h-3" />;
    }
    return null;
  };

  const isBuilding = () => buildStatus().state === 'compiling';
  const showProgress = () => isBuilding() && buildStatus().progress > 0;

  return (
    <div class="flex items-center gap-2">
      {/* Build Status */}
      <div class={`flex items-center gap-1 ${getStatusColor()}`}>
        {getStatusIcon()}
        <div class="flex flex-col">
          <span class="text-xs">{buildStatus().message}</span>
          <Show when={showProgress()}>
            <div class="w-24 h-1 bg-base-300 rounded-full overflow-hidden mt-0.5">
              <div
                class="h-full bg-warning transition-all duration-300"
                style={{ width: `${buildStatus().progress}%` }}
              />
            </div>
          </Show>
        </div>
      </div>

      {/* Rebuild Button */}
      <button
        onClick={handleRebuild}
        disabled={isBuilding()}
        class="btn btn-xs btn-ghost disabled:opacity-50"
        title="Trigger rebuild"
      >
        <IconRefresh class={`w-3 h-3 ${isBuilding() ? 'animate-spin' : ''}`} />
      </button>
    </div>
  );
};

export default BuildProgress;
