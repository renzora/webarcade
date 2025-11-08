import { createSignal, onMount, onCleanup } from 'solid-js';
import { IconServer } from '@tabler/icons-solidjs';

export default class BridgePluginClass {
  constructor() {
    this.name = 'Bridge Plugin';
    this.version = '1.0.0';
    this.description = 'Provides bridge server connectivity and file operations';
    this.apiPrefix = 'http://localhost:3001';
    
    // Status tracking
    this.statusSignal = createSignal(false);
    this.statusCheckInterval = null;
  }

  // Plugin lifecycle methods
  async initialize() {
    // Plugin initialization - equivalent to onActivate but following the expected interface
  }

  async onActivate(api) {
    this.api = api;
    
    // Register bridge status in the top bar
    this.api.registerStatusBarItem('bridge-status', {
      component: () => this.createStatusComponent(),
      position: 'right',
      priority: 100
    });

    // Register bridge viewport
    this.api.registerViewport('bridge-server', {
      title: 'Bridge Server',
      icon: IconServer,
      component: () => import('./BridgeViewport.jsx'),
      closable: true
    });

    // Start status monitoring
    this.startStatusMonitoring();
  }

  async onDeactivate() {
    this.stopStatusMonitoring();
  }

  // Status monitoring
  startStatusMonitoring() {
    const checkStatus = async () => {
      try {
        const { isServerConnected } = await import('./server.jsx');
        const connected = await isServerConnected();
        this.statusSignal[1](connected);
      } catch {
        this.statusSignal[1](false);
      }
    };

    // Initial check
    checkStatus();
    
    // Check every 5 seconds
    this.statusCheckInterval = setInterval(checkStatus, 5000);
  }

  stopStatusMonitoring() {
    if (this.statusCheckInterval) {
      clearInterval(this.statusCheckInterval);
      this.statusCheckInterval = null;
    }
  }

  // Status bar component
  createStatusComponent() {
    const [connected] = this.statusSignal;
    
    return (
      <button
        onClick={() => this.api?.openViewport('bridge-server')}
        class={`px-2 py-1 rounded text-xs font-medium transition-colors ${
          connected() 
            ? 'bg-success/20 text-success hover:bg-success/30' 
            : 'bg-error/20 text-error hover:bg-error/30'
        }`}
        title={`Bridge Server: ${connected() ? 'Connected' : 'Disconnected'}`}
      >
        <IconServer class="w-4 h-4 inline mr-1" />
        {connected() ? 'Bridge OK' : 'Bridge Error'}
      </button>
    );
  }

  // Bridge service methods - delegate to API
  async readFile(path) {
    const { readFile } = await import('./files.jsx');
    return readFile(path);
  }

  async writeFile(path, content) {
    const { writeFile } = await import('./files.jsx');
    return writeFile(path, content);
  }

  async deleteFile(path) {
    const { deleteFile } = await import('./files.jsx');
    return deleteFile(path);
  }

  async listDirectory(path = '') {
    const { listDirectory } = await import('./files.jsx');
    return listDirectory(path);
  }

  async getFileUrl(path) {
    const { getFileUrl } = await import('./files.jsx');
    return getFileUrl(path);
  }
}