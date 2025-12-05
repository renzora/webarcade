import panelStore, { PANELS } from './panels';

/**
 * Create a plugin with the new unified API
 *
 * @example
 * export default plugin({
 *     id: 'my-plugin',
 *     name: 'My Plugin',
 *     version: '1.0.0',
 *
 *     start(api) {
 *         api.add({ panel: 'tab', label: 'My Plugin', icon: MyIcon });
 *         api.add({ panel: 'viewport', id: 'main', component: MainView });
 *         api.add({ panel: 'left', id: 'explorer', component: Explorer });
 *         api.add({ panel: 'right', id: 'props', component: Properties });
 *         api.add({ panel: 'bottom', id: 'console', component: Console });
 *     },
 *
 *     active(api) {
 *         // Called when plugin becomes active
 *     },
 *
 *     inactive(api) {
 *         // Called when plugin becomes inactive
 *     },
 *
 *     stop(api) {
 *         // Called when plugin is stopped/unloaded
 *     }
 * });
 */
export function plugin(config) {
  const {
    id,
    name,
    version,
    description = 'No description provided',
    author = 'Unknown',
    icon = null,
    start = async () => {},
    active = async () => {},
    inactive = async () => {},
    stop = async () => {}
  } = config;

  if (!id) throw new Error('Plugin must have an id');
  if (!name) throw new Error('Plugin must have a name');
  if (!version) throw new Error('Plugin must have a version');

  return function Plugin(engineAPI) {
    let initialized = false;
    let started = false;
    const registeredItems = []; // Track registered items for cleanup

    // Create plugin-scoped API
    const api = {
      // Core engine API passthrough - bind methods explicitly since spread doesn't copy prototype methods
      menu: (menuId, config) => engineAPI.menu(menuId, config),
      toolbar: (toolbarId, config) => engineAPI.toolbar(toolbarId, config),
      toolbarGroup: (groupId, config) => engineAPI.toolbarGroup(groupId, config),
      footer: (footerId, config) => engineAPI.footer(footerId, config),
      topMenuButton: (buttonId, config) => engineAPI.topMenuButton(buttonId, config),
      bottomTab: (tabId, config) => engineAPI.bottomTab(tabId, config),
      open: (typeId, options) => engineAPI.open(typeId, options),
      showAll: (visible) => engineAPI.showAll(visible),
      hideAll: () => engineAPI.hideAll(),
      showFooter: (visible) => engineAPI.showFooter(visible),
      hideFooter: () => engineAPI.hideFooter(),
      showMenu: (enabled) => engineAPI.showMenu(enabled),
      hideMenu: () => engineAPI.hideMenu(),
      showTabs: (visible) => engineAPI.showTabs(visible),
      hideTabs: () => engineAPI.hideTabs(),
      toggleTabs: () => engineAPI.toggleTabs(),
      showPluginTabs: (visible) => engineAPI.showPluginTabs(visible),
      hidePluginTabs: () => engineAPI.hidePluginTabs(),
      togglePluginTabs: () => engineAPI.togglePluginTabs(),
      showBottomPanel: (visible) => engineAPI.showBottomPanel(visible),
      hideBottomPanel: () => engineAPI.hideBottomPanel(),
      toggleBottomPanel: () => engineAPI.toggleBottomPanel(),
      showToolbar: (visible) => engineAPI.showToolbar(visible),
      hideToolbar: () => engineAPI.hideToolbar(),
      toggleToolbar: () => engineAPI.toggleToolbar(),
      fullscreen: (enabled) => engineAPI.fullscreen(enabled),
      toggleFullscreen: () => engineAPI.toggleFullscreen(),

      /**
       * Add a component to a panel
       * @param {Object} config
       * @param {string} config.panel - 'tab' | 'viewport' | 'left' | 'right' | 'bottom'
       * @param {string} [config.id] - Unique ID for this item (auto-generated if not provided)
       * @param {Component} [config.component] - SolidJS component to render
       * @param {string} [config.label] - Display label
       * @param {Component} [config.icon] - Icon component
       * @param {boolean} [config.visible=true] - Initial visibility
       * @param {boolean} [config.shared=false] - Allow other plugins to use this
       * @param {number} [config.order=0] - Sort order
       * @param {boolean} [config.closable=true] - Can user close this tab
       * @param {Function} [config.start] - Called first time component mounts
       * @param {Function} [config.active] - Called when plugin becomes active
       * @param {Function} [config.inactive] - Called when plugin becomes inactive
       */
      add: (itemConfig) => {
        const panel = itemConfig.panel;

        if (!panel || !Object.values(PANELS).includes(panel)) {
          console.error(`[Plugin:${id}] Invalid panel type: ${panel}. Must be one of: ${Object.values(PANELS).join(', ')}`);
          return null;
        }

        // Wrap callbacks to pass api
        const wrappedConfig = {
          ...itemConfig,
          start: itemConfig.start ? () => itemConfig.start(api) : null,
          active: itemConfig.active ? () => itemConfig.active(api) : null,
          inactive: itemConfig.inactive ? () => itemConfig.inactive(api) : null
        };

        const fullId = panelStore.register(id, wrappedConfig);
        if (fullId) {
          registeredItems.push(fullId);
        }
        return fullId;
      },

      /**
       * Remove a registered item
       * @param {string} itemId - The item ID (not full ID)
       */
      remove: (itemId) => {
        const fullId = `${id}:${itemId}`;
        panelStore.unregister(fullId);
        const index = registeredItems.indexOf(fullId);
        if (index > -1) {
          registeredItems.splice(index, 1);
        }
      },

      /**
       * Get a shared panel config from another plugin
       * @param {string} fullId - Full ID in format "pluginId:itemId"
       */
      useShared: (fullId) => {
        return panelStore.useSharedPanel(fullId);
      },

      /**
       * Add a shared panel from another plugin to this plugin
       * @param {string} fullId - Full ID in format "pluginId:itemId"
       * @param {Object} [overrides] - Override label, icon, order, etc.
       * @returns {string|null} The new fullId or null if failed
       *
       * @example
       * // Add widget's clock to my left panel
       * api.addShared('widgets:clock', { panel: 'left', label: 'My Clock' });
       */
      addShared: (fullId, overrides = {}) => {
        return panelStore.addSharedPanel(id, fullId, overrides);
      },

      /**
       * Show/hide panels
       */
      showLeft: (visible = true) => panelStore.setPanelVisible('left', visible),
      showRight: (visible = true) => panelStore.setPanelVisible('right', visible),
      showBottom: (visible = true) => panelStore.setPanelVisible('bottom', visible),

      hideLeft: () => panelStore.setPanelVisible('left', false),
      hideRight: () => panelStore.setPanelVisible('right', false),
      hideBottom: () => panelStore.setPanelVisible('bottom', false),

      toggleLeft: () => panelStore.togglePanel('left'),
      toggleRight: () => panelStore.togglePanel('right'),
      toggleBottom: () => panelStore.togglePanel('bottom'),

      /**
       * Emit plugin-scoped events
       */
      emit: (eventType, data) => engineAPI.emit(`${id}:${eventType}`, data),
      on: (eventType, callback) => engineAPI.on(eventType, callback),
      onSelf: (eventType, callback) => engineAPI.on(`${id}:${eventType}`, callback),

      /**
       * Get plugin info
       */
      getPluginId: () => id,
      getPluginName: () => name,
      getPluginVersion: () => version
    };

    const pluginInstance = {
      // Metadata getters
      getId: () => id,
      getName: () => name,
      getVersion: () => version,
      getDescription: () => description,
      getAuthor: () => author,
      getIcon: () => icon,

      async onInit() {
        if (initialized) return;
        initialized = true;
      },

      async onStart() {
        if (!initialized) throw new Error('Plugin must be initialized before starting');
        if (started) return;

        // Call plugin's start function
        await start(api);
        started = true;

        // If this is the first plugin or should be active, activate it
        if (!panelStore.getActivePlugin()) {
          panelStore.setActivePlugin(id);
        }
      },

      async onActive() {
        // Called when this plugin becomes active
        panelStore.setActivePlugin(id);
        panelStore.applyUserPrefs(id);
        await active(api);
      },

      async onInactive() {
        // Called when this plugin becomes inactive
        await inactive(api);
      },

      async onStop() {
        if (!started) return;
        await stop(api);
        started = false;
      },

      async onDispose() {
        if (started) await pluginInstance.onStop();

        // Unregister all items
        registeredItems.forEach(fullId => {
          panelStore.unregister(fullId);
        });
        registeredItems.length = 0;

        initialized = false;
      },

      getStatus: () => ({
        id,
        name,
        version,
        initialized,
        started,
        registeredItems: registeredItems.length
      }),

      // Expose API for external use
      api
    };

    return pluginInstance;
  };
}

// Keep createPlugin as alias for backwards compatibility
export const createPlugin = plugin;

// Base Plugin class for class-based plugins (legacy support)
export class Plugin {
  constructor(engineAPI) {
    this.engineAPI = engineAPI;
    this.api = engineAPI;
    this.id = null;
    this.name = null;
    this.version = null;
    this.description = null;
  }

  async initialize() {
    // Override in subclasses
  }
}
