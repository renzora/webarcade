import { createSignal, createEffect, onCleanup, For } from 'solid-js';

export default function DemoBottomPanel() {
  const [logs, setLogs] = createSignal([
    { id: 1, type: 'info', message: 'Demo plugin initialized', timestamp: new Date() },
    { id: 2, type: 'success', message: 'All components registered successfully', timestamp: new Date() },
    { id: 3, type: 'info', message: 'Viewport ready', timestamp: new Date() }
  ]);
  const [filter, setFilter] = createSignal('all');

  createEffect(() => {
    const messages = [
      { type: 'info', message: 'Processing request...' },
      { type: 'success', message: 'Operation completed' },
      { type: 'warning', message: 'Resource usage high' },
      { type: 'error', message: 'Connection timeout' },
      { type: 'info', message: 'User action detected' },
      { type: 'success', message: 'Data saved successfully' }
    ];

    const interval = setInterval(() => {
      const randomMessage = messages[Math.floor(Math.random() * messages.length)];
      setLogs(prev => [...prev.slice(-49), {
        id: Date.now(),
        ...randomMessage,
        timestamp: new Date()
      }]);
    }, 3000);

    onCleanup(() => clearInterval(interval));
  });

  const filteredLogs = () => {
    if (filter() === 'all') return logs();
    return logs().filter(log => log.type === filter());
  };

  const getLogColor = (type) => {
    switch (type) {
      case 'success': return 'text-success';
      case 'warning': return 'text-warning';
      case 'error': return 'text-error';
      default: return 'text-info';
    }
  };

  const getLogBadge = (type) => {
    switch (type) {
      case 'success': return 'badge-success';
      case 'warning': return 'badge-warning';
      case 'error': return 'badge-error';
      default: return 'badge-info';
    }
  };

  const formatTime = (date) => {
    return date.toLocaleTimeString('en-US', {
      hour12: false,
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit'
    });
  };

  return (
    <div class="h-full flex flex-col bg-base-200">
      <div class="flex items-center justify-between px-3 py-1.5 border-b border-base-300 bg-base-100">
        <div class="flex items-center gap-2">
          <span class="text-xs font-medium text-base-content/70">Console Output</span>
          <span class="badge badge-sm">{filteredLogs().length}</span>
        </div>
        <div class="flex items-center gap-1">
          <select
            class="select select-xs select-bordered"
            value={filter()}
            onChange={(e) => setFilter(e.target.value)}
          >
            <option value="all">All</option>
            <option value="info">Info</option>
            <option value="success">Success</option>
            <option value="warning">Warning</option>
            <option value="error">Error</option>
          </select>
          <button class="btn btn-ghost btn-xs" onClick={() => setLogs([])}>Clear</button>
        </div>
      </div>

      <div class="flex-1 overflow-auto font-mono text-xs p-2 space-y-0.5">
        {filteredLogs().length === 0 ? (
          <div class="flex items-center justify-center h-full text-base-content/30">
            No logs to display
          </div>
        ) : (
          <For each={filteredLogs()}>
            {(log) => (
              <div class="flex items-start gap-2 px-2 py-0.5 hover:bg-base-300 rounded">
                <span class="text-base-content/30 whitespace-nowrap">
                  {formatTime(log.timestamp)}
                </span>
                <span class={`badge badge-xs ${getLogBadge(log.type)} uppercase`}>
                  {log.type}
                </span>
                <span class={getLogColor(log.type)}>
                  {log.message}
                </span>
              </div>
            )}
          </For>
        )}
      </div>

      <div class="px-2 py-1.5 border-t border-base-300 bg-base-100">
        <div class="flex items-center gap-2">
          <span class="text-primary font-mono text-xs">{'>'}</span>
          <input
            type="text"
            placeholder="Type a command..."
            class="input input-ghost input-xs flex-1 font-mono"
            onKeyDown={(e) => {
              if (e.key === 'Enter' && e.target.value.trim()) {
                setLogs(prev => [...prev, {
                  id: Date.now(),
                  type: 'info',
                  message: `Command: ${e.target.value}`,
                  timestamp: new Date()
                }]);
                e.target.value = '';
              }
            }}
          />
        </div>
      </div>
    </div>
  );
}
