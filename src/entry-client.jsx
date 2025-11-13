import { render } from 'solid-js/web'
import * as SolidJS from 'solid-js'
import * as SolidJSWeb from 'solid-js/web'
import * as SolidJSStore from 'solid-js/store'
import { createPlugin, usePluginAPI, viewportTypes } from './api/plugin'
import { bridge, BRIDGE_API, WEBARCADE_WS } from './api/bridge'
import App from './App'

// Expose SolidJS globally for runtime plugins
window.SolidJS = SolidJS
window.SolidJSWeb = SolidJSWeb
window.SolidJSStore = SolidJSStore

// Expose plugin API globally
window.WebArcadeAPI = {
  createPlugin,
  usePluginAPI,
  viewportTypes,
  bridge,
  BRIDGE_API,
  WEBARCADE_WS
}

const root = document.getElementById('root')

if (import.meta.hot) {
  import.meta.hot.dispose(() => root.textContent = '')
}

render(() => <App />, root)
