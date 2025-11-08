import { createSignal, createEffect, onCleanup, Show, For } from 'solid-js';
import { invoke } from '@tauri-apps/api/core';
import { IconTerminal, IconX, IconRefresh, IconAlertTriangle, IconExternalLink } from '@tabler/icons-solidjs';

const DebugConsole = () => {
  const [isOpen, setIsOpen] = createSignal(false);
  const [logs, setLogs] = createSignal([]);
  const [bridgeHealth, setBridgeHealth] = createSignal(null);
  const [healthError, setHealthError] = createSignal(null);
  const [autoScroll, setAutoScroll] = createSignal(true);
  let consoleRef;

  // Open logs in a new browser window for debugging
  const openLogsInWindow = () => {
    const logWindow = window.open('', 'Bridge Logs', 'width=1000,height=800');
    if (logWindow) {
      const currentLogs = logs();
      const healthStatus = bridgeHealth() ? 'Connected ✅' : `Disconnected ❌: ${healthError() || 'Unknown error'}`;

      logWindow.document.write(`
        <!DOCTYPE html>
        <html>
          <head>
            <title>WebArcade Bridge Logs</title>
            <style>
              body {
                margin: 0;
                padding: 20px;
                font-family: 'Consolas', 'Monaco', monospace;
                background: #1a1a1a;
                color: #e0e0e0;
                font-size: 13px;
              }
              .header {
                padding: 15px;
                background: #2d2d2d;
                border-radius: 8px;
                margin-bottom: 20px;
              }
              .status {
                font-size: 16px;
                font-weight: bold;
                margin-bottom: 10px;
              }
              .log-container {
                background: #0d0d0d;
                padding: 15px;
                border-radius: 8px;
                max-height: calc(100vh - 200px);
                overflow-y: auto;
              }
              .log-entry {
                padding: 4px 0;
                border-bottom: 1px solid #2d2d2d;
              }
              .log-entry:last-child {
                border-bottom: none;
              }
              .error { color: #ff6b6b; }
              .warning { color: #ffd93d; }
              .success { color: #6bcf7f; }
              .info { color: #4dabf7; }
              .line-number {
                color: #6c6c6c;
                margin-right: 10px;
                user-select: none;
              }
              .refresh-btn {
                margin-top: 10px;
                padding: 8px 16px;
                background: #4dabf7;
                color: white;
                border: none;
                border-radius: 4px;
                cursor: pointer;
                font-size: 14px;
              }
              .refresh-btn:hover {
                background: #339af0;
              }
            </style>
          </head>
          <body>
            <div class="header">
              <div class="status">Bridge Status: ${healthStatus}</div>
              <div>Total Logs: ${currentLogs.length}</div>
              <button class="refresh-btn" onclick="location.reload()">Refresh Logs</button>
            </div>
            <div class="log-container">
              ${currentLogs.map((log, i) => {
                let className = '';
                if (log.includes('[ERROR]') || log.includes('[BRIDGE ERROR]')) className = 'error';
                else if (log.includes('[WARN]')) className = 'warning';
                else if (log.includes('[SUCCESS]') || log.includes('✅') || log.includes('🟢')) className = 'success';
                else if (log.includes('[BRIDGE]') || log.includes('[SETUP]')) className = 'info';

                return `<div class="log-entry ${className}">
                  <span class="line-number">${i + 1}.</span>${log}
                </div>`;
              }).join('')}
            </div>
            <script>
              // Keep console open for debugging
              console.log('WebArcade Bridge Logs - Use browser DevTools (F12) to inspect further');
              // Auto-scroll to bottom
              window.scrollTo(0, document.body.scrollHeight);
            </script>
          </body>
        </html>
      `);
      logWindow.document.close();
    }
  };

  const fetchLogs = async () => {
    try {
      const bridgeLogs = await invoke('get_bridge_logs');
      setLogs(bridgeLogs);

      if (autoScroll() && consoleRef) {
        consoleRef.scrollTop = consoleRef.scrollHeight;
      }
    } catch (error) {
      console.error('Failed to fetch bridge logs:', error);
    }
  };

  const checkHealth = async () => {
    try {
      const health = await invoke('check_bridge_health');
      setBridgeHealth(health);
      setHealthError(null);
    } catch (error) {
      setHealthError(error);
      setBridgeHealth(null);
    }
  };


  createEffect(() => {
    if (isOpen()) {
      // Fetch logs immediately when opened
      fetchLogs();
      checkHealth();

      // Poll for new logs every 500ms while console is open
      const interval = setInterval(() => {
        fetchLogs();
        checkHealth();
      }, 500);

      onCleanup(() => clearInterval(interval));
    }
  });

  const getLogClass = (log) => {
    if (log.includes('[ERROR]') || log.includes('[BRIDGE ERROR]')) {
      return 'text-error';
    } else if (log.includes('[WARN]')) {
      return 'text-warning';
    } else if (log.includes('[SUCCESS]') || log.includes('✅') || log.includes('🟢')) {
      return 'text-success';
    } else if (log.includes('[BRIDGE]') || log.includes('[SETUP]')) {
      return 'text-info';
    }
    return 'text-base-content/70';
  };

  return (
    <>
      {/* Toggle Button */}
      <button
        class="btn btn-xs btn-ghost gap-1"
        onClick={() => setIsOpen(!isOpen())}
        title="Open Debug Console"
      >
        <IconTerminal class="w-3 h-3" />
        Debug
      </button>

      {/* Debug Console Modal */}
      <Show when={isOpen()}>
        <div class="fixed inset-0 bg-black/50 backdrop-blur-sm z-[9999] flex items-end justify-center p-4">
          <div class="bg-base-300 rounded-lg shadow-xl w-full max-w-6xl h-[70vh] flex flex-col">
            {/* Header */}
            <div class="flex items-center justify-between p-4 border-b border-base-content/10">
              <div class="flex items-center gap-3">
                <IconTerminal class="w-5 h-5 text-primary" />
                <h3 class="text-lg font-bold">Debug Console</h3>

                {/* Health Status */}
                <Show when={bridgeHealth()}>
                  <div class="badge badge-success gap-1">
                    <div class="w-2 h-2 rounded-full bg-success animate-pulse"></div>
                    Bridge Connected
                  </div>
                </Show>
                <Show when={healthError()}>
                  <div class="badge badge-error gap-1">
                    <IconAlertTriangle class="w-3 h-3" />
                    Bridge Disconnected
                  </div>
                </Show>
              </div>

              <div class="flex items-center gap-2">
                <button
                  class="btn btn-xs btn-ghost"
                  onClick={fetchLogs}
                  title="Refresh Logs"
                >
                  <IconRefresh class="w-4 h-4" />
                </button>

                <button
                  class="btn btn-xs btn-ghost gap-1"
                  onClick={openLogsInWindow}
                  title="Open logs in new window with browser DevTools"
                >
                  <IconExternalLink class="w-4 h-4" />
                  New Window
                </button>

                <label class="flex items-center gap-2 text-xs">
                  <input
                    type="checkbox"
                    class="checkbox checkbox-xs"
                    checked={autoScroll()}
                    onChange={(e) => setAutoScroll(e.target.checked)}
                  />
                  Auto-scroll
                </label>

                <button
                  class="btn btn-sm btn-ghost"
                  onClick={() => setIsOpen(false)}
                >
                  <IconX class="w-4 h-4" />
                </button>
              </div>
            </div>

            {/* Health Error Details */}
            <Show when={healthError()}>
              <div class="mx-4 mt-4 p-3 bg-error/10 border border-error/20 rounded-lg">
                <div class="flex items-start gap-2">
                  <IconAlertTriangle class="w-5 h-5 text-error flex-shrink-0 mt-0.5" />
                  <div class="flex-1">
                    <h4 class="font-bold text-error mb-1">Bridge Not Responding</h4>
                    <p class="text-sm text-base-content/70">{healthError()}</p>
                    <p class="text-xs text-base-content/50 mt-2">
                      This is why you're seeing "Loading..." in the footer. The bridge server may have failed to start.
                    </p>
                  </div>
                </div>
              </div>
            </Show>

            {/* Console Output */}
            <div
              ref={consoleRef}
              class="flex-1 overflow-y-auto p-4 font-mono text-xs bg-base-200"
              onScroll={(e) => {
                // Disable auto-scroll if user scrolls up
                const el = e.target;
                if (el.scrollTop < el.scrollHeight - el.clientHeight - 50) {
                  setAutoScroll(false);
                }
              }}
            >
              <Show when={logs().length === 0}>
                <div class="text-base-content/50 text-center py-8">
                  No logs available yet. Waiting for bridge to start...
                </div>
              </Show>

              <For each={logs()}>
                {(log, index) => (
                  <div class={`py-0.5 ${getLogClass(log)}`}>
                    <span class="text-base-content/40 select-none">{index() + 1}. </span>
                    {log}
                  </div>
                )}
              </For>
            </div>

            {/* Footer Info */}
            <div class="p-2 px-4 border-t border-base-content/10 text-xs text-base-content/50 flex justify-between items-center">
              <div>
                {logs().length} log entries
              </div>
              <div>
                Click "New Window" to open logs in a separate window where F12 works | Press Ctrl+C to copy selected text
              </div>
            </div>
          </div>
        </div>
      </Show>
    </>
  );
};

export default DebugConsole;
