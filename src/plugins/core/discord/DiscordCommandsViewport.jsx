import { createSignal, createEffect, onCleanup, For, Show } from 'solid-js';
import { IconRobot, IconPlus, IconTrash, IconInfoCircle, IconEdit, IconRefresh } from '@tabler/icons-solidjs';

const BRIDGE_URL = '';

export default function DiscordCommandsViewport() {
  const [commands, setCommands] = createSignal([]);
  const [loading, setLoading] = createSignal(false);
  const [saving, setSaving] = createSignal(false);
  const [reloading, setReloading] = createSignal(false);

  // Form fields for new command
  const [newName, setNewName] = createSignal('');
  const [newAliases, setNewAliases] = createSignal('');
  const [newResponse, setNewResponse] = createSignal('');
  const [newDescription, setNewDescription] = createSignal('');
  const [newPermission, setNewPermission] = createSignal('Everyone');
  const [newCooldown, setNewCooldown] = createSignal(0);
  const [newEnabled, setNewEnabled] = createSignal(true);

  // Edit modal state
  const [editingCommand, setEditingCommand] = createSignal(null);
  const [editName, setEditName] = createSignal('');
  const [editAliases, setEditAliases] = createSignal('');
  const [editResponse, setEditResponse] = createSignal('');
  const [editDescription, setEditDescription] = createSignal('');
  const [editPermission, setEditPermission] = createSignal('Everyone');
  const [editCooldown, setEditCooldown] = createSignal(0);
  const [editEnabled, setEditEnabled] = createSignal(true);

  // Fetch commands
  const fetchCommands = async () => {
    try {
      setLoading(true);
      console.log('[DiscordCommands] Fetching commands');
      const response = await fetch(`${BRIDGE_URL}/discord/commands`);
      const data = await response.json();
      console.log('[DiscordCommands] Received commands:', data);
      if (data.success && data.data) {
        setCommands(data.data);
      }
    } catch (error) {
      console.error('[DiscordCommands] Failed to fetch commands:', error);
    } finally {
      setLoading(false);
    }
  };

  // Fetch commands on mount and every 10 seconds
  createEffect(() => {
    fetchCommands();
    const interval = setInterval(fetchCommands, 10000);
    onCleanup(() => clearInterval(interval));
  });

  const handleAddCommand = async (e) => {
    e.preventDefault();
    const name = newName().trim();
    const response = newResponse().trim();

    if (!name || !response) {
      alert('Please fill in at least command name and response');
      return;
    }

    // Parse aliases from comma-separated string
    const aliases = newAliases()
      .split(',')
      .map(a => a.trim())
      .filter(a => a.length > 0);

    try {
      setSaving(true);
      const res = await fetch(`${BRIDGE_URL}/discord/commands`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          name,
          aliases,
          response,
          description: newDescription().trim(),
          permission: newPermission(),
          cooldown: parseInt(newCooldown()),
          enabled: newEnabled()
        })
      });

      const data = await res.json();
      if (res.ok && data.success) {
        // Clear form
        setNewName('');
        setNewAliases('');
        setNewResponse('');
        setNewDescription('');
        setNewPermission('Everyone');
        setNewCooldown(0);
        setNewEnabled(true);

        // Refresh commands
        await fetchCommands();
      } else {
        alert(`Failed to add command: ${data.error || 'Unknown error'}`);
      }
    } catch (error) {
      alert(`Error: ${error.message}`);
    } finally {
      setSaving(false);
    }
  };

  const handleEditCommand = (cmd) => {
    setEditingCommand(cmd);
    setEditName(cmd.name);
    setEditAliases(cmd.aliases.join(', '));
    setEditResponse(cmd.response);
    setEditDescription(cmd.description);
    setEditPermission(cmd.permission);
    setEditCooldown(cmd.cooldown);
    setEditEnabled(cmd.enabled);
    document.getElementById('edit_modal').showModal();
  };

  const handleSaveEdit = async (e) => {
    e.preventDefault();
    const cmd = editingCommand();
    if (!cmd) return;

    // Parse aliases from comma-separated string
    const aliases = editAliases()
      .split(',')
      .map(a => a.trim())
      .filter(a => a.length > 0);

    try {
      setSaving(true);
      const res = await fetch(`${BRIDGE_URL}/discord/commands/update`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          id: cmd.id,
          name: editName().trim(),
          aliases,
          response: editResponse().trim(),
          description: editDescription().trim(),
          permission: editPermission(),
          cooldown: parseInt(editCooldown()),
          enabled: editEnabled()
        })
      });

      const data = await res.json();
      if (res.ok && data.success) {
        document.getElementById('edit_modal').close();
        await fetchCommands();
      } else {
        alert(`Failed to update command: ${data.error || 'Unknown error'}`);
      }
    } catch (error) {
      alert(`Error: ${error.message}`);
    } finally {
      setSaving(false);
    }
  };

  const handleDeleteCommand = async (cmd) => {
    if (!confirm(`Delete command "${cmd.name}"?`)) return;

    try {
      const res = await fetch(`${BRIDGE_URL}/discord/commands/delete`, {
        method: 'DELETE',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ id: cmd.id })
      });

      const data = await res.json();
      if (res.ok && data.success) {
        await fetchCommands();
      } else {
        alert(`Failed to delete command: ${data.error || 'Unknown error'}`);
      }
    } catch (error) {
      alert(`Error: ${error.message}`);
    }
  };

  const handleToggleEnabled = async (cmd) => {
    try {
      const res = await fetch(`${BRIDGE_URL}/discord/commands/toggle`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          id: cmd.id,
          enabled: !cmd.enabled
        })
      });

      const data = await res.json();
      if (res.ok && data.success) {
        await fetchCommands();
      } else {
        alert(`Failed to toggle command: ${data.error || 'Unknown error'}`);
      }
    } catch (error) {
      alert(`Error: ${error.message}`);
    }
  };

  const handleReloadCommands = async () => {
    try {
      setReloading(true);
      const res = await fetch(`${BRIDGE_URL}/discord/commands/reload`, {
        method: 'POST'
      });

      const data = await res.json();
      if (res.ok && data.success) {
        await fetchCommands();
        alert('Commands reloaded successfully!');
      } else {
        alert(`Failed to reload commands: ${data.error || 'Unknown error'}`);
      }
    } catch (error) {
      alert(`Error: ${error.message}`);
    } finally {
      setReloading(false);
    }
  };

  return (
    <div class="h-full overflow-y-auto bg-gradient-to-br from-base-300 to-base-200 p-6">
      <div class="max-w-5xl mx-auto space-y-6">
        {/* Header */}
        <div class="card bg-base-100 shadow-xl">
          <div class="card-body">
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-3">
                <div class="p-3 bg-primary/20 rounded-lg">
                  <IconRobot size={32} class="text-primary" />
                </div>
                <div>
                  <h2 class="card-title text-2xl">Discord Commands</h2>
                  <p class="text-sm text-base-content/60">Manage custom Discord bot commands</p>
                </div>
              </div>
              <button
                class={`btn btn-secondary gap-2 ${reloading() ? 'loading' : ''}`}
                onClick={handleReloadCommands}
                disabled={reloading()}
              >
                {!reloading() && <IconRefresh size={20} />}
                {reloading() ? 'Reloading...' : 'Reload Commands'}
              </button>
            </div>
          </div>
        </div>

        {/* Info Card */}
        <div class="alert alert-info">
          <IconInfoCircle size={24} />
          <div>
            <h3 class="font-bold">Command Permissions:</h3>
            <p class="text-sm">
              <strong>Everyone</strong> - Anyone can use •
              <strong>Admin</strong> - Server admins only •
              <strong>Owner</strong> - Server owner only •
              <strong>Role:RoleName</strong> - Users with specific role
            </p>
          </div>
        </div>

        {/* Add Command Form */}
        <div class="card bg-base-100 shadow-xl">
          <div class="card-body">
            <h3 class="card-title">Add New Command</h3>
            <form onSubmit={handleAddCommand} class="space-y-4">
              <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                <div class="form-control">
                  <label class="label">
                    <span class="label-text">Command Name (without !)</span>
                  </label>
                  <input
                    type="text"
                    placeholder="welcome"
                    class="input input-bordered"
                    value={newName()}
                    onInput={(e) => setNewName(e.target.value)}
                    required
                  />
                </div>
                <div class="form-control">
                  <label class="label">
                    <span class="label-text">Aliases (comma-separated)</span>
                  </label>
                  <input
                    type="text"
                    placeholder="hi, hello"
                    class="input input-bordered"
                    value={newAliases()}
                    onInput={(e) => setNewAliases(e.target.value)}
                  />
                </div>
              </div>

              <div class="form-control">
                <label class="label">
                  <span class="label-text">Response Text</span>
                </label>
                <textarea
                  placeholder="Welcome to our Discord server!"
                  class="textarea textarea-bordered h-24"
                  value={newResponse()}
                  onInput={(e) => setNewResponse(e.target.value)}
                  required
                />
              </div>

              <div class="form-control">
                <label class="label">
                  <span class="label-text">Description</span>
                </label>
                <input
                  type="text"
                  placeholder="Welcomes users to the server"
                  class="input input-bordered"
                  value={newDescription()}
                  onInput={(e) => setNewDescription(e.target.value)}
                />
              </div>

              <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
                <div class="form-control">
                  <label class="label">
                    <span class="label-text">Permission Level</span>
                  </label>
                  <select
                    class="select select-bordered"
                    value={newPermission()}
                    onChange={(e) => setNewPermission(e.target.value)}
                  >
                    <option value="Everyone">Everyone</option>
                    <option value="Admin">Admin</option>
                    <option value="Owner">Owner</option>
                  </select>
                </div>
                <div class="form-control">
                  <label class="label">
                    <span class="label-text">Cooldown (seconds)</span>
                  </label>
                  <input
                    type="number"
                    min="0"
                    class="input input-bordered"
                    value={newCooldown()}
                    onInput={(e) => setNewCooldown(e.target.value)}
                  />
                </div>
                <div class="form-control">
                  <label class="label cursor-pointer">
                    <span class="label-text">Enabled</span>
                    <input
                      type="checkbox"
                      class="toggle toggle-success"
                      checked={newEnabled()}
                      onChange={(e) => setNewEnabled(e.target.checked)}
                    />
                  </label>
                </div>
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
                    <div class={`p-4 rounded-lg ${cmd.enabled ? 'bg-base-200' : 'bg-base-200/50 opacity-60'}`}>
                      <div class="flex items-start justify-between gap-4">
                        <div class="flex-1 min-w-0">
                          <div class="flex items-center gap-2 mb-2 flex-wrap">
                            <code class="text-primary font-bold">!{cmd.name}</code>
                            <Show when={cmd.aliases && cmd.aliases.length > 0}>
                              <span class="badge badge-sm">
                                Aliases: {cmd.aliases.join(', ')}
                              </span>
                            </Show>
                            <Show when={cmd.cooldown > 0}>
                              <span class="badge badge-warning badge-sm">
                                {cmd.cooldown}s cooldown
                              </span>
                            </Show>
                            <Show when={cmd.permission !== 'Everyone'}>
                              <span class="badge badge-error badge-sm">
                                {cmd.permission}
                              </span>
                            </Show>
                            <Show when={!cmd.enabled}>
                              <span class="badge badge-ghost badge-sm">Disabled</span>
                            </Show>
                          </div>
                          <Show when={cmd.description}>
                            <p class="text-sm text-base-content/70 mb-2">{cmd.description}</p>
                          </Show>
                          <p class="text-sm text-base-content/80 whitespace-pre-wrap break-words font-mono bg-base-300 p-2 rounded">
                            {cmd.response}
                          </p>
                        </div>
                        <div class="flex gap-2 flex-shrink-0">
                          <button
                            class={`btn btn-sm ${cmd.enabled ? 'btn-warning' : 'btn-success'}`}
                            onClick={() => handleToggleEnabled(cmd)}
                            title={cmd.enabled ? 'Disable' : 'Enable'}
                          >
                            {cmd.enabled ? 'Disable' : 'Enable'}
                          </button>
                          <button
                            class="btn btn-info btn-sm gap-2"
                            onClick={() => handleEditCommand(cmd)}
                          >
                            <IconEdit size={16} />
                            Edit
                          </button>
                          <button
                            class="btn btn-error btn-sm gap-2"
                            onClick={() => handleDeleteCommand(cmd)}
                          >
                            <IconTrash size={16} />
                            Delete
                          </button>
                        </div>
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
        <div class="modal-box max-w-2xl">
          <h3 class="font-bold text-lg mb-4">Edit Command: !{editName()}</h3>
          <form onSubmit={handleSaveEdit} class="space-y-4">
            <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
              <div class="form-control">
                <label class="label">
                  <span class="label-text">Command Name</span>
                </label>
                <input
                  type="text"
                  class="input input-bordered"
                  value={editName()}
                  onInput={(e) => setEditName(e.target.value)}
                  required
                />
              </div>
              <div class="form-control">
                <label class="label">
                  <span class="label-text">Aliases (comma-separated)</span>
                </label>
                <input
                  type="text"
                  class="input input-bordered"
                  value={editAliases()}
                  onInput={(e) => setEditAliases(e.target.value)}
                />
              </div>
            </div>

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
              <label class="label">
                <span class="label-text">Description</span>
              </label>
              <input
                type="text"
                class="input input-bordered"
                value={editDescription()}
                onInput={(e) => setEditDescription(e.target.value)}
              />
            </div>

            <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
              <div class="form-control">
                <label class="label">
                  <span class="label-text">Permission Level</span>
                </label>
                <select
                  class="select select-bordered"
                  value={editPermission()}
                  onChange={(e) => setEditPermission(e.target.value)}
                >
                  <option value="Everyone">Everyone</option>
                  <option value="Admin">Admin</option>
                  <option value="Owner">Owner</option>
                </select>
              </div>
              <div class="form-control">
                <label class="label">
                  <span class="label-text">Cooldown (seconds)</span>
                </label>
                <input
                  type="number"
                  min="0"
                  class="input input-bordered"
                  value={editCooldown()}
                  onInput={(e) => setEditCooldown(e.target.value)}
                />
              </div>
              <div class="form-control">
                <label class="label cursor-pointer">
                  <span class="label-text">Enabled</span>
                  <input
                    type="checkbox"
                    class="toggle toggle-success"
                    checked={editEnabled()}
                    onChange={(e) => setEditEnabled(e.target.checked)}
                  />
                </label>
              </div>
            </div>

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
