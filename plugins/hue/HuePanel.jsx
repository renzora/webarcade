import { createSignal, onMount, For, Show } from 'solid-js';
import { bridgeFetch } from '@/api/bridge.js';
import { IconBulb, IconBulbOff, IconAlertCircle } from '@tabler/icons-solidjs';

export default function HuePanel() {
  const [configured, setConfigured] = createSignal(false);
  const [loading, setLoading] = createSignal(true);
  const [allLightsOn, setAllLightsOn] = createSignal(false);
  const [customScenes, setCustomScenes] = createSignal([]);
  const [animatedScenes, setAnimatedScenes] = createSignal([]);

  let statusCheckTimeout = null;

  onMount(async () => {
    await loadConfig();
    await loadCustomScenes();
    await loadAnimatedScenes();
    setLoading(false);
  });

  const loadConfig = async () => {
    try {
      const response = await bridgeFetch('/hue/config');
      const data = await response.json();
      setConfigured(data.configured);

      if (data.configured) {
        await checkLightsStatus();
      }
    } catch (e) {
      console.error('Failed to load Hue config:', e);
    }
  };

  const checkLightsStatus = async () => {
    try {
      const response = await bridgeFetch('/hue/lights');
      const lights = await response.json();
      const anyOn = lights.some(light => light.on);
      setAllLightsOn(anyOn);
    } catch (e) {
      console.error('Failed to check lights status:', e);
    }
  };

  const toggleAllLights = async () => {
    const newState = !allLightsOn();

    // Optimistically update UI
    setAllLightsOn(newState);

    try {
      // Get all lights and toggle them
      const response = await bridgeFetch('/hue/lights');
      const lights = await response.json();

      // Toggle all lights in parallel
      await Promise.all(lights.map(light =>
        bridgeFetch('/hue/lights/power', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ light_id: light.id, on: newState }),
        })
      ));
    } catch (e) {
      console.error('Failed to toggle lights:', e);
      // Revert on error
      setAllLightsOn(!newState);
    }
  };

  const setScene = async (scene) => {
    // Optimistically update UI (scenes turn lights on)
    setAllLightsOn(true);

    try {
      await bridgeFetch('/hue/scene', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ scene }),
      });
    } catch (e) {
      console.error('Failed to set scene:', e);
      // Check actual state on error
      if (statusCheckTimeout) clearTimeout(statusCheckTimeout);
      statusCheckTimeout = setTimeout(checkLightsStatus, 500);
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

  const loadAnimatedScenes = async () => {
    try {
      const response = await bridgeFetch('/hue/animated-scenes');
      const data = await response.json();
      setAnimatedScenes(data);
    } catch (e) {
      console.error('Failed to load animated scenes:', e);
    }
  };

  const playAnimatedScene = async (sceneId) => {
    try {
      await bridgeFetch('/hue/animated-scenes/play', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ scene_id: sceneId }),
      });
      setAllLightsOn(true);
    } catch (e) {
      console.error('Failed to play scene:', e);
    }
  };

  const scenes = [
    { name: 'red', label: 'Red', color: '#FF0000' },
    { name: 'orange', label: 'Orange', color: '#FF8800' },
    { name: 'yellow', label: 'Yellow', color: '#FFFF00' },
    { name: 'green', label: 'Green', color: '#00FF00' },
    { name: 'cyan', label: 'Cyan', color: '#00FFFF' },
    { name: 'blue', label: 'Blue', color: '#0000FF' },
    { name: 'purple', label: 'Purple', color: '#8800FF' },
    { name: 'pink', label: 'Pink', color: '#FF00FF' },
    { name: 'white', label: 'White', color: '#FFFFFF' },
  ];

  return (
    <div class="h-full flex flex-col bg-base-200">
      {/* Header */}
      <div class="flex items-center justify-between bg-base-100 border-b border-base-300 px-3 py-2">
        <div class="flex items-center gap-2">
          <IconBulb size={16} class="text-primary" />
          <span class="text-sm font-semibold">Hue Lights</span>
        </div>
      </div>

      <div class="flex-1 overflow-y-auto p-3">
        <Show
          when={!loading()}
          fallback={
            <div class="flex items-center justify-center h-full">
              <span class="loading loading-spinner loading-sm"></span>
            </div>
          }
        >
          <Show
            when={configured()}
            fallback={
              <div class="text-center py-6">
                <IconAlertCircle size={32} class="mx-auto mb-3 opacity-30" />
                <p class="text-xs font-semibold mb-2">Hue Not Configured</p>
                <p class="text-xs text-base-content/60 mb-3">
                  Set up your Philips Hue bridge in the Hue viewport
                </p>
              </div>
            }
          >
            <div class="space-y-3">
              {/* Power Toggle */}
              <div class="card bg-base-100 shadow-sm">
                <div class="card-body p-3">
                  <button
                    class={`btn btn-block gap-2 ${allLightsOn() ? 'btn-warning' : 'btn-ghost'}`}
                    onClick={toggleAllLights}
                  >
                    {allLightsOn() ? <IconBulb size={18} /> : <IconBulbOff size={18} />}
                    {allLightsOn() ? 'Lights On' : 'Lights Off'}
                  </button>
                </div>
              </div>

              {/* Quick Presets */}
              <div class="card bg-base-100 shadow-sm">
                <div class="card-body p-3">
                  <h4 class="text-xs font-semibold mb-2">Quick Presets</h4>
                  <div class="grid grid-cols-3 gap-2">
                    <For each={scenes}>
                      {(scene) => (
                        <button
                          class="btn btn-xs h-12 flex flex-col gap-1 p-1"
                          style={{
                            'background-color': scene.color,
                            'border-color': scene.color,
                            'color': scene.name === 'white' || scene.name === 'yellow' ? '#000' : '#fff'
                          }}
                          onClick={() => setScene(scene.name)}
                          title={scene.label}
                        >
                          <div
                            class="w-6 h-6 rounded-full border-2"
                            style={{
                              'background-color': scene.color,
                              'border-color': scene.name === 'white' || scene.name === 'yellow' ? '#000' : '#fff'
                            }}
                          />
                          <span class="text-[9px] font-semibold">{scene.label}</span>
                        </button>
                      )}
                    </For>
                  </div>
                </div>
              </div>

              {/* Custom Presets */}
              <Show when={customScenes().length > 0}>
                <div class="card bg-base-100 shadow-sm">
                  <div class="card-body p-3">
                    <h4 class="text-xs font-semibold mb-2">Custom Presets</h4>
                    <div class="grid grid-cols-3 gap-2">
                      <For each={customScenes()}>
                        {(scene) => (
                          <button
                            class="btn btn-xs h-12 flex flex-col gap-1 p-1"
                            style={{
                              'background-color': `rgb(${scene.red}, ${scene.green}, ${scene.blue})`,
                              'border-color': `rgb(${scene.red}, ${scene.green}, ${scene.blue})`,
                              'color': (scene.red + scene.green + scene.blue) / 3 > 128 ? '#000' : '#fff'
                            }}
                            onClick={() => setScene(scene.name)}
                            title={scene.name}
                          >
                            <div
                              class="w-6 h-6 rounded-full border-2"
                              style={{
                                'background-color': `rgb(${scene.red}, ${scene.green}, ${scene.blue})`,
                                'border-color': (scene.red + scene.green + scene.blue) / 3 > 128 ? '#000' : '#fff'
                              }}
                            />
                            <span class="text-[9px] font-semibold truncate w-full">{scene.name}</span>
                          </button>
                        )}
                      </For>
                    </div>
                  </div>
                </div>
              </Show>

              {/* Animated Scenes */}
              <Show when={animatedScenes().length > 0}>
                <div class="card bg-base-100 shadow-sm">
                  <div class="card-body p-3">
                    <h4 class="text-xs font-semibold mb-2">Animated Scenes</h4>
                    <div class="space-y-2">
                      <For each={animatedScenes()}>
                        {(scene) => (
                          <button
                            class="btn btn-xs btn-block justify-start gap-2 h-auto py-2"
                            onClick={() => playAnimatedScene(scene.id)}
                          >
                            <span class="font-mono text-[10px] bg-base-300 px-1.5 py-0.5 rounded">
                              !{scene.tag}
                            </span>
                            <span class="text-[11px] truncate flex-1 text-left">
                              {scene.name}
                            </span>
                            <span class="text-[9px] text-base-content/60">
                              {scene.steps.length} colors
                            </span>
                          </button>
                        )}
                      </For>
                    </div>
                  </div>
                </div>
              </Show>
            </div>
          </Show>
        </Show>
      </div>
    </div>
  );
}
