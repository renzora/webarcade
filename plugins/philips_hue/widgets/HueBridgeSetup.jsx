import { createSignal, onMount, For } from 'solid-js';
import { IconBulb, IconPlus, IconTrash, IconRefresh, IconSettings } from '@tabler/icons-solidjs';

export default function HueBridgeSetup() {
  const [bridges, setBridges] = createSignal([]);
  const [loading, setLoading] = createSignal(false);
  const [showAddForm, setShowAddForm] = createSignal(false);
  const [newBridge, setNewBridge] = createSignal({ name: '', ip_address: '' });
  const [pairingBridgeId, setPairingBridgeId] = createSignal(null);
  const [message, setMessage] = createSignal(null);

  const loadBridges = async () => {
    setLoading(true);
    try {
      const response = await fetch('http://localhost:3001/philips-hue/bridges');
      const data = await response.json();
      setBridges(data.bridges || []);
    } catch (error) {
      console.error('Failed to load bridges:', error);
      setMessage({ type: 'error', text: 'Failed to load bridges' });
    } finally {
      setLoading(false);
    }
  };

  const addBridge = async () => {
    if (!newBridge().name || !newBridge().ip_address) {
      setMessage({ type: 'error', text: 'Please fill in all fields' });
      return;
    }

    setLoading(true);
    try {
      const response = await fetch('http://localhost:3001/philips-hue/bridges', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(newBridge()),
      });
      const data = await response.json();

      if (data.success) {
        setMessage({ type: 'success', text: 'Bridge added successfully' });
        setNewBridge({ name: '', ip_address: '' });
        setShowAddForm(false);
        await loadBridges();
      }
    } catch (error) {
      console.error('Failed to add bridge:', error);
      setMessage({ type: 'error', text: 'Failed to add bridge' });
    } finally {
      setLoading(false);
    }
  };

  const deleteBridge = async (bridgeId) => {
    if (!confirm('Are you sure you want to delete this bridge?')) return;

    setLoading(true);
    try {
      const response = await fetch(`http://localhost:3001/philips-hue/bridges/${bridgeId}`, {
        method: 'DELETE',
      });
      const data = await response.json();

      if (data.success) {
        setMessage({ type: 'success', text: 'Bridge deleted' });
        await loadBridges();
      }
    } catch (error) {
      console.error('Failed to delete bridge:', error);
      setMessage({ type: 'error', text: 'Failed to delete bridge' });
    } finally {
      setLoading(false);
    }
  };

  const pairBridge = async (bridgeId) => {
    setPairingBridgeId(bridgeId);
    setMessage({ type: 'info', text: 'Press the button on your Hue Bridge now...' });

    try {
      const response = await fetch(`http://localhost:3001/philips-hue/bridges/${bridgeId}/pair`, {
        method: 'POST',
      });
      const data = await response.json();

      if (data.success) {
        setMessage({ type: 'success', text: 'Bridge paired successfully!' });
        await loadBridges();
      } else {
        setMessage({ type: 'error', text: data.message || 'Pairing failed' });
      }
    } catch (error) {
      console.error('Failed to pair bridge:', error);
      setMessage({ type: 'error', text: 'Failed to pair bridge' });
    } finally {
      setPairingBridgeId(null);
    }
  };

  const discoverLights = async (bridgeId) => {
    setLoading(true);
    setMessage({ type: 'info', text: 'Discovering lights...' });

    try {
      const response = await fetch(`http://localhost:3001/philips-hue/bridges/${bridgeId}/discover`, {
        method: 'GET',
      });
      const data = await response.json();

      if (data.success) {
        setMessage({ type: 'success', text: `Found ${data.count} lights!` });
      } else {
        setMessage({ type: 'error', text: 'Failed to discover lights' });
      }
    } catch (error) {
      console.error('Failed to discover lights:', error);
      setMessage({ type: 'error', text: 'Failed to discover lights' });
    } finally {
      setLoading(false);
    }
  };

  onMount(() => {
    loadBridges();
  });

  return (
    <div class="card bg-gradient-to-br from-purple-500/20 to-purple-500/5 bg-base-100 shadow-lg h-full flex flex-col p-3">
      {/* Header */}
      <div class="flex items-center justify-between mb-2">
        <div class="flex items-center gap-1.5">
          <IconSettings size={16} class="text-purple-500 opacity-80" />
          <span class="text-xs font-medium opacity-70">Hue Bridge Setup</span>
        </div>
        <button
          class="btn btn-xs btn-ghost"
          onClick={() => setShowAddForm(!showAddForm())}
          title="Add Bridge"
        >
          <IconPlus size={14} />
        </button>
      </div>

      {/* Message */}
      {message() && (
        <div class={`alert alert-${message().type} alert-sm mb-2`}>
          <span class="text-xs">{message().text}</span>
        </div>
      )}

      {/* Add Bridge Form */}
      {showAddForm() && (
        <div class="bg-base-200 rounded-lg p-2 mb-2">
          <div class="form-control mb-2">
            <label class="label py-1">
              <span class="label-text text-xs">Bridge Name</span>
            </label>
            <input
              type="text"
              class="input input-sm input-bordered"
              placeholder="Living Room Bridge"
              value={newBridge().name}
              onInput={(e) => setNewBridge({ ...newBridge(), name: e.target.value })}
            />
          </div>
          <div class="form-control mb-2">
            <label class="label py-1">
              <span class="label-text text-xs">IP Address</span>
            </label>
            <input
              type="text"
              class="input input-sm input-bordered"
              placeholder="192.168.1.100"
              value={newBridge().ip_address}
              onInput={(e) => setNewBridge({ ...newBridge(), ip_address: e.target.value })}
            />
          </div>
          <div class="flex gap-2">
            <button
              class="btn btn-sm btn-primary flex-1"
              onClick={addBridge}
              disabled={loading()}
            >
              Add
            </button>
            <button
              class="btn btn-sm btn-ghost flex-1"
              onClick={() => setShowAddForm(false)}
            >
              Cancel
            </button>
          </div>
        </div>
      )}

      {/* Bridges List */}
      <div class="flex-1 overflow-y-auto">
        {loading() && bridges().length === 0 ? (
          <div class="text-center text-xs opacity-50 py-4">Loading...</div>
        ) : bridges().length === 0 ? (
          <div class="text-center text-xs opacity-50 py-4">
            No bridges configured. Click + to add one.
          </div>
        ) : (
          <For each={bridges()}>
            {(bridge) => (
              <div class="bg-base-200 rounded-lg p-2 mb-2">
                <div class="flex items-start justify-between mb-1">
                  <div class="flex-1">
                    <div class="font-medium text-sm">{bridge.name}</div>
                    <div class="text-xs opacity-70">{bridge.ip_address}</div>
                    <div class="text-xs opacity-50 mt-0.5">
                      {bridge.username ? (
                        <span class="text-success">✓ Paired</span>
                      ) : (
                        <span class="text-warning">⚠ Not Paired</span>
                      )}
                    </div>
                  </div>
                  <button
                    class="btn btn-xs btn-ghost btn-square"
                    onClick={() => deleteBridge(bridge.id)}
                    title="Delete"
                  >
                    <IconTrash size={14} />
                  </button>
                </div>

                <div class="flex gap-1 mt-2">
                  {!bridge.username && (
                    <button
                      class="btn btn-xs btn-primary flex-1"
                      onClick={() => pairBridge(bridge.id)}
                      disabled={pairingBridgeId() === bridge.id}
                    >
                      {pairingBridgeId() === bridge.id ? 'Pairing...' : 'Pair'}
                    </button>
                  )}
                  {bridge.username && (
                    <button
                      class="btn btn-xs btn-success flex-1"
                      onClick={() => discoverLights(bridge.id)}
                      disabled={loading()}
                    >
                      <IconRefresh size={12} />
                      Discover Lights
                    </button>
                  )}
                </div>
              </div>
            )}
          </For>
        )}
      </div>

      {/* Info */}
      <div class="text-xs opacity-50 mt-2 border-t border-base-300 pt-2">
        <div>💡 To find your bridge IP, check your router's DHCP list</div>
        <div class="mt-1">📱 Press the bridge button before pairing</div>
      </div>
    </div>
  );
}
