import { createSignal, createEffect, For, Show } from 'solid-js';
import { IconPlus, IconTrash, IconEdit, IconDeviceDesktop, IconMicrophone, IconSettings, IconCheck, IconX, IconBrandAmazon } from '@tabler/icons-solidjs';

const BRIDGE_URL = 'http://localhost:3001';

export default function AlexaViewport() {
  const [commands, setCommands] = createSignal([]);
  const [config, setConfig] = createSignal({
    obs_host: 'localhost',
    obs_port: 4455,
    obs_password: '',
    skill_id: '',
    enabled: false,
  });
  const [obsScenes, setObsScenes] = createSignal([]);
  const [obsStatus, setObsStatus] = createSignal({ connected: false });
  const [loading, setLoading] = createSignal(false);
  const [showCommandModal, setShowCommandModal] = createSignal(false);
  const [editingCommand, setEditingCommand] = createSignal(null);
  const [formData, setFormData] = createSignal({
    id: 0,
    name: '',
    intent_name: '',
    action_type: 'obs_scene',
    action_value: '',
    response_text: '',
    enabled: true,
  });

  // Fetch commands
  const fetchCommands = async () => {
    try {
      const response = await fetch(`${BRIDGE_URL}/alexa/commands`);
      const data = await response.json();
      if (data.success && data.content) {
        const commands = typeof data.content === 'string' ? JSON.parse(data.content) : data.content;
        setCommands(commands || []);
      }
    } catch (error) {
      console.error('Failed to fetch commands:', error);
    }
  };

  // Fetch config
  const fetchConfig = async () => {
    try {
      const response = await fetch(`${BRIDGE_URL}/alexa/config`);
      const data = await response.json();
      if (data.success && data.content) {
        const config = typeof data.content === 'string' ? JSON.parse(data.content) : data.content;
        setConfig(config);
      }
    } catch (error) {
      console.error('Failed to fetch config:', error);
    }
  };

  // Fetch OBS status
  const fetchObsStatus = async () => {
    try {
      const response = await fetch(`${BRIDGE_URL}/alexa/obs/status`);
      const data = await response.json();
      console.log('OBS Status Response:', data);
      if (data.success && data.content) {
        // Parse the content if it's a string
        const status = typeof data.content === 'string' ? JSON.parse(data.content) : data.content;
        console.log('Parsed OBS Status:', status);
        setObsStatus(status);
      }
    } catch (error) {
      console.error('Failed to fetch OBS status:', error);
    }
  };

  // Fetch OBS scenes
  const fetchObsScenes = async () => {
    try {
      const response = await fetch(`${BRIDGE_URL}/alexa/obs/scenes`);
      const data = await response.json();
      if (data.success && data.content) {
        const scenes = typeof data.content === 'string' ? JSON.parse(data.content) : data.content;
        setObsScenes(scenes || []);
      }
    } catch (error) {
      console.error('Failed to fetch OBS scenes:', error);
    }
  };

  createEffect(() => {
    fetchCommands();
    fetchConfig();
    fetchObsStatus();
  });

  // Save config
  const handleSaveConfig = async () => {
    try {
      setLoading(true);
      const response = await fetch(`${BRIDGE_URL}/alexa/config`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(config()),
      });

      const data = await response.json();
      if (data.success) {
        alert('Configuration saved successfully!');
        fetchObsStatus();
      } else {
        alert('Failed to save configuration: ' + (data.error || 'Unknown error'));
      }
    } catch (error) {
      console.error('Failed to save config:', error);
      alert('Failed to save configuration');
    } finally {
      setLoading(false);
    }
  };

  // Connect to OBS
  const handleObsConnect = async () => {
    try {
      setLoading(true);
      const response = await fetch(`${BRIDGE_URL}/alexa/obs/connect`, {
        method: 'POST',
      });

      const data = await response.json();
      if (data.success) {
        // Wait a bit for connection to establish, then fetch status
        setTimeout(async () => {
          await fetchObsStatus();
          await fetchObsScenes();
        }, 500);
        alert('Connected to OBS successfully!');
      } else {
        alert('Failed to connect to OBS: ' + (data.error || 'Unknown error'));
      }
    } catch (error) {
      console.error('Failed to connect to OBS:', error);
      alert('Failed to connect to OBS');
    } finally {
      setLoading(false);
    }
  };

  // Disconnect from OBS
  const handleObsDisconnect = async () => {
    try {
      setLoading(true);
      const response = await fetch(`${BRIDGE_URL}/alexa/obs/disconnect`, {
        method: 'POST',
      });

      const data = await response.json();
      if (data.success) {
        setObsStatus({ connected: false });
        setObsScenes([]);
      }
    } catch (error) {
      console.error('Failed to disconnect from OBS:', error);
    } finally {
      setLoading(false);
    }
  };

  // Open command modal for new command
  const handleNewCommand = () => {
    setFormData({
      id: 0,
      name: '',
      intent_name: '',
      action_type: 'obs_scene',
      action_value: '',
      response_text: '',
      enabled: true,
    });
    setEditingCommand(null);
    setShowCommandModal(true);
    if (obsStatus().connected && obsScenes().length === 0) {
      fetchObsScenes();
    }
  };

  // Open command modal for editing
  const handleEditCommand = (command) => {
    setFormData(command);
    setEditingCommand(command);
    setShowCommandModal(true);
    if (obsStatus().connected && obsScenes().length === 0) {
      fetchObsScenes();
    }
  };

  // Save command
  const handleSaveCommand = async () => {
    if (!formData().name || !formData().intent_name) {
      alert('Please fill in name and intent name');
      return;
    }

    try {
      setLoading(true);
      const response = await fetch(`${BRIDGE_URL}/alexa/commands`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(formData()),
      });

      const data = await response.json();
      if (data.success) {
        setShowCommandModal(false);
        fetchCommands();
      } else {
        alert('Failed to save command: ' + (data.error || 'Unknown error'));
      }
    } catch (error) {
      console.error('Failed to save command:', error);
      alert('Failed to save command');
    } finally {
      setLoading(false);
    }
  };

  // Delete command
  const handleDeleteCommand = async (id) => {
    if (!confirm('Are you sure you want to delete this command?')) return;

    try {
      const response = await fetch(`${BRIDGE_URL}/alexa/commands/${id}`, {
        method: 'DELETE',
      });

      const data = await response.json();
      if (data.success) {
        fetchCommands();
      } else {
        alert('Failed to delete command: ' + (data.error || 'Unknown error'));
      }
    } catch (error) {
      console.error('Failed to delete command:', error);
      alert('Failed to delete command');
    }
  };

  return (
    <div class="h-full flex flex-col bg-base-200">
      {/* Header */}
      <div class="p-6 bg-base-100 border-b border-base-300">
        <div class="flex items-center gap-3">
          <IconBrandAmazon size={32} class="text-primary" />
          <div>
            <h1 class="text-2xl font-bold">Amazon Alexa Integration</h1>
            <p class="text-sm text-base-content/60">Control OBS and stream settings with voice commands</p>
          </div>
        </div>
      </div>

      <div class="flex-1 overflow-y-auto p-6 space-y-6">
        {/* OBS Connection Status */}
        <div class="card bg-base-100 shadow-xl">
          <div class="card-body">
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-3">
                <IconDeviceDesktop size={24} />
                <div>
                  <h2 class="card-title">OBS WebSocket Connection</h2>
                  <p class="text-sm text-base-content/60">
                    Status: <span class={`font-semibold ${obsStatus().connected ? 'text-success' : 'text-error'}`}>
                      {obsStatus().connected ? 'Connected' : 'Disconnected'}
                    </span>
                  </p>
                </div>
              </div>
              <div class="flex gap-2">
                <Show when={!obsStatus().connected}>
                  <button
                    class="btn btn-success gap-2"
                    onClick={handleObsConnect}
                    disabled={loading()}
                  >
                    <IconCheck size={20} />
                    Connect
                  </button>
                </Show>
                <Show when={obsStatus().connected}>
                  <button
                    class="btn btn-error gap-2"
                    onClick={handleObsDisconnect}
                    disabled={loading()}
                  >
                    <IconX size={20} />
                    Disconnect
                  </button>
                </Show>
              </div>
            </div>
          </div>
        </div>

        {/* Configuration */}
        <div class="card bg-base-100 shadow-xl">
          <div class="card-body">
            <h2 class="card-title flex items-center gap-2">
              <IconSettings size={24} />
              Configuration
            </h2>

            <div class="grid grid-cols-2 gap-4">
              <div class="form-control">
                <label class="label">
                  <span class="label-text">OBS Host</span>
                </label>
                <input
                  type="text"
                  class="input input-bordered"
                  value={config().obs_host}
                  onInput={(e) => setConfig({ ...config(), obs_host: e.target.value })}
                  placeholder="localhost"
                />
              </div>

              <div class="form-control">
                <label class="label">
                  <span class="label-text">OBS Port</span>
                </label>
                <input
                  type="number"
                  class="input input-bordered"
                  value={config().obs_port}
                  onInput={(e) => setConfig({ ...config(), obs_port: parseInt(e.target.value) })}
                  placeholder="4455"
                />
              </div>

              <div class="form-control">
                <label class="label">
                  <span class="label-text">OBS Password</span>
                </label>
                <input
                  type="password"
                  class="input input-bordered"
                  value={config().obs_password || ''}
                  onInput={(e) => setConfig({ ...config(), obs_password: e.target.value })}
                  placeholder="Optional"
                />
              </div>

              <div class="form-control">
                <label class="label">
                  <span class="label-text">Alexa Skill ID</span>
                </label>
                <input
                  type="text"
                  class="input input-bordered"
                  value={config().skill_id || ''}
                  onInput={(e) => setConfig({ ...config(), skill_id: e.target.value })}
                  placeholder="Optional"
                />
              </div>
            </div>

            <div class="form-control">
              <label class="label cursor-pointer justify-start gap-4">
                <input
                  type="checkbox"
                  class="toggle toggle-success"
                  checked={config().enabled}
                  onChange={(e) => setConfig({ ...config(), enabled: e.target.checked })}
                />
                <span class="label-text">Enable Alexa Integration</span>
              </label>
            </div>

            <div class="card-actions justify-end mt-4">
              <button
                class="btn btn-primary gap-2"
                onClick={handleSaveConfig}
                disabled={loading()}
              >
                <IconCheck size={20} />
                Save Configuration
              </button>
            </div>
          </div>
        </div>

        {/* Voice Commands */}
        <div class="card bg-base-100 shadow-xl">
          <div class="card-body">
            <div class="flex items-center justify-between mb-4">
              <h2 class="card-title flex items-center gap-2">
                <IconMicrophone size={24} />
                Voice Commands
              </h2>
              <button
                class="btn btn-primary gap-2"
                onClick={handleNewCommand}
              >
                <IconPlus size={20} />
                Add Command
              </button>
            </div>

            <Show
              when={commands().length > 0}
              fallback={
                <div class="text-center py-12 text-base-content/60">
                  <IconMicrophone size={48} class="mx-auto mb-4 opacity-30" />
                  <p>No voice commands yet</p>
                  <p class="text-sm">Create your first Alexa command to get started</p>
                </div>
              }
            >
              <div class="overflow-x-auto">
                <table class="table table-zebra">
                  <thead>
                    <tr>
                      <th>Name</th>
                      <th>Intent</th>
                      <th>Action</th>
                      <th>Response</th>
                      <th>Status</th>
                      <th>Actions</th>
                    </tr>
                  </thead>
                  <tbody>
                    <For each={commands()}>
                      {(command) => (
                        <tr>
                          <td class="font-semibold">{command.name}</td>
                          <td class="font-mono text-sm">{command.intent_name}</td>
                          <td>
                            <div class="badge badge-ghost">
                              {command.action_type}
                            </div>
                            <div class="text-xs text-base-content/60 mt-1">
                              {command.action_value}
                            </div>
                          </td>
                          <td class="text-sm">{command.response_text}</td>
                          <td>
                            <div class={`badge ${command.enabled ? 'badge-success' : 'badge-error'}`}>
                              {command.enabled ? 'Enabled' : 'Disabled'}
                            </div>
                          </td>
                          <td>
                            <div class="flex gap-2">
                              <button
                                class="btn btn-sm btn-ghost"
                                onClick={() => handleEditCommand(command)}
                              >
                                <IconEdit size={16} />
                              </button>
                              <button
                                class="btn btn-sm btn-error btn-ghost"
                                onClick={() => handleDeleteCommand(command.id)}
                              >
                                <IconTrash size={16} />
                              </button>
                            </div>
                          </td>
                        </tr>
                      )}
                    </For>
                  </tbody>
                </table>
              </div>
            </Show>
          </div>
        </div>

        {/* Setup Instructions */}
        <div class="card bg-base-100 shadow-xl">
          <div class="card-body">
            <h2 class="card-title">Setup Instructions</h2>
            <div class="prose max-w-none">
              <ol class="text-sm space-y-2">
                <li>Install and configure OBS WebSocket plugin (OBS 28+ has it built-in)</li>
                <li>Enter your OBS WebSocket connection details above and save</li>
                <li>Click "Connect" to establish connection with OBS</li>
                <li>Create voice commands that map to OBS scenes or other actions</li>
                <li>Set up your Alexa skill in the Amazon Developer Console</li>
                <li>Configure your skill to send requests to: <code>http://localhost:3001/alexa/request</code></li>
                <li>Use tunnel services like ngrok to expose your local server to Alexa</li>
              </ol>
            </div>
          </div>
        </div>
      </div>

      {/* Command Modal */}
      <Show when={showCommandModal()}>
        <div class="modal modal-open">
          <div class="modal-box max-w-2xl">
            <h3 class="font-bold text-lg mb-4">
              {editingCommand() ? 'Edit Command' : 'New Command'}
            </h3>

            <div class="space-y-4">
              <div class="form-control">
                <label class="label">
                  <span class="label-text">Command Name</span>
                </label>
                <input
                  type="text"
                  class="input input-bordered"
                  value={formData().name}
                  onInput={(e) => setFormData({ ...formData(), name: e.target.value })}
                  placeholder="e.g., Switch to Gaming Scene"
                />
              </div>

              <div class="form-control">
                <label class="label">
                  <span class="label-text">Intent Name</span>
                </label>
                <input
                  type="text"
                  class="input input-bordered font-mono"
                  value={formData().intent_name}
                  onInput={(e) => setFormData({ ...formData(), intent_name: e.target.value })}
                  placeholder="e.g., SwitchToGamingIntent"
                />
                <label class="label">
                  <span class="label-text-alt">Must match the intent name in your Alexa skill</span>
                </label>
              </div>

              <div class="form-control">
                <label class="label">
                  <span class="label-text">Action Type</span>
                </label>
                <select
                  class="select select-bordered"
                  value={formData().action_type}
                  onChange={(e) => setFormData({ ...formData(), action_type: e.target.value })}
                >
                  <option value="obs_scene">Switch OBS Scene</option>
                  <option value="obs_source_visibility">Toggle OBS Source</option>
                  <option value="obs_filter_toggle">Toggle OBS Filter</option>
                </select>
              </div>

              <Show when={formData().action_type === 'obs_scene'}>
                <div class="form-control">
                  <label class="label">
                    <span class="label-text">OBS Scene</span>
                  </label>
                  <Show
                    when={obsStatus().connected && obsScenes().length > 0}
                    fallback={
                      <input
                        type="text"
                        class="input input-bordered"
                        value={formData().action_value}
                        onInput={(e) => setFormData({ ...formData(), action_value: e.target.value })}
                        placeholder="Scene name"
                      />
                    }
                  >
                    <select
                      class="select select-bordered"
                      value={formData().action_value}
                      onChange={(e) => setFormData({ ...formData(), action_value: e.target.value })}
                    >
                      <option value="">Select a scene...</option>
                      <For each={obsScenes()}>
                        {(scene) => <option value={scene}>{scene}</option>}
                      </For>
                    </select>
                  </Show>
                </div>
              </Show>

              <Show when={formData().action_type === 'obs_source_visibility'}>
                <div class="form-control">
                  <label class="label">
                    <span class="label-text">Action Value</span>
                  </label>
                  <input
                    type="text"
                    class="input input-bordered font-mono"
                    value={formData().action_value}
                    onInput={(e) => setFormData({ ...formData(), action_value: e.target.value })}
                    placeholder="scene_name:source_name:true"
                  />
                  <label class="label">
                    <span class="label-text-alt">Format: scene:source:visible (true/false)</span>
                  </label>
                </div>
              </Show>

              <Show when={formData().action_type === 'obs_filter_toggle'}>
                <div class="form-control">
                  <label class="label">
                    <span class="label-text">Action Value</span>
                  </label>
                  <input
                    type="text"
                    class="input input-bordered font-mono"
                    value={formData().action_value}
                    onInput={(e) => setFormData({ ...formData(), action_value: e.target.value })}
                    placeholder="source_name:filter_name"
                  />
                  <label class="label">
                    <span class="label-text-alt">Format: source:filter</span>
                  </label>
                </div>
              </Show>

              <div class="form-control">
                <label class="label">
                  <span class="label-text">Alexa Response</span>
                </label>
                <input
                  type="text"
                  class="input input-bordered"
                  value={formData().response_text}
                  onInput={(e) => setFormData({ ...formData(), response_text: e.target.value })}
                  placeholder="What Alexa should say after executing the command"
                />
              </div>

              <div class="form-control">
                <label class="label cursor-pointer justify-start gap-4">
                  <input
                    type="checkbox"
                    class="toggle toggle-success"
                    checked={formData().enabled}
                    onChange={(e) => setFormData({ ...formData(), enabled: e.target.checked })}
                  />
                  <span class="label-text">Enabled</span>
                </label>
              </div>
            </div>

            <div class="modal-action">
              <button
                class="btn"
                onClick={() => setShowCommandModal(false)}
              >
                Cancel
              </button>
              <button
                class="btn btn-primary"
                onClick={handleSaveCommand}
                disabled={loading()}
              >
                Save Command
              </button>
            </div>
          </div>
          <div class="modal-backdrop" onClick={() => setShowCommandModal(false)}></div>
        </div>
      </Show>
    </div>
  );
}
