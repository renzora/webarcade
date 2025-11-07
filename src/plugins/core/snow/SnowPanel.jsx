import { createSignal, createEffect, onCleanup } from 'solid-js';
import {
  IconSnowflake,
  IconPlayerPlay,
  IconPlayerPause,
  IconTrash,
  IconSettings,
  IconWind
} from '@tabler/icons-solidjs';

const BRIDGE_URL = 'http://localhost:3001';

export default function SnowPanel() {
  const [enabled, setEnabled] = createSignal(true);
  const [intensity, setIntensity] = createSignal(50);
  const [speed, setSpeed] = createSignal(1);
  const [windSpeed, setWindSpeed] = createSignal(0.5);
  const [minSize, setMinSize] = createSignal(2);
  const [maxSize, setMaxSize] = createSignal(8);
  const [minOpacity, setMinOpacity] = createSignal(0.3);
  const [maxOpacity, setMaxOpacity] = createSignal(0.8);

  // Send settings update to overlay
  const broadcastSettings = () => {
    fetch(`${BRIDGE_URL}/system/broadcast`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        type: 'snow_overlay_update',
        settings: {
          enabled: enabled(),
          intensity: intensity(),
          speed: speed(),
          windSpeed: windSpeed(),
          size: { min: minSize(), max: maxSize() },
          opacity: { min: minOpacity(), max: maxOpacity() }
        }
      })
    }).catch(err => console.error('Failed to broadcast settings:', err));
  };

  // Toggle enabled state
  const toggleEnabled = () => {
    setEnabled(!enabled());
    broadcastSettings();
  };

  // Update intensity
  const updateIntensity = (value) => {
    setIntensity(value);
    broadcastSettings();
  };

  // Update speed
  const updateSpeed = (value) => {
    setSpeed(value);
    broadcastSettings();
  };

  // Update wind speed
  const updateWindSpeed = (value) => {
    setWindSpeed(value);
    broadcastSettings();
  };

  // Update min size
  const updateMinSize = (value) => {
    setMinSize(value);
    // Ensure max size is always >= min size
    if (maxSize() < value) {
      setMaxSize(value);
    }
    broadcastSettings();
  };

  // Update max size
  const updateMaxSize = (value) => {
    setMaxSize(value);
    // Ensure min size is always <= max size
    if (minSize() > value) {
      setMinSize(value);
    }
    broadcastSettings();
  };

  // Update min opacity
  const updateMinOpacity = (value) => {
    setMinOpacity(value);
    // Ensure max opacity is always >= min opacity
    if (maxOpacity() < value) {
      setMaxOpacity(value);
    }
    broadcastSettings();
  };

  // Update max opacity
  const updateMaxOpacity = (value) => {
    setMaxOpacity(value);
    // Ensure min opacity is always <= max opacity
    if (minOpacity() > value) {
      setMinOpacity(value);
    }
    broadcastSettings();
  };

  // Clear all snowflakes
  const clearSnow = () => {
    fetch(`${BRIDGE_URL}/system/broadcast`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        type: 'snow_overlay_clear'
      })
    }).catch(err => console.error('Failed to clear snow:', err));
  };

  // Apply preset: Light Snow
  const applyLightPreset = () => {
    setIntensity(30);
    setSpeed(0.8);
    setWindSpeed(0.3);
    setMinSize(2);
    setMaxSize(5);
    setMinOpacity(0.4);
    setMaxOpacity(0.7);
    broadcastSettings();
  };

  // Apply preset: Heavy Snow
  const applyHeavyPreset = () => {
    setIntensity(100);
    setSpeed(1.5);
    setWindSpeed(0.7);
    setMinSize(3);
    setMaxSize(10);
    setMinOpacity(0.5);
    setMaxOpacity(0.9);
    broadcastSettings();
  };

  // Apply preset: Blizzard
  const applyBlizzardPreset = () => {
    setIntensity(150);
    setSpeed(2);
    setWindSpeed(1.5);
    setMinSize(2);
    setMaxSize(8);
    setMinOpacity(0.3);
    setMaxOpacity(0.8);
    broadcastSettings();
  };

  return (
    <div class="h-full flex flex-col bg-base-200">
      {/* Header */}
      <div class="p-4 bg-base-100 border-b border-base-300">
        <h2 class="text-lg font-semibold flex items-center gap-2">
          <IconSnowflake size={24} />
          Snow Overlay Control
        </h2>
      </div>

      {/* Controls */}
      <div class="flex-1 overflow-y-auto p-4 space-y-4">
        {/* Enable/Disable Toggle */}
        <div class="card bg-base-100 shadow-sm">
          <div class="card-body p-4">
            <h3 class="font-semibold mb-3">Status</h3>
            <div class="flex items-center justify-between">
              <span class="text-sm">Snow Overlay Enabled</span>
              <button
                class={`btn btn-sm gap-2 ${enabled() ? 'btn-error' : 'btn-success'}`}
                onClick={toggleEnabled}
              >
                {enabled() ? (
                  <>
                    <IconPlayerPause size={18} />
                    Stop
                  </>
                ) : (
                  <>
                    <IconPlayerPlay size={18} />
                    Start
                  </>
                )}
              </button>
            </div>
          </div>
        </div>

        {/* Presets */}
        <div class="card bg-base-100 shadow-sm">
          <div class="card-body p-4">
            <h3 class="font-semibold mb-3">Quick Presets</h3>
            <div class="grid grid-cols-3 gap-2">
              <button
                class="btn btn-sm btn-outline"
                onClick={applyLightPreset}
              >
                Light
              </button>
              <button
                class="btn btn-sm btn-outline"
                onClick={applyHeavyPreset}
              >
                Heavy
              </button>
              <button
                class="btn btn-sm btn-outline"
                onClick={applyBlizzardPreset}
              >
                Blizzard
              </button>
            </div>
          </div>
        </div>

        {/* Settings */}
        <div class="card bg-base-100 shadow-sm">
          <div class="card-body p-4">
            <h3 class="font-semibold mb-3 flex items-center gap-2">
              <IconSettings size={20} />
              Snow Settings
            </h3>

            {/* Intensity */}
            <div class="form-control">
              <label class="label">
                <span class="label-text">Intensity (Snowflakes)</span>
                <span class="label-text-alt">{intensity()}</span>
              </label>
              <input
                type="range"
                min="10"
                max="200"
                step="10"
                value={intensity()}
                onInput={(e) => updateIntensity(parseFloat(e.target.value))}
                class="range range-sm range-primary"
              />
              <div class="w-full flex justify-between text-xs px-2 text-base-content/60">
                <span>Light</span>
                <span>Heavy</span>
              </div>
            </div>

            {/* Speed */}
            <div class="form-control mt-4">
              <label class="label">
                <span class="label-text">Fall Speed</span>
                <span class="label-text-alt">{speed().toFixed(1)}x</span>
              </label>
              <input
                type="range"
                min="0.2"
                max="3"
                step="0.1"
                value={speed()}
                onInput={(e) => updateSpeed(parseFloat(e.target.value))}
                class="range range-sm range-secondary"
              />
              <div class="w-full flex justify-between text-xs px-2 text-base-content/60">
                <span>Slow</span>
                <span>Fast</span>
              </div>
            </div>

            {/* Wind Speed */}
            <div class="form-control mt-4">
              <label class="label">
                <span class="label-text flex items-center gap-1">
                  <IconWind size={16} />
                  Wind Speed
                </span>
                <span class="label-text-alt">{windSpeed().toFixed(1)}</span>
              </label>
              <input
                type="range"
                min="0"
                max="2"
                step="0.1"
                value={windSpeed()}
                onInput={(e) => updateWindSpeed(parseFloat(e.target.value))}
                class="range range-sm range-accent"
              />
              <div class="w-full flex justify-between text-xs px-2 text-base-content/60">
                <span>Calm</span>
                <span>Windy</span>
              </div>
            </div>

            {/* Size Range */}
            <div class="form-control mt-4">
              <label class="label">
                <span class="label-text">Min Size</span>
                <span class="label-text-alt">{minSize()}px</span>
              </label>
              <input
                type="range"
                min="1"
                max="10"
                step="1"
                value={minSize()}
                onInput={(e) => updateMinSize(parseFloat(e.target.value))}
                class="range range-sm range-info"
              />
            </div>

            <div class="form-control mt-2">
              <label class="label">
                <span class="label-text">Max Size</span>
                <span class="label-text-alt">{maxSize()}px</span>
              </label>
              <input
                type="range"
                min="1"
                max="15"
                step="1"
                value={maxSize()}
                onInput={(e) => updateMaxSize(parseFloat(e.target.value))}
                class="range range-sm range-info"
              />
            </div>

            {/* Opacity Range */}
            <div class="form-control mt-4">
              <label class="label">
                <span class="label-text">Min Opacity</span>
                <span class="label-text-alt">{(minOpacity() * 100).toFixed(0)}%</span>
              </label>
              <input
                type="range"
                min="0.1"
                max="1"
                step="0.05"
                value={minOpacity()}
                onInput={(e) => updateMinOpacity(parseFloat(e.target.value))}
                class="range range-sm range-warning"
              />
            </div>

            <div class="form-control mt-2">
              <label class="label">
                <span class="label-text">Max Opacity</span>
                <span class="label-text-alt">{(maxOpacity() * 100).toFixed(0)}%</span>
              </label>
              <input
                type="range"
                min="0.1"
                max="1"
                step="0.05"
                value={maxOpacity()}
                onInput={(e) => updateMaxOpacity(parseFloat(e.target.value))}
                class="range range-sm range-warning"
              />
            </div>
          </div>
        </div>

        {/* Actions */}
        <div class="card bg-base-100 shadow-sm">
          <div class="card-body p-4">
            <h3 class="font-semibold mb-3">Actions</h3>
            <button
              class="btn btn-error btn-block gap-2"
              onClick={clearSnow}
            >
              <IconTrash size={18} />
              Clear All Snowflakes
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
