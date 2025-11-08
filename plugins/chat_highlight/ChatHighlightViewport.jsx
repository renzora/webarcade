import { editorActions } from '@/layout/stores/EditorStore';
import { onMount } from 'solid-js';

const BRIDGE_URL = 'http://localhost:3001';

export default function ChatHighlightViewport() {
  let iframeRef;

  // Auto-select the chat-highlight-panel when this viewport loads
  onMount(() => {
    editorActions.setScenePanelOpen(true);
    editorActions.setSelectedTool('chat-highlight-panel');
  });

  return (
    <div class="h-full flex flex-col bg-base-200">
      {/* Header */}
      <div class="p-4 bg-base-100 border-b border-base-300 flex items-center justify-between">
        <div class="flex items-center gap-4">
          <span class="font-semibold">Chat Highlight Preview</span>
          <div class="badge badge-primary">Live Preview</div>
        </div>
      </div>

      {/* Iframe Preview */}
      <div class="flex-1 relative bg-base-300">
        <iframe
          ref={iframeRef}
          src={`${BRIDGE_URL}/overlay/chat-highlight`}
          class="absolute inset-0 w-full h-full border-none"
          title="Chat Highlight Preview"
        />
      </div>

      {/* Info Bar */}
      <div class="p-2 bg-base-100 border-t border-base-300 text-xs text-base-content/60">
        Preview of chat highlight overlay. Users can use !highlight [message] to display highlighted messages.
      </div>
    </div>
  );
}
