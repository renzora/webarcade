import { createSignal, createEffect, onCleanup, For, Show } from 'solid-js';
import { IconPlus, IconTrash, IconCopy, IconCheck, IconDeviceTv } from '@tabler/icons-solidjs';

const BRIDGE_URL = 'http://localhost:3001';

export default function OverlayManagerPanel() {
  const [overlayFiles, setOverlayFiles] = createSignal([]);
  const [loading, setLoading] = createSignal(false);
  const [copiedUrl, setCopiedUrl] = createSignal(false);
  const [selectedFile, setSelectedFile] = createSignal(null);

  const fetchOverlayFiles = async () => {
    try {
      setLoading(true);
      const response = await fetch(`${BRIDGE_URL}/overlay-manager/files`);
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

  // Listen for overlay selection from viewport
  createEffect(() => {
    const handleOverlaySelected = (e) => {
      setSelectedFile(e.detail);
    };

    window.addEventListener('overlay-selected', handleOverlaySelected);

    onCleanup(() => {
      window.removeEventListener('overlay-selected', handleOverlaySelected);
    });
  });

  const handleCopyUrl = (fileName) => {
    const url = `${BRIDGE_URL}/overlay/${fileName}`;
    navigator.clipboard.writeText(url);
    setCopiedUrl(fileName);
    setTimeout(() => setCopiedUrl(false), 2000);
  };

  const handleDeleteOverlay = async (file) => {
    if (!confirm(`Delete overlay "${file.name}"?`)) return;

    try {
      const response = await fetch(`${BRIDGE_URL}/overlay-manager/files/${file.name}.jsx`, {
        method: 'DELETE'
      });

      if (response.ok) {
        // Trigger rebuild
        await fetch(`${BRIDGE_URL}/overlay-manager/rebuild`, { method: 'POST' });
        await fetchOverlayFiles();
      } else {
        throw new Error('Failed to delete overlay');
      }
    } catch (error) {
      console.error('Failed to delete overlay:', error);
      alert('Failed to delete overlay');
    }
  };

  // Emit a custom event when an overlay is selected
  const handleSelectFile = (file) => {
    window.dispatchEvent(new CustomEvent('overlay-select', { detail: file }));
  };

  const handleNewOverlay = () => {
    window.dispatchEvent(new CustomEvent('overlay-new'));
  };

  return (
    <div class="h-full flex flex-col bg-base-100">
      <div class="p-4 border-b border-base-300">
        <button class="btn btn-primary w-full gap-2" onClick={handleNewOverlay}>
          <IconPlus size={20} />
          New Overlay
        </button>
      </div>

      <div class="flex-1 overflow-y-auto p-4 space-y-2">
        <Show when={loading()} fallback={
          <Show when={overlayFiles().length === 0}>
            <div class="text-center py-8 text-base-content/60">
              <IconDeviceTv size={48} class="mx-auto mb-2 opacity-50" />
              <p>No overlays yet</p>
              <p class="text-sm">Create your first overlay!</p>
            </div>
          </Show>
        }>
          <div class="flex justify-center py-8">
            <span class="loading loading-spinner loading-lg"></span>
          </div>
        </Show>

        <For each={overlayFiles()}>
          {(file) => (
            <div
              class={`card bg-base-200 cursor-pointer transition-all ${
                selectedFile()?.name === file.name ? 'ring-2 ring-primary' : 'hover:bg-base-300'
              }`}
              onClick={() => handleSelectFile(file)}
            >
              <div class="card-body p-3">
                <div class="flex items-start justify-between gap-2">
                  <div class="flex-1 min-w-0">
                    <h3 class="font-semibold truncate">{file.name}</h3>
                    <p class="text-xs text-base-content/60 truncate">{file.path}</p>
                  </div>
                  <button
                    class="btn btn-error btn-xs btn-circle"
                    onClick={(e) => {
                      e.stopPropagation();
                      handleDeleteOverlay(file);
                    }}
                  >
                    <IconTrash size={14} />
                  </button>
                </div>
                <button
                  class={`btn btn-xs gap-1 ${copiedUrl() === file.name ? 'btn-success' : 'btn-ghost'}`}
                  onClick={(e) => {
                    e.stopPropagation();
                    handleCopyUrl(file.name);
                  }}
                >
                  {copiedUrl() === file.name ? <IconCheck size={14} /> : <IconCopy size={14} />}
                  {copiedUrl() === file.name ? 'Copied!' : 'Copy URL'}
                </button>
              </div>
            </div>
          )}
        </For>
      </div>
    </div>
  );
}
