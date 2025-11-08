import { createSignal, createEffect } from 'solid-js';

const BRIDGE_URL = 'http://localhost:3001';

// Default overlay sizes
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
  text: { width: 400, height: 100, name: 'Text' },
  image: { width: 400, height: 400, name: 'Image' },
};

// Shared state
const [availableOverlays, setAvailableOverlays] = createSignal([]);
const [savedLayouts, setSavedLayouts] = createSignal([]);
const [overlaysInLayout, setOverlaysInLayout] = createSignal([]);
const [layoutName, setLayoutName] = createSignal('');
const [currentLayout, setCurrentLayout] = createSignal(null);
const [selectedOverlay, setSelectedOverlay] = createSignal(null);

// Fetch available overlays
const fetchAvailableOverlays = async () => {
  try {
    const response = await fetch(`${BRIDGE_URL}/overlay-manager/files`);
    const data = await response.json();
    setAvailableOverlays(data.map(f => f.name));
  } catch (error) {
    console.error('Failed to fetch overlays:', error);
  }
};

// Fetch saved layouts
const fetchLayouts = async () => {
  try {
    const response = await fetch(`${BRIDGE_URL}/layouts`);
    const data = await response.json();
    setSavedLayouts(data);
  } catch (error) {
    console.error('Failed to fetch layouts:', error);
    setSavedLayouts([]);
  }
};

// Add overlay to canvas
const addOverlay = (overlayType) => {
  const defaults = OVERLAY_DEFAULTS[overlayType] || { width: 400, height: 300, name: overlayType };
  const id = Date.now();

  const baseOverlay = {
    id,
    type: overlayType,
    x: 100,
    y: 100,
    width: defaults.width,
    height: defaults.height,
    zIndex: overlaysInLayout().length + 1,
    fitMode: 'contain', // 'contain', 'cover', or 'fill'
  };

  // Add type-specific properties
  if (overlayType === 'text') {
    baseOverlay.text = 'Text Layer';
    baseOverlay.fontSize = 48;
    baseOverlay.fontFamily = 'Arial';
    baseOverlay.color = '#ffffff';
    baseOverlay.backgroundColor = 'transparent';
    baseOverlay.textAlign = 'center';
    baseOverlay.fontWeight = 'normal';
    baseOverlay.fontStyle = 'normal';
  } else if (overlayType === 'image') {
    baseOverlay.imageUrl = '';
    baseOverlay.opacity = 1;
  }

  setOverlaysInLayout([...overlaysInLayout(), baseOverlay]);
};

// Load layout
const loadLayout = async (name) => {
  try {
    const response = await fetch(`${BRIDGE_URL}/layouts/${encodeURIComponent(name)}`);
    const layout = await response.json();
    setLayoutName(layout.name);
    setOverlaysInLayout(layout.overlays || []);
    setCurrentLayout(name);
  } catch (error) {
    console.error('Failed to load layout:', error);
  }
};

// Delete layout
const deleteLayout = async (name) => {
  if (!confirm(`Delete layout "${name}"?`)) return;

  try {
    await fetch(`${BRIDGE_URL}/layouts/${encodeURIComponent(name)}`, {
      method: 'DELETE',
    });
    await fetchLayouts();
    if (currentLayout() === name) {
      setLayoutName('');
      setOverlaysInLayout([]);
      setCurrentLayout(null);
    }
  } catch (error) {
    console.error('Failed to delete layout:', error);
  }
};

// Save layout
const saveLayout = async () => {
  const name = layoutName().trim();
  if (!name) {
    alert('Please enter a layout name');
    return;
  }

  const layout = {
    name,
    overlays: overlaysInLayout(),
  };

  try {
    const response = await fetch(`${BRIDGE_URL}/layouts/${encodeURIComponent(name)}`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(layout),
    });

    if (response.ok) {
      await fetchLayouts();
      alert('Layout saved!');
    }
  } catch (error) {
    console.error('Failed to save layout:', error);
    alert('Failed to save layout');
  }
};

// Broadcast for real-time updates (no debouncing)
const broadcastLayoutUpdate = async () => {
  const name = layoutName().trim();
  if (!name) return; // Don't broadcast if no layout name

  const layout = {
    name,
    overlays: overlaysInLayout(),
  };

  try {
    await fetch(`${BRIDGE_URL}/layouts/${encodeURIComponent(name)}`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(layout),
    });
    console.log('📡 Broadcast layout update:', name);
  } catch (error) {
    console.error('Failed to broadcast layout:', error);
  }
};

// New layout
const newLayout = () => {
  setLayoutName('');
  setOverlaysInLayout([]);
  setCurrentLayout(null);
  setSelectedOverlay(null);
};

// Reorder overlays (for z-index management)
// fromIndex and toIndex are from the sorted array (by zIndex descending)
const reorderOverlays = (fromIndex, toIndex) => {
  // Get sorted array (highest zIndex first = top layer)
  const sorted = [...overlaysInLayout()].sort((a, b) => b.zIndex - a.zIndex);

  // Reorder in the sorted array
  const [removed] = sorted.splice(fromIndex, 1);
  sorted.splice(toIndex, 0, removed);

  // Reassign z-index based on new order (highest index = highest zIndex = top layer)
  const updated = sorted.map((item, index) => ({
    ...item,
    zIndex: sorted.length - index,
  }));

  setOverlaysInLayout(updated);
};

// Remove overlay by id
const removeOverlayById = (id) => {
  // Clear selection if the removed overlay was selected
  if (selectedOverlay()?.id === id) {
    setSelectedOverlay(null);
  }
  setOverlaysInLayout(overlaysInLayout().filter(o => o.id !== id));
};

// Initialize store
const init = () => {
  fetchAvailableOverlays();
  fetchLayouts();
};

export const layoutManagerStore = {
  // Signals
  availableOverlays,
  savedLayouts,
  overlaysInLayout,
  setOverlaysInLayout,
  layoutName,
  setLayoutName,
  currentLayout,
  setCurrentLayout,
  selectedOverlay,
  setSelectedOverlay,

  // Actions
  addOverlay,
  loadLayout,
  deleteLayout,
  saveLayout,
  newLayout,
  fetchLayouts,
  init,
  reorderOverlays,
  removeOverlayById,
  broadcastLayoutUpdate,
};

export { OVERLAY_DEFAULTS };
