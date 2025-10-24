import { createSignal, createEffect, onCleanup, For, Show } from 'solid-js';
import { IconTerminal2, IconPlus, IconTrash, IconInfoCircle, IconEdit } from '@tabler/icons-solidjs';

const BRIDGE_URL = '';

export default function TextCommandsViewport() {
  const [commands, setCommands] = createSignal([]);
  const [channel, setChannel] = createSignal('');
  const [newCommand, setNewCommand] = createSignal('');
  const [newResponse, setNewResponse] = createSignal('');
  const [loading, setLoading] = createSignal(false);
  const [saving, setSaving] = createSignal(false);
  const [editingCommand, setEditingCommand] = createSignal(null);
  const [editResponse, setEditResponse] = createSignal('');
  const [editAutoPost, setEditAutoPost] = createSignal(false);
  const [editInterval, setEditInterval] = createSignal(10);

  // Fetch Twitch config to get channel
  createEffect(() => {
    const fetchConfig = async () => {
      try {
        const response = await fetch(`${BRIDGE_URL}/twitch/config`);
        const config = await response.json();
        if (config.channels && config.channels.length > 0) {
          setChannel(config.channels[0]);
        }
      } catch (error) {
        console.error('Failed to fetch config:', error);
      }
    };

    fetchConfig();
  });

  // Fetch text commands when channel is available
  createEffect(() => {
    const ch = channel();
    if (!ch) {
      console.log('[TextCommands] No channel set yet');
      return;
    }

    const fetchCommands = async () => {
      try {
        setLoading(true);
        console.log('[TextCommands] Fetching commands for channel:', ch);
        const response = await fetch(`${BRIDGE_URL}/twitch/text-commands?channel=${ch}`);
        const data = await response.json();
        console.log('[TextCommands] Received commands:', data);
        setCommands(data);
      } catch (error) {
        console.error('[TextCommands] Failed to fetch commands:', error);
      } finally {
        setLoading(false);
      }
    };

    fetchCommands();
    const interval = setInterval(fetchCommands, 5000);
    onCleanup(() => clearInterval(interval));
  });

  const handleAddCommand = async (e) => {
    e.preventDefault();
    const ch = channel();
    const cmd = newCommand().trim();
    const resp = newResponse().trim();

    if (!cmd || !resp) {
      alert('Please fill in both command name and response');
      return;
    }

    try {
      setSaving(true);
      const response = await fetch(`${BRIDGE_URL}/twitch/text-commands/add`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          channel: ch,
          command: cmd,
          response: resp
        })
      });

      if (response.ok) {
        setNewCommand('');
        setNewResponse('');
        // Refresh commands
        const refreshResponse = await fetch(`${BRIDGE_URL}/twitch/text-commands?channel=${ch}`);
        const data = await refreshResponse.json();
        setCommands(data);
      } else {
        const error = await response.text();
        alert(`Failed to add command: ${error}`);
      }
    } catch (error) {
      alert(`Error: ${error.message}`);
    } finally {
      setSaving(false);
    }
  };

  const handleEditCommand = (cmd) => {
    setEditingCommand(cmd.command);
    setEditResponse(cmd.response);
    setEditAutoPost(cmd.auto_post || false);
    setEditInterval(cmd.interval_minutes || 10);
    document.getElementById('edit_modal').showModal();
  };

  const handleSaveEdit = async (e) => {
    e.preventDefault();
    const ch = channel();
    const cmd = editingCommand();

    try {
      setSaving(true);
      const response = await fetch(`${BRIDGE_URL}/twitch/text-commands/edit`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          channel: ch,
          command: cmd,
          response: editResponse(),
          auto_post: editAutoPost(),
          interval_minutes: editInterval()
        })
      });

      if (response.ok) {
        document.getElementById('edit_modal').close();
        // Refresh commands
        const refreshResponse = await fetch(`${BRIDGE_URL}/twitch/text-commands?channel=${ch}`);
        const data = await refreshResponse.json();
        setCommands(data);
      } else {
        const error = await response.text();
        alert(`Failed to update command: ${error}`);
      }
    } catch (error) {
      alert(`Error: ${error.message}`);
    } finally {
      setSaving(false);
    }
  };

  const handleDeleteCommand = async (cmd) => {
    const ch = channel();
    if (!confirm(`Delete command "${cmd}"?`)) return;

    try {
      const response = await fetch(`${BRIDGE_URL}/twitch/text-commands`, {
        method: 'DELETE',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          channel: ch,
          command: cmd
        })
      });

      if (response.ok) {
        // Refresh commands
        const refreshResponse = await fetch(`${BRIDGE_URL}/twitch/text-commands?channel=${ch}`);
        const data = await refreshResponse.json();
        setCommands(data);
      } else {
        const error = await response.text();
        alert(`Failed to delete command: ${error}`);
      }
    } catch (error) {
      alert(`Error: ${error.message}`);
    }
  };

  return (
    <div class="h-full overflow-y-auto bg-gradient-to-br from-base-300 to-base-200 p-6">
      <div class="max-w-4xl mx-auto space-y-6">
        {/* Header */}
        <div class="card bg-base-100 shadow-xl">
          <div class="card-body">
            <div class="flex items-center gap-3">
              <div class="p-3 bg-primary/20 rounded-lg">
                <IconTerminal2 size={32} class="text-primary" />
              </div>
              <div>
                <h2 class="card-title text-2xl">Text Commands</h2>
                <p class="text-sm text-base-content/60">Create custom chat commands with dynamic variables</p>
              </div>
            </div>
          </div>
        </div>

        {/* Info Card */}
        <div class="alert alert-info">
          <IconInfoCircle size={24} />
          <div>
            <h3 class="font-bold">Available Variables:</h3>
            <p class="text-sm">
              <code>{'{username}'}</code> - User who triggered the command •
              <code>{'{displayname}'}</code> - User's display name •
              <code>{'{channel}'}</code> - Channel name •
              <code>{'{args}'}</code> - All arguments •
              <code>{'{count}'}</code> - Number of arguments
            </p>
          </div>
        </div>

        {/* Add Command Form */}
        <div class="card bg-base-100 shadow-xl">
          <div class="card-body">
            <h3 class="card-title">Add New Command</h3>
            <form onSubmit={handleAddCommand} class="space-y-4">
              <div class="form-control">
                <label class="label">
                  <span class="label-text">Command Name (without !)</span>
                </label>
                <input
                  type="text"
                  placeholder="discord"
                  class="input input-bordered"
                  value={newCommand()}
                  onInput={(e) => setNewCommand(e.target.value)}
                  required
                />
              </div>
              <div class="form-control">
                <label class="label">
                  <span class="label-text">Response Text</span>
                </label>
                <textarea
                  placeholder="Join our Discord: https://discord.gg/example"
                  class="textarea textarea-bordered h-24"
                  value={newResponse()}
                  onInput={(e) => setNewResponse(e.target.value)}
                  required
                />
                <label class="label">
                  <span class="label-text-alt">Example: "Hey {'{username}'}, welcome to the stream!"</span>
                </label>
              </div>
              <button
                type="submit"
                class={`btn btn-primary gap-2 ${saving() ? 'loading' : ''}`}
                disabled={saving()}
              >
                {!saving() && <IconPlus size={20} />}
                {saving() ? 'Adding...' : 'Add Command'}
              </button>
            </form>
          </div>
        </div>

        {/* Commands List */}
        <div class="card bg-base-100 shadow-xl">
          <div class="card-body">
            <h3 class="card-title">Your Commands</h3>
            <Show when={loading()}>
              <div class="flex justify-center py-8">
                <span class="loading loading-spinner loading-lg"></span>
              </div>
            </Show>
            <Show when={!loading() && commands().length === 0}>
              <div class="text-center py-8 text-base-content/60">
                No commands yet. Add your first command above!
              </div>
            </Show>
            <Show when={!loading() && commands().length > 0}>
              <div class="space-y-2">
                <For each={commands()}>
                  {(cmd) => (
                    <div class="flex items-start gap-3 p-4 bg-base-200 rounded-lg">
                      <div class="flex-1 min-w-0">
                        <div class="flex items-center gap-2 mb-2">
                          <code class="text-primary font-bold">!{cmd.command}</code>
                          <Show when={cmd.auto_post}>
                            <span class="badge badge-success badge-sm">Auto-post every {cmd.interval_minutes}m</span>
                          </Show>
                        </div>
                        <p class="text-sm text-base-content/80 whitespace-pre-wrap break-words">{cmd.response}</p>
                      </div>
                      <div class="flex gap-2 flex-shrink-0">
                        <button
                          class="btn btn-info btn-sm gap-2"
                          onClick={() => handleEditCommand(cmd)}
                        >
                          <IconEdit size={16} />
                          Edit
                        </button>
                        <button
                          class="btn btn-error btn-sm gap-2"
                          onClick={() => handleDeleteCommand(cmd.command)}
                        >
                          <IconTrash size={16} />
                          Delete
                        </button>
                      </div>
                    </div>
                  )}
                </For>
              </div>
            </Show>
          </div>
        </div>
      </div>

      {/* Edit Modal */}
      <dialog id="edit_modal" class="modal">
        <div class="modal-box">
          <h3 class="font-bold text-lg mb-4">Edit Command: !{editingCommand()}</h3>
          <form onSubmit={handleSaveEdit} class="space-y-4">
            <div class="form-control">
              <label class="label">
                <span class="label-text">Response Text</span>
              </label>
              <textarea
                class="textarea textarea-bordered h-24"
                value={editResponse()}
                onInput={(e) => setEditResponse(e.target.value)}
                required
              />
            </div>

            <div class="form-control">
              <label class="label cursor-pointer">
                <span class="label-text">Auto-post this command</span>
                <input
                  type="checkbox"
                  class="toggle toggle-success"
                  checked={editAutoPost()}
                  onChange={(e) => setEditAutoPost(e.target.checked)}
                />
              </label>
            </div>

            <Show when={editAutoPost()}>
              <div class="form-control">
                <label class="label">
                  <span class="label-text">Post every</span>
                </label>
                <select
                  class="select select-bordered w-full"
                  value={editInterval()}
                  onChange={(e) => setEditInterval(parseInt(e.target.value))}
                >
                  <option value={5}>5 minutes</option>
                  <option value={10}>10 minutes</option>
                  <option value={15}>15 minutes</option>
                  <option value={20}>20 minutes</option>
                  <option value={30}>30 minutes</option>
                  <option value={60}>1 hour</option>
                </select>
                <label class="label">
                  <span class="label-text-alt">Command will be posted automatically every {editInterval()} minutes while streaming</span>
                </label>
              </div>
            </Show>

            <div class="modal-action">
              <button
                type="button"
                class="btn"
                onClick={() => document.getElementById('edit_modal').close()}
              >
                Cancel
              </button>
              <button
                type="submit"
                class={`btn btn-primary ${saving() ? 'loading' : ''}`}
                disabled={saving()}
              >
                {saving() ? 'Saving...' : 'Save Changes'}
              </button>
            </div>
          </form>
        </div>
        <form method="dialog" class="modal-backdrop">
          <button>close</button>
        </form>
      </dialog>
    </div>
  );
}
