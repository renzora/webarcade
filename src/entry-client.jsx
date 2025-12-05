import { render } from 'solid-js/web'
import * as SolidJS from 'solid-js'
import * as SolidJSWeb from 'solid-js/web'
import * as SolidJSStore from 'solid-js/store'
import {
  plugin, createPlugin, usePluginAPI, viewportTypes, pluginAPI,
  panelStore, panels, activePlugin, panelVisibility, PANELS,
  horizontalMenuButtonsEnabled, footerVisible, viewportTabsVisible, pluginTabsVisible,
  leftPanelVisible, propertiesPanelVisible, bottomPanelVisible, toolbarVisible, fullscreenMode
} from './api/plugin'
import { api, BRIDGE_API, WEBARCADE_WS } from './api/bridge'
import App from './App'

// Expose SolidJS globally for runtime plugins
// Note: @tabler/icons-solidjs is NOT exposed globally - plugins bundle their own icons for tree-shaking
window.SolidJS = SolidJS
window.SolidJSWeb = SolidJSWeb
window.SolidJSStore = SolidJSStore

// Expose plugin API globally
window.WebArcadeAPI = {
  plugin,
  createPlugin,
  usePluginAPI,
  viewportTypes,
  pluginAPI,
  panelStore,
  panels,
  activePlugin,
  panelVisibility,
  PANELS,
  horizontalMenuButtonsEnabled,
  footerVisible,
  viewportTabsVisible,
  pluginTabsVisible,
  leftPanelVisible,
  propertiesPanelVisible,
  bottomPanelVisible,
  toolbarVisible,
  fullscreenMode,
  api,
  BRIDGE_API,
  WEBARCADE_WS
}

const root = document.getElementById('root')

if (import.meta.hot) {
  import.meta.hot.dispose(() => root.textContent = '')
}

render(() => <App />, root)
