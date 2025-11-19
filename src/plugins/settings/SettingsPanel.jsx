import { For } from 'solid-js';
import { editorStore, editorActions } from '@/layout/stores/EditorStore.jsx';

const SettingsPanel = () => {
  // Helper functions to convert between RGB and hex
  const rgbToHex = (rgb) => {
    const toHex = (n) => {
      const hex = n.toString(16);
      return hex.length === 1 ? '0' + hex : hex;
    };
    return `#${toHex(rgb.r)}${toHex(rgb.g)}${toHex(rgb.b)}`;
  };

  const hexToRgb = (hex) => {
    const result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex);
    return result ? {
      r: parseInt(result[1], 16),
      g: parseInt(result[2], 16),
      b: parseInt(result[3], 16)
    } : { r: 0, g: 0, b: 0 };
  };

  return (
    <div class="h-full flex flex-col p-4 overflow-y-auto">
      <h2 class="text-lg font-semibold text-base-content mb-4">Application Settings</h2>

      {/* Glass Theme Settings - Only show when dark-glass theme is active */}
      {editorStore.theme === 'dark-glass' && (
        <div class="mb-6">
          <h3 class="text-sm font-semibold text-base-content mb-3">Glass Theme Settings</h3>
          <div class="space-y-4 bg-base-200 p-4 rounded-lg">
            {/* Base 100 Opacity */}
            <div>
              <label class="flex justify-between text-xs text-base-content/80 mb-1">
                <span>Panel Opacity (Base 100)</span>
                <span>{Math.round(editorStore.glassTheme.base100Opacity * 100)}%</span>
              </label>
              <input
                type="range"
                min="0"
                max="1"
                step="0.05"
                value={editorStore.glassTheme.base100Opacity}
                onInput={(e) => editorActions.updateGlassThemeSetting('base100Opacity', parseFloat(e.target.value))}
                class="range range-primary range-sm"
              />
            </div>

            {/* Base 200 Opacity */}
            <div>
              <label class="flex justify-between text-xs text-base-content/80 mb-1">
                <span>Secondary Panel Opacity (Base 200)</span>
                <span>{Math.round(editorStore.glassTheme.base200Opacity * 100)}%</span>
              </label>
              <input
                type="range"
                min="0"
                max="1"
                step="0.05"
                value={editorStore.glassTheme.base200Opacity}
                onInput={(e) => editorActions.updateGlassThemeSetting('base200Opacity', parseFloat(e.target.value))}
                class="range range-primary range-sm"
              />
            </div>

            {/* Base 300 Opacity */}
            <div>
              <label class="flex justify-between text-xs text-base-content/80 mb-1">
                <span>Tertiary Panel Opacity (Base 300)</span>
                <span>{Math.round(editorStore.glassTheme.base300Opacity * 100)}%</span>
              </label>
              <input
                type="range"
                min="0"
                max="1"
                step="0.05"
                value={editorStore.glassTheme.base300Opacity}
                onInput={(e) => editorActions.updateGlassThemeSetting('base300Opacity', parseFloat(e.target.value))}
                class="range range-primary range-sm"
              />
            </div>

            {/* Root Opacity */}
            <div>
              <label class="flex justify-between text-xs text-base-content/80 mb-1">
                <span>Root Background Opacity</span>
                <span>{Math.round(editorStore.glassTheme.rootOpacity * 100)}%</span>
              </label>
              <input
                type="range"
                min="0"
                max="1"
                step="0.05"
                value={editorStore.glassTheme.rootOpacity}
                onInput={(e) => editorActions.updateGlassThemeSetting('rootOpacity', parseFloat(e.target.value))}
                class="range range-primary range-sm"
              />
            </div>

            {/* Blur Amount */}
            <div>
              <label class="flex justify-between text-xs text-base-content/80 mb-1">
                <span>Blur Amount</span>
                <span>{editorStore.glassTheme.blurAmount}px</span>
              </label>
              <input
                type="range"
                min="0"
                max="30"
                step="1"
                value={editorStore.glassTheme.blurAmount}
                onInput={(e) => editorActions.updateGlassThemeSetting('blurAmount', parseInt(e.target.value))}
                class="range range-primary range-sm"
              />
            </div>

            <div class="divider my-2 text-xs">Colors</div>

            {/* Base 100 Color */}
            <div>
              <label class="text-xs text-base-content/80 mb-1 block">Panel Color (Base 100)</label>
              <input
                type="color"
                value={rgbToHex(editorStore.glassTheme.base100Color)}
                onInput={(e) => editorActions.updateGlassThemeSetting('base100Color', hexToRgb(e.target.value))}
                class="w-full h-10 rounded cursor-pointer"
              />
            </div>

            {/* Base 200 Color */}
            <div>
              <label class="text-xs text-base-content/80 mb-1 block">Secondary Panel Color (Base 200)</label>
              <input
                type="color"
                value={rgbToHex(editorStore.glassTheme.base200Color)}
                onInput={(e) => editorActions.updateGlassThemeSetting('base200Color', hexToRgb(e.target.value))}
                class="w-full h-10 rounded cursor-pointer"
              />
            </div>

            {/* Base 300 Color */}
            <div>
              <label class="text-xs text-base-content/80 mb-1 block">Tertiary Panel Color (Base 300)</label>
              <input
                type="color"
                value={rgbToHex(editorStore.glassTheme.base300Color)}
                onInput={(e) => editorActions.updateGlassThemeSetting('base300Color', hexToRgb(e.target.value))}
                class="w-full h-10 rounded cursor-pointer"
              />
            </div>
          </div>
        </div>
      )}

      {/* Power Mode Settings */}
      <div class="mb-6">
        <h3 class="text-sm font-semibold text-base-content mb-3">Power Mode</h3>
        <div class="space-y-3 bg-base-200 p-4 rounded-lg">
          <p class="text-xs text-base-content/70 mb-2">
            Add visual effects when typing in code editors
          </p>

          {/* Enable Power Mode */}
          <div class="flex items-center justify-between">
            <span class="text-sm text-base-content">Enable Power Mode</span>
            <input
              type="checkbox"
              checked={editorStore.powerMode.enabled}
              onChange={(e) => editorActions.updatePowerModeSetting('enabled', e.target.checked)}
              class="toggle toggle-primary"
            />
          </div>

          {/* Show when power mode is enabled */}
          {editorStore.powerMode.enabled && (
            <>
              <div class="divider my-2"></div>

              {/* Particles Toggle */}
              <div class="flex items-center justify-between">
                <span class="text-sm text-base-content">Particles</span>
                <input
                  type="checkbox"
                  checked={editorStore.powerMode.particles}
                  onChange={(e) => editorActions.updatePowerModeSetting('particles', e.target.checked)}
                  class="toggle toggle-primary toggle-sm"
                />
              </div>

              {/* Screen Shake Toggle */}
              <div class="flex items-center justify-between">
                <span class="text-sm text-base-content">Screen Shake</span>
                <input
                  type="checkbox"
                  checked={editorStore.powerMode.shake}
                  onChange={(e) => editorActions.updatePowerModeSetting('shake', e.target.checked)}
                  class="toggle toggle-primary toggle-sm"
                />
              </div>

              {/* Shake Intensity */}
              {editorStore.powerMode.shake && (
                <div>
                  <label class="flex justify-between text-xs text-base-content/80 mb-1">
                    <span>Shake Intensity</span>
                    <span>{editorStore.powerMode.shakeIntensity}</span>
                  </label>
                  <input
                    type="range"
                    min="1"
                    max="10"
                    step="1"
                    value={editorStore.powerMode.shakeIntensity}
                    onInput={(e) => editorActions.updatePowerModeSetting('shakeIntensity', parseInt(e.target.value))}
                    class="range range-primary range-sm"
                  />
                </div>
              )}

              {/* Particle Size */}
              {editorStore.powerMode.particles && (
                <div>
                  <label class="flex justify-between text-xs text-base-content/80 mb-1">
                    <span>Particle Size</span>
                    <span>{editorStore.powerMode.particleSize}px</span>
                  </label>
                  <input
                    type="range"
                    min="2"
                    max="20"
                    step="1"
                    value={editorStore.powerMode.particleSize}
                    onInput={(e) => editorActions.updatePowerModeSetting('particleSize', parseInt(e.target.value))}
                    class="range range-primary range-sm"
                  />
                </div>
              )}

              {/* Max Particles */}
              {editorStore.powerMode.particles && (
                <div>
                  <label class="flex justify-between text-xs text-base-content/80 mb-1">
                    <span>Max Particles</span>
                    <span>{editorStore.powerMode.maxParticles}</span>
                  </label>
                  <input
                    type="range"
                    min="100"
                    max="1000"
                    step="50"
                    value={editorStore.powerMode.maxParticles}
                    onInput={(e) => editorActions.updatePowerModeSetting('maxParticles', parseInt(e.target.value))}
                    class="range range-primary range-sm"
                  />
                </div>
              )}
            </>
          )}
        </div>
      </div>
    </div>
  );
};

export default SettingsPanel;
