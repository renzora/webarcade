import { createSignal, createEffect, For, Show, onMount, onCleanup } from 'solid-js';
import { IconPlus, IconTrash, IconCopy, IconDeviceFloppy, IconZoomIn, IconZoomOut, IconGridDots, IconChevronDown, IconEye, IconEyeOff } from '@tabler/icons-solidjs';
import { layoutManagerStore } from './LayoutManagerStore';
import { WEBARCADE_WS } from '@/api/bridge';
import { editorActions, editorStore } from '@/layout/stores/EditorStore';
import { propertyTabs } from '@/api/plugin';
import { viewportStore } from '@/panels/viewport/store';

const BRIDGE_URL = 'http://localhost:3001';
const CANVAS_WIDTH = 1920;
const CANVAS_HEIGHT = 1080;
const SNAP_THRESHOLD = 10; // Pixels to snap

// Default overlay sizes (for display purposes)
const OVERLAY_DEFAULTS = {
  alerts: { width: 1920, height: 1080, name: 'Alerts' },
  goals: { width: 1920, height: 200, name: 'Goals' },
  status: { width: 400, height: 100, name: 'Status' },
  ticker: { width: 1920, height: 48, name: 'Ticker' },
  chat: { width: 420, height: 800, name: 'Chat' },
  timer: { width: 300, height: 200, name: 'Timer' },
  'watchtime-leaderboard': { width: 400, height: 600, name: 'Leaderboard' },
  todos: { width: 400, height: 500, name: 'Todos' },
  weight: { width: 300, height: 200, name: 'Weight' },
  wheel: { width: 600, height: 600, name: 'Wheel' },
  effect: { width: 1920, height: 1080, name: 'Effect' },
  levelup: { width: 1920, height: 1080, name: 'Level Up' },
};

export default function LayoutManagerViewport() {
  const [isDragging, setIsDragging] = createSignal(false);
  const [isResizing, setIsResizing] = createSignal(false);
  const [resizeDirection, setResizeDirection] = createSignal(null); // 'n', 's', 'e', 'w', 'ne', 'nw', 'se', 'sw'
  const [dragStart, setDragStart] = createSignal({ x: 0, y: 0 });
  const [copiedUrl, setCopiedUrl] = createSignal(false);
  const [zoom, setZoom] = createSignal(0.4); // Zoom level (0.1 to 1.0)
  const [isShiftPressed, setIsShiftPressed] = createSignal(false);
  const [snapEnabled, setSnapEnabled] = createSignal(true);
  const [snapAmount, setSnapAmount] = createSignal(100);
  const [previewMode, setPreviewMode] = createSignal(false);
  const [isPanning, setIsPanning] = createSignal(false);
  const [panStart, setPanStart] = createSignal({ x: 0, y: 0, scrollX: 0, scrollY: 0 });

  let canvasContainerRef;
  let ws;

  // Use shared selected overlay from store
  const selectedOverlay = () => layoutManagerStore.selectedOverlay();
  const setSelectedOverlay = (overlay) => layoutManagerStore.setSelectedOverlay(overlay);

  // Initialize store on mount
  onMount(() => {
    layoutManagerStore.init();

    // Auto-select the layout-manager-panel tab when this viewport loads
    editorActions.setScenePanelOpen(true);
    editorActions.setSelectedTool('layout-manager-panel');

    // Connect to WebSocket for real-time updates
    const connectWebSocket = () => {
      ws = new WebSocket(WEBARCADE_WS);

      ws.onopen = () => {
        console.log('✅ Layout Manager connected to WebSocket');
      };

      ws.onclose = () => {
        console.log('❌ WebSocket disconnected, reconnecting...');
        setTimeout(connectWebSocket, 3000);
      };

      ws.onerror = (error) => {
        console.error('WebSocket error:', error);
      };
    };

    connectWebSocket();
  });

  onCleanup(() => {
    if (ws) {
      ws.close();
    }

    // When viewport is closed, switch away from the layout-manager-panel tab
    const currentTool = editorStore.ui.selectedTool;
    if (currentTool === 'layout-manager-panel') {
      // Find the first available tab that isn't the layout-manager-panel
      const availableTabs = Array.from(propertyTabs().values())
        .filter(tab => tab.id !== 'layout-manager-panel' && (!tab.condition || tab.condition()))
        .sort((a, b) => (a.order || 0) - (b.order || 0));

      if (availableTabs.length > 0) {
        editorActions.setSelectedTool(availableTabs[0].id);
      } else {
        // If no other tabs available, just set to null
        editorActions.setSelectedTool(null);
      }
    }
  });

  // Watch for viewport changes and auto-switch to the panel tab
  createEffect(() => {
    const activeTab = viewportStore.tabs.find(tab => tab.id === viewportStore.activeTabId);
    if (activeTab?.type === 'layout-manager') {
      editorActions.setScenePanelOpen(true);
      editorActions.setSelectedTool('layout-manager-panel');
    }
  });

  // Auto-broadcast layout updates when overlays change (for real-time preview)
  createEffect(() => {
    // Track overlays changes
    const overlays = layoutManagerStore.overlaysInLayout();
    const name = layoutManagerStore.layoutName();

    // Only broadcast if we have a layout name and WebSocket is ready
    if (name && overlays.length > 0 && ws && ws.readyState === WebSocket.OPEN) {
      // Send update via WebSocket for instant real-time updates
      const message = {
        type: 'layout_update',
        layout_name: name,
        layout: {
          name,
          overlays
        }
      };
      ws.send(JSON.stringify(message));
      console.log('📡 Sent layout update via WebSocket:', name);
    }
  });

  // Keyboard event listeners for shift key
  createEffect(() => {
    const handleKeyDown = (e) => {
      if (e.key === 'Shift') setIsShiftPressed(true);
    };
    const handleKeyUp = (e) => {
      if (e.key === 'Shift') setIsShiftPressed(false);
    };

    window.addEventListener('keydown', handleKeyDown);
    window.addEventListener('keyup', handleKeyUp);

    return () => {
      window.removeEventListener('keydown', handleKeyDown);
      window.removeEventListener('keyup', handleKeyUp);
    };
  });

  // Snapping helper function
  const snapValue = (value, snapTo) => {
    if (!snapEnabled()) return value;
    const snapped = Math.round(value / snapTo) * snapTo;
    return Math.abs(value - snapped) < SNAP_THRESHOLD ? snapped : value;
  };

  // Snap to grid and other overlays
  const snapPosition = (x, y, currentOverlay) => {
    if (!snapEnabled()) return { x, y };

    let snappedX = snapValue(x, snapAmount());
    let snappedY = snapValue(y, snapAmount());

    // Snap to other overlays
    layoutManagerStore.overlaysInLayout().forEach(overlay => {
      if (overlay.id === currentOverlay.id) return;

      // Snap to edges
      if (Math.abs(x - overlay.x) < SNAP_THRESHOLD) snappedX = overlay.x;
      if (Math.abs(x - (overlay.x + overlay.width)) < SNAP_THRESHOLD) snappedX = overlay.x + overlay.width;
      if (Math.abs((x + currentOverlay.width) - overlay.x) < SNAP_THRESHOLD) snappedX = overlay.x - currentOverlay.width;
      if (Math.abs((x + currentOverlay.width) - (overlay.x + overlay.width)) < SNAP_THRESHOLD) snappedX = overlay.x + overlay.width - currentOverlay.width;

      if (Math.abs(y - overlay.y) < SNAP_THRESHOLD) snappedY = overlay.y;
      if (Math.abs(y - (overlay.y + overlay.height)) < SNAP_THRESHOLD) snappedY = overlay.y + overlay.height;
      if (Math.abs((y + currentOverlay.height) - overlay.y) < SNAP_THRESHOLD) snappedY = overlay.y - currentOverlay.height;
      if (Math.abs((y + currentOverlay.height) - (overlay.y + overlay.height)) < SNAP_THRESHOLD) snappedY = overlay.y + overlay.height - currentOverlay.height;
    });

    return { x: snappedX, y: snappedY };
  };

  // Zoom controls
  const zoomIn = () => setZoom(Math.min(1.0, zoom() + 0.1));
  const zoomOut = () => setZoom(Math.max(0.1, zoom() - 0.1));
  const resetZoom = () => setZoom(0.4);

  // Handle mouse wheel zoom (zoom toward cursor)
  const handleWheel = (e) => {
    e.preventDefault();

    if (!canvasContainerRef) return;

    const container = canvasContainerRef;
    const rect = container.getBoundingClientRect();

    // Get mouse position relative to container
    const mouseX = e.clientX - rect.left;
    const mouseY = e.clientY - rect.top;

    // Get scroll position before zoom
    const scrollX = container.scrollLeft;
    const scrollY = container.scrollTop;

    // Calculate mouse position in canvas coordinates (before zoom)
    const canvasMouseX = scrollX + mouseX;
    const canvasMouseY = scrollY + mouseY;

    // Calculate zoom delta
    const oldZoom = zoom();
    const zoomDelta = e.deltaY > 0 ? -0.05 : 0.05;
    const newZoom = Math.max(0.1, Math.min(1.0, oldZoom + zoomDelta));

    // Update zoom
    setZoom(newZoom);

    // Calculate new scroll position to keep mouse point stable
    const zoomRatio = newZoom / oldZoom;
    const newScrollX = canvasMouseX * zoomRatio - mouseX;
    const newScrollY = canvasMouseY * zoomRatio - mouseY;

    // Apply new scroll position on next frame
    requestAnimationFrame(() => {
      container.scrollLeft = newScrollX;
      container.scrollTop = newScrollY;
    });
  };

  // Handle middle mouse panning
  const handleCanvasMouseDown = (e) => {
    if (e.button === 1) { // Middle mouse button
      e.preventDefault();
      setIsPanning(true);
      setPanStart({
        x: e.clientX,
        y: e.clientY,
        scrollX: canvasContainerRef.scrollLeft,
        scrollY: canvasContainerRef.scrollTop,
      });
    }
  };

  const handleCanvasMouseMove = (e) => {
    if (isPanning()) {
      e.preventDefault();
      const deltaX = e.clientX - panStart().x;
      const deltaY = e.clientY - panStart().y;
      canvasContainerRef.scrollLeft = panStart().scrollX - deltaX;
      canvasContainerRef.scrollTop = panStart().scrollY - deltaY;
    }
  };

  const handleCanvasMouseUp = (e) => {
    if (e.button === 1) { // Middle mouse button
      setIsPanning(false);
    }
  };

  // Remove overlay from canvas
  const removeOverlay = (id) => {
    layoutManagerStore.removeOverlayById(id);
  };

  // Handle mouse down on overlay
  const handleOverlayMouseDown = (e, overlay) => {
    e.stopPropagation();

    // If clicking on a different overlay when one is already selected, ignore
    if (selectedOverlay() && selectedOverlay()?.id !== overlay.id) {
      return; // Don't allow selecting different overlays
    }

    // If no overlay selected, select this one
    if (!selectedOverlay()) {
      setSelectedOverlay(overlay);
      return; // Don't start dragging yet
    }

    // If clicking on already-selected overlay, allow dragging
    setIsDragging(true);
    setDragStart({
      x: e.clientX - overlay.x * zoom(),
      y: e.clientY - overlay.y * zoom(),
    });
  };

  // Handle mouse down on resize handle
  const handleResizeMouseDown = (e, overlay, direction) => {
    e.stopPropagation();

    // Only allow resizing the selected overlay
    if (selectedOverlay()?.id !== overlay.id) {
      return; // Resize handle should only be visible on selected overlay anyway
    }

    setIsResizing(true);
    setResizeDirection(direction);
    setDragStart({
      x: e.clientX,
      y: e.clientY,
      overlayX: overlay.x,
      overlayY: overlay.y,
      width: overlay.width,
      height: overlay.height,
      aspectRatio: overlay.width / overlay.height,
    });
  };

  // Handle mouse move
  const handleMouseMove = (e) => {
    if (isDragging() && selectedOverlay()) {
      const rawX = (e.clientX - dragStart().x) / zoom();
      const rawY = (e.clientY - dragStart().y) / zoom();

      // Apply bounds
      const boundedX = Math.max(0, Math.min(CANVAS_WIDTH - selectedOverlay().width, rawX));
      const boundedY = Math.max(0, Math.min(CANVAS_HEIGHT - selectedOverlay().height, rawY));

      // Apply snapping
      const snapped = snapPosition(boundedX, boundedY, selectedOverlay());

      layoutManagerStore.setOverlaysInLayout(
        layoutManagerStore.overlaysInLayout().map(o =>
          o.id === selectedOverlay().id ? { ...o, x: Math.round(snapped.x), y: Math.round(snapped.y) } : o
        )
      );
      setSelectedOverlay({ ...selectedOverlay(), x: Math.round(snapped.x), y: Math.round(snapped.y) });
    } else if (isResizing() && selectedOverlay()) {
      const deltaX = e.clientX - dragStart().x;
      const deltaY = e.clientY - dragStart().y;
      const direction = resizeDirection();

      let newWidth = dragStart().width;
      let newHeight = dragStart().height;
      let newX = dragStart().overlayX;
      let newY = dragStart().overlayY;

      // Calculate new dimensions based on resize direction
      if (direction.includes('e')) { // East (right)
        newWidth = Math.max(50, dragStart().width + deltaX / zoom());
      }
      if (direction.includes('w')) { // West (left)
        const widthChange = deltaX / zoom();
        newWidth = Math.max(50, dragStart().width - widthChange);
        newX = dragStart().overlayX + (dragStart().width - newWidth);
      }
      if (direction.includes('s')) { // South (bottom)
        newHeight = Math.max(50, dragStart().height + deltaY / zoom());
      }
      if (direction.includes('n')) { // North (top)
        const heightChange = deltaY / zoom();
        newHeight = Math.max(50, dragStart().height - heightChange);
        newY = dragStart().overlayY + (dragStart().height - newHeight);
      }

      // Maintain aspect ratio if shift is pressed
      if (isShiftPressed()) {
        const aspectRatio = dragStart().aspectRatio;

        // Determine which dimension changed more
        const widthChange = Math.abs(newWidth - dragStart().width);
        const heightChange = Math.abs(newHeight - dragStart().height);

        if (widthChange > heightChange) {
          // Width changed more, adjust height
          const oldHeight = newHeight;
          newHeight = newWidth / aspectRatio;

          // If resizing from north, adjust Y position
          if (direction.includes('n')) {
            newY = dragStart().overlayY + (dragStart().height - newHeight);
          }
        } else {
          // Height changed more, adjust width
          const oldWidth = newWidth;
          newWidth = newHeight * aspectRatio;

          // If resizing from west, adjust X position
          if (direction.includes('w')) {
            newX = dragStart().overlayX + (dragStart().width - newWidth);
          }
        }
      }

      // Snap dimensions to grid
      if (snapEnabled()) {
        newWidth = snapValue(newWidth, snapAmount());
        newHeight = snapValue(newHeight, snapAmount());
      }

      // Apply bounds
      newX = Math.max(0, Math.min(CANVAS_WIDTH - newWidth, newX));
      newY = Math.max(0, Math.min(CANVAS_HEIGHT - newHeight, newY));

      layoutManagerStore.setOverlaysInLayout(
        layoutManagerStore.overlaysInLayout().map(o =>
          o.id === selectedOverlay().id
            ? {
                ...o,
                x: Math.round(newX),
                y: Math.round(newY),
                width: Math.round(newWidth),
                height: Math.round(newHeight)
              }
            : o
        )
      );
      setSelectedOverlay({
        ...selectedOverlay(),
        x: Math.round(newX),
        y: Math.round(newY),
        width: Math.round(newWidth),
        height: Math.round(newHeight)
      });
    }
  };

  // Handle mouse up
  const handleMouseUp = () => {
    setIsDragging(false);
    setIsResizing(false);
    setResizeDirection(null);
  };

  // Copy layout URL
  const copyLayoutUrl = () => {
    const name = layoutManagerStore.layoutName().trim();
    if (!name) {
      alert('Please save the layout first');
      return;
    }

    const url = `${BRIDGE_URL}/overlay/layout/${encodeURIComponent(name)}`;
    navigator.clipboard.writeText(url);
    setCopiedUrl(true);
    setTimeout(() => setCopiedUrl(false), 2000);
  };

  // Get overlay color
  const getOverlayColor = (type) => {
    const colors = {
      alerts: 'bg-red-500/30 border-red-500',
      goals: 'bg-purple-500/30 border-purple-500',
      status: 'bg-blue-500/30 border-blue-500',
      ticker: 'bg-yellow-500/30 border-yellow-500',
      chat: 'bg-green-500/30 border-green-500',
      timer: 'bg-pink-500/30 border-pink-500',
    };
    return colors[type] || 'bg-gray-500/30 border-gray-500';
  };

  return (
    <div class="flex flex-col h-full bg-base-200" onMouseMove={handleMouseMove} onMouseUp={handleMouseUp}>
      {/* Header */}
      <div class="navbar bg-base-300 shadow-lg">
        <div class="flex-1">
          <h2 class="text-2xl font-bold">Layout Manager</h2>
          <Show when={isShiftPressed()}>
            <div class="badge badge-info ml-4">Shift: Aspect Ratio Lock</div>
          </Show>
        </div>
        <div class="flex-none gap-2">
          {/* Preview Mode Toggle */}
          <button
            class={`btn btn-sm ${previewMode() ? 'btn-primary' : 'btn-outline'}`}
            onClick={() => setPreviewMode(!previewMode())}
            title={previewMode() ? 'Exit Preview Mode' : 'Enter Preview Mode'}
          >
            {previewMode() ? <IconEye size={16} /> : <IconEyeOff size={16} />}
            Preview
          </button>

          {/* Snap Controls */}
          <div class="join">
            <button
              class={`btn btn-sm join-item ${snapEnabled() ? 'btn-active' : ''}`}
              onClick={() => setSnapEnabled(!snapEnabled())}
              title="Toggle Snap"
            >
              <IconGridDots size={16} />
            </button>
            <input
              type="number"
              class="input input-sm join-item w-20 text-center"
              value={snapAmount()}
              onInput={(e) => setSnapAmount(Math.max(1, parseInt(e.target.value) || 1))}
              disabled={!snapEnabled()}
              placeholder="Grid"
              title="Snap Grid Size"
            />
          </div>

          {/* Zoom Controls */}
          <div class="join">
            <button class="btn btn-sm join-item" onClick={zoomOut} title="Zoom Out">
              <IconZoomOut size={16} />
            </button>
            <button class="btn btn-sm join-item" onClick={resetZoom} title="Reset Zoom">
              {Math.round(zoom() * 100)}%
            </button>
            <button class="btn btn-sm join-item" onClick={zoomIn} title="Zoom In">
              <IconZoomIn size={16} />
            </button>
          </div>

          <button class="btn btn-sm btn-primary" onClick={() => layoutManagerStore.newLayout()}>
            <IconPlus size={16} /> New Layout
          </button>
        </div>
      </div>

      <div class="flex flex-1 overflow-hidden">
        {/* Main Canvas Area */}
        <div class="flex-1 flex flex-col p-4">
          {/* Controls */}
          <div class="card bg-base-300 shadow-xl mb-4">
            <div class="card-body p-4">
              <div class="flex gap-4 items-center">
                {/* Saved Layouts Dropdown */}
                <div class="dropdown">
                  <label tabindex="0" class="btn btn-outline btn-sm gap-2">
                    <Show when={layoutManagerStore.currentLayout()} fallback="Load Layout">
                      {layoutManagerStore.currentLayout()}
                    </Show>
                    <IconChevronDown size={16} />
                  </label>
                  <ul tabindex="0" class="dropdown-content z-[1] menu p-2 shadow bg-base-100 rounded-box w-52 mt-2">
                    <Show when={layoutManagerStore.savedLayouts().length === 0}>
                      <li class="text-center text-sm opacity-60 py-2">No saved layouts</li>
                    </Show>
                    <For each={layoutManagerStore.savedLayouts()}>
                      {(layout) => (
                        <li>
                          <div class="flex justify-between items-center">
                            <button
                              class="flex-1 text-left"
                              onClick={() => layoutManagerStore.loadLayout(layout)}
                            >
                              {layout}
                            </button>
                            <button
                              class="btn btn-xs btn-ghost btn-circle"
                              onClick={(e) => {
                                e.stopPropagation();
                                layoutManagerStore.deleteLayout(layout);
                              }}
                            >
                              <IconTrash size={12} />
                            </button>
                          </div>
                        </li>
                      )}
                    </For>
                  </ul>
                </div>

                <div class="flex-1">
                  <label class="label">
                    <span class="label-text">Layout Name</span>
                  </label>
                  <input
                    type="text"
                    class="input input-bordered w-full"
                    placeholder="My Awesome Layout"
                    value={layoutManagerStore.layoutName()}
                    onInput={(e) => layoutManagerStore.setLayoutName(e.target.value)}
                  />
                </div>
                <div class="flex gap-2 items-end">
                  <button class="btn btn-primary" onClick={() => layoutManagerStore.saveLayout()}>
                    <IconDeviceFloppy size={18} /> Save Layout
                  </button>
                  <button
                    class={`btn ${copiedUrl() ? 'btn-success' : 'btn-secondary'}`}
                    onClick={copyLayoutUrl}
                  >
                    {copiedUrl() ? 'Copied!' : <><IconCopy size={18} /> Copy URL</>}
                  </button>
                </div>
              </div>

              {/* Selected Overlay Info - Fixed Height */}
              <div class="min-h-[3rem] mt-2">
                <Show when={selectedOverlay()}>
                  <div class="pt-2 border-t border-base-content/10">
                    <div class="grid grid-cols-5 gap-2 text-sm mb-2">
                      <div>
                        <span class="font-bold">Type:</span> {selectedOverlay().type}
                      </div>
                      <div>
                        <span class="font-bold">X:</span> {selectedOverlay().x}px
                      </div>
                      <div>
                        <span class="font-bold">Y:</span> {selectedOverlay().y}px
                      </div>
                      <div>
                        <span class="font-bold">W:</span> {selectedOverlay().width}px
                      </div>
                      <div>
                        <span class="font-bold">H:</span> {selectedOverlay().height}px
                      </div>
                    </div>
                    <div class="flex items-center gap-2 text-sm">
                      <span class="font-bold">Fit Mode:</span>
                      <div class="btn-group">
                        <button
                          class={`btn btn-xs ${(selectedOverlay().fitMode || 'contain') === 'contain' ? 'btn-primary' : 'btn-outline'}`}
                          onClick={() => {
                            layoutManagerStore.setOverlaysInLayout(
                              layoutManagerStore.overlaysInLayout().map(o =>
                                o.id === selectedOverlay().id ? { ...o, fitMode: 'contain' } : o
                              )
                            );
                            setSelectedOverlay({ ...selectedOverlay(), fitMode: 'contain' });
                          }}
                          title="Fit inside while maintaining aspect ratio"
                        >
                          Contain
                        </button>
                        <button
                          class={`btn btn-xs ${(selectedOverlay().fitMode || 'contain') === 'cover' ? 'btn-primary' : 'btn-outline'}`}
                          onClick={() => {
                            layoutManagerStore.setOverlaysInLayout(
                              layoutManagerStore.overlaysInLayout().map(o =>
                                o.id === selectedOverlay().id ? { ...o, fitMode: 'cover' } : o
                              )
                            );
                            setSelectedOverlay({ ...selectedOverlay(), fitMode: 'cover' });
                          }}
                          title="Fill while maintaining aspect ratio (crop overflow)"
                        >
                          Cover
                        </button>
                        <button
                          class={`btn btn-xs ${(selectedOverlay().fitMode || 'contain') === 'fill' ? 'btn-primary' : 'btn-outline'}`}
                          onClick={() => {
                            layoutManagerStore.setOverlaysInLayout(
                              layoutManagerStore.overlaysInLayout().map(o =>
                                o.id === selectedOverlay().id ? { ...o, fitMode: 'fill' } : o
                              )
                            );
                            setSelectedOverlay({ ...selectedOverlay(), fitMode: 'fill' });
                          }}
                          title="Stretch to fill exactly (may distort)"
                        >
                          Fill
                        </button>
                      </div>
                    </div>
                  </div>
                </Show>
              </div>
            </div>
          </div>

          {/* Canvas */}
          <div
            ref={canvasContainerRef}
            class={`flex-1 flex items-center justify-center bg-base-100 rounded-lg p-4 overflow-auto ${isPanning() ? 'cursor-grabbing' : 'cursor-default'}`}
            onWheel={handleWheel}
            onMouseDown={handleCanvasMouseDown}
            onMouseMove={handleCanvasMouseMove}
            onMouseUp={handleCanvasMouseUp}
            onMouseLeave={() => setIsPanning(false)}
          >
            <div
              class="relative bg-black border border-primary shadow-2xl"
              style={{
                width: `${CANVAS_WIDTH * zoom()}px`,
                height: `${CANVAS_HEIGHT * zoom()}px`,
              }}
            >
              {/* Grid overlay */}
              <Show when={snapEnabled()}>
                <div
                  class="absolute inset-0 opacity-10 pointer-events-none"
                  style={{
                    'background-image': 'linear-gradient(rgba(255,255,255,0.1) 1px, transparent 1px), linear-gradient(90deg, rgba(255,255,255,0.1) 1px, transparent 1px)',
                    'background-size': `${snapAmount() * zoom()}px ${snapAmount() * zoom()}px`,
                  }}
                ></div>
              </Show>

              {/* Canvas dimensions label */}
              <div class="absolute top-2 left-2 text-white text-xs bg-black/50 px-2 py-1 rounded">
                1920 × 1080
              </div>

              {/* Overlays */}
              <For each={layoutManagerStore.overlaysInLayout()}>
                {(overlay) => (
                  <div
                    class={`absolute border ${
                      selectedOverlay()?.id === overlay.id
                        ? 'ring-2 ring-white cursor-move pointer-events-auto'
                        : selectedOverlay()
                        ? 'cursor-not-allowed pointer-events-auto opacity-60'
                        : 'cursor-pointer pointer-events-auto'
                    } ${!previewMode() ? `flex items-center justify-center font-bold text-white ${getOverlayColor(overlay.type)}` : ''}`}
                    style={{
                      left: `${overlay.x * zoom()}px`,
                      top: `${overlay.y * zoom()}px`,
                      width: `${overlay.width * zoom()}px`,
                      height: `${overlay.height * zoom()}px`,
                      'z-index': selectedOverlay()?.id === overlay.id ? 9999 : overlay.zIndex,
                    }}
                    onMouseDown={(e) => handleOverlayMouseDown(e, overlay)}
                  >
                    <Show when={previewMode()} fallback={
                      /* Placeholder view */
                      <div class="flex flex-col items-center gap-1 pointer-events-none">
                        <div class="text-sm">{OVERLAY_DEFAULTS[overlay.type]?.name || overlay.type}</div>
                        <div class="text-xs opacity-75">
                          {overlay.width} × {overlay.height}
                        </div>
                      </div>
                    }>
                      {/* Preview view - actual iframe */}
                      {(() => {
                        const fitMode = overlay.fitMode || 'contain';
                        const nativeWidth = OVERLAY_DEFAULTS[overlay.type]?.width || 1920;
                        const nativeHeight = OVERLAY_DEFAULTS[overlay.type]?.height || 1080;
                        const targetWidth = overlay.width;
                        const targetHeight = overlay.height;

                        let iframeWidth, iframeHeight, scale;

                        if (fitMode === 'fill') {
                          // Stretch to fill - non-uniform scaling
                          iframeWidth = nativeWidth;
                          iframeHeight = nativeHeight;
                          const scaleX = targetWidth / nativeWidth;
                          const scaleY = targetHeight / nativeHeight;
                          scale = `scale(${scaleX * zoom()}, ${scaleY * zoom()})`;
                        } else if (fitMode === 'cover') {
                          // Fill while maintaining aspect ratio - crop overflow
                          iframeWidth = nativeWidth;
                          iframeHeight = nativeHeight;
                          const scaleX = targetWidth / nativeWidth;
                          const scaleY = targetHeight / nativeHeight;
                          const uniformScale = Math.max(scaleX, scaleY);
                          scale = `scale(${uniformScale * zoom()})`;
                        } else {
                          // contain - fit inside while maintaining aspect ratio
                          iframeWidth = nativeWidth;
                          iframeHeight = nativeHeight;
                          const scaleX = targetWidth / nativeWidth;
                          const scaleY = targetHeight / nativeHeight;
                          const uniformScale = Math.min(scaleX, scaleY);
                          scale = `scale(${uniformScale * zoom()})`;
                        }

                        return (
                          <div
                            class="pointer-events-none"
                            style={{
                              width: '100%',
                              height: '100%',
                              overflow: 'hidden',
                              display: 'flex',
                              'align-items': 'center',
                              'justify-content': 'center',
                            }}
                          >
                            <iframe
                              src={`${BRIDGE_URL}/overlay/${overlay.type}`}
                              class="border-none pointer-events-none"
                              style={{
                                width: `${iframeWidth}px`,
                                height: `${iframeHeight}px`,
                                transform: scale,
                                'transform-origin': 'center',
                              }}
                              title={overlay.type}
                            />
                          </div>
                        );
                      })()}
                    </Show>

                    {/* Controls - only show for selected overlay */}
                    <Show when={selectedOverlay()?.id === overlay.id}>
                      {/* Delete button */}
                      <button
                        class="absolute top-1 right-1 btn btn-xs btn-error btn-circle"
                        onClick={(e) => {
                          e.stopPropagation();
                          removeOverlay(overlay.id);
                        }}
                      >
                        <IconTrash size={12} />
                      </button>

                      {/* Resize Handles - 8 directions */}
                      {/* Top-left corner */}
                      <div
                        class="absolute top-0 left-0 w-2 h-2 bg-blue-500 cursor-nwse-resize"
                        style={{ transform: 'translate(-50%, -50%)' }}
                        onMouseDown={(e) => handleResizeMouseDown(e, overlay, 'nw')}
                      ></div>

                      {/* Top-center edge */}
                      <div
                        class="absolute top-0 left-1/2 w-2 h-2 bg-blue-500 cursor-ns-resize"
                        style={{ transform: 'translate(-50%, -50%)' }}
                        onMouseDown={(e) => handleResizeMouseDown(e, overlay, 'n')}
                      ></div>

                      {/* Top-right corner */}
                      <div
                        class="absolute top-0 right-0 w-2 h-2 bg-blue-500 cursor-nesw-resize"
                        style={{ transform: 'translate(50%, -50%)' }}
                        onMouseDown={(e) => handleResizeMouseDown(e, overlay, 'ne')}
                      ></div>

                      {/* Middle-left edge */}
                      <div
                        class="absolute top-1/2 left-0 w-2 h-2 bg-blue-500 cursor-ew-resize"
                        style={{ transform: 'translate(-50%, -50%)' }}
                        onMouseDown={(e) => handleResizeMouseDown(e, overlay, 'w')}
                      ></div>

                      {/* Middle-right edge */}
                      <div
                        class="absolute top-1/2 right-0 w-2 h-2 bg-blue-500 cursor-ew-resize"
                        style={{ transform: 'translate(50%, -50%)' }}
                        onMouseDown={(e) => handleResizeMouseDown(e, overlay, 'e')}
                      ></div>

                      {/* Bottom-left corner */}
                      <div
                        class="absolute bottom-0 left-0 w-2 h-2 bg-blue-500 cursor-nesw-resize"
                        style={{ transform: 'translate(-50%, 50%)' }}
                        onMouseDown={(e) => handleResizeMouseDown(e, overlay, 'sw')}
                      ></div>

                      {/* Bottom-center edge */}
                      <div
                        class="absolute bottom-0 left-1/2 w-2 h-2 bg-blue-500 cursor-ns-resize"
                        style={{ transform: 'translate(-50%, 50%)' }}
                        onMouseDown={(e) => handleResizeMouseDown(e, overlay, 's')}
                      ></div>

                      {/* Bottom-right corner */}
                      <div
                        class="absolute bottom-0 right-0 w-2 h-2 bg-blue-500 cursor-nwse-resize"
                        style={{ transform: 'translate(50%, 50%)' }}
                        onMouseDown={(e) => handleResizeMouseDown(e, overlay, 'se')}
                      ></div>
                    </Show>
                  </div>
                )}
              </For>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
