import { createSignal, createEffect, onCleanup, onMount, Show } from 'solid-js';
import { IconEye, IconRefresh, IconCode, IconDeviceTv } from '@tabler/icons-solidjs';
import MonacoEditor from '@/plugins/core/code/MonacoEditor';
import { editorActions, editorStore } from '@/layout/stores/EditorStore';
import { propertyTabs } from '@/api/plugin';
import { viewportStore } from '@/panels/viewport/store';

const BRIDGE_URL = 'http://localhost:3001';
const DEFAULT_OVERLAY_JSX = `import { render } from 'solid-js/web';
import { createSignal, createEffect, onCleanup } from 'solid-js';
import '@/index.css';

const WEBARCADE_WS = 'ws://localhost:3002';

function MyOverlay() {
  const [isConnected, setIsConnected] = createSignal(false);
  const [message, setMessage] = createSignal('Hello from overlay!');
  let ws;

  // Connect to WebSocket
  const connectWebSocket = () => {
    ws = new WebSocket(WEBARCADE_WS);

    ws.onopen = () => {
      console.log('✅ Connected to WebArcade');
      setIsConnected(true);
    };

    ws.onclose = () => {
      console.log('❌ Disconnected');
      setIsConnected(false);
      setTimeout(connectWebSocket, 3000);
    };

    ws.onmessage = (event) => {
      const data = JSON.parse(event.data);
      if (data.type === 'twitch_event' && data.event.type === 'chat_message') {
        setMessage(data.event.message);
      }
    };
  };

  createEffect(() => {
    connectWebSocket();
    onCleanup(() => ws?.close());
  });

  return (
    <div class="fixed inset-0 pointer-events-none overflow-hidden" data-theme="dark">
      {/* Connection Status */}
      <div class="absolute top-5 right-5 pointer-events-auto">
        <div class={\`badge \${isConnected() ? 'badge-success' : 'badge-error'} gap-2 shadow-lg py-3 px-4\`}>
          <div class="w-2 h-2 bg-white rounded-full"></div>
          {isConnected() ? 'Live' : 'Offline'}
        </div>
      </div>

      {/* Your content here */}
      <div class="absolute bottom-8 left-8">
        <div class="card bg-base-100 shadow-xl">
          <div class="card-body">
            <h2 class="card-title">WebArcade Overlay</h2>
            <p>{message()}</p>
          </div>
        </div>
      </div>
    </div>
  );
}

render(() => <MyOverlay />, document.getElementById('root'));
`;

export default function OverlayManagerViewport() {
  const [overlayFiles, setOverlayFiles] = createSignal([]);
  const [selectedFile, setSelectedFile] = createSignal(null);
  const [fileName, setFileName] = createSignal('');
  const [fileContent, setFileContent] = createSignal('');
  const [loading, setLoading] = createSignal(false);
  const [saving, setSaving] = createSignal(false);
  const [building, setBuilding] = createSignal(false);
  const [showPreview, setShowPreview] = createSignal(false);
  const [editorKey, setEditorKey] = createSignal(0);

  const fetchOverlayFiles = async () => {
    try {
      setLoading(true);
      const response = await fetch(`${BRIDGE_URL}/api/overlay-files`);
      const data = await response.json();
      setOverlayFiles(data);
    } catch (error) {
      console.error('Failed to fetch overlay files:', error);
    } finally {
      setLoading(false);
    }
  };

  createEffect(() => {
    fetchOverlayFiles();
  });

  // Auto-select the overlay-manager-panel tab when this viewport loads
  onMount(() => {
    editorActions.setScenePanelOpen(true);
    editorActions.setSelectedTool('overlay-manager-panel');

    // When viewport is closed, switch away from the overlay-manager-panel tab
    onCleanup(() => {
      const currentTool = editorStore.ui.selectedTool;
      if (currentTool === 'overlay-manager-panel') {
        // Find the first available tab that isn't the overlay-manager-panel
        const availableTabs = Array.from(propertyTabs().values())
          .filter(tab => tab.id !== 'overlay-manager-panel' && (!tab.condition || tab.condition()))
          .sort((a, b) => (a.order || 0) - (b.order || 0));

        if (availableTabs.length > 0) {
          editorActions.setSelectedTool(availableTabs[0].id);
        } else {
          // If no other tabs available, just set to null
          editorActions.setSelectedTool(null);
        }
      }
    });
  });

  // Watch for viewport changes and auto-switch to the panel tab
  createEffect(() => {
    const activeTab = viewportStore.tabs.find(tab => tab.id === viewportStore.activeTabId);
    if (activeTab?.type === 'overlay-manager') {
      editorActions.setScenePanelOpen(true);
      editorActions.setSelectedTool('overlay-manager-panel');
    }
  });

  // Listen for events from the panel
  createEffect(() => {
    const handleOverlaySelect = (e) => {
      handleSelectFile(e.detail);
    };

    const handleOverlayNew = () => {
      handleNewOverlay();
    };

    window.addEventListener('overlay-select', handleOverlaySelect);
    window.addEventListener('overlay-new', handleOverlayNew);

    onCleanup(() => {
      window.removeEventListener('overlay-select', handleOverlaySelect);
      window.removeEventListener('overlay-new', handleOverlayNew);
    });
  });

  const handleNewOverlay = () => {
    setSelectedFile({ name: 'new-overlay', path: 'new' });
    setFileName('my-overlay');
    setFileContent(DEFAULT_OVERLAY_JSX);
    setEditorKey(prev => prev + 1);
  };

  const handleSelectFile = async (file) => {
    console.log('🔵 handleSelectFile called with:', file);

    // Clear selectedFile first to force Monaco unmount when switching files
    console.log('🔵 Clearing selectedFile to force unmount');
    setSelectedFile(null);

    try {
      const url = `${BRIDGE_URL}/api/overlay-files/${file.name}.jsx`;
      console.log('🔵 Fetching from URL:', url);

      const response = await fetch(url);
      console.log('🔵 Response status:', response.status, response.statusText);

      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`);
      }

      const content = await response.text();
      console.log('🔵 Content length:', content.length);
      console.log('🔵 Content preview:', content.substring(0, 100));

      // Update content and fileName
      console.log('🔵 Setting fileName:', file.name);
      setFileName(file.name);

      console.log('🔵 Setting fileContent with length:', content.length);
      setFileContent(content);

      console.log('🔵 Incrementing editorKey');
      setEditorKey(prev => {
        console.log('🔵 editorKey changing from', prev, 'to', prev + 1);
        return prev + 1;
      });

      // Set selectedFile LAST so Monaco renders with the updated content
      console.log('🔵 Setting selectedFile:', file);
      setSelectedFile(file);

      console.log('🔵 State updated successfully');

      // Broadcast selection to panel for highlighting
      window.dispatchEvent(new CustomEvent('overlay-selected', { detail: file }));
    } catch (error) {
      console.error('❌ Failed to load overlay file:', error);
      alert(`Failed to load overlay file: ${error.message}`);
    }
  };

  const handleSaveOverlay = async () => {
    if (!fileName().trim()) {
      alert('Please enter a file name');
      return;
    }

    // Ensure filename ends with .jsx
    let finalFileName = fileName().trim();
    if (!finalFileName.endsWith('.jsx')) {
      finalFileName += '.jsx';
    }

    try {
      setSaving(true);

      const response = await fetch(`${BRIDGE_URL}/api/overlay-files/${finalFileName}`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ content: fileContent() })
      });

      if (response.ok) {
        // Trigger rebuild
        setBuilding(true);
        const rebuildResponse = await fetch(`${BRIDGE_URL}/api/rebuild-overlays`, {
          method: 'POST'
        });

        if (rebuildResponse.ok) {
          await fetchOverlayFiles();
          setSelectedFile({ name: finalFileName.replace('.jsx', ''), path: `src/overlays/${finalFileName}` });
          setFileName(finalFileName.replace('.jsx', ''));
          alert('Overlay saved and built successfully!');
        } else {
          throw new Error('Failed to rebuild overlays');
        }
      } else {
        throw new Error('Failed to save overlay');
      }
    } catch (error) {
      console.error('Failed to save overlay:', error);
      alert('Failed to save overlay: ' + error.message);
    } finally {
      setSaving(false);
      setBuilding(false);
    }
  };

  const handleRefreshPreview = () => {
    const iframe = document.getElementById('preview-iframe');
    if (iframe) {
      iframe.src = iframe.src;
    }
  };

  return (
    <div class="h-full flex flex-col bg-base-200">
      {/* Main Editor Area */}
      <div class="flex-1 flex flex-col">
        {/* Editor Header */}
        <div class="p-4 bg-base-100 border-b border-base-300 flex items-center justify-between gap-4">
          <div class="flex-1 flex items-center gap-4">
            <input
              type="text"
              placeholder="overlay-name"
              class="input input-bordered flex-1 max-w-md font-mono"
              value={fileName()}
              onInput={(e) => setFileName(e.target.value)}
            />
            <span class="text-base-content/60">.jsx</span>
          </div>
          <div class="flex gap-2">
            <button
              class="btn btn-ghost gap-2"
              onClick={() => setShowPreview(!showPreview())}
            >
              <IconEye size={20} />
              {showPreview() ? 'Hide' : 'Show'} Preview
            </button>
            <button
              class={`btn btn-primary gap-2 ${saving() || building() ? 'loading' : ''}`}
              onClick={handleSaveOverlay}
              disabled={saving() || building()}
            >
              {!saving() && !building() && <IconCode size={20} />}
              {building() ? 'Building...' : saving() ? 'Saving...' : 'Save & Build'}
            </button>
          </div>
        </div>

        {/* Editor Content */}
        <div class="flex-1 flex overflow-hidden">
          {/* JSX Editor */}
          <div class="flex-1 flex flex-col overflow-hidden">
            <div class="p-2 bg-base-300 text-sm font-semibold">JSX Editor</div>
            <div class="flex-1 relative">
              <Show
                when={selectedFile()}
                fallback={
                  <div class="flex items-center justify-center h-full bg-base-300/20">
                    <div class="text-center">
                      <IconCode size={64} class="mx-auto mb-4 opacity-30" />
                      <p class="text-base-content/60">Select an overlay or create a new one</p>
                    </div>
                  </div>
                }
              >
                <div class="absolute inset-0">
                  <MonacoEditor
                    key={editorKey()}
                    value={(() => {
                      const content = fileContent();
                      console.log('🟢 Monaco value prop:', content.length, 'chars, editorKey:', editorKey());
                      return content;
                    })()}
                    onChange={(value) => {
                      console.log('🟡 Monaco onChange called with length:', value.length);
                      setFileContent(value);
                    }}
                    language="javascript"
                    theme="vs-dark"
                    height="100%"
                    width="100%"
                    options={{
                      formatOnPaste: true,
                      formatOnType: true,
                      autoClosingBrackets: 'always',
                      autoClosingQuotes: 'always',
                      autoIndent: 'full',
                      minimap: { enabled: false },
                      scrollBeyondLastLine: false,
                      wordWrap: 'on',
                      fontSize: 14,
                    }}
                  />
                </div>
              </Show>
            </div>
          </div>

          {/* Live Preview */}
          <Show when={showPreview()}>
            <div class="w-1/2 flex flex-col border-l border-base-300">
              <div class="p-2 bg-base-300 text-sm font-semibold flex items-center justify-between">
                <span>Live Preview</span>
                <button class="btn btn-xs btn-ghost" onClick={handleRefreshPreview}>
                  <IconRefresh size={14} />
                </button>
              </div>
              <div class="flex-1 bg-black/10 overflow-hidden">
                <Show when={selectedFile()}>
                  <iframe
                    id="preview-iframe"
                    src={`${BRIDGE_URL}/overlay/${selectedFile().name}`}
                    class="w-full h-full border-0"
                    style={{
                      transform: 'scale(0.5)',
                      'transform-origin': 'top left',
                      width: '200%',
                      height: '200%'
                    }}
                  />
                </Show>
                <Show when={!selectedFile()}>
                  <div class="flex items-center justify-center h-full text-base-content/60">
                    <div class="text-center">
                      <IconDeviceTv size={64} class="mx-auto mb-4 opacity-30" />
                      <p>Save overlay to see preview</p>
                    </div>
                  </div>
                </Show>
              </div>
            </div>
          </Show>
        </div>

        {/* Instructions */}
        <div class="p-4 bg-base-100 border-t border-base-300">
          <div class="alert alert-info">
            <IconDeviceTv size={24} />
            <div>
              <h3 class="font-bold">Single-File Overlay System</h3>
              <p class="text-sm">
                Each .jsx file is a complete overlay with SolidJS, Tailwind, and DaisyUI.
                Save & Build to compile, then copy URL for OBS Browser Source (1920x1080 recommended).
              </p>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
