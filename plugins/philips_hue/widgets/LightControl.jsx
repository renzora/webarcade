import { createSignal, onMount, For, Show, createEffect } from 'solid-js';
import { IconBulb, IconBulbOff, IconRefresh, IconChevronLeft } from '@tabler/icons-solidjs';

export default function LightControl() {
  const [lights, setLights] = createSignal([]);
  const [loading, setLoading] = createSignal(false);
  const [selectedLight, setSelectedLight] = createSignal(null);

  const loadLights = async () => {
    setLoading(true);
    try {
      const response = await fetch('http://localhost:3001/philips-hue/lights');
      const data = await response.json();
      setLights(data.lights || []);
    } catch (error) {
      console.error('Failed to load lights:', error);
    } finally {
      setLoading(false);
    }
  };

  const toggleLight = async (light) => {
    try {
      const newState = !light.state.on;
      await fetch(`http://localhost:3001/philips-hue/lights/${light.id}/state`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ on: newState }),
      });

      // Update local state
      setLights(lights().map(l =>
        l.id === light.id
          ? { ...l, state: { ...l.state, on: newState } }
          : l
      ));

      // Update selected light if it's the one being toggled
      if (selectedLight()?.id === light.id) {
        setSelectedLight({ ...light, state: { ...light.state, on: newState } });
      }
    } catch (error) {
      console.error('Failed to toggle light:', error);
    }
  };

  const setBrightness = async (light, brightness) => {
    try {
      await fetch(`http://localhost:3001/philips-hue/lights/${light.id}/state`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ on: true, bri: brightness }),
      });

      // Update local state
      setLights(lights().map(l =>
        l.id === light.id
          ? { ...l, state: { ...l.state, on: true, bri: brightness } }
          : l
      ));

      if (selectedLight()?.id === light.id) {
        setSelectedLight({ ...light, state: { ...light.state, on: true, bri: brightness } });
      }
    } catch (error) {
      console.error('Failed to set brightness:', error);
    }
  };

  const setColorFromPicker = async (light, hexColor) => {
    // Convert hex to HSL
    const r = parseInt(hexColor.slice(1, 3), 16) / 255;
    const g = parseInt(hexColor.slice(3, 5), 16) / 255;
    const b = parseInt(hexColor.slice(5, 7), 16) / 255;

    const max = Math.max(r, g, b);
    const min = Math.min(r, g, b);
    const delta = max - min;

    let h = 0;
    let s = 0;

    if (delta !== 0) {
      s = delta / max;

      if (max === r) {
        h = ((g - b) / delta) % 6;
      } else if (max === g) {
        h = (b - r) / delta + 2;
      } else {
        h = (r - g) / delta + 4;
      }

      h = Math.round(h * 60);
      if (h < 0) h += 360;
    }

    // Convert to Hue values (0-65535 for hue, 0-254 for saturation)
    const hueValue = Math.round((h / 360) * 65535);
    const satValue = Math.round(s * 254);

    try {
      await fetch(`http://localhost:3001/philips-hue/lights/${light.id}/state`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ on: true, hue: hueValue, sat: satValue }),
      });

      // Update local state
      setLights(lights().map(l =>
        l.id === light.id
          ? { ...l, state: { ...l.state, on: true, hue: hueValue, sat: satValue } }
          : l
      ));

      if (selectedLight()?.id === light.id) {
        setSelectedLight({ ...light, state: { ...light.state, on: true, hue: hueValue, sat: satValue } });
      }
    } catch (error) {
      console.error('Failed to set color:', error);
    }
  };

  const setColorTemp = async (light, ct) => {
    try {
      await fetch(`http://localhost:3001/philips-hue/lights/${light.id}/state`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ on: true, ct: parseInt(ct) }),
      });

      setLights(lights().map(l =>
        l.id === light.id
          ? { ...l, state: { ...l.state, on: true, ct: parseInt(ct) } }
          : l
      ));

      if (selectedLight()?.id === light.id) {
        setSelectedLight({ ...light, state: { ...light.state, on: true, ct: parseInt(ct) } });
      }
    } catch (error) {
      console.error('Failed to set color temperature:', error);
    }
  };

  const getHueColor = (hue, sat = 254) => {
    if (!hue) return '#ffffff';
    const h = (hue / 65535) * 360;
    const s = (sat / 254) * 100;
    return `hsl(${h}, ${s}%, 50%)`;
  };

  const hueToHex = (hue, sat = 254) => {
    if (!hue) return '#ffffff';
    const h = (hue / 65535) * 360;
    const s = (sat / 254) / 100;
    const l = 0.5;

    const c = (1 - Math.abs(2 * l - 1)) * s;
    const x = c * (1 - Math.abs((h / 60) % 2 - 1));
    const m = l - c / 2;

    let r = 0, g = 0, b = 0;

    if (h >= 0 && h < 60) {
      r = c; g = x; b = 0;
    } else if (h >= 60 && h < 120) {
      r = x; g = c; b = 0;
    } else if (h >= 120 && h < 180) {
      r = 0; g = c; b = x;
    } else if (h >= 180 && h < 240) {
      r = 0; g = x; b = c;
    } else if (h >= 240 && h < 300) {
      r = x; g = 0; b = c;
    } else {
      r = c; g = 0; b = x;
    }

    const toHex = (val) => {
      const hex = Math.round((val + m) * 255).toString(16);
      return hex.length === 1 ? '0' + hex : hex;
    };

    return `#${toHex(r)}${toHex(g)}${toHex(b)}`;
  };

  onMount(() => {
    loadLights();
  });

  return (
    <div class="card bg-gradient-to-br from-amber-500/20 to-amber-500/5 bg-base-100 shadow-lg h-full flex flex-col p-3 overflow-hidden">
      {/* Header */}
      <div class="flex items-center justify-between mb-2 flex-shrink-0">
        <div class="flex items-center gap-1.5">
          <Show when={selectedLight()}>
            <button
              class="btn btn-xs btn-ghost btn-square"
              onClick={() => setSelectedLight(null)}
              title="Back to list"
            >
              <IconChevronLeft size={14} />
            </button>
          </Show>
          <IconBulb size={16} class="text-amber-500 opacity-80" />
          <span class="text-xs font-medium opacity-70">
            {selectedLight() ? selectedLight().name : 'Light Control'}
          </span>
        </div>
        <button
          class="btn btn-xs btn-ghost"
          onClick={loadLights}
          disabled={loading()}
          title="Refresh"
        >
          <IconRefresh size={14} />
        </button>
      </div>

      <div class="flex-1 overflow-y-auto min-h-0">
        {/* List View */}
        <Show when={!selectedLight()}>
          {loading() && lights().length === 0 ? (
            <div class="text-center text-xs opacity-50 py-4">Loading...</div>
          ) : lights().length === 0 ? (
            <div class="text-center text-xs opacity-50 py-4">
              No lights found. Set up your Hue Bridge first.
            </div>
          ) : (
            <For each={lights()}>
              {(light) => (
                <div
                  class="bg-base-200 rounded-lg p-3 mb-2 cursor-pointer hover:bg-base-300 transition-colors"
                  onClick={() => setSelectedLight(light)}
                >
                  <div class="flex items-center justify-between">
                    <div class="flex items-center gap-2 flex-1">
                      {light.state.on ? (
                        <IconBulb
                          size={20}
                          class="text-amber-400"
                          style={{ color: light.state.hue ? getHueColor(light.state.hue, light.state.sat) : undefined }}
                        />
                      ) : (
                        <IconBulbOff size={20} class="opacity-50" />
                      )}
                      <div class="flex-1">
                        <div class="font-medium text-sm">{light.name}</div>
                        <div class="text-xs opacity-50">{light.type}</div>
                      </div>
                    </div>
                    <div class="flex items-center gap-2">
                      <Show when={light.state.on && light.state.bri !== null}>
                        <span class="text-xs opacity-70">
                          {Math.round((light.state.bri / 254) * 100)}%
                        </span>
                      </Show>
                      <button
                        class={`btn btn-xs ${light.state.on ? 'btn-warning' : 'btn-outline'}`}
                        onClick={(e) => {
                          e.stopPropagation();
                          toggleLight(light);
                        }}
                      >
                        {light.state.on ? 'ON' : 'OFF'}
                      </button>
                    </div>
                  </div>
                  {!light.reachable && (
                    <div class="text-xs text-error mt-1">⚠ Unreachable</div>
                  )}
                </div>
              )}
            </For>
          )}
        </Show>

        {/* Detail View */}
        <Show when={selectedLight()}>
          {(light) => (
            <div class="space-y-3">
              {/* Status */}
              <div class="bg-base-200 rounded-lg p-3 text-center">
                {light().state.on ? (
                  <IconBulb
                    size={48}
                    class="text-amber-400 mx-auto mb-2"
                    style={{ color: light().state.hue ? getHueColor(light().state.hue, light().state.sat) : undefined }}
                  />
                ) : (
                  <IconBulbOff size={48} class="opacity-50 mx-auto mb-2" />
                )}
                <div class="font-medium">{light().name}</div>
                <div class="text-xs opacity-50">{light().type}</div>
                <button
                  class={`btn btn-sm mt-2 w-full ${light().state.on ? 'btn-warning' : 'btn-outline'}`}
                  onClick={() => toggleLight(light())}
                >
                  {light().state.on ? 'Turn Off' : 'Turn On'}
                </button>
              </div>

              {/* Brightness Control */}
              <Show when={light().state.on && light().state.bri !== null}>
                <div class="bg-base-200 rounded-lg p-3">
                  <label class="label py-0">
                    <span class="label-text text-sm font-medium">Brightness</span>
                    <span class="label-text text-sm opacity-70">
                      {Math.round((light().state.bri / 254) * 100)}%
                    </span>
                  </label>
                  <input
                    type="range"
                    min="1"
                    max="254"
                    value={light().state.bri || 254}
                    class="range range-warning mt-2"
                    onInput={(e) => setBrightness(light(), parseInt(e.target.value))}
                  />
                </div>
              </Show>

              {/* Color Picker for Color Lights */}
              <Show when={light().state.on && light().state.hue !== null}>
                <div class="bg-base-200 rounded-lg p-3">
                  <label class="label py-0 mb-2">
                    <span class="label-text text-sm font-medium">Color</span>
                  </label>

                  {/* Full Color Picker */}
                  <div class="relative">
                    <input
                      type="color"
                      value={hueToHex(light().state.hue, light().state.sat)}
                      class="w-full h-32 cursor-pointer rounded-lg"
                      style={{
                        appearance: 'none',
                        '-webkit-appearance': 'none',
                        border: 'none',
                        cursor: 'pointer'
                      }}
                      onInput={(e) => setColorFromPicker(light(), e.target.value)}
                    />
                  </div>

                  {/* Quick Color Presets */}
                  <div class="grid grid-cols-6 gap-2 mt-3">
                    <button
                      class="btn btn-sm btn-square"
                      style={{ background: '#ff0000', border: 'none' }}
                      onClick={() => setColorFromPicker(light(), '#ff0000')}
                      title="Red"
                    />
                    <button
                      class="btn btn-sm btn-square"
                      style={{ background: '#ff8800', border: 'none' }}
                      onClick={() => setColorFromPicker(light(), '#ff8800')}
                      title="Orange"
                    />
                    <button
                      class="btn btn-sm btn-square"
                      style={{ background: '#ffff00', border: 'none' }}
                      onClick={() => setColorFromPicker(light(), '#ffff00')}
                      title="Yellow"
                    />
                    <button
                      class="btn btn-sm btn-square"
                      style={{ background: '#00ff00', border: 'none' }}
                      onClick={() => setColorFromPicker(light(), '#00ff00')}
                      title="Green"
                    />
                    <button
                      class="btn btn-sm btn-square"
                      style={{ background: '#00ffff', border: 'none' }}
                      onClick={() => setColorFromPicker(light(), '#00ffff')}
                      title="Cyan"
                    />
                    <button
                      class="btn btn-sm btn-square"
                      style={{ background: '#0000ff', border: 'none' }}
                      onClick={() => setColorFromPicker(light(), '#0000ff')}
                      title="Blue"
                    />
                    <button
                      class="btn btn-sm btn-square"
                      style={{ background: '#8800ff', border: 'none' }}
                      onClick={() => setColorFromPicker(light(), '#8800ff')}
                      title="Purple"
                    />
                    <button
                      class="btn btn-sm btn-square"
                      style={{ background: '#ff00ff', border: 'none' }}
                      onClick={() => setColorFromPicker(light(), '#ff00ff')}
                      title="Magenta"
                    />
                    <button
                      class="btn btn-sm btn-square"
                      style={{ background: '#ff0088', border: 'none' }}
                      onClick={() => setColorFromPicker(light(), '#ff0088')}
                      title="Pink"
                    />
                    <button
                      class="btn btn-sm btn-square"
                      style={{ background: '#ffffff', border: '1px solid #666' }}
                      onClick={() => setColorFromPicker(light(), '#ffffff')}
                      title="White"
                    />
                    <button
                      class="btn btn-sm btn-square"
                      style={{ background: '#ffa500', border: 'none' }}
                      onClick={() => setColorFromPicker(light(), '#ffa500')}
                      title="Warm White"
                    />
                    <button
                      class="btn btn-sm btn-square"
                      style={{ background: '#87ceeb', border: 'none' }}
                      onClick={() => setColorFromPicker(light(), '#87ceeb')}
                      title="Cool White"
                    />
                  </div>
                </div>
              </Show>

              {/* White Temperature for White Ambiance */}
              <Show when={light().state.on && light().state.ct !== null && light().state.hue === null}>
                <div class="bg-base-200 rounded-lg p-3">
                  <label class="label py-0">
                    <span class="label-text text-sm font-medium">Temperature</span>
                    <span class="label-text text-sm opacity-70">
                      {light().state.ct}K
                    </span>
                  </label>
                  <input
                    type="range"
                    min="153"
                    max="500"
                    value={light().state.ct || 326}
                    class="range range-sm mt-2"
                    onInput={(e) => setColorTemp(light(), e.target.value)}
                  />
                  <div class="flex justify-between text-xs opacity-50 mt-1">
                    <span>Cool</span>
                    <span>Warm</span>
                  </div>
                </div>
              </Show>

              {!light().reachable && (
                <div class="alert alert-error alert-sm">
                  <span class="text-xs">⚠ Light is unreachable</span>
                </div>
              )}
            </div>
          )}
        </Show>
      </div>

      {/* Info */}
      <div class="text-xs opacity-50 mt-2 border-t border-base-300 pt-2 flex-shrink-0">
        💡 {selectedLight() ? 'Click back to see all lights' : 'Click a light to control it'}
      </div>
    </div>
  );
}
