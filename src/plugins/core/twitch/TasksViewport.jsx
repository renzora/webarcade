import { createSignal, onMount, For, Show } from 'solid-js';
import twitchStore from './TwitchStore.jsx';
import { bridgeFetch } from '@/api/bridge.js';
import { IconChecklist, IconTrash, IconCheck, IconAlertCircle, IconPlus } from '@tabler/icons-solidjs';

// Helper function to format "time ago"
function formatTimeAgo(timestamp) {
  const now = Math.floor(Date.now() / 1000);
  const secondsAgo = now - timestamp;

  if (secondsAgo < 60) {
    return 'just now';
  }

  const minutesAgo = Math.floor(secondsAgo / 60);
  if (minutesAgo < 60) {
    return `${minutesAgo} ${minutesAgo === 1 ? 'minute' : 'minutes'} ago`;
  }

  const hoursAgo = Math.floor(minutesAgo / 60);
  if (hoursAgo < 24) {
    return `${hoursAgo} ${hoursAgo === 1 ? 'hour' : 'hours'} ago`;
  }

  const daysAgo = Math.floor(hoursAgo / 24);
  if (daysAgo < 7) {
    return `${daysAgo} ${daysAgo === 1 ? 'day' : 'days'} ago`;
  }

  const weeksAgo = Math.floor(daysAgo / 7);
  return `${weeksAgo} ${weeksAgo === 1 ? 'week' : 'weeks'} ago`;
}

export default function TasksViewport() {
  const [tasks, setTasks] = createSignal([]);
  const [loading, setLoading] = createSignal(true);
  const [selectedChannel, setSelectedChannel] = createSignal('');
  const [newTask, setNewTask] = createSignal('');
  const [status, setStatus] = createSignal({ status: 'disconnected', connected_channels: [] });
  const [currentTime, setCurrentTime] = createSignal(Math.floor(Date.now() / 1000));
  const [visibleCount, setVisibleCount] = createSignal(10);

  // Update current time every 30 seconds to refresh "time ago" display
  onMount(() => {
    const interval = setInterval(() => {
      setCurrentTime(Math.floor(Date.now() / 1000));
    }, 30000);

    return () => clearInterval(interval);
  });

  onMount(async () => {
    const currentStatus = await twitchStore.fetchStatus();
    if (currentStatus) {
      setStatus(currentStatus);
      if (currentStatus.connected_channels && currentStatus.connected_channels.length > 0) {
        setSelectedChannel(currentStatus.connected_channels[0]);
        await loadTasks(currentStatus.connected_channels[0]);
      }
    }
    setLoading(false);
  });

  const loadTasks = async (channel) => {
    if (!channel || channel.trim() === '') {
      console.log('Skipping task load - channel is empty');
      return;
    }

    try {
      setLoading(true);
      // Load all tasks for the channel (no username filter)
      const response = await bridgeFetch(`/database/todos?channel=${encodeURIComponent(channel)}`);
      const data = await response.json();
      setTasks(data);
    } catch (e) {
      console.error('Failed to load tasks:', e);
    } finally {
      setLoading(false);
    }
  };

  const handleChannelChange = async (channel) => {
    setSelectedChannel(channel);
    setNewTask(''); // Clear input when changing channels
    setVisibleCount(10); // Reset visible count when changing channels
    await loadTasks(channel);
  };

  const loadMoreTasks = () => {
    setVisibleCount((prev) => prev + 10);
  };

  // Get only the visible tasks
  const visibleTasks = () => tasks().slice(0, visibleCount());
  const hasMoreTasks = () => tasks().length > visibleCount();

  const addTask = async () => {
    const task = newTask().trim();
    if (!task || !selectedChannel()) return;

    try {
      // Get bot config to use bot username
      const config = await twitchStore.fetchConfig();
      if (!config || !config.bot_username) return;

      const response = await bridgeFetch('/database/todos/add', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          channel: selectedChannel(),
          username: config.bot_username,
          task,
        }),
      });

      if (response.ok) {
        setNewTask('');
        await loadTasks(selectedChannel());
      }
    } catch (e) {
      console.error('Failed to add task:', e);
    }
  };

  const completeTask = async (id, username) => {
    try {
      const response = await bridgeFetch('/database/todos/complete', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          channel: selectedChannel(),
          username: username,
          id,
        }),
      });

      if (response.ok) {
        await loadTasks(selectedChannel());
      }
    } catch (e) {
      console.error('Failed to complete task:', e);
    }
  };

  const deleteTask = async (id, username) => {
    try {
      const response = await bridgeFetch('/database/todos', {
        method: 'DELETE',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          channel: selectedChannel(),
          username: username,
          id,
        }),
      });

      if (response.ok) {
        await loadTasks(selectedChannel());
      }
    } catch (e) {
      console.error('Failed to delete task:', e);
    }
  };

  return (
    <div class="h-full flex flex-col bg-base-200">
      {/* Header */}
      <div class="flex items-center gap-3 bg-base-100 border-b border-base-300 px-4 py-3">
        <IconChecklist size={20} class="text-primary" />
        <h2 class="text-lg font-semibold">Channel Tasks</h2>

        <Show when={status().connected_channels.length > 0}>
          <select
            class="select select-bordered select-sm"
            value={selectedChannel()}
            onChange={(e) => handleChannelChange(e.target.value)}
          >
            {status().connected_channels.map((channel) => (
              <option value={channel}>#{channel}</option>
            ))}
          </select>
        </Show>

        <div class="badge badge-neutral ml-auto">
          {tasks().length} {tasks().length === 1 ? 'task' : 'tasks'}
        </div>
      </div>

      {/* Add Task */}
      <div class="p-4 bg-base-100 border-b border-base-300">
        <div class="flex gap-2">
          <input
            type="text"
            placeholder="Add a new task..."
            class="input input-bordered input-sm flex-1"
            value={newTask()}
            onInput={(e) => setNewTask(e.target.value)}
            onKeyPress={(e) => e.key === 'Enter' && addTask()}
          />
          <button
            class="btn btn-primary btn-sm gap-2"
            onClick={addTask}
            disabled={!newTask().trim() || !selectedChannel()}
          >
            <IconPlus size={16} />
            Add
          </button>
        </div>
      </div>

      {/* Tasks List */}
      <div class="flex-1 overflow-y-auto p-4">
        <Show
          when={!loading() && selectedChannel()}
          fallback={
            <div class="h-full flex items-center justify-center">
              <div class="text-center">
                <IconAlertCircle size={48} class="opacity-30 mx-auto" />
                <p class="text-sm text-base-content/60 mt-2">
                  {loading() ? 'Loading tasks...' : 'Select a channel'}
                </p>
              </div>
            </div>
          }
        >
          <Show
            when={tasks().length > 0}
            fallback={
              <div class="text-center">
                <IconChecklist size={48} class="opacity-30 mx-auto" />
                <p class="text-sm font-semibold mt-2">No tasks yet</p>
                <p class="text-xs text-base-content/60">
                  Add a task for the channel above
                </p>
              </div>
            }
          >
            <div class="space-y-2">
              <For each={visibleTasks()}>
                {(task) => (
                  <div class="card bg-base-100 shadow-sm hover:shadow-md transition-shadow">
                    <div class="card-body p-3">
                      <div class="flex items-center gap-3">
                        <button
                          class="btn btn-circle btn-sm btn-success btn-outline"
                          onClick={() => completeTask(task.id, task.username)}
                          title="Complete"
                        >
                          <IconCheck size={16} />
                        </button>

                        <div class="flex-1">
                          <div class="flex items-baseline gap-2">
                            <span class="text-xs font-semibold text-primary">@{task.username}</span>
                            <p class="text-sm">{task.task}</p>
                          </div>
                          <Show when={task.created_at}>
                            <p class="text-xs text-base-content/50 mt-1">
                              {formatTimeAgo(task.created_at)}
                            </p>
                          </Show>
                        </div>

                        <button
                          class="btn btn-circle btn-sm btn-ghost"
                          onClick={() => deleteTask(task.id, task.username)}
                          title="Delete"
                        >
                          <IconTrash size={16} />
                        </button>
                      </div>
                    </div>
                  </div>
                )}
              </For>

              {/* Load More Button */}
              <Show when={hasMoreTasks()}>
                <div class="text-center pt-2">
                  <button
                    class="btn btn-outline btn-sm"
                    onClick={loadMoreTasks}
                  >
                    Load More ({tasks().length - visibleCount()} remaining)
                  </button>
                </div>
              </Show>
            </div>
          </Show>
        </Show>
      </div>
    </div>
  );
}
