import { createSignal, createEffect, onCleanup } from 'solid-js';
import { bridge } from '@/api/bridge';

// Build status store
const [buildStatus, setBuildStatus] = createSignal({
  frontend: {
    status: 'idle', // idle, building, success, error
    message: '',
    timestamp: null
  },
  backend: {
    status: 'checking', // checking, running, stopped, error
    message: '',
    timestamp: null
  }
});

// Check if we're in development mode
const isDev = () => {
  return import.meta.hot !== undefined || window.location.port === '8080';
};

// Monitor HMR WebSocket for build status
export const initBuildStatusMonitor = () => {
  if (!isDev()) {
    setBuildStatus({
      frontend: { status: 'production', message: 'Production build', timestamp: Date.now() },
      backend: { status: 'running', message: 'Running', timestamp: Date.now() }
    });
    return;
  }

  // Monitor HMR WebSocket
  if (import.meta.hot) {
    setBuildStatus(prev => ({
      ...prev,
      frontend: { status: 'success', message: 'Ready', timestamp: Date.now() }
    }));

    // Listen for HMR updates
    import.meta.hot.on('vite:beforeUpdate', () => {
      setBuildStatus(prev => ({
        ...prev,
        frontend: { status: 'building', message: 'Rebuilding...', timestamp: Date.now() }
      }));
    });

    import.meta.hot.on('vite:afterUpdate', () => {
      setBuildStatus(prev => ({
        ...prev,
        frontend: { status: 'success', message: 'Ready', timestamp: Date.now() }
      }));
    });

    import.meta.hot.on('vite:error', (err) => {
      setBuildStatus(prev => ({
        ...prev,
        frontend: { status: 'error', message: err.message || 'Build error', timestamp: Date.now() }
      }));
    });
  }

  // Check backend status via polling
  const checkBackendStatus = async () => {
    try {
      const response = await bridge('/api/health', {
        method: 'GET',
        signal: AbortSignal.timeout(2000)
      });

      if (response.ok) {
        setBuildStatus(prev => ({
          ...prev,
          backend: { status: 'running', message: 'Ready', timestamp: Date.now() }
        }));
      } else {
        setBuildStatus(prev => ({
          ...prev,
          backend: { status: 'error', message: 'Server error', timestamp: Date.now() }
        }));
      }
    } catch (error) {
      setBuildStatus(prev => ({
        ...prev,
        backend: { status: 'stopped', message: 'Not responding', timestamp: Date.now() }
      }));
    }
  };

  // Poll backend status every 3 seconds
  checkBackendStatus();
  const interval = setInterval(checkBackendStatus, 3000);

  return () => clearInterval(interval);
};

// Alternative: Monitor rspack HMR via WebSocket
export const monitorRspackHMR = () => {
  if (!isDev()) return;

  let ws;
  let reconnectTimer;
  const wsProtocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
  const wsUrl = `${wsProtocol}//${window.location.host}/ws`;

  const connect = () => {
    try {
      ws = new WebSocket(wsUrl);

      ws.addEventListener('open', () => {
        console.log('🔌 Connected to rspack HMR');
        setBuildStatus(prev => ({
          ...prev,
          frontend: { status: 'success', message: 'Ready', timestamp: Date.now() }
        }));
      });

      ws.addEventListener('message', (event) => {
        try {
          const data = JSON.parse(event.data);

          if (data.type === 'building') {
            setBuildStatus(prev => ({
              ...prev,
              frontend: { status: 'building', message: 'Rebuilding...', timestamp: Date.now() }
            }));
          } else if (data.type === 'built' || data.type === 'ok') {
            setBuildStatus(prev => ({
              ...prev,
              frontend: { status: 'success', message: 'Ready', timestamp: Date.now() }
            }));
          } else if (data.type === 'errors') {
            setBuildStatus(prev => ({
              ...prev,
              frontend: { status: 'error', message: 'Build failed', timestamp: Date.now() }
            }));
          }
        } catch (e) {
          // Ignore parse errors
        }
      });

      ws.addEventListener('close', () => {
        console.log('🔌 Disconnected from rspack HMR');
        setBuildStatus(prev => ({
          ...prev,
          frontend: { status: 'idle', message: 'Disconnected', timestamp: Date.now() }
        }));

        // Attempt to reconnect after 3 seconds
        reconnectTimer = setTimeout(connect, 3000);
      });

      ws.addEventListener('error', () => {
        setBuildStatus(prev => ({
          ...prev,
          frontend: { status: 'error', message: 'Connection error', timestamp: Date.now() }
        }));
      });
    } catch (err) {
      console.error('Failed to connect to HMR:', err);
    }
  };

  connect();

  return () => {
    if (ws) ws.close();
    if (reconnectTimer) clearTimeout(reconnectTimer);
  };
};

export { buildStatus, setBuildStatus };
