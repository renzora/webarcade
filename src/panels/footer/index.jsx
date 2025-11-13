import { createSignal, createEffect, onCleanup, For } from 'solid-js';
import { editorStore } from '@/layout/stores/EditorStore';
import { IconAlertTriangle, IconBrandGithub, IconBrandDiscord } from '@tabler/icons-solidjs';
import { footerButtons } from '@/api/plugin';
import BuildProgress from './BuildProgress';
import DebugConsole from './DebugConsole';
import { setShowAbout } from '../../plugins/menu/index.jsx';
import { bridge } from '@/api/bridge';

const Footer = () => {
  const [engineInfo] = createSignal('Engine Ready');
  const [systemStats, setSystemStats] = createSignal(null);

  // Ultra-fast system stats polling (100ms for near real-time)
  createEffect(() => {
    const fetchSystemStats = async () => {
      try {
        const response = await bridge('/system/stats');
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
  
  // Get reactive values from editor store
  const selection = () => editorStore.selection;
  const selectedEntity = () => selection().entity;
  const transformMode = () => selection().transformMode;
  
  const getSelectionInfo = () => {
    if (selectedEntity()) {
      return `Selected: ${selectedEntity()}`;
    }
    return null; // Return null instead of "No selection"
  };
  
  const getTransformModeInfo = () => {
    if (transformMode() && transformMode() !== 'select') {
      return ` | ${transformMode().charAt(0).toUpperCase() + transformMode().slice(1)} mode`;
    }
    return '';
  };
  
  
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
  
  const renderSystemStatsInfo = () => {
    const stats = systemStats();
    if (!stats) return <span class="text-base-content/90">Loading...</span>;
    
    const cpu = Math.round(stats.cpu_usage);
    const memory = Math.round(stats.memory_usage);
    const gpu = stats.gpu_usage ? Math.round(stats.gpu_usage) : null;
    
    return (
      <div class="flex items-center gap-2">
        {/* CPU */}
        <div class="flex items-center gap-1">
          <span class={getUsageColor(cpu, 'cpu')}>
            CPU: {cpu}%
          </span>
          {needsWarningIcon(cpu, 'cpu') && (
            <IconAlertTriangle class="w-3 h-3 text-error animate-pulse" />
          )}
        </div>
        
        <span class="text-base-content/30">|</span>
        
        {/* RAM */}
        <div class="flex items-center gap-1">
          <span class={getUsageColor(memory, 'memory')}>
            RAM: {memory}%
          </span>
          {needsWarningIcon(memory, 'memory') && (
            <IconAlertTriangle class="w-3 h-3 text-error animate-pulse" />
          )}
        </div>
        
        {/* GPU */}
        {gpu !== null && (
          <>
            <span class="text-base-content/30">|</span>
            <div class="flex items-center gap-1">
              <span class={getUsageColor(gpu, 'gpu')}>
                GPU: {gpu}%
              </span>
              {needsWarningIcon(gpu, 'gpu') && (
                <IconAlertTriangle class="w-3 h-3 text-error animate-pulse" />
              )}
            </div>
          </>
        )}
      </div>
    );
  };
  
  return (
    <div class="fixed bottom-0 left-0 right-0 h-6 bg-base-200 backdrop-blur-md border-t border-base-content/10 text-xs flex items-center justify-between px-3 pointer-events-auto z-50 rounded-t-none">
      {/* Left side - WebArcade branding and links */}
      <div class="flex items-center gap-3">
        {getSelectionInfo() ? (
          <span class="text-base-content/90">
            {getSelectionInfo()}{getTransformModeInfo()}
          </span>
        ) : (
          <>
            <button
              onClick={() => setShowAbout(true)}
              class="text-base-content/70 hover:text-primary font-medium transition-colors cursor-pointer"
              title="About WebArcade"
            >
              WebArcade v1.0.0
            </button>
            <span class="text-base-content/30">|</span>
            <a
              href="https://github.com/pianoplayerjames/webarcade"
              target="_blank"
              rel="noopener noreferrer"
              class="flex items-center gap-1 text-base-content/60 hover:text-primary transition-colors"
              title="GitHub Repository"
            >
              <IconBrandGithub class="w-3.5 h-3.5" />
              <span>GitHub</span>
            </a>
            <a
              href="https://discord.gg/G9WBkSu6Ta"
              target="_blank"
              rel="noopener noreferrer"
              class="flex items-center gap-1 text-base-content/60 hover:text-primary transition-colors"
              title="Join Discord"
            >
              <IconBrandDiscord class="w-3.5 h-3.5" />
              <span>Discord</span>
            </a>
          </>
        )}
      </div>
      
      {/* Right side - Status info */}
      <div class="flex items-center gap-4">
        {renderSystemStatsInfo()}

        <span class="text-base-content/30">|</span>

        {/* Build Progress */}
        <BuildProgress />

        <span class="text-base-content/90">
          {engineInfo()}
        </span>

        {/* Debug Console */}
        <span class="text-base-content/30">|</span>
        <DebugConsole />

        {/* Plugin footer buttons */}
        <For each={Array.from(footerButtons().entries())}>
          {([_id, button]) => {
            const Component = button.component;
            return Component ? <Component /> : null;
          }}
        </For>
      </div>
    </div>
  );
};

export default Footer;