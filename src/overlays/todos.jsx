import { render } from 'solid-js/web';
import { createSignal, createEffect, onCleanup, For, Show } from 'solid-js';
import '@/index.css';
import { BRIDGE_API } from '@/api/bridge';

const BRIDGE_URL = BRIDGE_API;
const REFRESH_INTERVAL = 5000; // Refresh every 5 seconds

function TodosOverlay() {
  const [todos, setTodos] = createSignal([]);
  const [isLoading, setIsLoading] = createSignal(true);

  let intervalId = null;

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

  // Fetch todos on mount and set up interval
  createEffect(() => {
    fetchTodos();
    intervalId = setInterval(fetchTodos, REFRESH_INTERVAL);

    onCleanup(() => {
      if (intervalId) {
        clearInterval(intervalId);
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

  return (
    <div class="fixed inset-0 pointer-events-none overflow-hidden font-sans">
      {/* Todos Panel */}
      <div class="absolute top-0 left-0 w-full h-full overflow-hidden">
        <div class="bg-black backdrop-blur-xl shadow-2xl overflow-hidden h-full flex flex-col">
          {/* Header */}
          <div class="px-6 py-4 border-b border-white/10">
            <h2 class="text-2xl font-bold text-white flex items-center gap-2">
              <svg xmlns="http://www.w3.org/2000/svg" class="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2" />
              </svg>
              Community Tasks
            </h2>
            <p class="text-sm text-white/60 mt-1">
              {todos().length} task{todos().length !== 1 ? 's' : ''}
            </p>
          </div>

          {/* Content */}
          <div class="flex-1 overflow-hidden">
            <Show when={!isLoading()} fallback={
              <div class="flex items-center justify-center py-12">
                <span class="loading loading-spinner loading-lg text-primary"></span>
              </div>
            }>
              <Show when={todos().length > 0} fallback={
                <div class="text-center py-12 px-6">
                  <div class="text-white/40 text-lg">No tasks yet</div>
                  <div class="text-white/30 text-sm mt-2">Tasks will appear here when added</div>
                </div>
              }>
                <div class="p-4 space-y-2">
                  <For each={todos()}>
                    {(task) => (
                      <div class="flex items-start gap-2 group">
                        {/* Checkbox */}
                        <div class="mt-0.5">
                          <Show when={task.completed} fallback={
                            <div class="w-5 h-5 rounded border-2 border-white/30"></div>
                          }>
                            <div class="w-5 h-5 rounded bg-success border-2 border-success flex items-center justify-center">
                              <svg xmlns="http://www.w3.org/2000/svg" class="w-3 h-3 text-white" viewBox="0 0 20 20" fill="currentColor">
                                <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd" />
                              </svg>
                            </div>
                          </Show>
                        </div>

                        {/* Task Text */}
                        <div class="flex-1 min-w-0">
                          <p class={`text-sm ${task.completed ? 'line-through text-white/40' : 'text-white/90'}`}>
                            <span class="text-white/40 mr-2">#{task.id}</span>
                            {task.task_text}
                          </p>
                          <Show when={task.created_at}>
                            <p class="text-xs text-white/30 mt-1">
                              {task.username} • {getMinutesAgo(task.created_at)}
                            </p>
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
    </div>
  );
}

render(() => <TodosOverlay />, document.getElementById('root'));
