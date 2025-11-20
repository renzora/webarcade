import { createStore } from 'solid-js/store';

const [editorStore, setEditorStore] = createStore({
  isOpen: false,
  
  selection: {
    entity: null,
    entities: [], // Array of selected entity IDs for multi-selection
    transformMode: 'select'
  },
  
  ui: {
    rightPanelWidth: 300,
    bottomPanelHeight: 300,
    scenePropertiesHeight: Math.floor(window.innerHeight * 0.7),
    selectedTool: 'scene',
    selectedBottomTab: 'assets'
  },

  
  panels: {
    isResizingPanels: false,
    isScenePanelOpen: true,
    isAssetPanelOpen: true
  },
  
  console: {
    contextMenuHandler: null,
    messages: []
  },
  

  theme: 'dark',

  powerMode: {
    enabled: true,
    shake: true,
    particles: true,
    maxParticles: 500,
    particleSize: 8,
    shakeIntensity: 3
  },

  backgroundMode: false, // When true: maximized, no decorations, always on bottom

  settings: {
    viewport: {
      backgroundColor: '#1a202c',
      renderingEngine: 'webgl'
    },
    editor: {
      showStats: true,
      panelPosition: 'right',
      scriptReloadDebounceMs: 500,
      renderPaused: false
    },
    grid: {
      enabled: true,
      unit: 'centimeters',
      size: 20,
      cellSize: 100,
      sectionSize: 10,
      infiniteGrid: true,
      position: [0, 0, 0],
      cellColor: '#334155',
      sectionColor: '#475569'
    }
  }
})

export const editorActions = {
  toggleOpen: () => {
    setEditorStore('isOpen', !editorStore.isOpen)
  },
  
  setSelectedTool: (tool) => {
    setEditorStore('ui', 'selectedTool', tool)
  },

  setSelectedBottomTab: (tab) => {
    setEditorStore('ui', 'selectedBottomTab', tab)
  },
  
  selectEntity: (entityId, entityIds = null) => {
    setEditorStore('selection', 'entity', entityId);
    if (entityIds !== null) {
      setEditorStore('selection', 'entities', entityIds);
    } else {
      setEditorStore('selection', 'entities', entityId ? [entityId] : []);
    }
  },
  
  setScenePanelOpen: (isOpen) => {
    setEditorStore('panels', 'isScenePanelOpen', isOpen);
  },

  setAssetPanelOpen: (isOpen) => {
    setEditorStore('panels', 'isAssetPanelOpen', isOpen);
  },
  
  setResizingPanels: (isResizing) => {
    setEditorStore('panels', 'isResizingPanels', isResizing)
  },

  setRightPanelWidth: (width) => {
    setEditorStore('ui', 'rightPanelWidth', width)
  },

  setBottomPanelHeight: (height) => {
    setEditorStore('ui', 'bottomPanelHeight', height)
  },
  
  setScenePropertiesHeight: (height) => {
    setEditorStore('ui', 'scenePropertiesHeight', height)
  },
  
  setContextMenuHandler: (handler) => {
    setEditorStore('console', 'contextMenuHandler', handler)
  },
  
  addConsoleMessage: (message, type = 'info') => {
    setEditorStore('console', 'messages', editorStore.console.messages.length, {
      message,
      type,
      timestamp: Date.now()
    })
  },
  
  setTransformMode: (mode) => {
    setEditorStore('selection', 'transformMode', mode)
  },
  
  updateViewportSettings: (settings) => {
    setEditorStore('settings', 'viewport', settings);
    editorActions.saveToProject();
  },
  
  toggleStats: () => {
    setEditorStore('settings', 'editor', 'showStats', !editorStore.settings.editor.showStats)
  },
  
  updateEditorSettings: (settings) => {
    setEditorStore('settings', 'editor', settings);
    editorActions.saveToProject();
  },
  
  updateGridSettings: (settings) => {
    setEditorStore('settings', 'grid', settings);
    editorActions.saveToProject();
  },
  
  setToolbarTabOrder: (order) => {
    setEditorStore('ui', 'toolbarTabOrder', order)
  },
  
  setToolbarBottomTabOrder: (order) => {
    setEditorStore('ui', 'toolbarBottomTabOrder', order)
  },
  
  setBottomTabOrder: (order) => {
    setEditorStore('ui', 'bottomTabOrder', order)
  },

  setAssetsLibraryWidth: (width) => {
    setEditorStore('ui', 'assetsLibraryWidth', width)
  },


  toggleScriptExecution: () => {
    const newState = !editorStore.scripts.isPlaying;
    setEditorStore('scripts', 'isPlaying', newState);
    editorActions.saveToProject();
    
    // Trigger script manager state change
    const event = new CustomEvent('engine:script-execution-toggle', {
      detail: { isPlaying: newState }
    });
    document.dispatchEvent(event);
  },

  setScriptExecution: (isPlaying) => {
    setEditorStore('scripts', 'isPlaying', isPlaying);
    editorActions.saveToProject();
    
    // Trigger script manager state change
    const event = new CustomEvent('engine:script-execution-toggle', {
      detail: { isPlaying }
    });
    document.dispatchEvent(event);
  },

  setCurrentMode: (mode) => {
    setEditorStore('ui', 'currentMode', mode);
    editorActions.saveToProject();
    
    // Trigger mode change event
    const event = new CustomEvent('engine:mode-change', {
      detail: { mode, previousMode: editorStore.ui.currentMode }
    });
    document.dispatchEvent(event);
  },

  setTheme: (theme) => {
    setEditorStore('theme', theme);
    editorActions.saveToProject();

    // Apply theme to DOM
    document.documentElement.setAttribute('data-theme', theme);
  },

  updatePowerModeSetting: (key, value) => {
    setEditorStore('powerMode', key, value);
    editorActions.saveToProject();
  },

  toggleBackgroundMode: async () => {
    const newMode = !editorStore.backgroundMode;
    setEditorStore('backgroundMode', newMode);

    // Apply or remove background-mode class to body for styling
    if (newMode) {
      document.body.classList.add('background-mode');
    } else {
      document.body.classList.remove('background-mode');
    }

    if (typeof window !== 'undefined' && window.__TAURI_INTERNALS__) {
      try {
        const { getCurrentWindow } = await import('@tauri-apps/api/window');
        const currentWindow = getCurrentWindow();

        if (newMode) {
          // Entering background mode - maximize window and stay behind other windows
          await currentWindow.maximize();
          await currentWindow.setAlwaysOnBottom(true);
          // Don't use setIgnoreCursorEvents - we'll use CSS pointer-events instead
        } else {
          // Exiting background mode - restore window
          await currentWindow.setAlwaysOnBottom(false);
          await currentWindow.unmaximize();
        }
      } catch (error) {
        console.error('Failed to toggle background mode:', error);
      }
    }
  },

  // Save settings to project file instead of localStorage
  saveToProject: () => {
  },

  // Load settings from project data
  loadFromProject: (projectSettings) => {
    if (!projectSettings) return;

    // Load all settings from project
    if (projectSettings.settings) {
      setEditorStore('settings', projectSettings.settings);
    }

    if (projectSettings.ui) {
      setEditorStore('ui', 'currentMode', projectSettings.ui.currentMode || 'standard');
    }

    if (projectSettings.scripts) {
      setEditorStore('scripts', projectSettings.scripts);
    }

    if (projectSettings.theme) {
      setEditorStore('theme', projectSettings.theme);
      // Apply theme to DOM
      document.documentElement.setAttribute('data-theme', projectSettings.theme);
    }
  },

  // Control system actions
  setControlEnabled: (category, control, enabled) => {
    setEditorStore('controls', category, control, enabled);
  },

  setControlsEnabled: (category, controls) => {
    Object.entries(controls).forEach(([control, enabled]) => {
      setEditorStore('controls', category, control, enabled);
    });
  },

  // Utility to get effective control state (base + mode overrides)
  getEffectiveControls: () => {
    const currentMode = editorStore.ui.currentMode;
    const baseControls = editorStore.controls;
    const modeOverrides = baseControls.overrides[currentMode] || {};
    
    // Deep merge base controls with mode-specific overrides
    const effectiveControls = JSON.parse(JSON.stringify(baseControls));
    
    Object.entries(modeOverrides).forEach(([category, overrides]) => {
      if (effectiveControls[category]) {
        Object.assign(effectiveControls[category], overrides);
      }
    });
    
    return effectiveControls;
  },

  // Convenient getters for specific control states
  canDeselect: () => {
    const controls = editorActions.getEffectiveControls();
    return controls.selection.enabled && controls.selection.allowDeselection;
  },

  canPanCamera: (button = 0) => {
    const controls = editorActions.getEffectiveControls();
    if (button === 0) return controls.camera.leftClickPanEnabled;
    if (button === 2) return controls.camera.rightClickPanEnabled;
    return controls.camera.leftClickPanEnabled || controls.camera.rightClickPanEnabled;
  },

  canOrbitCamera: (button = 1) => {
    const controls = editorActions.getEffectiveControls();
    if (button === 1) return controls.camera.middleClickOrbitEnabled;
    if (button === 2) return controls.camera.rightClickOrbitEnabled;
    return controls.camera.middleClickOrbitEnabled || controls.camera.rightClickOrbitEnabled;
  },

  canZoomCamera: () => {
    const controls = editorActions.getEffectiveControls();
    return controls.camera.zoomEnabled;
  },

  canPickObjects: () => {
    const controls = editorActions.getEffectiveControls();
    return controls.interaction.objectPickingEnabled;
  },

  canUseKeyboardShortcuts: () => {
    const controls = editorActions.getEffectiveControls();
    return controls.interaction.keyboardShortcutsEnabled;
  },

  canUseMouseWheel: () => {
    const controls = editorActions.getEffectiveControls();
    return controls.interaction.mouseWheelEnabled;
  },

  canShowGizmos: () => {
    const controls = editorActions.getEffectiveControls();
    return controls.transform.gizmosEnabled;
  },

  canTransform: (type = 'position') => {
    const controls = editorActions.getEffectiveControls();
    return controls.transform.gizmosEnabled && controls.transform[`${type}Enabled`];
  }
}

export { editorStore }

if (typeof window !== 'undefined') {
  window.editorStore = editorStore
  window.editorActions = editorActions
}
