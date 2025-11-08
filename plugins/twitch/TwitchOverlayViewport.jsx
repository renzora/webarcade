import { createSignal, onMount, For, Show } from 'solid-js';
import twitchStore from './TwitchStore.jsx';
import { IconPlus, IconTrash, IconCode, IconCommand, IconSparkles, IconCopy, IconDownload, IconAlertCircle, IconCheck, IconUser, IconUsers, IconShield, IconCrown } from '@tabler/icons-solidjs';

export default function TwitchOverlayViewport() {
  const [activeTab, setActiveTab] = createSignal('commands');
  const [commands, setCommands] = createSignal([]);
  const [loading, setLoading] = createSignal(true);
  const [saving, setSaving] = createSignal(false);

  // Command Form
  const [commandName, setCommandName] = createSignal('');
  const [commandAliases, setCommandAliases] = createSignal('');
  const [commandDescription, setCommandDescription] = createSignal('');
  const [commandResponse, setCommandResponse] = createSignal('');
  const [commandPermission, setCommandPermission] = createSignal('everyone');

  // Overlay Settings
  const [overlayHtml, setOverlayHtml] = createSignal(`<!DOCTYPE html>
<html>
<head>
  <style>
    body {
      margin: 0;
      padding: 20px;
      font-family: 'Arial', sans-serif;
      background: transparent;
    }

    .alert {
      background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
      padding: 20px 30px;
      border-radius: 10px;
      color: white;
      font-size: 24px;
      font-weight: bold;
      box-shadow: 0 10px 30px rgba(0,0,0,0.3);
      animation: slideIn 0.5s ease;
    }

    @keyframes slideIn {
      from { transform: translateX(-100%); opacity: 0; }
      to { transform: translateX(0); opacity: 1; }
    }

    .username { color: #FFD700; }
    .message { font-weight: normal; margin-top: 5px; }
  </style>
</head>
<body>
  <div id="overlay"></div>

  <script>
    // Connect to WebSocket for Twitch events
    const ws = new WebSocket('ws://localhost:3002');

    ws.onmessage = (event) => {
      const data = JSON.parse(event.data);

      if (data.type === 'twitch_event' && data.event.type === 'chat_message') {
        const msg = data.event;
        showAlert(msg.username, msg.message);
      }
    };

    function showAlert(username, message) {
      const overlay = document.getElementById('overlay');
      const alertDiv = document.createElement('div');
      alertDiv.className = 'alert';
      alertDiv.innerHTML = \`
        <div class="username">@\${username}</div>
        <div class="message">\${message}</div>
      \`;

      overlay.appendChild(alertDiv);

      // Remove after 5 seconds
      setTimeout(() => {
        alertDiv.style.animation = 'slideIn 0.5s ease reverse';
        setTimeout(() => overlay.removeChild(alertDiv), 500);
      }, 5000);
    }
  </script>
</body>
</html>`);

  onMount(async () => {
    await loadCommands();
    setLoading(false);
  });

  const loadCommands = async () => {
    const cmds = await twitchStore.fetchCommands();
    setCommands(cmds);
  };

  const getPermissionIcon = (permission) => {
    switch (permission) {
      case 'broadcaster':
        return <IconCrown size={16} class="text-error" />;
      case 'moderator':
        return <IconShield size={16} class="text-success" />;
      case 'vip':
        return <IconSparkles size={16} class="text-secondary" />;
      case 'subscriber':
        return <IconUsers size={16} class="text-primary" />;
      default:
        return <IconUser size={16} class="text-base-content/50" />;
    }
  };

  const getPermissionBadgeClass = (permission) => {
    switch (permission) {
      case 'broadcaster':
        return 'badge-error';
      case 'moderator':
        return 'badge-success';
      case 'vip':
        return 'badge-secondary';
      case 'subscriber':
        return 'badge-primary';
      default:
        return 'badge-ghost';
    }
  };

  const handleAddCommand = async () => {
    if (!commandName() || !commandResponse()) {
      return;
    }

    setSaving(true);
    try {
      const aliases = commandAliases()
        .split(',')
        .map((a) => a.trim())
        .filter((a) => a.length > 0);

      await twitchStore.registerCommand({
        name: commandName(),
        aliases,
        description: commandDescription(),
        response: commandResponse(),
        permission: commandPermission()
      });

      // Clear form
      setCommandName('');
      setCommandAliases('');
      setCommandDescription('');
      setCommandResponse('');
      setCommandPermission('everyone');

      // Reload commands
      await loadCommands();
    } catch (e) {
      console.error('Failed to register command:', e);
    } finally {
      setSaving(false);
    }
  };

  const handleDeleteCommand = async (name) => {
    if (!confirm(`Delete command '${name}'?`)) return;

    try {
      await twitchStore.unregisterCommand(name);
      await loadCommands();
    } catch (e) {
      alert(`Failed to delete command: ${e.message}`);
    }
  };

  const copyOverlayUrl = () => {
    // In a real scenario, you'd serve this HTML file via your bridge
    // For now, users can copy the HTML and host it themselves
    navigator.clipboard.writeText(overlayHtml());
    alert('Overlay HTML copied to clipboard! You can paste this into a local HTML file and add it as a Browser Source in OBS.');
  };

  const exportOverlay = () => {
    const blob = new Blob([overlayHtml()], { type: 'text/html' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = 'twitch-overlay.html';
    a.click();
    URL.revokeObjectURL(url);
  };

  return (
    <div class="h-full flex flex-col bg-base-200">
      {/* Tabs */}
      <div class="tabs tabs-boxed p-4 bg-base-100 border-b border-base-300">
        <a
          class={`tab tab-lg ${activeTab() === 'commands' ? 'tab-active' : ''}`}
          onClick={() => setActiveTab('commands')}
        >
          <IconCommand size={20} class="mr-2" />
          Commands
        </a>
        <a
          class={`tab tab-lg ${activeTab() === 'overlay' ? 'tab-active' : ''}`}
          onClick={() => setActiveTab('overlay')}
        >
          <IconCode size={20} class="mr-2" />
          Overlay Editor
        </a>
      </div>

      {/* Content */}
      <div class="flex-1 overflow-y-auto p-6">
        <Show when={activeTab() === 'commands'}>
          <div class="max-w-4xl mx-auto space-y-6">
            {/* Add Command Form */}
            <div class="card bg-base-100 shadow-xl">
              <div class="card-body">
                <h2 class="card-title">Add New Command</h2>

                <div class="grid grid-cols-2 gap-4">
                  <div class="form-control">
                    <label class="label">
                      <span class="label-text">Command Name</span>
                    </label>
                    <input
                      type="text"
                      placeholder="hello"
                      class="input input-bordered"
                      value={commandName()}
                      onInput={(e) => setCommandName(e.target.value)}
                    />
                    <label class="label">
                      <span class="label-text-alt">Users will type: !{commandName() || 'hello'}</span>
                    </label>
                  </div>

                  <div class="form-control">
                    <label class="label">
                      <span class="label-text">Aliases (comma-separated)</span>
                    </label>
                    <input
                      type="text"
                      placeholder="hi, hey"
                      class="input input-bordered"
                      value={commandAliases()}
                      onInput={(e) => setCommandAliases(e.target.value)}
                    />
                  </div>
                </div>

                <div class="form-control">
                  <label class="label">
                    <span class="label-text">Description</span>
                  </label>
                  <input
                    type="text"
                    placeholder="Say hello to users"
                    class="input input-bordered"
                    value={commandDescription()}
                    onInput={(e) => setCommandDescription(e.target.value)}
                  />
                </div>

                <div class="form-control">
                  <label class="label">
                    <span class="label-text">Response</span>
                  </label>
                  <textarea
                    class="textarea textarea-bordered h-24"
                    placeholder="Hello! Welcome to the stream!"
                    value={commandResponse()}
                    onInput={(e) => setCommandResponse(e.target.value)}
                  />
                </div>

                <div class="form-control">
                  <label class="label">
                    <span class="label-text">Permission Level</span>
                  </label>
                  <select
                    class="select select-bordered"
                    value={commandPermission()}
                    onChange={(e) => setCommandPermission(e.target.value)}
                  >
                    <option value="everyone">Everyone</option>
                    <option value="subscriber">Subscribers</option>
                    <option value="vip">VIPs</option>
                    <option value="moderator">Moderators</option>
                    <option value="broadcaster">Broadcaster Only</option>
                  </select>
                </div>

                <div class="card-actions justify-end">
                  <button class="btn btn-primary" onClick={handleAddCommand}>
                    <IconPlus size={20} />
                    Add Command
                  </button>
                </div>
              </div>
            </div>

            {/* Commands List */}
            <div class="card bg-base-100 shadow-xl">
              <div class="card-body">
                <h2 class="card-title">Registered Commands ({commands().length})</h2>

                <Show
                  when={commands().length > 0}
                  fallback={<p class="text-base-content/50">No commands registered yet</p>}
                >
                  <div class="space-y-2">
                    <For each={commands()}>
                      {(cmd) => (
                        <div class="flex items-center justify-between p-3 bg-base-200 rounded-lg">
                          <div class="flex-1">
                            <div class="flex items-center gap-2">
                              <span class="font-mono font-bold">!{cmd.name}</span>
                              <Show when={cmd.aliases && cmd.aliases.length > 0}>
                                <div class="flex gap-1">
                                  <For each={cmd.aliases}>
                                    {(alias) => (
                                      <span class="badge badge-sm">!{alias}</span>
                                    )}
                                  </For>
                                </div>
                              </Show>
                              <span class="badge badge-primary badge-sm">{cmd.permission}</span>
                            </div>
                            <p class="text-sm text-base-content/70 mt-1">{cmd.description}</p>
                          </div>
                          <button
                            class="btn btn-error btn-sm"
                            onClick={() => handleDeleteCommand(cmd.name)}
                          >
                            <IconTrash size={16} />
                          </button>
                        </div>
                      )}
                    </For>
                  </div>
                </Show>
              </div>
            </div>
          </div>
        </Show>

        <Show when={activeTab() === 'overlay'}>
          <div class="max-w-6xl mx-auto space-y-6">
            <div class="card bg-base-100 shadow-xl">
              <div class="card-body">
                <h2 class="card-title">Stream Overlay Editor</h2>
                <p class="text-base-content/70">
                  Create custom HTML overlays for OBS or other streaming software. The overlay will automatically receive Twitch events via WebSocket.
                </p>

                <div class="form-control">
                  <label class="label">
                    <span class="label-text font-semibold">HTML Code</span>
                  </label>
                  <textarea
                    class="textarea textarea-bordered font-mono text-sm"
                    rows={20}
                    value={overlayHtml()}
                    onInput={(e) => setOverlayHtml(e.target.value)}
                  />
                </div>

                <div class="card-actions justify-end gap-2">
                  <button class="btn btn-secondary" onClick={copyOverlayUrl}>
                    Copy HTML
                  </button>
                  <button class="btn btn-primary" onClick={exportOverlay}>
                    Export as HTML File
                  </button>
                </div>
              </div>
            </div>

            <div class="card bg-base-100 shadow-xl">
              <div class="card-body">
                <h2 class="card-title">How to Use</h2>
                <ol class="list-decimal list-inside space-y-2 text-base-content/80">
                  <li>Customize the HTML/CSS/JavaScript above</li>
                  <li>Click "Export as HTML File" to download the overlay</li>
                  <li>In OBS, add a "Browser Source"</li>
                  <li>Set the source to the exported HTML file (local file)</li>
                  <li>Set width/height as needed (1920x1080 recommended)</li>
                  <li>The overlay will automatically connect to your Twitch bot via WebSocket</li>
                </ol>

                <div class="alert alert-info mt-4">
                  <IconCode size={24} />
                  <div>
                    <h3 class="font-bold">Available Events</h3>
                    <p class="text-sm">
                      The overlay receives events from <code class="bg-base-300 px-1 rounded">ws://localhost:3002</code>.
                      Event types: <code>chat_message</code>, <code>connected</code>, <code>disconnected</code>, etc.
                    </p>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </Show>
      </div>
    </div>
  );
}
