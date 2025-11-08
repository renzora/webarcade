import { createSignal, onMount, onCleanup, Show } from 'solid-js';
import { isServerConnected } from './server.jsx';

export default function BridgeStatus({ onOpenViewport }) {
  const [bridgeConnected, setBridgeConnected] = createSignal(false);
  const [showTooltip, setShowTooltip] = createSignal(false);
  
  let statusCheckInterval;

  // Check bridge connection status
  const checkBridgeStatus = async () => {
    try {
      const connected = await isServerConnected();
      setBridgeConnected(connected);
    } catch {
      setBridgeConnected(false);
    }
  };

  const handleClick = () => {
    if (onOpenViewport) {
      onOpenViewport();
    }
  };

  onMount(() => {
    // Initial check
    checkBridgeStatus();
    
    // Check every 5 seconds
    statusCheckInterval = setInterval(checkBridgeStatus, 5000);
  });

  onCleanup(() => {
    if (statusCheckInterval) {
      clearInterval(statusCheckInterval);
    }
  });

  return (
    <div class="relative">
      <button
        onClick={handleClick}
        onMouseEnter={() => setShowTooltip(true)}
        onMouseLeave={() => setShowTooltip(false)}
        class={`p-2 rounded-full transition-colors ${
          bridgeConnected() 
            ? 'text-green-500 hover:bg-green-500/10' 
            : 'text-red-500 hover:bg-red-500/10'
        }`}
        title={bridgeConnected() ? 'Bridge Connected' : 'Bridge Disconnected'}
      >
        <div class={`w-3 h-3 rounded-full ${
          bridgeConnected() ? 'bg-green-500' : 'bg-red-500'
        }`} />
      </button>
      
      <Show when={showTooltip()}>
        <div class="absolute bottom-full right-0 mb-2 px-2 py-1 bg-gray-800 text-white text-xs rounded whitespace-nowrap z-50">
          {bridgeConnected() ? 'Bridge Server: Connected' : 'Bridge Server: Disconnected'}
        </div>
      </Show>
    </div>
  );
}