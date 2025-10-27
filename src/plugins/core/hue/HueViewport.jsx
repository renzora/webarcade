import { createSignal, onMount, For, Show } from 'solid-js';
import { bridgeFetch } from '@/api/bridge.js';
import { IconBulb, IconPlug, IconSettings, IconRefresh, IconSearch, IconCheck } from '@tabler/icons-solidjs';

export default function HueViewport() {
  const [configured, setConfigured] = createSignal(false);
  const [bridgeIP, setBridgeIP] = createSignal('');
  const [discovering, setDiscovering] = createSignal(false);
  const [pairing, setPairing] = createSignal(false);
  const [lights, setLights] = createSignal([]);
  const [loading, setLoading] = createSignal(true);
  const [customScenes, setCustomScenes] = createSignal([]);
  const [newSceneName, setNewSceneName] = createSignal('');
  const [newSceneColor, setNewSceneColor] = createSignal('#FF0000');
  const [transitionTime, setTransitionTime] = createSignal(4); // 0.4 seconds default

  let brightnessTimeout = null;

  onMount(async () => {
    await loadConfig();
    await loadCustomScenes();
    setLoading(false);
  });

  const loadConfig = async () => {
    try {
      const response = await bridgeFetch('/hue/config');
      const data = await response.json();
      setConfigured(data.configured);
      if (data.bridge_ip) {
        setBridgeIP(data.bridge_ip);
        await loadLights();
      }
    } catch (e) {
      console.error('Failed to load Hue config:', e);
    }
  };

  const discoverBridge = async () => {
    setDiscovering(true);
    try {
      const response = await bridgeFetch('/hue/discover');
      const data = await response.json();

      if (data.success && data.bridge_ip) {
        setBridgeIP(data.bridge_ip);
      }
    } catch (e) {
      console.error('Failed to discover bridge:', e);
      alert('No Hue bridge found on your network. Make sure it\'s powered on and connected.');
    } finally {
      setDiscovering(false);
    }
  };

  const pairBridge = async () => {
    if (!bridgeIP()) {
      alert('Please discover or enter a bridge IP first');
      return;
    }

    setPairing(true);
    try {
      const response = await bridgeFetch('/hue/pair', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ bridge_ip: bridgeIP() }),
      });

      if (response.ok) {
        setConfigured(true);
        await loadLights();
      } else {
        const data = await response.json();
        alert(data.error || 'Pairing failed. Did you press the button on your bridge?');
      }
    } catch (e) {
      console.error('Failed to pair:', e);
      alert('Pairing failed. Make sure to press the button on your Hue bridge!');
    } finally {
      setPairing(false);
    }
  };

  const unpair = async () => {
    if (!confirm('Remove Hue bridge configuration?')) return;

    try {
      await bridgeFetch('/hue/config', { method: 'DELETE' });
      setConfigured(false);
      setBridgeIP('');
      setLights([]);
    } catch (e) {
      console.error('Failed to unpair:', e);
    }
  };

  const loadLights = async () => {
    try {
      const response = await bridgeFetch('/hue/lights');
      const data = await response.json();
      // Sort by ID to maintain consistent order
      data.sort((a, b) => a.id.localeCompare(b.id));
      setLights(data);
    } catch (e) {
      console.error('Failed to load lights:', e);
    }
  };

  const toggleLight = async (lightId, currentState) => {
    // Optimistically update UI
    setLights(lights().map(light =>
      light.id === lightId ? { ...light, on: !currentState } : light
    ));

    try {
      await bridgeFetch('/hue/lights/power', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ light_id: lightId, on: !currentState }),
      });
    } catch (e) {
      console.error('Failed to toggle light:', e);
      // Reload to get actual state on error
      await loadLights();
    }
  };

  const setBrightness = (lightId, brightness) => {
    const brightnessValue = Math.round(brightness);

    // Optimistically update UI
    setLights(lights().map(light =>
      light.id === lightId ? { ...light, brightness: brightnessValue } : light
    ));

    // Debounce API call
    if (brightnessTimeout) clearTimeout(brightnessTimeout);
    brightnessTimeout = setTimeout(async () => {
      try {
        await bridgeFetch('/hue/lights/brightness', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ light_id: lightId, brightness: brightnessValue }),
        });
      } catch (e) {
        console.error('Failed to set brightness:', e);
        // Reload to get actual state on error
        await loadLights();
      }
    }, 300);
  };

  const setScene = async (scene) => {
    try {
      await bridgeFetch('/hue/scene', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ scene }),
      });
      await loadLights();
    } catch (e) {
      console.error('Failed to set scene:', e);
    }
  };

  const loadCustomScenes = async () => {
    try {
      const response = await bridgeFetch('/hue/scenes');
      const data = await response.json();
      setCustomScenes(data);
    } catch (e) {
      console.error('Failed to load custom scenes:', e);
    }
  };

  const saveCustomScene = async () => {
    const name = newSceneName().trim().toLowerCase();
    if (!name) {
      alert('Please enter a preset name');
      return;
    }

    // Convert hex to RGB
    const hex = newSceneColor().replace('#', '');
    const r = parseInt(hex.substring(0, 2), 16);
    const g = parseInt(hex.substring(2, 4), 16);
    const b = parseInt(hex.substring(4, 6), 16);

    try {
      await bridgeFetch('/hue/scenes', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ name, red: r, green: g, blue: b }),
      });
      setNewSceneName('');
      setNewSceneColor('#FF0000');
      await loadCustomScenes();
    } catch (e) {
      console.error('Failed to save preset:', e);
      alert('Failed to save preset');
    }
  };

  const deleteCustomScene = async (name) => {
    if (!confirm(`Delete preset '${name}'?`)) return;

    try {
      await bridgeFetch('/hue/scenes', {
        method: 'DELETE',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ name }),
      });
      await loadCustomScenes();
    } catch (e) {
      console.error('Failed to delete preset:', e);
      alert('Failed to delete preset');
    }
  };

  const sceneButtons = [
    { name: 'red', color: '#FF0000' },
    { name: 'orange', color: '#FF8800' },
    { name: 'yellow', color: '#FFFF00' },
    { name: 'green', color: '#00FF00' },
    { name: 'cyan', color: '#00FFFF' },
    { name: 'blue', color: '#0000FF' },
    { name: 'purple', color: '#8800FF' },
    { name: 'pink', color: '#FF00FF' },
    { name: 'white', color: '#FFFFFF' },
  ];

  return (
    <div class="h-full flex flex-col bg-base-200">
      {/* Header */}
      <div class="flex items-center justify-between bg-base-100 border-b border-base-300 px-4 py-3">
        <div class="flex items-center gap-3">
          <IconBulb size={20} class="text-primary" />
          <h2 class="text-lg font-semibold">Philips Hue</h2>
        </div>

        <Show when={configured()}>
          <button class="btn btn-sm btn-ghost" onClick={unpair}>
            Disconnect
          </button>
        </Show>
      </div>

      <div class="flex-1 overflow-y-auto">
        <Show
          when={!loading()}
          fallback={
            <div class="flex items-center justify-center h-full">
              <span class="loading loading-spinner loading-lg"></span>
            </div>
          }
        >
          <Show
            when={configured()}
            fallback={
              <div class="p-6 max-w-2xl mx-auto">
                {/* Setup Instructions */}
                <div class="card bg-base-100 shadow-lg mb-6">
                  <div class="card-body">
                    <h3 class="card-title">Setup Philips Hue</h3>
                    <div class="space-y-4">
                      <div class="alert alert-info">
                        <IconSettings size={20} />
                        <div class="text-sm">
                          <p class="font-semibold">Requirements:</p>
                          <ul class="list-disc list-inside mt-2">
                            <li>Philips Hue Bridge connected to your network</li>
                            <li>Bridge and computer on the same network</li>
                            <li>Access to the physical bridge button</li>
                          </ul>
                        </div>
                      </div>

                      {/* Step 1: Discover */}
                      <div class="form-control">
                        <label class="label">
                          <span class="label-text font-semibold">Step 1: Find your bridge</span>
                        </label>
                        <div class="flex gap-2">
                          <input
                            type="text"
                            placeholder="Bridge IP (e.g., 192.168.1.100)"
                            class="input input-bordered flex-1"
                            value={bridgeIP()}
                            onInput={(e) => setBridgeIP(e.target.value)}
                          />
                          <button
                            class="btn btn-primary gap-2"
                            onClick={discoverBridge}
                            disabled={discovering()}
                          >
                            <IconSearch size={16} />
                            {discovering() ? 'Searching...' : 'Auto-discover'}
                          </button>
                        </div>
                      </div>

                      {/* Step 2: Pair */}
                      <Show when={bridgeIP()}>
                        <div class="form-control">
                          <label class="label">
                            <span class="label-text font-semibold">Step 2: Pair with bridge</span>
                          </label>
                          <div class="alert alert-warning mb-3">
                            <IconPlug size={20} />
                            <span class="text-sm">Press the button on your Hue bridge, then click Pair</span>
                          </div>
                          <button
                            class="btn btn-success gap-2"
                            onClick={pairBridge}
                            disabled={pairing()}
                          >
                            <IconCheck size={16} />
                            {pairing() ? 'Pairing...' : 'Pair Bridge'}
                          </button>
                        </div>
                      </Show>
                    </div>
                  </div>
                </div>
              </div>
            }
          >
            <div class="p-4 space-y-4">
              {/* Transition Settings */}
              <div class="card bg-base-100 shadow-sm">
                <div class="card-body p-4">
                  <h3 class="font-semibold mb-3">Transition Settings</h3>
                  <div class="form-control">
                    <label class="label">
                      <span class="label-text">Transition Speed: {transitionTime() / 10}s</span>
                    </label>
                    <input
                      type="range"
                      min="0"
                      max="40"
                      value={transitionTime()}
                      class="range range-sm range-primary"
                      onInput={(e) => setTransitionTime(parseInt(e.target.value))}
                    />
                    <div class="flex justify-between text-xs text-base-content/60 px-2 mt-1">
                      <span>Instant</span>
                      <span>Smooth (4s)</span>
                    </div>
                  </div>
                </div>
              </div>

              {/* Quick Color Presets */}
              <div class="card bg-base-100 shadow-sm">
                <div class="card-body p-4">
                  <h3 class="font-semibold mb-3">Quick Color Presets</h3>
                  <div class="flex flex-wrap gap-2">
                    <For each={sceneButtons}>
                      {(scene) => (
                        <button
                          class="btn btn-sm"
                          style={{
                            'background-color': scene.color,
                            'border-color': scene.color,
                            'color': scene.name === 'white' || scene.name === 'yellow' ? '#000' : '#fff'
                          }}
                          onClick={() => setScene(scene.name)}
                        >
                          {scene.name}
                        </button>
                      )}
                    </For>
                  </div>
                </div>
              </div>

              {/* Custom Color Presets */}
              <div class="card bg-base-100 shadow-sm">
                <div class="card-body p-4">
                  <h3 class="font-semibold mb-3">Custom Color Presets</h3>

                  {/* Create New Preset */}
                  <div class="form-control mb-4">
                    <div class="flex gap-2">
                      <input
                        type="text"
                        placeholder="Preset name (e.g., sunset, ocean)"
                        class="input input-bordered input-sm flex-1"
                        value={newSceneName()}
                        onInput={(e) => setNewSceneName(e.target.value)}
                      />
                      <input
                        type="color"
                        class="w-12 h-8 rounded cursor-pointer"
                        value={newSceneColor()}
                        onInput={(e) => setNewSceneColor(e.target.value)}
                      />
                      <button
                        class="btn btn-sm btn-primary"
                        onClick={saveCustomScene}
                      >
                        Add
                      </button>
                    </div>
                  </div>

                  {/* Custom Presets List */}
                  <Show
                    when={customScenes().length > 0}
                    fallback={
                      <p class="text-sm text-base-content/60">No custom presets yet. Create one above!</p>
                    }
                  >
                    <div class="flex flex-wrap gap-2">
                      <For each={customScenes()}>
                        {(scene) => (
                          <div class="flex items-center gap-1 bg-base-200 rounded-lg pl-3 pr-1 py-1">
                            <button
                              class="btn btn-xs h-6 min-h-0 px-2"
                              style={{
                                'background-color': `rgb(${scene.red}, ${scene.green}, ${scene.blue})`,
                                'border-color': `rgb(${scene.red}, ${scene.green}, ${scene.blue})`,
                                'color': (scene.red + scene.green + scene.blue) / 3 > 128 ? '#000' : '#fff'
                              }}
                              onClick={() => setScene(scene.name)}
                            >
                              {scene.name}
                            </button>
                            <button
                              class="btn btn-xs btn-ghost h-6 min-h-0 px-1"
                              onClick={() => deleteCustomScene(scene.name)}
                              title="Delete"
                            >
                              ×
                            </button>
                          </div>
                        )}
                      </For>
                    </div>
                  </Show>
                </div>
              </div>

              {/* Individual Lights */}
              <div class="card bg-base-100 shadow-sm">
                <div class="card-body p-4">
                  <div class="flex items-center justify-between mb-3">
                    <h3 class="font-semibold">Lights ({lights().length})</h3>
                    <button class="btn btn-sm btn-ghost gap-2" onClick={loadLights}>
                      <IconRefresh size={16} />
                      Refresh
                    </button>
                  </div>

                  <Show
                    when={lights().length > 0}
                    fallback={
                      <p class="text-sm text-base-content/60">No lights found</p>
                    }
                  >
                    <div class="space-y-3">
                      <For each={lights()}>
                        {(light) => (
                          <div class="card bg-base-200">
                            <div class="card-body p-3">
                              <div class="flex items-center justify-between mb-2">
                                <div class="flex items-center gap-3">
                                  <IconBulb
                                    size={20}
                                    class={light.on ? 'text-warning' : 'text-base-content/30'}
                                  />
                                  <div>
                                    <div class="font-medium">{light.name}</div>
                                    <div class="text-xs text-base-content/60">
                                      {light.reachable ? 'Connected' : 'Unreachable'}
                                    </div>
                                  </div>
                                </div>
                                <input
                                  type="checkbox"
                                  class="toggle toggle-success"
                                  checked={light.on}
                                  onChange={() => toggleLight(light.id, light.on)}
                                  disabled={!light.reachable}
                                />
                              </div>

                              <Show when={light.on && light.brightness !== null}>
                                <div class="form-control">
                                  <label class="label py-1">
                                    <span class="label-text text-xs">Brightness: {Math.round((light.brightness / 254) * 100)}%</span>
                                  </label>
                                  <input
                                    type="range"
                                    min="1"
                                    max="254"
                                    value={light.brightness || 127}
                                    class="range range-xs range-primary"
                                    onInput={(e) => setBrightness(light.id, e.target.value)}
                                  />
                                </div>
                              </Show>
                            </div>
                          </div>
                        )}
                      </For>
                    </div>
                  </Show>
                </div>
              </div>
            </div>
          </Show>
        </Show>
      </div>
    </div>
  );
}
