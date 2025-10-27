import { createSignal, Show } from 'solid-js';
import { IconX, IconServer, IconAlertCircle } from '@tabler/icons-solidjs';

export default function BridgeModal({ isOpen, onClose }) {
  const [bridgeStatus, setBridgeStatus] = createSignal({
    connected: false,
    checking: false
  });

  const checkBridgeConnection = async () => {
    setBridgeStatus(prev => ({ ...prev, checking: true }));
    
    try {
      const { isServerConnected } = await import('./server.jsx');
      const connected = await isServerConnected();
      setBridgeStatus({
        connected,
        checking: false
      });
    } catch {
      setBridgeStatus({
        connected: false,
        checking: false
      });
    }
  };

  const handleTest = () => {
    checkBridgeConnection();
  };

  return (
    <Show when={isOpen()}>
      <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
        <div class="bg-base-200 rounded-xl shadow-2xl w-[500px] max-w-[90vw]">
          {/* Header */}
          <div class="flex items-center justify-between p-6 border-b border-base-300">
            <div class="flex items-center gap-3">
              <IconServer class="w-6 h-6 text-primary" />
              <h2 class="text-xl font-semibold text-base-content">Bridge Server</h2>
            </div>
            <button
              onClick={onClose}
              class="p-1 hover:bg-base-300 rounded transition-colors"
            >
              <IconX class="w-5 h-5" />
            </button>
          </div>

          {/* Content */}
          <div class="p-6 space-y-6">
            <div class="text-center">
              <IconServer class="w-16 h-16 mx-auto mb-4 text-primary" />
              <h3 class="text-lg font-medium text-base-content mb-2">
                Bridge Server Status
              </h3>
              <p class="text-base-content/60">
                The bridge server handles file operations and system monitoring
              </p>
            </div>

            {/* Status Section */}
            <div class="bg-base-100 rounded-lg p-4">
              <div class="flex items-center justify-between mb-3">
                <span class="font-medium">Connection Status</span>
                <button
                  onClick={handleTest}
                  disabled={bridgeStatus().checking}
                  class="btn btn-xs btn-outline"
                >
                  {bridgeStatus().checking ? 'Testing...' : 'Test Connection'}
                </button>
              </div>
              
              <div class="flex items-center gap-2">
                {bridgeStatus().checking ? (
                  <>
                    <div class="loading loading-spinner loading-xs"></div>
                    <span class="text-base-content/60">Checking connection...</span>
                  </>
                ) : bridgeStatus().connected ? (
                  <>
                    <div class="w-3 h-3 bg-success rounded-full"></div>
                    <span class="text-success">Connected to http://localhost:3001</span>
                  </>
                ) : (
                  <>
                    <IconAlertCircle class="w-4 h-4 text-error" />
                    <span class="text-error">Not connected</span>
                  </>
                )}
              </div>
            </div>

            {/* Information */}
            <div class="space-y-3">
              <h4 class="font-medium">Available Operations:</h4>
              <ul class="text-sm space-y-1 text-base-content/80">
                <li>• File reading and writing</li>
                <li>• Directory listing</li>
                <li>• Binary file operations</li>
                <li>• File system monitoring</li>
                <li>• System resource monitoring</li>
                <li>• Cache management</li>
              </ul>
            </div>

            {!bridgeStatus().connected && (
              <div class="alert alert-warning">
                <IconAlertCircle class="w-5 h-5" />
                <div>
                  <div class="font-medium">Bridge server not available</div>
                  <div class="text-sm">Make sure the bridge server is running on port 3001</div>
                </div>
              </div>
            )}
          </div>

          {/* Footer */}
          <div class="flex justify-end gap-3 p-6 border-t border-base-300">
            <button
              onClick={onClose}
              class="btn btn-ghost"
            >
              Close
            </button>
          </div>
        </div>
      </div>
    </Show>
  );
}