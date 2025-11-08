import { createSignal, onMount, For } from 'solid-js';

export default function DiscordViewport() {
  const [config, setConfig] = createSignal({
    bot_token: '',
    command_prefix: '!',
    enabled: false
  });
  const [status, setStatus] = createSignal({
    is_running: false,
    command_prefix: '!'
  });
  const [commands, setCommands] = createSignal([]);
  const [editingCommand, setEditingCommand] = createSignal(null);
  const [newCommand, setNewCommand] = createSignal({
    name: '',
    description: '',
    response: '',
    enabled: true
  });

  onMount(async () => {
    await loadConfig();
    await loadStatus();
    await loadCommands();
  });

  async function loadConfig() {
    try {
      const response = await fetch('http://localhost:3001/discord/config');
      const data = await response.json();
      if (data.success) {
        setConfig(data.data);
      }
    } catch (error) {
      console.error('Failed to load config:', error);
    }
  }

  async function loadStatus() {
    try {
      const response = await fetch('http://localhost:3001/discord/status');
      const data = await response.json();
      if (data.success) {
        setStatus(data.data);
      }
    } catch (error) {
      console.error('Failed to load status:', error);
    }
  }

  async function loadCommands() {
    try {
      const response = await fetch('http://localhost:3001/discord/commands');
      const data = await response.json();
      if (data.success) {
        setCommands(data.data);
      }
    } catch (error) {
      console.error('Failed to load commands:', error);
    }
  }

  async function saveConfig() {
    try {
      const response = await fetch('http://localhost:3001/discord/config', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(config())
      });
      const data = await response.json();
      if (data.success) {
        alert('Configuration saved successfully!');
        await loadStatus();
      } else {
        alert('Failed to save configuration');
      }
    } catch (error) {
      console.error('Failed to save config:', error);
      alert('Failed to save configuration');
    }
  }

  async function startBot() {
    try {
      const response = await fetch('http://localhost:3001/discord/start', {
        method: 'POST'
      });
      const data = await response.json();
      if (data.success) {
        alert('Bot started successfully!');
        await loadStatus();
      } else {
        alert('Failed to start bot');
      }
    } catch (error) {
      console.error('Failed to start bot:', error);
      alert('Failed to start bot');
    }
  }

  async function stopBot() {
    try {
      const response = await fetch('http://localhost:3001/discord/stop', {
        method: 'POST'
      });
      const data = await response.json();
      if (data.success) {
        alert('Bot stopped successfully!');
        await loadStatus();
      } else {
        alert('Failed to stop bot');
      }
    } catch (error) {
      console.error('Failed to stop bot:', error);
      alert('Failed to stop bot');
    }
  }

  async function saveCommand(command) {
    try {
      const response = await fetch('http://localhost:3001/discord/commands', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(command)
      });
      const data = await response.json();
      if (data.success) {
        await loadCommands();
        setEditingCommand(null);
        setNewCommand({
          name: '',
          description: '',
          response: '',
          enabled: true
        });
      } else {
        alert('Failed to save command');
      }
    } catch (error) {
      console.error('Failed to save command:', error);
      alert('Failed to save command');
    }
  }

  async function deleteCommand(id) {
    if (!confirm('Are you sure you want to delete this command?')) return;

    try {
      const response = await fetch(`http://localhost:3001/discord/commands/${id}`, {
        method: 'DELETE'
      });
      const data = await response.json();
      if (data.success) {
        await loadCommands();
      } else {
        alert('Failed to delete command');
      }
    } catch (error) {
      console.error('Failed to delete command:', error);
      alert('Failed to delete command');
    }
  }

  return (
    <div class="p-6 space-y-6">
      {/* Bot Status */}
      <div class="bg-base-200 rounded-lg p-4">
        <h2 class="text-xl font-bold mb-4">Discord Bot Status</h2>
        <div class="flex items-center gap-4">
          <div class="flex items-center gap-2">
            <div class={`w-3 h-3 rounded-full ${status().is_running ? 'bg-success' : 'bg-error'}`}></div>
            <span class="font-semibold">
              {status().is_running ? 'Running' : 'Stopped'}
            </span>
          </div>
          {status().is_running && (
            <button class="btn btn-sm btn-error" onClick={stopBot}>
              Stop Bot
            </button>
          )}
          {!status().is_running && (
            <button class="btn btn-sm btn-success" onClick={startBot}>
              Start Bot
            </button>
          )}
        </div>
      </div>

      {/* Bot Configuration */}
      <div class="bg-base-200 rounded-lg p-4">
        <h2 class="text-xl font-bold mb-4">Bot Configuration</h2>
        <div class="space-y-4">
          <div class="form-control">
            <label class="label">
              <span class="label-text">Bot Token</span>
            </label>
            <input
              type="password"
              class="input input-bordered"
              value={config().bot_token || ''}
              onInput={(e) => setConfig({ ...config(), bot_token: e.target.value })}
              placeholder="Enter your Discord bot token"
            />
          </div>

          <div class="form-control">
            <label class="label">
              <span class="label-text">Command Prefix</span>
            </label>
            <input
              type="text"
              class="input input-bordered"
              value={config().command_prefix}
              onInput={(e) => setConfig({ ...config(), command_prefix: e.target.value })}
              placeholder="!"
            />
          </div>

          <div class="form-control">
            <label class="label cursor-pointer">
              <span class="label-text">Auto-start bot on launch</span>
              <input
                type="checkbox"
                class="toggle toggle-primary"
                checked={config().enabled}
                onChange={(e) => setConfig({ ...config(), enabled: e.target.checked })}
              />
            </label>
          </div>

          <button class="btn btn-primary" onClick={saveConfig}>
            Save Configuration
          </button>
        </div>
      </div>

      {/* Custom Commands */}
      <div class="bg-base-200 rounded-lg p-4">
        <h2 class="text-xl font-bold mb-4">Custom Commands</h2>

        {/* Add New Command */}
        <div class="bg-base-300 rounded-lg p-4 mb-4">
          <h3 class="font-semibold mb-3">Add New Command</h3>
          <div class="space-y-3">
            <input
              type="text"
              class="input input-bordered input-sm w-full"
              value={newCommand().name}
              onInput={(e) => setNewCommand({ ...newCommand(), name: e.target.value })}
              placeholder="Command name (without prefix)"
            />
            <input
              type="text"
              class="input input-bordered input-sm w-full"
              value={newCommand().description}
              onInput={(e) => setNewCommand({ ...newCommand(), description: e.target.value })}
              placeholder="Description"
            />
            <textarea
              class="textarea textarea-bordered w-full"
              value={newCommand().response}
              onInput={(e) => setNewCommand({ ...newCommand(), response: e.target.value })}
              placeholder="Response"
              rows="3"
            ></textarea>
            <button
              class="btn btn-sm btn-primary"
              onClick={() => saveCommand(newCommand())}
              disabled={!newCommand().name || !newCommand().response}
            >
              Add Command
            </button>
          </div>
        </div>

        {/* Commands List */}
        <div class="space-y-2">
          <For each={commands()}>
            {(command) => (
              <div class="bg-base-300 rounded-lg p-3">
                {editingCommand()?.id === command.id ? (
                  <div class="space-y-2">
                    <input
                      type="text"
                      class="input input-bordered input-sm w-full"
                      value={editingCommand().name}
                      onInput={(e) => setEditingCommand({ ...editingCommand(), name: e.target.value })}
                    />
                    <input
                      type="text"
                      class="input input-bordered input-sm w-full"
                      value={editingCommand().description || ''}
                      onInput={(e) => setEditingCommand({ ...editingCommand(), description: e.target.value })}
                    />
                    <textarea
                      class="textarea textarea-bordered w-full"
                      value={editingCommand().response}
                      onInput={(e) => setEditingCommand({ ...editingCommand(), response: e.target.value })}
                      rows="3"
                    ></textarea>
                    <div class="flex gap-2">
                      <button
                        class="btn btn-sm btn-success"
                        onClick={() => saveCommand(editingCommand())}
                      >
                        Save
                      </button>
                      <button
                        class="btn btn-sm"
                        onClick={() => setEditingCommand(null)}
                      >
                        Cancel
                      </button>
                    </div>
                  </div>
                ) : (
                  <div class="flex items-start justify-between">
                    <div class="flex-1">
                      <div class="font-semibold">
                        {status().command_prefix}{command.name}
                      </div>
                      {command.description && (
                        <div class="text-sm opacity-70">{command.description}</div>
                      )}
                      <div class="text-sm mt-1">{command.response}</div>
                    </div>
                    <div class="flex gap-2">
                      <button
                        class="btn btn-sm btn-ghost"
                        onClick={() => setEditingCommand(command)}
                      >
                        Edit
                      </button>
                      <button
                        class="btn btn-sm btn-ghost text-error"
                        onClick={() => deleteCommand(command.id)}
                      >
                        Delete
                      </button>
                    </div>
                  </div>
                )}
              </div>
            )}
          </For>
        </div>

        {commands().length === 0 && (
          <div class="text-center py-8 opacity-50">
            No custom commands yet. Add one above to get started!
          </div>
        )}
      </div>

      {/* Built-in Commands */}
      <div class="bg-base-200 rounded-lg p-4">
        <h2 class="text-xl font-bold mb-4">Built-in Commands</h2>
        <div class="space-y-2">
          <div class="flex items-center gap-3">
            <code class="bg-base-300 px-2 py-1 rounded">{status().command_prefix}ping</code>
            <span class="text-sm opacity-70">Check if bot is responsive</span>
          </div>
          <div class="flex items-center gap-3">
            <code class="bg-base-300 px-2 py-1 rounded">{status().command_prefix}help</code>
            <span class="text-sm opacity-70">Show available commands</span>
          </div>
          <div class="flex items-center gap-3">
            <code class="bg-base-300 px-2 py-1 rounded">{status().command_prefix}info</code>
            <span class="text-sm opacity-70">Show bot information</span>
          </div>
        </div>
      </div>
    </div>
  );
}
