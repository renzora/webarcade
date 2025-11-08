import { editorActions } from '@/layout/stores/EditorStore';
import { onMount } from 'solid-js';

const BRIDGE_URL = 'http://localhost:3001';

export default function SnowViewport() {
  let iframeRef;

  // Auto-select the snow-panel when this viewport loads
  onMount(() => {
    editorActions.setScenePanelOpen(true);
    editorActions.setSelectedTool('snow-panel');
  });

  return (
    <div class="h-full flex flex-col bg-base-200">
      {/* Header */}
      <div class="p-4 bg-base-100 border-b border-base-300 flex items-center justify-between">
        <div class="flex items-center gap-4">
          <span class="font-semibold">Snow Overlay Preview</span>
          <div class="badge badge-primary">Live Preview</div>
        </div>
      </div>

      {/* Iframe Preview */}
      <div class="flex-1 relative bg-gradient-to-b from-slate-800 to-slate-900">
        <iframe
          ref={iframeRef}
          src={`${BRIDGE_URL}/overlay/snow`}
          class="absolute inset-0 w-full h-full border-none"
          title="Snow Overlay Preview"
        />
      </div>

      {/* Info Bar */}
      <div class="p-2 bg-base-100 border-t border-base-300 text-xs text-base-content/60">
        Preview of falling snow overlay. Use the Snow panel to control settings.
      </div>
    </div>
  );
}
