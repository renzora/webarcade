import { render } from 'solid-js/web';
import { createSignal, createEffect, onCleanup, For, Show } from 'solid-js';
import '@/index.css';
import { WEBARCADE_WS } from '@/api/bridge';

const BRIDGE_URL = 'http://localhost:3001';
const FRONTEND_URL = window.location.origin; // Use the same origin as the layout overlay

function LayoutOverlay() {
  let containerRef;
  const [layoutName, setLayoutName] = createSignal('');
  const [overlays, setOverlays] = createSignal([]);
  const [error, setError] = createSignal(null);
  const [loading, setLoading] = createSignal(true);
  let ws;

  // Track iframes by overlay ID to prevent recreation
  const iframeMap = new Map();

  // Get layout name from URL
  const getLayoutName = () => {
    const path = window.location.pathname;
    const match = path.match(/\/overlay\/layout\/([^/]+)/);
    return match ? decodeURIComponent(match[1]) : null;
  };

  // Load layout configuration
  const loadLayout = async () => {
    const name = getLayoutName();
    if (!name) {
      setError('No layout name specified in URL');
      setLoading(false);
      return;
    }

    setLayoutName(name);

    try {
      const response = await fetch(`${BRIDGE_URL}/layouts/${encodeURIComponent(name)}`);

      if (!response.ok) {
        throw new Error(`Failed to load layout: ${response.statusText}`);
      }

      const layout = await response.json();
      console.log('Loaded layout:', layout);

      if (!layout.overlays || layout.overlays.length === 0) {
        setError('Layout has no overlays');
        setLoading(false);
        return;
      }

      setOverlays(layout.overlays);
      setLoading(false);
    } catch (err) {
      console.error('Failed to load layout:', err);
      setError(err.message);
      setLoading(false);
    }
  };

  // Connect to WebSocket for real-time updates
  const connectWebSocket = () => {
    ws = new WebSocket(WEBARCADE_WS);

    ws.onopen = () => {
      console.log('✅ Connected to WebArcade WebSocket');
    };

    ws.onclose = () => {
      console.log('❌ Disconnected from WebSocket');
      setTimeout(connectWebSocket, 3000);
    };

    ws.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data);

        // Handle layout update events
        if (data.type === 'layout_update') {
          const currentLayoutName = layoutName();
          console.log('📡 Received layout update. Current:', currentLayoutName, 'Update for:', data.layout_name);
          // Only update if this is the layout we're displaying
          if (data.layout_name === currentLayoutName && data.layout?.overlays) {
            console.log('✅ Updating overlays (iframes will reposition, not reload)');
            setOverlays(data.layout.overlays);
          }
        }
      } catch (error) {
        console.error('Error parsing WebSocket message:', error);
      }
    };
  };

  // Manually manage iframes to prevent recreation on updates
  createEffect(() => {
    if (!containerRef || loading() || error()) return;

    const currentOverlays = overlays();
    const currentIds = new Set(currentOverlays.map(o => o.id));

    // Remove iframes that are no longer in the overlay list
    for (const [id, iframe] of iframeMap.entries()) {
      if (!currentIds.has(id)) {
        iframe.remove();
        iframeMap.delete(id);
      }
    }

    // Update or create iframes
    currentOverlays.forEach(overlay => {
      let iframe = iframeMap.get(overlay.id);

      if (!iframe) {
        // Create new iframe with cache-busting parameter
        iframe = document.createElement('iframe');
        const cacheBuster = Date.now();
        iframe.src = `${FRONTEND_URL}/overlay/${overlay.type}?v=${cacheBuster}`;
        iframe.className = 'absolute border-none pointer-events-auto';
        iframe.style.display = 'block';
        iframe.style.border = 'none';
        iframe.style.margin = '0';
        iframe.style.padding = '0';
        iframe.title = overlay.type;
        containerRef.appendChild(iframe);
        iframeMap.set(overlay.id, iframe);
      }

      // Update iframe position and size
      iframe.style.left = `${overlay.x}px`;
      iframe.style.top = `${overlay.y}px`;
      iframe.style.width = `${overlay.width}px`;
      iframe.style.height = `${overlay.height}px`;
      iframe.style.zIndex = overlay.zIndex || 1;
    });
  });

  createEffect(() => {
    loadLayout();
    connectWebSocket();

    onCleanup(() => {
      if (ws) {
        ws.close();
      }
      // Clean up all iframes
      iframeMap.forEach(iframe => iframe.remove());
      iframeMap.clear();
    });
  });

  return (
    <div class="fixed inset-0 pointer-events-none overflow-hidden bg-transparent">
      {/* Error Display */}
      <Show when={error()}>
        <div class="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 pointer-events-auto">
          <div class="alert alert-error shadow-lg">
            <div>
              <svg xmlns="http://www.w3.org/2000/svg" class="stroke-current flex-shrink-0 h-6 w-6" fill="none" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z" />
              </svg>
              <div>
                <h3 class="font-bold">Layout Error</h3>
                <div class="text-xs">{error()}</div>
              </div>
            </div>
          </div>
        </div>
      </Show>

      {/* Loading Display */}
      <Show when={loading()}>
        <div class="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2">
          <div class="loading loading-spinner loading-lg text-primary"></div>
          <div class="text-white mt-4 text-center">Loading layout...</div>
        </div>
      </Show>

      {/* Overlay Iframes Container */}
      <div ref={containerRef} class="fixed inset-0 pointer-events-none bg-transparent"></div>

      {/* Debug info (hidden by default, can be shown for testing) */}
      <Show when={false}>
        <div class="absolute bottom-4 right-4 bg-black/80 text-white p-4 rounded text-xs pointer-events-auto max-w-md">
          <div class="font-bold mb-2">Layout Debug Info</div>
          <div>Name: {layoutName()}</div>
          <div>Overlays: {overlays().length}</div>
          <For each={overlays()}>
            {(overlay) => (
              <div class="mt-2 border-t border-white/20 pt-2">
                <div>Type: {overlay.type}</div>
                <div>Position: {overlay.x}, {overlay.y}</div>
                <div>Size: {overlay.width} × {overlay.height}</div>
                <div>Z-Index: {overlay.zIndex}</div>
              </div>
            )}
          </For>
        </div>
      </Show>
    </div>
  );
}

// Only render when used as standalone (for OBS browser sources)
if (document.getElementById('root')) {
  render(() => <LayoutOverlay />, document.getElementById('root'));
}
