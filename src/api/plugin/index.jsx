import { createSignal, createContext, useContext, onMount, onCleanup, createRoot } from 'solid-js';
import pluginStore, { PLUGIN_STATES } from './store.jsx';

const PluginAPIContext = createContext();

const [topMenuItems, setTopMenuItems] = createSignal(new Map());
const [propertyTabs, setPropertyTabs] = createSignal(new Map());
const [viewportTypes, setViewportTypes] = createSignal(new Map());
const [footerButtons, setFooterButtons] = createSignal(new Map());
const [leftPanelMenuItems, setLeftPanelMenuItems] = createSignal(new Map());
const [registeredPlugins, setRegisteredPlugins] = createSignal(new Map());
const [widgets, setWidgets] = createSignal(new Map());
const [propertiesPanelVisible, setPropertiesPanelVisible] = createSignal(true);
const [leftPanelVisible, setLeftPanelVisible] = createSignal(true);
const [horizontalMenuButtonsEnabled, setHorizontalMenuButtonsEnabled] = createSignal(true);
const [footerVisible, setFooterVisible] = createSignal(true);
const [viewportTabsVisible, setViewportTabsVisible] = createSignal(true);
const [layoutComponents, setLayoutComponents] = createSignal(new Map());

class PluginLoader {
  constructor(PluginAPI) {
    this.PluginAPI = PluginAPI;
    this.updateInterval = null;
    this.pluginDirectories = [
      '/src/plugins'
    ];
  }

  isCorePlugin(pluginPath) {
    // Core plugins are bridge, default, and plugins manager
    const corePluginIds = ['bridge', 'default', 'plugins'];
    const pluginId = pluginPath.split('/').filter(Boolean).pop();
    return corePluginIds.includes(pluginId);
  }

  async discoverPlugins() {
    // Auto-discovering plugins
    const discovered = new Map();

    const autoDiscoveredPlugins = await this.scanForPlugins();
    
    autoDiscoveredPlugins.forEach(plugin => {
      discovered.set(plugin.id, plugin);
      this.setPluginState(plugin.id, PLUGIN_STATES.DISCOVERED);
    });

    // Auto-discovery completed
    return discovered;
  }

  async scanForPlugins() {
    const plugins = [];
    
    // Get plugin configs from store instead of importing JSON
    const pluginConfigs = pluginStore.getPluginConfigs();
    
    for (const [id, pluginConfig] of pluginConfigs) {
      try {
        if (pluginConfig.id.includes('test') && process.env.NODE_ENV === 'production') {
          continue;
        }

        const pathParts = pluginConfig.path.split('/').filter(p => p && p !== 'src' && p !== 'plugins' && p !== 'ui');
        const pluginName = pluginConfig.name || (pathParts
          .map(part => part.split(/[-_]/)
            .map(word => word.charAt(0).toUpperCase() + word.slice(1))
            .join(' '))
          .join(' ') + ' Plugin');

        const isCore = this.isCorePlugin(pluginConfig.path);
        
        const plugin = {
          id: pluginConfig.id,
          path: pluginConfig.path,
          enabled: isCore ? true : pluginConfig.enabled, // Core plugins are always enabled
          isCore: isCore,
          manifest: {
            name: pluginName,
            version: pluginConfig.version || '1.0.0',
            description: pluginConfig.description || `Plugin: ${pluginName}`,
            author: pluginConfig.author || 'Renzora Engine Team',
            main: pluginConfig.main,
            dependencies: [],
            permissions: this.inferPermissions(pluginConfig.path),
            apiVersion: '1.0.0',
            priority: pluginConfig.priority || 1,
            isCore: isCore
          }
        };

        plugins.push(plugin);
      } catch (error) {
      }
    }

    plugins.sort((a, b) => a.manifest.priority - b.manifest.priority);
    return plugins;
  }

  async loadPluginDynamic(id, path, mainFile) {

    try {
      // Use require.context to create a webpack context for all plugins
      // This scans the plugins directory at build time
      // Path from src/api/plugin/index.jsx to src/plugins/
      const pluginContext = require.context('../../plugins', true, /\.(jsx|js)$/);

      // Path format: /src/plugins/plugin_name -> ./plugin_name/index.jsx
      const relativePath = path.replace('/src/plugins', '.');
      const mainPath = `${relativePath}/${mainFile}`;

      // Check if this exact path exists
      if (pluginContext.keys().includes(mainPath)) {
        const pluginModule = pluginContext(mainPath);
        return pluginModule;
      }

      // Try without extension
      const pathWithoutExt = mainPath.replace(/\.(jsx|js)$/, '');
      const tryPaths = [
        `${pathWithoutExt}.jsx`,
        `${pathWithoutExt}.js`,
        `${pathWithoutExt}/index.jsx`,
        `${pathWithoutExt}/index.js`
      ];

      for (const tryPath of tryPaths) {
        if (pluginContext.keys().includes(tryPath)) {
          const pluginModule = pluginContext(tryPath);
          return pluginModule;
        }
      }

      throw new Error(`Plugin file not found: ${mainPath}. Available: ${pluginContext.keys().join(', ')}`);

    } catch (error) {
      throw error;
    }
  }

  inferPermissions(pluginPath) {
    const permissions = [];
    
    if (pluginPath.includes('/core/')) {
      permissions.push('core-engine', 'ui-core');
    }
    if (pluginPath.includes('/editor')) {
      permissions.push('ui-core', 'file-access', 'viewport-management');
    }
    if (pluginPath.includes('/splash')) {
      permissions.push('ui-core', 'viewport-management');
    }
    if (pluginPath.includes('/menu')) {
      permissions.push('ui-core');
    }
    if (pluginPath.includes('/bridge')) {
      permissions.push('file-access', 'network-access');
    }
    if (pluginPath.includes('/render')) {
      permissions.push('rendering', 'gpu-access');
    }
    
    return permissions.length > 0 ? permissions : ['ui-core'];
  }

  async loadPlugin(pluginInfo) {
    const { id, path, manifest } = pluginInfo;

    try {
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

        // Handle class-based plugins
        if (PluginFactory.prototype && PluginFactory.prototype.constructor) {
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
          pluginInstance = PluginFactory(this.PluginAPI);
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
    const pluginData = pluginStore.getPluginInstance(pluginId);
    const pluginConfig = pluginStore.getPluginConfig(pluginId);
    if (!pluginData || !pluginData.instance) {
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
    const pluginData = pluginStore.getPluginInstance(pluginId);
    const pluginConfig = pluginStore.getPluginConfig(pluginId);
    if (!pluginData || !pluginData.instance) {
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

      // Auto-load widget if it exists
      await this.autoLoadWidget(pluginId, plugin);

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

  // Helper method to load and register a widget
  loadWidgetComponent(pluginId, widgetPath, widgetConfig) {
    try {
      const pluginContext = require.context('../../plugins', true, /\.(jsx|js)$/);
      const relativePath = widgetPath.replace('/src/plugins', '.');

      if (pluginContext.keys().includes(relativePath)) {
        const widgetModule = pluginContext(relativePath);
        const WidgetComponent = widgetModule.default;

        if (WidgetComponent) {
          this.PluginAPI.registerWidget(widgetConfig.id, {
            ...widgetConfig,
            component: WidgetComponent
          });
          return true;
        }
      }
      return false;
    } catch (error) {
      return false;
    }
  }

  async autoLoadWidget(pluginId, plugin) {
    try {
      const pluginConfig = pluginStore.getPluginConfig(pluginId);
      if (!pluginConfig?.widget) return;

      const widgetPath = `${plugin.path}/${pluginConfig.widget}`;
      const widgetId = `${pluginId}-widget`;
      const widgetTitle = plugin.manifest.name || pluginId.split('-').map(w =>
        w.charAt(0).toUpperCase() + w.slice(1)
      ).join(' ');

      let widgetIcon = null;
      if (plugin.instance && typeof plugin.instance.getIcon === 'function') {
        widgetIcon = plugin.instance.getIcon();
      }

      this.loadWidgetComponent(pluginId, widgetPath, {
        id: widgetId,
        title: widgetTitle,
        component: null, // Will be set by loadWidgetComponent
        icon: widgetIcon,
        description: `${widgetTitle} widget`,
        defaultSize: { w: 2, h: 3 },
        order: plugin.manifest.priority || 100
      });
    } catch (error) {
      // Silently fail widget auto-loading
    }
  }

  async autoLoadWidgetsFromDirectory(pluginId) {
    try {
      const pluginConfig = pluginStore.getPluginConfig(pluginId);
      if (!pluginConfig?.widgets || pluginConfig.widgets.length === 0) return;

      for (const widgetFile of pluginConfig.widgets) {
        const widgetPath = `${pluginConfig.path}/widgets/${widgetFile}`;
        const widgetName = widgetFile.replace('.jsx', '').replace(/Widget$/, '');
        const widgetId = `${pluginId}-${widgetName.toLowerCase()}`;

        let widgetIcon = null;
        try {
          const icons = require('@tabler/icons-solidjs');
          const iconName = `Icon${widgetName}`;
          widgetIcon = icons[iconName] || icons.IconBox;
        } catch (e) {
          // Icon not found, will use default
        }

        const loaded = this.loadWidgetComponent(pluginId, widgetPath, {
          id: widgetId,
          title: widgetName,
          component: null, // Will be set by loadWidgetComponent
          icon: widgetIcon,
          description: `${widgetName} widget`,
          defaultSize: { w: 1, h: 1 },
          order: 100
        });

        if (loaded) {
          console.log(`  ✓ Auto-loaded widget: ${widgetId}`);
        }
      }
    } catch (error) {
      // Silently fail
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

    // Auto-load widgets from widgets directories
    const runningPlugins = pluginStore.getRunningPlugins();
    for (const plugin of runningPlugins) {
      await this.autoLoadWidgetsFromDirectory(plugin.id);
    }

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
      
      // Remove property tabs
      setPropertyTabs(prev => {
        const newMap = new Map(prev);
        for (const [key, tab] of newMap) {
          if (tab.plugin === pluginId) {
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

      // Remove left panel menu items
      setLeftPanelMenuItems(prev => {
        const newMap = new Map(prev);
        for (const [key, item] of newMap) {
          if (item.plugin === pluginId) {
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

      // Remove widgets
      setWidgets(prev => {
        const newMap = new Map(prev);
        for (const [key, widget] of newMap) {
          if (widget.plugin === pluginId) {
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
    const menuItem = {
      id,
      label: config.label,
      onClick: config.onClick,
      icon: config.icon,
      submenu: config.submenu,
      order: config.order || 100,
      plugin: config.plugin || this.getCurrentPluginId() || 'unknown'
    };

    setTopMenuItems(prev => new Map(prev.set(id, menuItem)));
    return true;
  }

  registerPropertyTab(id, config) {
    const tab = {
      id,
      title: config.title,
      component: config.component,
      icon: config.icon,
      order: config.order || 100,
      condition: config.condition || (() => true),
      viewport: config.viewport || null, // Optional: associated viewport type ID
      plugin: config.plugin || this.getCurrentPluginId() || 'unknown'
    };

    setPropertyTabs(prev => new Map(prev.set(id, tab)));
    return true;
  }

  registerViewportType(id, config) {
    const viewportType = {
      id,
      label: config.label,
      component: config.component,
      icon: config.icon,
      description: config.description || `${config.label} viewport`,
      plugin: config.plugin || this.getCurrentPluginId() || 'unknown'
    };

    setViewportTypes(prev => new Map(prev.set(id, viewportType)));
    return true;
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

  registerLeftPanelMenuItem(id, config) {
    const menuItem = {
      id,
      label: config.label,
      icon: config.icon,
      description: config.description || '',
      onClick: config.onClick,
      order: config.order || 100,
      category: config.category || 'General',
      plugin: config.plugin || this.getCurrentPluginId() || 'unknown'
    };

    setLeftPanelMenuItems(prev => new Map(prev.set(id, menuItem)));
    return true;
  }

  registerWidget(id, config) {
    const widget = {
      id,
      title: config.title,
      component: config.component,
      icon: config.icon,
      description: config.description || '',
      defaultSize: config.defaultSize || { w: 2, h: 2 },
      minSize: config.minSize || { w: 1, h: 1 },
      maxSize: config.maxSize || { w: 12, h: 12 },
      order: config.order || 100,
      plugin: config.plugin || this.getCurrentPluginId() || 'unknown'
    };

    setWidgets(prev => new Map(prev.set(id, widget)));
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
  tab(id, config) { return this.registerPropertyTab(id, config); }
  viewport(id, config) { return this.registerViewportType(id, config); }
  footer(id, config) { return this.registerFooterButton(id, config); }
  widget(id, config) { return this.registerWidget(id, config); }
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
    // Left panel visibility changed
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
  
  showTabs(visible = true) { return this.setViewportTabsVisible(visible); }
  hideTabs() { return this.setViewportTabsVisible(false); }



  getTopMenuItems() {
    return Array.from(topMenuItems().values()).sort((a, b) => a.order - b.order);
  }

  getPropertyTabs() {
    return Array.from(propertyTabs().values()).sort((a, b) => a.order - b.order);
  }


  getViewportTypes() {
    return Array.from(viewportTypes().values());
  }


  getFooterButtons() {
    return Array.from(footerButtons().values()).sort((a, b) => a.order - b.order);
  }

  getLeftPanelMenuItems() {
    return Array.from(leftPanelMenuItems().values()).sort((a, b) => a.order - b.order);
  }

  getWidgets() {
    return Array.from(widgets().values()).sort((a, b) => a.order - b.order);
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
      registeredPropertyTabs: propertyTabs().size,
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
    // Starting Renzora Engine
    try {
      await pluginAPI.initialize();
      // Renzora Engine started successfully
    } catch (error) {
    }
  });

  onCleanup(async () => {
    // Shutting down Renzora Engine
    try {
      await pluginAPI.dispose();
      // Renzora Engine shut down successfully
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

export {
  topMenuItems,
  propertyTabs,
  viewportTypes,
  footerButtons,
  leftPanelMenuItems,
  registeredPlugins,
  widgets,
  propertiesPanelVisible,
  leftPanelVisible,
  horizontalMenuButtonsEnabled,
  footerVisible,
  viewportTabsVisible,
  layoutComponents,
  PLUGIN_STATES
};

