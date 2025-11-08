import { render } from 'solid-js/web';
import { createSignal, createEffect, onCleanup, For, Show } from 'solid-js';
import '@/index.css';
import { BRIDGE_API, WEBARCADE_WS } from '@/api/bridge';

const BRIDGE_URL = BRIDGE_API;
const REFRESH_INTERVAL = 5000; // Refresh every 5 seconds

function TodosOverlay() {
  const [todos, setTodos] = createSignal([]);
  const [isLoading, setIsLoading] = createSignal(true);
  // Default to true, will be fetched from database
  const [isEnabled, setIsEnabled] = createSignal(true);

  let intervalId = null;
  let ws;

  const fetchTodos = async () => {
    try {
      const response = await fetch(`${BRIDGE_URL}/database/todos`);
      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }
      const data = await response.json();

      // The API now returns the array directly, not wrapped in a success object
      if (Array.isArray(data)) {
        setTodos(data);
      } else if (data.success && data.todos) {
        setTodos(data.todos);
      }
      setIsLoading(false);
    } catch (error) {
      console.error('Failed to fetch todos:', error);
      setIsLoading(false);
    }
  };

  const fetchToggleState = async () => {
    try {
      const response = await fetch(`${BRIDGE_URL}/database/todos/toggle`);
      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }
      const data = await response.json();
      console.log('Fetched toggle state from database:', data.enabled);
      setIsEnabled(data.enabled);
    } catch (error) {
      console.error('Failed to fetch toggle state:', error);
    }
  };

  const connectWebSocket = () => {
    ws = new WebSocket(WEBARCADE_WS);

    ws.onopen = () => {
      console.log('WebSocket connected');
    };

    ws.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data);
        console.log('WebSocket message received:', data);

        if (data.type === 'community_tasks_toggle') {
          console.log('Setting isEnabled to:', data.enabled);
          setIsEnabled(data.enabled);
        }
      } catch (error) {
        console.error('WebSocket message error:', error);
      }
    };

    ws.onclose = () => {
      console.log('WebSocket disconnected');
      setTimeout(connectWebSocket, 3000);
    };

    ws.onerror = (error) => {
      console.error('WebSocket error:', error);
    };
  };

  // Fetch todos and toggle state on mount and set up interval
  createEffect(() => {
    fetchTodos();
    fetchToggleState();
    intervalId = setInterval(fetchTodos, REFRESH_INTERVAL);
    connectWebSocket();

    onCleanup(() => {
      if (intervalId) {
        clearInterval(intervalId);
      }
      if (ws) {
        ws.close();
      }
    });
  });

  // Calculate minutes ago from timestamp
  const getMinutesAgo = (timestamp) => {
    const now = Math.floor(Date.now() / 1000);
    const diffSeconds = now - timestamp;
    const diffMinutes = Math.floor(diffSeconds / 60);

    if (diffMinutes < 1) return 'just now';
    if (diffMinutes === 1) return '1 minute ago';
    if (diffMinutes < 60) return `${diffMinutes} minutes ago`;

    const diffHours = Math.floor(diffMinutes / 60);
    if (diffHours === 1) return '1 hour ago';
    if (diffHours < 24) return `${diffHours} hours ago`;

    const diffDays = Math.floor(diffHours / 24);
    if (diffDays === 1) return '1 day ago';
    return `${diffDays} days ago`;
  };

  // Debug log to track state changes
  createEffect(() => {
    console.log('Overlay isEnabled state changed to:', isEnabled());
  });

  return (
    <div class="fixed inset-0 pointer-events-none overflow-hidden font-sans bg-transparent">
      <style>{`
        @keyframes fadeIn {
          from { opacity: 0; transform: translateY(-10px); }
          to { opacity: 1; transform: translateY(0); }
        }
        .task-item {
          animation: fadeIn 0.3s ease-out;
        }
        .cozy-pattern {
          background-image:
            radial-gradient(circle at 20% 50%, rgba(251, 191, 36, 0.03) 0%, transparent 50%),
            radial-gradient(circle at 80% 80%, rgba(249, 115, 22, 0.03) 0%, transparent 50%);
        }
      `}</style>

      {/* Todos Panel - Only show when enabled */}
      <Show when={isEnabled()}>
        <div class="absolute top-0 left-0 w-full h-full overflow-hidden">
          <div class="bg-gradient-to-br from-amber-50/95 via-orange-50/95 to-rose-50/95 backdrop-blur-xl shadow-2xl overflow-hidden h-full flex flex-col cozy-pattern">
          {/* Header */}
          <div class="px-8 py-6 bg-gradient-to-r from-amber-100/40 to-orange-100/40 border-b-2 border-amber-200/30">
            <div class="flex items-center gap-3 mb-2">
              <div class="p-2.5 bg-gradient-to-br from-amber-400 to-orange-500 rounded-2xl shadow-lg">
                <svg xmlns="http://www.w3.org/2000/svg" class="w-7 h-7 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2" />
                </svg>
              </div>
              <h2 class="text-3xl font-bold bg-gradient-to-r from-amber-800 to-orange-700 bg-clip-text text-transparent">
                Community Tasks
              </h2>
            </div>
            <div class="flex items-center gap-2 ml-1">
              <div class="px-3 py-1 bg-amber-200/50 rounded-full border border-amber-300/50">
                <p class="text-sm font-semibold text-amber-900">
                  {todos().length} task{todos().length !== 1 ? 's' : ''}
                </p>
              </div>
            </div>
          </div>

          {/* Content */}
          <div class="flex-1 overflow-auto">
            <Show when={!isLoading()} fallback={
              <div class="flex items-center justify-center py-16">
                <div class="flex flex-col items-center gap-3">
                  <div class="w-12 h-12 border-4 border-amber-300 border-t-orange-500 rounded-full animate-spin"></div>
                  <span class="text-amber-800/60 font-medium">Loading tasks...</span>
                </div>
              </div>
            }>
              <Show when={todos().length > 0} fallback={
                <div class="text-center py-16 px-6">
                  <div class="inline-flex p-6 bg-amber-100/50 rounded-3xl mb-4">
                    <svg xmlns="http://www.w3.org/2000/svg" class="w-16 h-16 text-amber-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
                    </svg>
                  </div>
                  <div class="text-amber-900/70 text-xl font-semibold mb-2">All caught up!</div>
                  <div class="text-amber-800/50 text-sm">No tasks yet. New tasks will appear here.</div>
                </div>
              }>
              <div class="p-6 space-y-3">
                <For each={todos()}>
                  {(task) => (
                    <div class="task-item flex items-start gap-4 p-4 bg-white/60 backdrop-blur-sm rounded-2xl shadow-sm border border-amber-100/50 hover:shadow-md hover:bg-white/80 transition-all duration-200 group">
                        {/* Checkbox */}
                        <div class="mt-1">
                          <Show when={task.completed} fallback={
                            <div class="w-6 h-6 rounded-lg border-2 border-amber-300/70 bg-amber-50/30 group-hover:border-amber-400 transition-colors"></div>
                          }>
                            <div class="w-6 h-6 rounded-lg bg-gradient-to-br from-green-400 to-emerald-500 border-2 border-green-500 flex items-center justify-center shadow-sm">
                              <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4 text-white" viewBox="0 0 20 20" fill="currentColor">
                                <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd" />
                              </svg>
                            </div>
                          </Show>
                        </div>

                        {/* Task Text */}
                        <div class="flex-1 min-w-0">
                          <div class="flex items-start gap-2 mb-1">
                            <span class="px-2 py-0.5 text-xs font-bold text-amber-700 bg-amber-100/70 rounded-md border border-amber-200/50">
                              #{task.id}
                            </span>
                            <p class={`text-base flex-1 leading-relaxed ${task.completed ? 'line-through text-amber-900/40' : 'text-amber-950/90 font-medium'}`}>
                              {task.task_text}
                            </p>
                          </div>
                          <Show when={task.created_at}>
                            <div class="flex items-center gap-2 mt-2 text-xs text-amber-700/60">
                              <span class="px-2 py-0.5 bg-amber-100/50 rounded-full font-medium">
                                {task.username}
                              </span>
                              <span>•</span>
                              <span>{getMinutesAgo(task.created_at)}</span>
                            </div>
                          </Show>
                        </div>
                      </div>
                    )}
                  </For>
                </div>
              </Show>
            </Show>
          </div>
        </div>
      </div>
      </Show>
    </div>
  );
}

// Only render when used as standalone (for OBS browser sources)
if (document.getElementById('root')) {
  render(() => <TodosOverlay />, document.getElementById('root'));
}
