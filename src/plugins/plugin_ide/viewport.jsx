import { createSignal, onMount, onCleanup, Show } from 'solid-js';
import { CodeEditor } from './components/Editor';
import { NewPluginWizard } from './components/NewPluginWizard';
import { IconPlus, IconPackage } from '@tabler/icons-solidjs';
import { bridge } from '@/api/bridge';

export default function PluginIDE() {
  const [currentPlugin, setCurrentPlugin] = createSignal(null);
  const [currentFile, setCurrentFile] = createSignal(null);
  const [showWizard, setShowWizard] = createSignal(false);
  const [isBuilding, setIsBuilding] = createSignal(false);
  const [buildStatus, setBuildStatus] = createSignal(null);

  // Listen for file selection events from the Files panel
  onMount(() => {
    const handleFileSelect = (event) => {
      console.log('File select event received:', event.detail);
      console.log('File object:', event.detail.file);
      console.log('Plugin:', event.detail.plugin);
      setCurrentPlugin(event.detail.plugin);
      setCurrentFile(event.detail.file);
      console.log('Current file after set:', currentFile());
      console.log('Current plugin after set:', currentPlugin());
    };

    window.addEventListener('plugin-ide:file-select', handleFileSelect);

    onCleanup(() => {
      window.removeEventListener('plugin-ide:file-select', handleFileSelect);
    });
  });

  const handlePluginCreated = (plugin) => {
    console.log('Plugin created:', plugin);
    // Dispatch event to refresh files panel
    window.dispatchEvent(new CustomEvent('plugin-ide:refresh-plugins'));
  };

  const handleBuild = async () => {
    if (!currentPlugin() || isBuilding()) return;

    setIsBuilding(true);
    setBuildStatus({ type: 'loading', message: 'Building plugin...' });

    try {
      const response = await bridge(`/plugin_ide/build/${currentPlugin()}`, {
        method: 'POST',
      });

      if (!response.ok) {
        throw new Error('Build failed');
      }

      const result = await response.json();
      setBuildStatus({ type: 'success', message: 'Build successful!', data: result });

      // Auto-clear success message after 3 seconds
      setTimeout(() => setBuildStatus(null), 3000);
    } catch (error) {
      console.error('Build error:', error);
      setBuildStatus({ type: 'error', message: error.message || 'Build failed' });
    } finally {
      setIsBuilding(false);
    }
  };

  return (
    <div class="flex flex-col h-full bg-base-100">
      {/* Toolbar */}
      <div class="flex items-center justify-between px-4 py-2 bg-base-200 border-b border-base-content/10">
        <div class="flex items-center gap-2">
          <Show when={currentFile()}>
            <span class="text-sm text-base-content/60">
              {currentPlugin()} / {currentFile()?.name}
            </span>
          </Show>
        </div>

        <div class="flex items-center gap-2">
          <Show when={buildStatus()}>
            <div class={`alert alert-sm ${
              buildStatus().type === 'success' ? 'alert-success' :
              buildStatus().type === 'error' ? 'alert-error' :
              'alert-info'
            } py-1 px-3`}>
              <span class="text-xs">{buildStatus().message}</span>
            </div>
          </Show>

          <Show when={currentPlugin()}>
            <button
              onClick={handleBuild}
              disabled={isBuilding()}
              class="btn btn-sm btn-success gap-2"
            >
              {isBuilding() ? (
                <>
                  <span class="loading loading-spinner loading-xs"></span>
                  Building...
                </>
              ) : (
                <>
                  <IconPackage size={16} />
                  Build
                </>
              )}
            </button>
          </Show>

          <button
            onClick={() => setShowWizard(true)}
            class="btn btn-sm btn-primary gap-2"
          >
            <IconPlus size={16} />
            New Plugin
          </button>
        </div>
      </div>

      {/* Monaco Editor - Full Viewport */}
      <div class="flex-1">
        <CodeEditor
          file={currentFile()}
          currentPlugin={currentPlugin()}
          onSave={(file) => {
            console.log('File saved:', file.name);
          }}
        />
      </div>

      {/* New Plugin Wizard */}
      <Show when={showWizard()}>
        <NewPluginWizard
          onClose={() => setShowWizard(false)}
          onCreate={handlePluginCreated}
        />
      </Show>
    </div>
  );
}
