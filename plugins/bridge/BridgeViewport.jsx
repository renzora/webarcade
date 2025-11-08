import { createSignal, onMount, onCleanup, For } from 'solid-js';
import { IconServer, IconCircleCheck, IconCircleX, IconRefresh } from '@tabler/icons-solidjs';
import { getHealth, getStartupTime, isServerConnected } from './server.jsx';

export default function BridgeViewport() {
  const [bridgeStatus, setBridgeStatus] = createSignal({
    connected: false,
    startupTime: null,
    systemStats: null
  });
  const [loading, setLoading] = createSignal(false);
  
  let statusInterval;

  const fetchBridgeInfo = async () => {
    setLoading(true);
    try {
      const connected = await isServerConnected();
      
      let startupTime = null;
      let systemStats = null;
      
      if (connected) {
        try {
          const startupData = await getStartupTime();
          startupTime = startupData.startup_time;
        } catch (e) {
        }
        
        try {
          const response = await fetch('http://localhost:3001/system/stats');
          if (response.ok) {
            systemStats = await response.json();
          }
        } catch (e) {
        }
      }
      
      setBridgeStatus({
        connected,
        startupTime,
        systemStats
      });
    } catch (error) {
      setBridgeStatus({
        connected: false,
        startupTime: null,
        systemStats: null
      });
    } finally {
      setLoading(false);
    }
  };

  const handleRefresh = () => {
    fetchBridgeInfo();
  };

  onMount(() => {
    fetchBridgeInfo();
    statusInterval = setInterval(fetchBridgeInfo, 10000); // Update every 10 seconds
  });

  onCleanup(() => {
    if (statusInterval) {
      clearInterval(statusInterval);
    }
  });

  const formatUptime = (startupTime) => {
    if (!startupTime) return 'Unknown';
    const now = Date.now() / 1000;
    const uptime = now - startupTime;
    const hours = Math.floor(uptime / 3600);
    const minutes = Math.floor((uptime % 3600) / 60);
    const seconds = Math.floor(uptime % 60);
    return `${hours}h ${minutes}m ${seconds}s`;
  };

  const formatBytes = (bytes) => {
    if (!bytes) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  };

  return (
    <div class="p-6 bg-base-100 min-h-full">
      <div class="flex items-center justify-between mb-6">
        <div class="flex items-center gap-3">
          <IconServer class="w-8 h-8 text-primary" />
          <div>
            <h1 class="text-2xl font-bold text-base-content">Bridge Server Status</h1>
            <p class="text-base-content/60">File operations and system monitoring</p>
          </div>
        </div>
        
        <button
          onClick={handleRefresh}
          disabled={loading()}
          class="btn btn-circle btn-ghost"
          title="Refresh"
        >
          <IconRefresh class={`w-5 h-5 ${loading() ? 'animate-spin' : ''}`} />
        </button>
      </div>

      <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Connection Status */}
        <div class="card bg-base-200 shadow-lg">
          <div class="card-body">
            <h2 class="card-title flex items-center gap-2">
              {bridgeStatus().connected ? (
                <IconCircleCheck class="w-5 h-5 text-success" />
              ) : (
                <IconCircleX class="w-5 h-5 text-error" />
              )}
              Connection Status
            </h2>
            
            <div class="space-y-2">
              <div class="flex justify-between">
                <span>Status:</span>
                <span class={`font-medium ${
                  bridgeStatus().connected ? 'text-success' : 'text-error'
                }`}>
                  {bridgeStatus().connected ? 'Connected' : 'Disconnected'}
                </span>
              </div>
              
              {bridgeStatus().connected && (
                <>
                  <div class="flex justify-between">
                    <span>Endpoint:</span>
                    <span class="font-mono text-sm">http://localhost:3001</span>
                  </div>
                  
                  <div class="flex justify-between">
                    <span>Uptime:</span>
                    <span class="font-mono text-sm">
                      {formatUptime(bridgeStatus().startupTime)}
                    </span>
                  </div>
                </>
              )}
            </div>
          </div>
        </div>

        {/* System Statistics */}
        <div class="card bg-base-200 shadow-lg">
          <div class="card-body">
            <h2 class="card-title">System Resources</h2>
            
            {bridgeStatus().connected && bridgeStatus().systemStats ? (
              <div class="space-y-3">
                <div>
                  <div class="flex justify-between mb-1">
                    <span>CPU Usage</span>
                    <span class="font-mono text-sm">
                      {bridgeStatus().systemStats.cpu_usage?.toFixed(1) || 0}%
                    </span>
                  </div>
                  <progress 
                    class="progress progress-primary w-full" 
                    value={bridgeStatus().systemStats.cpu_usage || 0} 
                    max="100"
                  />
                </div>
                
                <div>
                  <div class="flex justify-between mb-1">
                    <span>Memory Usage</span>
                    <span class="font-mono text-sm">
                      {formatBytes(bridgeStatus().systemStats.memory_used)} / {formatBytes(bridgeStatus().systemStats.memory_total)}
                    </span>
                  </div>
                  <progress 
                    class="progress progress-secondary w-full" 
                    value={bridgeStatus().systemStats.memory_used || 0} 
                    max={bridgeStatus().systemStats.memory_total || 1}
                  />
                </div>
                
                <div class="grid grid-cols-2 gap-4 mt-4 text-sm">
                  <div>
                    <span class="text-base-content/60">CPU Cores:</span>
                    <span class="ml-2 font-mono">{bridgeStatus().systemStats.cpu_cores || 'N/A'}</span>
                  </div>
                  <div>
                    <span class="text-base-content/60">Load Average:</span>
                    <span class="ml-2 font-mono">{bridgeStatus().systemStats.load_average?.[0]?.toFixed(2) || 'N/A'}</span>
                  </div>
                </div>
              </div>
            ) : (
              <div class="text-center text-base-content/60">
                {bridgeStatus().connected ? 'Loading system stats...' : 'Bridge server not connected'}
              </div>
            )}
          </div>
        </div>
      </div>

      {/* Available Endpoints */}
      <div class="card bg-base-200 shadow-lg mt-6">
        <div class="card-body">
          <h2 class="card-title">Available Endpoints</h2>
          
          <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div>
              <h3 class="font-semibold mb-2 text-primary">File Operations</h3>
              <ul class="space-y-1 text-sm font-mono">
                <li><span class="text-success">GET</span> /list/&lt;path&gt;</li>
                <li><span class="text-info">GET</span> /read/&lt;path&gt;</li>
                <li><span class="text-warning">POST</span> /write/&lt;path&gt;</li>
                <li><span class="text-warning">POST</span> /write-binary/&lt;path&gt;</li>
                <li><span class="text-error">DELETE</span> /delete/&lt;path&gt;</li>
                <li><span class="text-info">GET</span> /file/&lt;path&gt;</li>
              </ul>
            </div>
            
            <div>
              <h3 class="font-semibold mb-2 text-primary">System & Monitoring</h3>
              <ul class="space-y-1 text-sm font-mono">
                <li><span class="text-success">GET</span> /health</li>
                <li><span class="text-success">GET</span> /startup-time</li>
                <li><span class="text-success">GET</span> /system/stats</li>
                <li><span class="text-warning">POST</span> /restart</li>
                <li><span class="text-warning">POST</span> /clear-cache</li>
                <li><span class="text-primary">WS</span> ws://localhost:3002 (File changes)</li>
              </ul>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}