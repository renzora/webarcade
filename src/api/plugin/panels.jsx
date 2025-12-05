import { createSignal, createMemo, createRoot } from 'solid-js';
import { createStore, produce } from 'solid-js/store';

// Panel locations
export const PANELS = {
  TAB: 'tab',         // Main plugin tab bar
  VIEWPORT: 'viewport', // Main content area
  LEFT: 'left',       // Left sidebar
  RIGHT: 'right',     // Right sidebar
  BOTTOM: 'bottom'    // Bottom panel
};

// Create stores in a root to avoid disposal warnings
let panels, setPanels;
let activePlugin, setActivePlugin;
let panelVisibility, setPanelVisibility;
let userPanelPrefs, setUserPanelPrefs;

createRoot(() => {
  // All registered panel items: Map<fullId, PanelItem>
  // fullId format: "pluginId:itemId" (e.g., "my-plugin:explorer")
  [panels, setPanels] = createSignal(new Map());

  // Currently active plugin ID
  [activePlugin, setActivePlugin] = createSignal(null);

  // Panel visibility state (user can toggle)
  [panelVisibility, setPanelVisibility] = createStore({
    left: true,
    right: true,
    bottom: true
  });

  // User preferences per plugin (remembers what user had open/closed)
  // Map<pluginId, { left: boolean, right: boolean, bottom: boolean }>
  [userPanelPrefs, setUserPanelPrefs] = createSignal(new Map());
});

// Generate full ID with namespace
const getFullId = (pluginId, itemId) => `${pluginId}:${itemId}`;

// Parse full ID back to parts
const parseFullId = (fullId) => {
  const [pluginId, ...rest] = fullId.split(':');
  return { pluginId, itemId: rest.join(':') };
};

// Panel item structure
const createPanelItem = (config) => ({
  id: config.id,
  fullId: config.fullId,
  pluginId: config.pluginId,
  panel: config.panel,
  component: config.component,
  label: config.label || config.id,
  icon: config.icon || null,
  visible: config.visible !== false, // Default to visible
  shared: config.shared || false,    // Can other plugins use this?
  order: config.order || 0,
  closable: config.closable !== false,
  // Lifecycle callbacks
  start: config.start || null,       // First time mounted
  active: config.active || null,     // Plugin became active
  inactive: config.inactive || null, // Plugin became inactive
  // Internal state
  _started: false
});

// Panel Store API
export const panelStore = {
  // Get all panels
  getPanels: () => panels(),

  // Get active plugin
  getActivePlugin: () => activePlugin(),

  // Set active plugin (triggers active/inactive callbacks)
  setActivePlugin: (pluginId) => {
    const previousPlugin = activePlugin();

    if (previousPlugin === pluginId) return;

    // Call inactive callbacks for previous plugin's items
    if (previousPlugin) {
      panels().forEach((item) => {
        if (item.pluginId === previousPlugin && item.inactive) {
          try {
            item.inactive();
          } catch (err) {
            console.error(`[Panels] Error in inactive callback for ${item.fullId}:`, err);
          }
        }
      });

      // Emit plugin inactive event
      document.dispatchEvent(new CustomEvent('plugin:inactive', {
        detail: { pluginId: previousPlugin }
      }));
    }

    setActivePlugin(pluginId);

    // Call active callbacks for new plugin's items
    if (pluginId) {
      panels().forEach((item) => {
        if (item.pluginId === pluginId && item.active) {
          try {
            item.active();
          } catch (err) {
            console.error(`[Panels] Error in active callback for ${item.fullId}:`, err);
          }
        }
      });

      // Emit plugin active event
      document.dispatchEvent(new CustomEvent('plugin:active', {
        detail: { pluginId }
      }));
    }
  },

  // Register a panel item
  register: (pluginId, config) => {
    const itemId = config.id || `${config.panel}-${Date.now()}`;
    const fullId = getFullId(pluginId, itemId);

    // Check for duplicate
    if (panels().has(fullId)) {
      console.warn(`[Panels] Item ${fullId} already registered`);
      return false;
    }

    const item = createPanelItem({
      ...config,
      id: itemId,
      fullId,
      pluginId
    });

    setPanels(prev => new Map(prev.set(fullId, item)));

    // Call start callback if plugin is already active
    if (item.start && activePlugin() === pluginId) {
      try {
        item.start();
        item._started = true;
      } catch (err) {
        console.error(`[Panels] Error in start callback for ${fullId}:`, err);
      }
    }

    // Emit registration event
    document.dispatchEvent(new CustomEvent('panel:registered', {
      detail: { fullId, item }
    }));

    return fullId;
  },

  // Unregister a panel item
  unregister: (fullId) => {
    setPanels(prev => {
      const newMap = new Map(prev);
      newMap.delete(fullId);
      return newMap;
    });

    document.dispatchEvent(new CustomEvent('panel:unregistered', {
      detail: { fullId }
    }));
  },

  // Unregister all items for a plugin
  unregisterPlugin: (pluginId) => {
    setPanels(prev => {
      const newMap = new Map(prev);
      for (const [fullId, item] of prev) {
        if (item.pluginId === pluginId) {
          newMap.delete(fullId);
        }
      }
      return newMap;
    });
  },

  // Get items for a specific panel and plugin
  getItemsForPanel: (panel, pluginId = null) => {
    const targetPlugin = pluginId || activePlugin();
    const items = [];

    panels().forEach((item) => {
      if (item.panel === panel) {
        // Include if: belongs to target plugin OR is shared
        if (item.pluginId === targetPlugin || item.shared) {
          items.push(item);
        }
      }
    });

    return items.sort((a, b) => a.order - b.order);
  },

  // Get all tabs (plugin tabs)
  getTabs: () => {
    return panelStore.getItemsForPanel(PANELS.TAB);
  },

  // Get viewports for active plugin
  getViewports: () => {
    return panelStore.getItemsForPanel(PANELS.VIEWPORT);
  },

  // Get left panel tabs for active plugin
  getLeftTabs: () => {
    return panelStore.getItemsForPanel(PANELS.LEFT);
  },

  // Get right panel tabs for active plugin
  getRightTabs: () => {
    return panelStore.getItemsForPanel(PANELS.RIGHT);
  },

  // Get bottom panel tabs for active plugin
  getBottomTabs: () => {
    return panelStore.getItemsForPanel(PANELS.BOTTOM);
  },

  // Panel visibility controls
  isPanelVisible: (panel) => panelVisibility[panel],

  setPanelVisible: (panel, visible) => {
    setPanelVisibility(panel, visible);

    // Save user preference for current plugin
    const currentPlugin = activePlugin();
    if (currentPlugin) {
      setUserPanelPrefs(prev => {
        const newMap = new Map(prev);
        const prefs = newMap.get(currentPlugin) || {};
        newMap.set(currentPlugin, { ...prefs, [panel]: visible });
        return newMap;
      });
    }
  },

  togglePanel: (panel) => {
    panelStore.setPanelVisible(panel, !panelVisibility[panel]);
  },

  // Apply user preferences when switching plugins
  applyUserPrefs: (pluginId) => {
    const prefs = userPanelPrefs().get(pluginId);
    if (prefs) {
      if (prefs.left !== undefined) setPanelVisibility('left', prefs.left);
      if (prefs.right !== undefined) setPanelVisibility('right', prefs.right);
      if (prefs.bottom !== undefined) setPanelVisibility('bottom', prefs.bottom);
    }
  },

  // Get an item by full ID
  getItem: (fullId) => panels().get(fullId),

  // Use a shared panel from another plugin
  useSharedPanel: (fullId) => {
    const item = panels().get(fullId);
    if (!item) {
      console.warn(`[Panels] Panel ${fullId} not found`);
      return null;
    }
    if (!item.shared) {
      console.warn(`[Panels] Panel ${fullId} is not shared`);
      return null;
    }
    return item;
  },

  // Add a shared panel from another plugin to the current plugin
  // This creates a reference in the current plugin's namespace
  addSharedPanel: (consumerPluginId, fullId, overrides = {}) => {
    const sourceItem = panels().get(fullId);
    if (!sourceItem) {
      console.warn(`[Panels] Panel ${fullId} not found`);
      return null;
    }
    if (!sourceItem.shared) {
      console.warn(`[Panels] Panel ${fullId} is not shared - cannot add to ${consumerPluginId}`);
      return null;
    }

    // Create a new panel item that references the shared component
    const newId = overrides.id || sourceItem.id;
    const newFullId = getFullId(consumerPluginId, newId);

    // Don't duplicate if already added
    if (panels().has(newFullId)) {
      return newFullId;
    }

    const item = createPanelItem({
      ...sourceItem,
      ...overrides,
      id: newId,
      fullId: newFullId,
      pluginId: consumerPluginId,
      shared: false, // The copy belongs to consumer now
      _sourceId: fullId // Track where it came from
    });

    setPanels(prev => new Map(prev.set(newFullId, item)));

    return newFullId;
  }
};

// Export reactive signals for direct use in components
export {
  panels,
  activePlugin,
  panelVisibility,
  PANELS as PanelTypes
};

export default panelStore;
