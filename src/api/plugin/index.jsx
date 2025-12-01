import { createSignal, createContext, useContext, onMount, onCleanup, createRoot } from 'solid-js';
import pluginStore, { PLUGIN_STATES, setPluginConfigs } from './store.jsx';

const PluginAPIContext = createContext();

const [topMenuItems, setTopMenuItems] = createSignal(new Map());
const [topMenuButtons, setTopMenuButtons] = createSignal(new Map());
const [leftPanelComponents, setLeftPanelComponents] = createSignal(new Map()); // Map<viewportType, panelConfig>
const [rightPanelComponents, setRightPanelComponents] = createSignal(new Map()); // Map<viewportType, panelConfig>
const [viewportTypes, setViewportTypes] = createSignal(new Map());
const [footerButtons, setFooterButtons] = createSignal(new Map());
const [registeredPlugins, setRegisteredPlugins] = createSignal(new Map());
const [bottomPanelTabs, setBottomPanelTabs] = createSignal(new Map()); // Each tab has a 'viewports' array
const [toolbarItems, setToolbarItems] = createSignal(new Map());
const [toolbarGroups, setToolbarGroups] = createSignal(new Map());
const [activeViewportType, setActiveViewportType] = createSignal(null); // Track currently active viewport type
const [propertiesPanelVisible, setPropertiesPanelVisible] = createSignal(true);
const [leftPanelVisible, setLeftPanelVisible] = createSignal(true);
const [horizontalMenuButtonsEnabled, setHorizontalMenuButtonsEnabled] = createSignal(true);
const [footerVisible, setFooterVisible] = createSignal(false);
const [viewportTabsVisible, setViewportTabsVisible] = createSignal(false);
const [bottomPanelVisible, setBottomPanelVisible] = createSignal(false);
const [toolbarVisible, setToolbarVisible] = createSignal(true);
const [fullscreenMode, setFullscreenMode] = createSignal(false);
const [layoutComponents, setLayoutComponents] = createSignal(new Map());

class PluginLoader {
  constructor(PluginAPI) {
    this.PluginAPI = PluginAPI;
    this.updateInterval = null;
  }

  async discoverPlugins() {
    // Discovering plugins from backend
    const discovered = new Map();

    const plugins = await this.scanForPlugins();

    plugins.forEach(plugin => {
      discovered.set(plugin.id, plugin);
      this.setPluginState(plugin.id, PLUGIN_STATES.DISCOVERED);
    });

    return discovered;
  }

  async scanForPlugins() {
    const plugins = [];

    // Fetch all plugins from backend (all plugins are now dynamic)
    try {
      console.log('[PluginLoader] Fetching plugins from backend...');
      const response = await fetch('http://localhost:3001/api/plugins/list');
      console.log('[PluginLoader] Plugins response:', response.status, response.ok);

      if (response.ok) {
        const data = await response.json();
        console.log('[PluginLoader] Plugins data:', data);

        for (const runtimePlugin of data.plugins) {
          // Check if this plugin has a frontend (plugin.js exists)
          const hasFrontend = runtimePlugin.has_plugin_js !== false;

          if (hasFrontend) {
            const pluginConfig = {
              id: runtimePlugin.id,
              path: `/runtime/${runtimePlugin.id}`,
              name: runtimePlugin.name || runtimePlugin.id,
              version: runtimePlugin.version || '1.0.0',
              description: runtimePlugin.description || `Plugin: ${runtimePlugin.name || runtimePlugin.id}`,
              author: runtimePlugin.author || 'Plugin Developer',
              main: 'plugin.js',
              enabled: true,
              priority: runtimePlugin.priority || 100,
              has_backend: runtimePlugin.has_dll || false
            };

            // Register plugin config in store
            setPluginConfigs(prev => new Map(prev.set(runtimePlugin.id, pluginConfig)));

            const plugin = {
              id: runtimePlugin.id,
              path: `/runtime/${runtimePlugin.id}`,
              enabled: true,
              isCore: false,
              isRuntime: true,
              manifest: {
                name: pluginConfig.name,
                version: pluginConfig.version,
                description: pluginConfig.description,
                author: pluginConfig.author,
                main: pluginConfig.main,
                dependencies: [],
                permissions: ['ui-core'],
                apiVersion: '1.0.0',
                priority: pluginConfig.priority
              }
            };
            console.log('[PluginLoader] Adding plugin:', plugin.id, '(has_backend:', pluginConfig.has_backend, ')');
            plugins.push(plugin);
          } else {
            console.log('[PluginLoader] Skipping plugin (no frontend):', runtimePlugin.id);
          }
        }
        console.log('[PluginLoader] Total plugins loaded:', plugins.length);
      }
    } catch (error) {
      console.error('[PluginLoader] Failed to fetch plugins:', error);
    }

    plugins.sort((a, b) => a.manifest.priority - b.manifest.priority);
    return plugins;
  }

  async loadPluginDynamic(id, path, mainFile) {
    try {
      // All plugins are loaded from the backend
      const pluginId = path.replace('/runtime/', '');
      const pluginUrl = `http://localhost:3001/api/plugins/${pluginId}/${mainFile}`;

      console.log(`[PluginLoader] Loading plugin "${id}" from ${pluginUrl}`);

      // Dynamically import the plugin.js file
      const pluginModule = await import(/* webpackIgnore: true */ pluginUrl);
      console.log(`[PluginLoader] Successfully loaded plugin "${id}"`, pluginModule);
      console.log(`[PluginLoader] Module keys for "${id}":`, Object.keys(pluginModule));
      console.log(`[PluginLoader] Module.default for "${id}":`, pluginModule.default);
      return pluginModule;
    } catch (error) {
      throw error;
    }
  }

  async loadPlugin(pluginInfo) {
    const { id, path, manifest } = pluginInfo;

    try {
      console.log(`[PluginLoader] loadPlugin called for "${id}" (isRuntime: ${pluginInfo.isRuntime})`);
      this.setPluginState(id, PLUGIN_STATES.LOADING);
      // Loading plugin: id

      let pluginModule = null;
      let pluginInstance = null;

      // Check if this is a widget-only plugin (no main file)
      if (!manifest.main) {
        // Widget-only plugin - create a minimal dummy instance
        pluginInstance = {
          id,
          name: manifest.name,
          version: manifest.version,
          getId: () => id,
          getName: () => manifest.name,
          getVersion: () => manifest.version,
          // No onInit or onStart needed for widget-only plugins
        };
      } else {
        // Regular plugin with a main file
        try {
          // Use a mapping approach that works with bundlers
          pluginModule = await this.loadPluginDynamic(id, path, manifest.main);
        } catch (importError) {
          throw new Error(`Could not load plugin from ${path}`);
        }

        if (!pluginModule.default && !pluginModule.Plugin) {
          throw new Error(`Plugin ${id} must export a default plugin function`);
        }

        const PluginFactory = pluginModule.default || pluginModule.Plugin;
        console.log(`[PluginLoader] PluginFactory for "${id}":`, typeof PluginFactory, PluginFactory);

        // Handle class-based plugins
        if (PluginFactory.prototype && PluginFactory.prototype.constructor) {
          console.log(`[PluginLoader] Creating class-based plugin instance for "${id}"`);
          pluginInstance = new PluginFactory(this.PluginAPI);
          // Add required methods for class-based plugins
          if (!pluginInstance.getId) {
            pluginInstance.getId = () => pluginInstance.id;
          }
          if (!pluginInstance.getName) {
            pluginInstance.getName = () => pluginInstance.name;
          }
          if (!pluginInstance.getVersion) {
            pluginInstance.getVersion = () => pluginInstance.version;
          }
          if (!pluginInstance.onInit) {
            pluginInstance.onInit = () => pluginInstance.initialize();
          }
        } else {
          // Handle function-based plugins
          console.log(`[PluginLoader] Creating function-based plugin instance for "${id}"`);
          pluginInstance = PluginFactory(this.PluginAPI);
          console.log(`[PluginLoader] Plugin instance created for "${id}":`, pluginInstance);
        }

        const requiredMethods = ['getId', 'getName', 'getVersion'];
        requiredMethods.forEach(method => {
          if (typeof pluginInstance[method] !== 'function') {
          }
        });
      }

      // Store plugin instance in store (single source of truth)
      pluginStore.setPluginInstance(id, pluginInstance, pluginModule);

      this.setPluginState(id, PLUGIN_STATES.LOADED);
      // Plugin loaded successfully

      return pluginInstance;
    } catch (error) {
      this.setPluginError(id, error);
      this.setPluginState(id, PLUGIN_STATES.ERROR);
      throw error;
    }
  }

  async initializePlugin(pluginId) {
    console.log(`[PluginLoader] initializePlugin called for "${pluginId}"`);
    const pluginData = pluginStore.getPluginInstance(pluginId);
    const pluginConfig = pluginStore.getPluginConfig(pluginId);
    if (!pluginData || !pluginData.instance) {
      console.error(`[PluginLoader] Plugin "${pluginId}" not loaded - no instance found`);
      throw new Error(`Plugin ${pluginId} not loaded`);
    }
    const plugin = {
      instance: pluginData.instance,
      manifest: {
        name: pluginConfig?.name || pluginId,
        version: pluginConfig?.version || '1.0.0',
        description: pluginConfig?.description || '',
        author: pluginConfig?.author || 'Unknown'
      }
    };

    try {
      this.setPluginState(pluginId, PLUGIN_STATES.INITIALIZING);
      // Initializing plugin

      // Set the plugin context for registration tracking
      this.PluginAPI.setCurrentPluginContext(pluginId);

      if (typeof plugin.instance.onInit === 'function') {
        await plugin.instance.onInit();
      }

      this.PluginAPI.registerPlugin(pluginId, {
        name: plugin.manifest.name,
        version: plugin.manifest.version,
        description: plugin.manifest.description,
        author: plugin.manifest.author,
        instance: plugin.instance
      });

      // Clear the plugin context
      this.PluginAPI.clearCurrentPluginContext();

      this.setPluginState(pluginId, PLUGIN_STATES.INITIALIZED);
      // Plugin initialized successfully
    } catch (error) {
      this.setPluginError(pluginId, error);
      this.setPluginState(pluginId, PLUGIN_STATES.ERROR);
      
      // Clear the plugin context on error
      this.PluginAPI.clearCurrentPluginContext();
      throw error;
    }
  }

  async startPlugin(pluginId) {
    console.log(`[PluginLoader] startPlugin called for "${pluginId}"`);
    const pluginData = pluginStore.getPluginInstance(pluginId);
    const pluginConfig = pluginStore.getPluginConfig(pluginId);
    if (!pluginData || !pluginData.instance) {
      console.error(`[PluginLoader] Plugin "${pluginId}" not started - no instance found`);
      throw new Error(`Plugin ${pluginId} not loaded`);
    }
    const plugin = {
      instance: pluginData.instance,
      path: pluginConfig?.path || '',
      manifest: {
        name: pluginConfig?.name || pluginId,
        priority: pluginConfig?.priority || 100
      }
    };

    try {
      this.setPluginState(pluginId, PLUGIN_STATES.STARTING);
      // Starting plugin

      // Set the plugin context for auto-registration
      this.PluginAPI.setCurrentPluginContext(pluginId);


      if (typeof plugin.instance.onStart === 'function') {
        // Wrap onStart in createRoot to properly handle SolidJS effects
        await new Promise((resolve, reject) => {
          createRoot(async (dispose) => {
            try {
              await plugin.instance.onStart(this.PluginAPI);
              // Store dispose function on plugin instance for cleanup
              plugin.instance._dispose = dispose;
              resolve();
            } catch (error) {
              dispose();
              reject(error);
            }
          });
        });
      }

      // Clear the plugin context after onStart
      this.PluginAPI.clearCurrentPluginContext();

      this.setPluginState(pluginId, PLUGIN_STATES.RUNNING);
      // Plugin started successfully
    } catch (error) {
      this.setPluginError(pluginId, error);
      this.setPluginState(pluginId, PLUGIN_STATES.ERROR);

      // Clear the plugin context on error
      this.PluginAPI.clearCurrentPluginContext();
      throw error;
    }
  }


  async loadAllPlugins() {
    // Loading all plugins

    const discovered = await this.discoverPlugins();
    const loadPromises = [];

    for (const [id, pluginInfo] of discovered) {
      // Skip loading disabled plugins
      if (!pluginInfo.enabled) {
        this.setPluginState(id, PLUGIN_STATES.DISABLED);
        continue;
      }

      loadPromises.push(
        this.loadPlugin(pluginInfo).catch(error => {
          pluginStore.setPluginError(id, error);
          pluginStore.setPluginState(id, PLUGIN_STATES.ERROR);
          return null;
        })
      );
    }

    await Promise.all(loadPromises);

    const initPromises = [];
    const loadedPlugins = pluginStore.getAllPlugins().filter(p =>
      p.state === PLUGIN_STATES.LOADED && p.enabled
    );
    for (const plugin of loadedPlugins) {
      initPromises.push(
        this.initializePlugin(plugin.id).catch(error => {
          pluginStore.setPluginError(plugin.id, error);
          pluginStore.setPluginState(plugin.id, PLUGIN_STATES.ERROR);
          return null;
        })
      );
    }

    await Promise.all(initPromises);

    const startPromises = [];
    const initializedPlugins = pluginStore.getAllPlugins().filter(p =>
      p.state === PLUGIN_STATES.INITIALIZED && p.enabled
    );
    for (const plugin of initializedPlugins) {
      startPromises.push(
        this.startPlugin(plugin.id).catch(error => {
          pluginStore.setPluginError(plugin.id, error);
          pluginStore.setPluginState(plugin.id, PLUGIN_STATES.ERROR);
          return null;
        })
      );
    }

    await Promise.all(startPromises);

    // Plugin loading completed
  }

  async reloadPluginRegistry() {
    // Refresh plugins configuration from registry via store
    try {
      const success = await pluginStore.reloadConfigsFromFile();
      if (success) {
      }
      return success;
    } catch (error) {
      return false;
    }
  }

  async loadSinglePlugin(pluginId, pluginPath, mainFile) {
    try {
      // Create plugin info from parameters
      const pluginInfo = {
        id: pluginId,
        path: pluginPath,
        enabled: true,
        manifest: {
          name: pluginId.split('-').map(word =>
            word.charAt(0).toUpperCase() + word.slice(1)
          ).join(' ') + ' Plugin',
          version: '1.0.0',
          description: `Dynamically loaded plugin: ${pluginId}`,
          author: 'Plugin Developer',
          main: mainFile,
          dependencies: [],
          permissions: this.inferPermissions(pluginPath),
          apiVersion: '1.0.0',
          priority: 1
        }
      };

      // Load the plugin
      await this.loadPlugin(pluginInfo);

      // Initialize the plugin
      await this.initializePlugin(pluginId);

      // Start the plugin
      await this.startPlugin(pluginId);

      return pluginStore.getPluginInstance(pluginId)?.instance;
    } catch (error) {
      throw error;
    }
  }

  startUpdateLoop() {
    if (this.updateInterval) return;

    // Starting plugin update loop
    this.updateInterval = setInterval(() => {
      this.updatePlugins();
    }, 1000 / 60);
  }

  stopUpdateLoop() {
    if (this.updateInterval) {
      clearInterval(this.updateInterval);
      this.updateInterval = null;
      // Plugin update loop stopped
    }
  }

  updatePlugins() {
    const runningPlugins = this.getRunningPlugins();
    
    runningPlugins.forEach(plugin => {
      try {
        if (typeof plugin.instance.onUpdate === 'function') {
          plugin.instance.onUpdate();
        }
      } catch (error) {
        this.setPluginError(plugin.id, error);
      }
    });
  }

  getRunningPlugins() {
    return pluginStore.getRunningPlugins();
  }

  getPluginState(pluginId) {
    return pluginStore.getPluginState(pluginId);
  }

  setPluginState(pluginId, state) {
    pluginStore.setPluginState(pluginId, state);

    this.PluginAPI.emit('plugin-state-changed', {
      pluginId,
      state,
      timestamp: Date.now()
    });
  }

  setPluginError(pluginId, error) {
    pluginStore.setPluginError(pluginId, error);
  }

  getPluginInfo(pluginId) {
    return pluginStore.getAllPlugins().find(p => p.id === pluginId);
  }

  getAllPlugins() {
    return pluginStore.getAllPlugins();
  }

  getStats() {
    return pluginStore.getStats();
  }
}

export class PluginAPI {
  constructor() {
    this.id = 'plugin-api';
    this.version = '1.0.0';
    this.pluginLoader = new PluginLoader(this);
    this.initialized = false;
    this.currentRegistringPlugin = null; // Track which plugin is currently registering

    // Set up plugin store event listeners for reactive UI updates
    this.setupPluginStoreListeners();

    // Set up global viewport type tracking
    this.setupViewportTypeTracking();
  }

  setupViewportTypeTracking() {
    // Track which viewport type is currently active
    document.addEventListener('viewport:tab-activated', async (event) => {
      const { tabId } = event.detail;
      try {
        const { viewportStore } = await import('@/panels/viewport/store');
        const tab = viewportStore.tabs.find(t => t.id === tabId);
        if (tab) {
          // Reset all UI visibility to hidden by default when switching viewports
          // Plugins must explicitly show panels/footer/tabs in their onActivate or onStart
          setFooterVisible(false);
          setViewportTabsVisible(false);
          setBottomPanelVisible(false);
          setLeftPanelVisible(false);
          setPropertiesPanelVisible(false);
          setToolbarVisible(false);

          setActiveViewportType(tab.type);
        }
      } catch (err) {
        console.error('[PluginAPI] Failed to track viewport type:', err);
      }
    });

    // Also check for existing active tab on init (in case tab was set before listener)
    setTimeout(async () => {
      if (!activeViewportType()) {
        try {
          const { viewportStore } = await import('@/panels/viewport/store');
          const activeTab = viewportStore.tabs.find(t => t.id === viewportStore.activeTabId);
          if (activeTab) {
            setActiveViewportType(activeTab.type);
          }
        } catch (err) {
          // Ignore
        }
      }
    }, 100);
  }
  
  // Helper method to get the current plugin ID for registration
  getCurrentPluginId() {
    return this.currentRegistringPlugin;
  }
  
  // Set the current plugin context for registration
  setCurrentPluginContext(pluginId) {
    this.currentRegistringPlugin = pluginId;
  }
  
  // Clear the current plugin context
  clearCurrentPluginContext() {
    this.currentRegistringPlugin = null;
  }
  
  setupPluginStoreListeners() {
    // Listen for plugin state changes and clean up UI elements
    pluginStore.on('plugin-disabled', (data) => {
      this.cleanupPluginUIElements(data.pluginId);
    });
    
    pluginStore.on('plugin-enabling', (data) => {
    });
  }
  
  // Clean up all UI elements registered by a plugin
  cleanupPluginUIElements(pluginId) {
    
    try {
      // Remove top menu items
      setTopMenuItems(prev => {
        const newMap = new Map(prev);
        for (const [key, item] of newMap) {
          if (item.plugin === pluginId) {
            newMap.delete(key);
          }
        }
        return newMap;
      });

      // Remove top menu buttons
      setTopMenuButtons(prev => {
        const newMap = new Map(prev);
        for (const [key, button] of newMap) {
          if (button.plugin === pluginId) {
            newMap.delete(key);
          }
        }
        return newMap;
      });

      // Remove left panel components owned by this plugin
      setLeftPanelComponents(prev => {
        const newMap = new Map(prev);
        for (const [key, panel] of newMap) {
          if (panel.plugin === pluginId) {
            newMap.delete(key);
          }
        }
        return newMap;
      });

      // Remove right panel components owned by this plugin
      setRightPanelComponents(prev => {
        const newMap = new Map(prev);
        for (const [key, panel] of newMap) {
          if (panel.plugin === pluginId) {
            newMap.delete(key);
          }
        }
        return newMap;
      });

      // Remove viewport types
      setViewportTypes(prev => {
        const newMap = new Map(prev);
        for (const [key, viewport] of newMap) {
          if (viewport.plugin === pluginId) {
            newMap.delete(key);
          }
        }
        return newMap;
      });
      
      // Remove footer buttons
      setFooterButtons(prev => {
        const newMap = new Map(prev);
        for (const [key, button] of newMap) {
          if (button.plugin === pluginId) {
            newMap.delete(key);
          }
        }
        return newMap;
      });

      // Remove bottom panel tabs
      setBottomPanelTabs(prev => {
        const newMap = new Map(prev);
        for (const [key, tab] of newMap) {
          if (tab.plugin === pluginId) {
            newMap.delete(key);
          }
        }
        return newMap;
      });

      // Remove toolbar items
      setToolbarItems(prev => {
        const newMap = new Map(prev);
        for (const [key, item] of newMap) {
          if (item.plugin === pluginId) {
            newMap.delete(key);
          }
        }
        return newMap;
      });

      // Remove toolbar groups
      setToolbarGroups(prev => {
        const newMap = new Map(prev);
        for (const [key, group] of newMap) {
          if (group.plugin === pluginId) {
            newMap.delete(key);
          }
        }
        return newMap;
      });

      // Remove layout components
      setLayoutComponents(prev => {
        const newMap = new Map(prev);
        for (const [key, component] of newMap) {
          if (component.plugin === pluginId) {
            newMap.delete(key);
          }
        }
        return newMap;
      });

      // Remove from registered plugins
      setRegisteredPlugins(prev => {
        const newMap = new Map(prev);
        newMap.delete(pluginId);
        return newMap;
      });

    } catch (error) {
    }
  }

  async initialize() {
    if (this.initialized) return;

    // Initializing Plugin API

    try {
      await this.pluginLoader.loadAllPlugins();
      this.pluginLoader.startUpdateLoop();
      this.initialized = true;

      this.emit('api-initialized', {
        pluginStats: this.pluginLoader.getStats()
      });

    } catch (error) {
      throw error;
    }
  }

  async dispose() {
    if (!this.initialized) return;

    // Disposing Plugin API
    this.pluginLoader.stopUpdateLoop();

    const plugins = this.pluginLoader.getAllPlugins();
    for (const plugin of plugins) {
      if (plugin.instance) {
        // Clean up SolidJS reactive context if it exists
        if (plugin.instance._dispose) {
          try {
            plugin.instance._dispose();
          } catch (error) {
          }
        }

        if (typeof plugin.instance.onDispose === 'function') {
          try {
            await plugin.instance.onDispose();
          } catch (error) {
          }
        }
      }
    }

    this.initialized = false;
    // Plugin API disposed
  }

  registerTopMenuItem(id, config) {
    // Get the viewport type from config, or from the currently active viewport
    const viewportType = config.viewport || activeViewportType() || 'global';

    const menuItem = {
      id,
      label: config.label,
      onClick: config.onClick,
      icon: config.icon,
      submenu: config.submenu,
      order: config.order || 100,
      plugin: config.plugin || this.getCurrentPluginId() || 'unknown',
      viewport: viewportType
    };

    setTopMenuItems(prev => new Map(prev.set(id, menuItem)));
    return true;
  }

  registerLeftPanel(config) {
    // Get the viewport type from config, or from the currently active viewport
    const viewportType = config.viewport || activeViewportType() || 'global';

    const panel = {
      component: config.component,
      plugin: config.plugin || this.getCurrentPluginId() || 'unknown',
      viewport: viewportType
    };

    setLeftPanelComponents(prev => new Map(prev.set(viewportType, panel)));
    return true;
  }

  registerRightPanel(config) {
    // Get the viewport type from config, or from the currently active viewport
    const viewportType = config.viewport || activeViewportType() || 'global';

    const panel = {
      component: config.component,
      plugin: config.plugin || this.getCurrentPluginId() || 'unknown',
      viewport: viewportType
    };

    setRightPanelComponents(prev => new Map(prev.set(viewportType, panel)));
    return true;
  }

  registerViewportType(id, config) {
    const viewportType = {
      id,
      label: config.label,
      component: config.component,
      icon: config.icon,
      description: config.description || `${config.label} viewport`,
      onActivate: config.onActivate || null,
      onDeactivate: config.onDeactivate || null,
      plugin: config.plugin || this.getCurrentPluginId() || 'unknown'
    };

    setViewportTypes(prev => new Map(prev.set(id, viewportType)));

    // Set up event listeners for this viewport type's lifecycle
    if (config.onActivate || config.onDeactivate) {
      this.setupViewportLifecycleListeners(id, config.onActivate, config.onDeactivate);
    }

    return true;
  }

  setupViewportLifecycleListeners(viewportTypeId, onActivate, onDeactivate) {
    // Listen for tab activation
    if (onActivate) {
      document.addEventListener('viewport:tab-activated', async (event) => {
        const { tabId } = event.detail;
        // Dynamically import viewportStore to check tab type
        try {
          const { viewportStore } = await import('@/panels/viewport/store');
          const tab = viewportStore.tabs.find(t => t.id === tabId);
          if (tab && tab.type === viewportTypeId) {
            onActivate(this, tab);
          }
        } catch (err) {
          console.error('[PluginAPI] Failed to check tab activation:', err);
        }
      });
    }

    // Listen for tab deactivation
    if (onDeactivate) {
      document.addEventListener('viewport:tab-deactivated', async (event) => {
        const { tabId } = event.detail;
        // Dynamically import viewportStore to check tab type
        try {
          const { viewportStore } = await import('@/panels/viewport/store');
          const tab = viewportStore.tabs.find(t => t.id === tabId);
          if (tab && tab.type === viewportTypeId) {
            onDeactivate(this, tab);
          }
        } catch (err) {
          console.error('[PluginAPI] Failed to check tab deactivation:', err);
        }
      });
    }
  }

  registerFooterButton(id, config) {
    const button = {
      id,
      component: config.component,
      order: config.order || 100,
      priority: config.priority || 100,
      section: config.section || 'status',
      plugin: config.plugin || this.getCurrentPluginId() || 'unknown'
    };

    setFooterButtons(prev => new Map(prev.set(id, button)));
    return true;
  }

  registerTopMenuButton(id, config) {
    const button = {
      id,
      component: config.component,
      order: config.order || 100,
      plugin: config.plugin || this.getCurrentPluginId() || 'unknown'
    };

    setTopMenuButtons(prev => new Map(prev.set(id, button)));
    return true;
  }

  registerBottomPanelTab(id, config) {
    // Get the viewport type from config, or from the currently active viewport
    const viewportType = config.viewport || activeViewportType() || 'global';

    const tab = {
      id,
      title: config.title,
      component: config.component,
      icon: config.icon,
      order: config.order || 100,
      closable: config.closable !== false,
      plugin: config.plugin || this.getCurrentPluginId() || 'unknown',
      viewport: viewportType
    };

    setBottomPanelTabs(prev => new Map(prev.set(id, tab)));

    // Auto-show bottom panel when a tab is registered (only if active viewport matches)
    if (config.autoShow !== false) {
      const currentViewport = activeViewportType();
      if (tab.viewport === currentViewport) {
        setBottomPanelVisible(true);
      }
    }

    return true;
  }

  unregisterBottomPanelTab(id) {
    setBottomPanelTabs(prev => {
      const newMap = new Map(prev);
      newMap.delete(id);

      // Auto-hide bottom panel if no tabs remain
      if (newMap.size === 0) {
        setBottomPanelVisible(false);
      }

      return newMap;
    });
    return true;
  }

  registerToolbarItem(id, config) {
    // Get the viewport type from config, or from the currently active viewport
    const viewportType = config.viewport || activeViewportType() || 'global';

    const item = {
      id,
      icon: config.icon,
      label: config.label,
      tooltip: config.tooltip || config.label,
      onClick: config.onClick,
      component: config.component, // Custom component instead of icon button
      group: config.group || 'default',
      order: config.order || 100,
      disabled: config.disabled || (() => false),
      active: config.active || (() => false),
      visible: config.visible || (() => true),
      separator: config.separator || false, // Add separator after this item
      plugin: config.plugin || this.getCurrentPluginId() || 'unknown',
      viewport: viewportType
    };

    setToolbarItems(prev => new Map(prev.set(id, item)));
    return true;
  }

  unregisterToolbarItem(id) {
    setToolbarItems(prev => {
      const newMap = new Map(prev);
      newMap.delete(id);
      return newMap;
    });
    return true;
  }

  registerToolbarGroup(id, config) {
    // Get the viewport type from config, or from the currently active viewport
    const viewportType = config.viewport || activeViewportType() || 'global';

    const group = {
      id,
      label: config.label,
      order: config.order || 100,
      collapsible: config.collapsible !== false,
      visible: config.visible || (() => true),
      plugin: config.plugin || this.getCurrentPluginId() || 'unknown',
      viewport: viewportType
    };

    setToolbarGroups(prev => new Map(prev.set(id, group)));
    return true;
  }

  unregisterToolbarGroup(id) {
    setToolbarGroups(prev => {
      const newMap = new Map(prev);
      newMap.delete(id);
      return newMap;
    });
    return true;
  }

  registerLayoutComponent(region, component) {
    const layoutComponent = {
      component,
      plugin: this.getCurrentPluginId() || 'unknown'
    };
    setLayoutComponents(prev => new Map(prev.set(region, layoutComponent)));
    return true;
  }

  unregisterLayoutComponent(region) {
    setLayoutComponents(prev => {
      const newMap = new Map(prev);
      newMap.delete(region);
      return newMap;
    });
    // Layout component unregistered
    return true;
  }

  getLayoutComponent(region) {
    const layoutComponent = layoutComponents().get(region);
    // Handle both old and new structure for backwards compatibility
    if (layoutComponent?.component) {
      return layoutComponent.component;
    }
    return layoutComponent;
  }

  getLayoutComponents() {
    return layoutComponents();
  }

  registerPlugin(id, plugin) {
    const pluginConfig = {
      id,
      name: plugin.name,
      version: plugin.version,
      description: plugin.description,
      author: plugin.author,
      api: plugin.api || {},
      registeredAt: Date.now()
    };

    setRegisteredPlugins(prev => new Map(prev.set(id, pluginConfig)));
    // Plugin registered
    return true;
  }

  menu(id, config) { return this.registerTopMenuItem(id, config); }
  leftPanel(config) { return this.registerLeftPanel(config); }
  rightPanel(config) { return this.registerRightPanel(config); }
  viewport(id, config) { return this.registerViewportType(id, config); }
  footer(id, config) { return this.registerFooterButton(id, config); }
  topMenuButton(id, config) { return this.registerTopMenuButton(id, config); }
  bottomTab(id, config) { return this.registerBottomPanelTab(id, config); }
  toolbar(id, config) { return this.registerToolbarItem(id, config); }
  toolbarGroup(id, config) { return this.registerToolbarGroup(id, config); }
  open(typeId, options) { return this.createViewportTab(typeId, options); }

  createViewportTab(typeId, options = {}) {
    // Creating viewport tab for typeId
    const viewportType = viewportTypes().get(typeId);
    if (!viewportType) {
      return false;
    }

    try {
      import('@/panels/viewport/store').then(({ viewportActions, viewportStore }) => {
        // Check if a tab with this type already exists
        const existingTab = viewportStore.tabs.find(tab => tab.type === typeId);

        if (existingTab) {
          // Tab already exists, just activate it
          viewportActions.setActiveViewportTab(existingTab.id);
          return;
        }

        // No existing tab found, create a new one
        const newTabId = `${typeId}_${Date.now()}`;
        const newTab = {
          id: newTabId,
          name: options.label || viewportType.label,
          label: options.label || viewportType.label,
          type: typeId,
          icon: viewportType.icon,
          component: viewportType.component,
          ...options
        };

        // Creating viewport tab with new ID
        viewportActions.addViewportTab(newTab);

        if (options.setActive !== false) {
          viewportActions.setActiveViewportTab(newTabId);
        }
      }).catch(err => {
      });

      return true;
    } catch (error) {
      return false;
    }
  }

  createSceneViewport(options = {}) {
    // Deprecated: 3d-viewport type no longer exists
    console.warn('createSceneViewport is deprecated - 3d-viewport type no longer exists');
    return false;
  }

  setPropertiesPanelVisible(visible) {
    setPropertiesPanelVisible(visible);
    // Properties panel visibility changed
  }

  showProps(visible = true) { return this.setPropertiesPanelVisible(visible); }
  hideProps() { return this.setPropertiesPanelVisible(false); }

  setLeftPanelVisible(visible) {
    setLeftPanelVisible(visible);
  }

  showLeftPanel(visible = true) { return this.setLeftPanelVisible(visible); }
  hideLeftPanel() { return this.setLeftPanelVisible(false); }

  setHorizontalMenuButtonsEnabled(enabled) {
    setHorizontalMenuButtonsEnabled(enabled);
    // Horizontal menu buttons toggled
  }
  
  showMenu(enabled = true) { return this.setHorizontalMenuButtonsEnabled(enabled); }
  hideMenu() { return this.setHorizontalMenuButtonsEnabled(false); }

  setFooterVisible(visible) {
    setFooterVisible(visible);
    // Footer visibility changed
  }
  
  showFooter(visible = true) { return this.setFooterVisible(visible); }
  hideFooter() { return this.setFooterVisible(false); }

  setViewportTabsVisible(visible) {
    setViewportTabsVisible(visible);
    // Viewport tabs visibility changed
  }

  setBottomPanelVisible(visible) {
    setBottomPanelVisible(visible);
  }

  showBottomPanel(visible = true) { return this.setBottomPanelVisible(visible); }
  hideBottomPanel() { return this.setBottomPanelVisible(false); }
  toggleBottomPanel() { return this.setBottomPanelVisible(!bottomPanelVisible()); }

  setToolbarVisible(visible) {
    setToolbarVisible(visible);
  }

  showToolbar(visible = true) { return this.setToolbarVisible(visible); }
  hideToolbar() { return this.setToolbarVisible(false); }
  toggleToolbar() { return this.setToolbarVisible(!toolbarVisible()); }

  showTabs(visible = true) { return this.setViewportTabsVisible(visible); }
  hideTabs() { return this.setViewportTabsVisible(false); }

  setFullscreen(enabled) {
    setFullscreenMode(enabled);
  }

  showFullscreen(enabled = true) { return this.setFullscreen(enabled); }
  hideFullscreen() { return this.setFullscreen(false); }
  toggleFullscreen() { return this.setFullscreen(!fullscreenMode()); }

  getFullscreen() {
    return fullscreenMode();
  }

  async exit() {
    try {
      // Check if we're running in Tauri
      if (typeof window !== 'undefined' && window.__TAURI_INTERNALS__) {
        const { emit } = await import('@tauri-apps/api/event');
        await emit('proceed-with-close');
      } else {
        // Fallback for non-Tauri environments (e.g., browser development)
        console.warn('[PluginAPI] exit() called but not running in Tauri environment');
      }
    } catch (error) {
      console.error('[PluginAPI] Failed to exit application:', error);
      // Try alternative close method
      try {
        const { getCurrentWindow } = await import('@tauri-apps/api/window');
        const window = getCurrentWindow();
        await window.close();
      } catch (closeError) {
        console.error('[PluginAPI] Failed to close window:', closeError);
      }
    }
  }

  async fullscreen(enabled = true) {
    try {
      if (typeof window !== 'undefined' && window.__TAURI_INTERNALS__) {
        const { getCurrentWindow } = await import('@tauri-apps/api/window');
        const appWindow = getCurrentWindow();
        await appWindow.setFullscreen(enabled);
      } else {
        console.warn('[PluginAPI] fullscreen() called but not running in Tauri environment');
      }
    } catch (error) {
      console.error('[PluginAPI] Failed to set fullscreen:', error);
    }
  }

  async setWindowSize(width, height) {
    try {
      if (typeof window !== 'undefined' && window.__TAURI_INTERNALS__) {
        const { getCurrentWindow } = await import('@tauri-apps/api/window');
        const { LogicalSize } = await import('@tauri-apps/api/dpi');
        const appWindow = getCurrentWindow();
        await appWindow.setSize(new LogicalSize(width, height));
      } else {
        console.warn('[PluginAPI] setWindowSize() called but not running in Tauri environment');
      }
    } catch (error) {
      console.error('[PluginAPI] Failed to set window size:', error);
    }
  }

  async getWindowSize() {
    try {
      if (typeof window !== 'undefined' && window.__TAURI_INTERNALS__) {
        const { getCurrentWindow } = await import('@tauri-apps/api/window');
        const appWindow = getCurrentWindow();
        const size = await appWindow.innerSize();
        return { width: size.width, height: size.height };
      } else {
        console.warn('[PluginAPI] getWindowSize() called but not running in Tauri environment');
        return { width: window.innerWidth, height: window.innerHeight };
      }
    } catch (error) {
      console.error('[PluginAPI] Failed to get window size:', error);
      return { width: 0, height: 0 };
    }
  }

  async setWindowPosition(x, y) {
    try {
      if (typeof window !== 'undefined' && window.__TAURI_INTERNALS__) {
        const { getCurrentWindow } = await import('@tauri-apps/api/window');
        const { LogicalPosition } = await import('@tauri-apps/api/dpi');
        const appWindow = getCurrentWindow();
        await appWindow.setPosition(new LogicalPosition(x, y));
      } else {
        console.warn('[PluginAPI] setWindowPosition() called but not running in Tauri environment');
      }
    } catch (error) {
      console.error('[PluginAPI] Failed to set window position:', error);
    }
  }

  async getWindowPosition() {
    try {
      if (typeof window !== 'undefined' && window.__TAURI_INTERNALS__) {
        const { getCurrentWindow } = await import('@tauri-apps/api/window');
        const appWindow = getCurrentWindow();
        const position = await appWindow.outerPosition();
        return { x: position.x, y: position.y };
      } else {
        console.warn('[PluginAPI] getWindowPosition() called but not running in Tauri environment');
        return { x: 0, y: 0 };
      }
    } catch (error) {
      console.error('[PluginAPI] Failed to get window position:', error);
      return { x: 0, y: 0 };
    }
  }

  async setWindowMinSize(width, height) {
    try {
      if (typeof window !== 'undefined' && window.__TAURI_INTERNALS__) {
        const { getCurrentWindow } = await import('@tauri-apps/api/window');
        const { LogicalSize } = await import('@tauri-apps/api/dpi');
        const appWindow = getCurrentWindow();
        await appWindow.setMinSize(new LogicalSize(width, height));
      } else {
        console.warn('[PluginAPI] setWindowMinSize() called but not running in Tauri environment');
      }
    } catch (error) {
      console.error('[PluginAPI] Failed to set window min size:', error);
    }
  }

  async setWindowMaxSize(width, height) {
    try {
      if (typeof window !== 'undefined' && window.__TAURI_INTERNALS__) {
        const { getCurrentWindow } = await import('@tauri-apps/api/window');
        const { LogicalSize } = await import('@tauri-apps/api/dpi');
        const appWindow = getCurrentWindow();
        await appWindow.setMaxSize(new LogicalSize(width, height));
      } else {
        console.warn('[PluginAPI] setWindowMaxSize() called but not running in Tauri environment');
      }
    } catch (error) {
      console.error('[PluginAPI] Failed to set window max size:', error);
    }
  }

  async centerWindow() {
    try {
      if (typeof window !== 'undefined' && window.__TAURI_INTERNALS__) {
        const { getCurrentWindow } = await import('@tauri-apps/api/window');
        const appWindow = getCurrentWindow();
        await appWindow.center();
      } else {
        console.warn('[PluginAPI] centerWindow() called but not running in Tauri environment');
      }
    } catch (error) {
      console.error('[PluginAPI] Failed to center window:', error);
    }
  }

  async maximizeWindow() {
    try {
      if (typeof window !== 'undefined' && window.__TAURI_INTERNALS__) {
        const { getCurrentWindow } = await import('@tauri-apps/api/window');
        const appWindow = getCurrentWindow();
        await appWindow.maximize();
      } else {
        console.warn('[PluginAPI] maximizeWindow() called but not running in Tauri environment');
      }
    } catch (error) {
      console.error('[PluginAPI] Failed to maximize window:', error);
    }
  }

  async minimizeWindow() {
    try {
      if (typeof window !== 'undefined' && window.__TAURI_INTERNALS__) {
        const { getCurrentWindow } = await import('@tauri-apps/api/window');
        const appWindow = getCurrentWindow();
        await appWindow.minimize();
      } else {
        console.warn('[PluginAPI] minimizeWindow() called but not running in Tauri environment');
      }
    } catch (error) {
      console.error('[PluginAPI] Failed to minimize window:', error);
    }
  }

  async unmaximizeWindow() {
    try {
      if (typeof window !== 'undefined' && window.__TAURI_INTERNALS__) {
        const { getCurrentWindow } = await import('@tauri-apps/api/window');
        const appWindow = getCurrentWindow();
        await appWindow.unmaximize();
      } else {
        console.warn('[PluginAPI] unmaximizeWindow() called but not running in Tauri environment');
      }
    } catch (error) {
      console.error('[PluginAPI] Failed to unmaximize window:', error);
    }
  }

  async setWindowTitle(title) {
    try {
      if (typeof window !== 'undefined' && window.__TAURI_INTERNALS__) {
        const { getCurrentWindow } = await import('@tauri-apps/api/window');
        const appWindow = getCurrentWindow();
        await appWindow.setTitle(title);
      } else {
        console.warn('[PluginAPI] setWindowTitle() called but not running in Tauri environment');
        document.title = title;
      }
    } catch (error) {
      console.error('[PluginAPI] Failed to set window title:', error);
    }
  }

  showAll(visible = true) {
    setFooterVisible(visible);
    setViewportTabsVisible(visible);
    setBottomPanelVisible(visible);
    setLeftPanelVisible(visible);
    setPropertiesPanelVisible(visible);
    setToolbarVisible(visible);
    setHorizontalMenuButtonsEnabled(visible);
  }

  hideAll() {
    return this.showAll(false);
  }

  getTopMenuItems() {
    return Array.from(topMenuItems().values()).sort((a, b) => a.order - b.order);
  }

  getViewportTypes() {
    return Array.from(viewportTypes().values());
  }


  getFooterButtons() {
    return Array.from(footerButtons().values()).sort((a, b) => a.order - b.order);
  }

  getBottomPanelTabs() {
    return Array.from(bottomPanelTabs().values()).sort((a, b) => a.order - b.order);
  }

  getBottomPanelVisible() {
    return bottomPanelVisible();
  }

  getToolbarItems() {
    return Array.from(toolbarItems().values()).sort((a, b) => a.order - b.order);
  }

  getToolbarGroups() {
    return Array.from(toolbarGroups().values()).sort((a, b) => a.order - b.order);
  }

  getToolbarVisible() {
    return toolbarVisible();
  }

  getPlugins() {
    return Array.from(registeredPlugins().values());
  }

  getPlugin(id) {
    return registeredPlugins().get(id);
  }

  getPropertiesPanelVisible() {
    return propertiesPanelVisible();
  }


  getHorizontalMenuButtonsEnabled() {
    return horizontalMenuButtonsEnabled();
  }


  getPluginLoader() {
    return this.pluginLoader;
  }

  getPluginStats() {
    return this.pluginLoader.getStats();
  }

  async loadPluginDynamically(pluginId, pluginPath, mainFile) {
    try {
      return await this.pluginLoader.loadSinglePlugin(pluginId, pluginPath, mainFile);
    } catch (error) {
      throw error;
    }
  }

  async reloadPlugins() {
    try {
      await this.pluginLoader.reloadPluginRegistry();

      // Discover and load any new plugins
      const discovered = await this.pluginLoader.discoverPlugins();
      const currentPlugins = new Set(pluginStore.getAllPlugins().map(p => p.id));

      // Load only new plugins
      for (const [id, pluginInfo] of discovered) {
        if (!currentPlugins.has(id)) {
          try {
            await this.pluginLoader.loadPlugin(pluginInfo);
            await this.pluginLoader.initializePlugin(id);
            await this.pluginLoader.startPlugin(id);
          } catch (error) {
          }
        }
      }

      return true;
    } catch (error) {
      return false;
    }
  }

  /**
   * Reload a single runtime plugin by ID (unload and reload)
   * This is useful for hot-reloading plugins during development
   */
  async reloadRuntimePlugin(pluginId) {
    console.log(`[PluginAPI] Reloading runtime plugin: ${pluginId}`);

    try {
      // Check if plugin is currently loaded
      const existingPlugin = pluginStore.getPluginInstance(pluginId);

      if (existingPlugin) {
        console.log(`[PluginAPI] Unloading existing plugin: ${pluginId}`);
        // Clean up UI elements registered by this plugin
        this.cleanupPluginUIElements(pluginId);

        // Unload the plugin instance
        await pluginStore.unloadPlugin(pluginId);

        // Remove from plugin configs so it gets re-discovered
        pluginStore.removePluginConfig(pluginId);
      }

      // Re-discover plugins to pick up the updated version
      const discovered = await this.pluginLoader.discoverPlugins();

      // Find the plugin we want to reload
      const pluginInfo = discovered.get(pluginId);

      if (pluginInfo) {
        console.log(`[PluginAPI] Loading updated plugin: ${pluginId}`);
        await this.pluginLoader.loadPlugin(pluginInfo);
        await this.pluginLoader.initializePlugin(pluginId);
        await this.pluginLoader.startPlugin(pluginId);
        console.log(`[PluginAPI] Plugin ${pluginId} reloaded successfully`);
        return true;
      } else {
        console.warn(`[PluginAPI] Plugin ${pluginId} not found after rediscovery`);
        return false;
      }
    } catch (error) {
      console.error(`[PluginAPI] Failed to reload plugin ${pluginId}:`, error);
      throw error;
    }
  }

  emit(eventType, data) {
    const event = new CustomEvent(`plugin:${eventType}`, { detail: data });
    document.dispatchEvent(event);
    // Event emitted: eventType
  }

  on(eventType, callback) {
    const handler = (event) => callback(event.detail);
    document.addEventListener(`plugin:${eventType}`, handler);
    return () => document.removeEventListener(`plugin:${eventType}`, handler);
  }

  getInfo() {
    return {
      id: this.id,
      version: this.version,
      registeredTopMenuItems: topMenuItems().size,
      registeredPlugins: registeredPlugins().size
    };
  }
}

export const pluginAPI = new PluginAPI();

export function PluginAPIProvider(props) {
  return (
    <PluginAPIContext.Provider value={pluginAPI}>
      {props.children}
    </PluginAPIContext.Provider>
  );
}

export function usePluginAPI() {
  const api = useContext(PluginAPIContext);
  if (!api) {
    throw new Error('usePluginAPI must be used within a PluginAPIProvider');
  }
  return api;
}

export function Engine(props) {
  onMount(async () => {
    // Starting WebArcade Engine
    try {
      await pluginAPI.initialize();
      // Engine started successfully
    } catch (error) {
      console.error('Failed to start engine:', error);
    }
  });

  onCleanup(async () => {
    // Shutting down WebArcade Engine
    try {
      await pluginAPI.dispose();
      // Engine shut down successfully
    } catch (error) {
    }
  });

  return (
    <PluginAPIProvider>
      {props.children}
    </PluginAPIProvider>
  );
}

export { createPlugin } from './Plugin.jsx';

// Computed signals that return the correct panel for the active viewport
const leftPanelComponent = () => {
  const currentViewport = activeViewportType();
  const panels = leftPanelComponents();

  // Only return panel if it exists for the current viewport type
  if (currentViewport && panels.has(currentViewport)) {
    return panels.get(currentViewport);
  }

  // No panel for this viewport - return null to hide it
  return null;
};

const rightPanelComponent = () => {
  const currentViewport = activeViewportType();
  const panels = rightPanelComponents();

  // Only return panel if it exists for the current viewport type
  if (currentViewport && panels.has(currentViewport)) {
    return panels.get(currentViewport);
  }

  // No panel for this viewport - return null to hide it
  return null;
};

// Computed signal that returns bottom panel tabs filtered by active viewport
const filteredBottomPanelTabs = () => {
  const currentViewport = activeViewportType();
  const allTabs = bottomPanelTabs();
  const result = new Map();

  for (const [id, tab] of allTabs) {
    // Only include tab if it matches the current viewport
    if (tab.viewport === currentViewport) {
      result.set(id, tab);
    }
  }

  return result;
};

// Computed signal that returns toolbar items filtered by active viewport
const filteredToolbarItems = () => {
  const currentViewport = activeViewportType();
  const allItems = toolbarItems();
  const result = new Map();

  for (const [id, item] of allItems) {
    // Only include item if it matches the current viewport
    if (item.viewport === currentViewport) {
      result.set(id, item);
    }
  }

  return result;
};

// Computed signal that returns toolbar groups filtered by active viewport
const filteredToolbarGroups = () => {
  const currentViewport = activeViewportType();
  const allGroups = toolbarGroups();
  const result = new Map();

  for (const [id, group] of allGroups) {
    // Only include group if it matches the current viewport
    if (group.viewport === currentViewport) {
      result.set(id, group);
    }
  }

  return result;
};

export {
  topMenuItems,
  topMenuButtons,
  leftPanelComponent,
  rightPanelComponent,
  leftPanelComponents,
  rightPanelComponents,
  viewportTypes,
  footerButtons,
  registeredPlugins,
  bottomPanelTabs,
  filteredBottomPanelTabs,
  toolbarItems,
  toolbarGroups,
  filteredToolbarItems,
  filteredToolbarGroups,
  propertiesPanelVisible,
  leftPanelVisible,
  horizontalMenuButtonsEnabled,
  footerVisible,
  viewportTabsVisible,
  bottomPanelVisible,
  toolbarVisible,
  fullscreenMode,
  layoutComponents,
  activeViewportType,
  PLUGIN_STATES
};

