import { For, onMount, createSignal, Show } from 'solid-js';
import { IconPlus, IconTrash, IconGripVertical, IconLetterT, IconPhoto } from '@tabler/icons-solidjs';
import { layoutManagerStore, OVERLAY_DEFAULTS } from './LayoutManagerStore';

export default function LayoutManagerPanel() {
  const [draggedIndex, setDraggedIndex] = createSignal(null);

  // Initialize store on mount
  onMount(() => {
    layoutManagerStore.init();
  });

  // Drag and drop handlers
  const handleDragStart = (e, index) => {
    setDraggedIndex(index);
    e.dataTransfer.effectAllowed = 'move';
  };

  const handleDragOver = (e) => {
    e.preventDefault();
    e.dataTransfer.dropEffect = 'move';
  };

  const handleDrop = (e, dropIndex) => {
    e.preventDefault();
    const dragIndex = draggedIndex();
    if (dragIndex !== null && dragIndex !== dropIndex) {
      layoutManagerStore.reorderOverlays(dragIndex, dropIndex);
    }
    setDraggedIndex(null);
  };

  const handleDragEnd = () => {
    setDraggedIndex(null);
  };

  // Get overlay display name
  const getOverlayName = (overlay) => {
    return OVERLAY_DEFAULTS[overlay.type]?.name || overlay.type;
  };

  // Get sorted overlays by zIndex (highest first = top layer)
  const sortedOverlays = () => {
    return [...layoutManagerStore.overlaysInLayout()].sort((a, b) => b.zIndex - a.zIndex);
  };

  return (
    <div class="h-full bg-base-200 p-4 overflow-y-auto">
      {/* Custom Layers */}
      <h3 class="font-bold text-lg mb-4">Custom Layers</h3>
      <div class="grid grid-cols-2 gap-2 mb-6">
        <button
          class="btn btn-sm btn-outline flex-col h-auto py-3 px-2 border-base-content/20 hover:border-blue-500"
          onClick={() => layoutManagerStore.addOverlay('text')}
          title="Add Text Layer"
        >
          <IconLetterT size={14} class="mb-1" />
          <div class="text-xs font-semibold leading-tight text-center">
            Text
          </div>
          <div class="text-[10px] opacity-60 mt-1">
            400×100
          </div>
        </button>
        <button
          class="btn btn-sm btn-outline flex-col h-auto py-3 px-2 border-base-content/20 hover:border-purple-500"
          onClick={() => layoutManagerStore.addOverlay('image')}
          title="Add Image Layer"
        >
          <IconPhoto size={14} class="mb-1" />
          <div class="text-xs font-semibold leading-tight text-center">
            Image
          </div>
          <div class="text-[10px] opacity-60 mt-1">
            400×400
          </div>
        </button>
      </div>

      <div class="divider"></div>

      {/* Overlay Library */}
      <h3 class="font-bold text-lg mb-4">Available Overlays</h3>
      <div class="grid grid-cols-3 gap-2 mb-6">
        <For each={layoutManagerStore.availableOverlays()}>
          {(overlay) => {
            const defaults = OVERLAY_DEFAULTS[overlay];
            return (
              <button
                class="btn btn-sm btn-outline flex-col h-auto py-3 px-2 border-base-content/20 hover:border-green-500"
                onClick={() => layoutManagerStore.addOverlay(overlay)}
                title={`${defaults?.width}×${defaults?.height}`}
              >
                <IconPlus size={14} class="mb-1" />
                <div class="text-xs font-semibold leading-tight text-center">
                  {defaults?.name || overlay}
                </div>
                <div class="text-[10px] opacity-60 mt-1">
                  {defaults?.width}×{defaults?.height}
                </div>
              </button>
            );
          }}
        </For>
      </div>

      {/* Layers */}
      <Show when={layoutManagerStore.overlaysInLayout().length > 0}>
        <div class="divider"></div>
        <h3 class="font-bold text-lg mb-4">Layers</h3>
        <div class="text-xs text-base-content/60 mb-2">Drag to reorder • Top = Front</div>
        <div class="flex flex-col gap-1">
          <For each={sortedOverlays()}>
            {(overlay, index) => (
              <div
                draggable={true}
                onDragStart={(e) => handleDragStart(e, index())}
                onDragOver={handleDragOver}
                onDrop={(e) => handleDrop(e, index())}
                onDragEnd={handleDragEnd}
                onClick={() => layoutManagerStore.setSelectedOverlay(overlay)}
                class={`flex items-center gap-2 p-2 rounded cursor-pointer transition-colors ${
                  draggedIndex() === index()
                    ? 'opacity-50'
                    : layoutManagerStore.selectedOverlay()?.id === overlay.id
                    ? 'bg-primary text-primary-content ring-2 ring-primary'
                    : 'bg-base-300 hover:bg-base-100'
                }`}
              >
                <IconGripVertical size={16} class="text-current opacity-40" />
                <div class="flex-1 truncate">
                  <div class="font-medium text-sm">{getOverlayName(overlay)}</div>
                  <div class="text-xs opacity-60">
                    {overlay.width} × {overlay.height} • z:{overlay.zIndex}
                  </div>
                </div>
                <button
                  class="btn btn-xs btn-ghost btn-circle"
                  onClick={(e) => {
                    e.stopPropagation();
                    layoutManagerStore.removeOverlayById(overlay.id);
                  }}
                  title="Remove layer"
                >
                  <IconTrash size={12} />
                </button>
              </div>
            )}
          </For>
        </div>
      </Show>
    </div>
  );
}
