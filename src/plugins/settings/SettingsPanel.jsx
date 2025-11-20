import { For, createSignal } from 'solid-js';
import { editorStore, editorActions } from '@/layout/stores/EditorStore.jsx';
import { backgroundLayers, pluginAPI } from '@/api/plugin';
import { IconGripVertical, IconArrowUp, IconArrowDown } from '@tabler/icons-solidjs';

const SettingsPanel = () => {
  const [draggedLayer, setDraggedLayer] = createSignal(null);
  const [dragOverLayer, setDragOverLayer] = createSignal(null);

  // Background layer ordering functions
  const moveLayerUp = (layerId) => {
    const layers = Array.from(backgroundLayers().entries())
      .sort((a, b) => {
        if (a[1].order !== b[1].order) return a[1].order - b[1].order;
        return (a[1].zIndex || 0) - (b[1].zIndex || 0);
      });

    const index = layers.findIndex(([id]) => id === layerId);

    if (index < layers.length - 1) {
      const currentZIndex = layers[index][1].zIndex || 0;
      const aboveZIndex = layers[index + 1][1].zIndex || 0;

      // Swap z-index values only (no re-rendering)
      pluginAPI.updateBackgroundLayerZIndex(layers[index][0], aboveZIndex);
      pluginAPI.updateBackgroundLayerZIndex(layers[index + 1][0], currentZIndex);
    }
  };

  const moveLayerDown = (layerId) => {
    const layers = Array.from(backgroundLayers().entries())
      .sort((a, b) => {
        if (a[1].order !== b[1].order) return a[1].order - b[1].order;
        return (a[1].zIndex || 0) - (b[1].zIndex || 0);
      });

    const index = layers.findIndex(([id]) => id === layerId);
    if (index > 0) {
      const currentZIndex = layers[index][1].zIndex || 0;
      const belowZIndex = layers[index - 1][1].zIndex || 0;

      // Swap z-index values only (no re-rendering)
      pluginAPI.updateBackgroundLayerZIndex(layers[index][0], belowZIndex);
      pluginAPI.updateBackgroundLayerZIndex(layers[index - 1][0], currentZIndex);
    }
  };

  // Drag and drop handlers
  const handleDragStart = (e, layerId) => {
    setDraggedLayer(layerId);
    e.dataTransfer.effectAllowed = 'move';
  };

  const handleDragOver = (e, layerId) => {
    e.preventDefault();
    e.dataTransfer.dropEffect = 'move';
    setDragOverLayer(layerId);
  };

  const handleDragLeave = () => {
    setDragOverLayer(null);
  };

  const handleDrop = (e, targetLayerId) => {
    e.preventDefault();
    const sourceLayerId = draggedLayer();

    if (sourceLayerId && targetLayerId && sourceLayerId !== targetLayerId) {
      const layers = Array.from(backgroundLayers().entries())
        .sort((a, b) => {
          if (a[1].order !== b[1].order) return a[1].order - b[1].order;
          return (a[1].zIndex || 0) - (b[1].zIndex || 0);
        });

      const sourceIndex = layers.findIndex(([id]) => id === sourceLayerId);
      const targetIndex = layers.findIndex(([id]) => id === targetLayerId);

      if (sourceIndex !== -1 && targetIndex !== -1) {
        const sourceZIndex = layers[sourceIndex][1].zIndex || 0;
        const targetZIndex = layers[targetIndex][1].zIndex || 0;

        // Swap z-index values only (no re-rendering)
        pluginAPI.updateBackgroundLayerZIndex(sourceLayerId, targetZIndex);
        pluginAPI.updateBackgroundLayerZIndex(targetLayerId, sourceZIndex);
      }
    }

    setDraggedLayer(null);
    setDragOverLayer(null);
  };

  const handleDragEnd = () => {
    setDraggedLayer(null);
    setDragOverLayer(null);
  };

  return (
    <div class="h-full flex flex-col p-4 overflow-y-auto">
      <h2 class="text-lg font-semibold text-base-content mb-4">Application Settings</h2>

      {/* Background Layers Ordering */}
      <div class="mb-6">
        <h3 class="text-sm font-semibold text-base-content mb-3">Background Layers</h3>
        <div class="space-y-2 bg-base-200 p-4 rounded-lg">
          <p class="text-xs text-base-content/70 mb-2">
            Reorder background layers (bottom = back, top = front)
          </p>
          <For each={Array.from(backgroundLayers().entries()).sort((a, b) => {
            if (a[1].order !== b[1].order) return a[1].order - b[1].order;
            return (a[1].zIndex || 0) - (b[1].zIndex || 0);
          })}>
            {([layerId, layer], index) => (
              <div
                draggable={true}
                onDragStart={(e) => handleDragStart(e, layerId)}
                onDragOver={(e) => handleDragOver(e, layerId)}
                onDragLeave={handleDragLeave}
                onDrop={(e) => handleDrop(e, layerId)}
                onDragEnd={handleDragEnd}
                class="flex items-center gap-2 bg-base-300 p-2 rounded cursor-move transition-all"
                classList={{
                  'opacity-50': draggedLayer() === layerId,
                  'ring-2 ring-primary': dragOverLayer() === layerId && draggedLayer() !== layerId,
                  'scale-105': dragOverLayer() === layerId && draggedLayer() !== layerId
                }}
              >
                <IconGripVertical class="w-4 h-4 text-base-content/40" />
                <div class="flex-1 text-sm text-base-content">
                  {layerId}
                  <span class="text-xs text-base-content/50 ml-2">
                    (order: {layer.order}, z: {layer.zIndex})
                  </span>
                </div>
                <div class="flex gap-1">
                  <button
                    onClick={() => moveLayerUp(layerId)}
                    disabled={index() === 0}
                    class="btn btn-xs btn-ghost"
                    classList={{ 'opacity-30': index() === 0 }}
                    title="Move up (towards front)"
                  >
                    <IconArrowUp class="w-3 h-3" />
                  </button>
                  <button
                    onClick={() => moveLayerDown(layerId)}
                    disabled={index() === backgroundLayers().size - 1}
                    class="btn btn-xs btn-ghost"
                    classList={{ 'opacity-30': index() === backgroundLayers().size - 1 }}
                    title="Move down (towards back)"
                  >
                    <IconArrowDown class="w-3 h-3" />
                  </button>
                </div>
              </div>
            )}
          </For>
        </div>
      </div>

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
