import { createSignal, createEffect, Show, For } from 'solid-js';
import { ProjectTree } from './ProjectTree';
import { bridge } from '@/api/bridge';

export default function FilesPanel(props) {
  const [currentPlugin, setCurrentPlugin] = createSignal(null);
  const [plugins, setPlugins] = createSignal([]);

  createEffect(() => {
    loadPlugins();
  });

  const loadPlugins = async () => {
    try {
      const response = await bridge('/plugin_ide/plugins');
      const data = await response.json();
      setPlugins(data);

      // Select first plugin if none selected
      if (data.length > 0 && !currentPlugin()) {
        setCurrentPlugin(data[0].id);
      }
    } catch (error) {
      console.error('Failed to load plugins:', error);
    }
  };

  const handleFileSelect = (file) => {
    console.log('File selected in panel:', file);
    if (file.type === 'file') {
      // Dispatch custom event to notify viewport
      window.dispatchEvent(new CustomEvent('plugin-ide:file-select', {
        detail: { file, plugin: currentPlugin() }
      }));
    }
  };

  const handleCreateFile = async (parentPath, type, name) => {
    try {
      // Extract relative path
      const pluginPath = `plugins/${currentPlugin()}`;
      let relativePath = '';

      if (parentPath.includes(pluginPath)) {
        relativePath = parentPath.split(pluginPath + '/')[1] || parentPath.split(pluginPath + '\\')[1] || '';
      }

      await bridge(`/plugin_ide/file/${currentPlugin()}`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          path: relativePath,
          type: type,
          name: name,
        }),
      });
    } catch (error) {
      console.error('Failed to create file:', error);
      alert('Failed to create file: ' + error.message);
    }
  };

  const handleDeleteFile = async (filePath) => {
    try {
      const pluginPath = `plugins/${currentPlugin()}`;
      let relativePath = filePath;

      if (relativePath.includes(pluginPath)) {
        relativePath = relativePath.split(pluginPath + '/')[1] || relativePath.split(pluginPath + '\\')[1];
      }

      await bridge(`/plugin_ide/file/${currentPlugin()}/${relativePath}`, {
        method: 'DELETE',
      });
    } catch (error) {
      console.error('Failed to delete file:', error);
      alert('Failed to delete file: ' + error.message);
    }
  };

  return (
    <div class="h-full flex flex-col bg-base-100">
      {/* Plugin Selector */}
      <div class="p-3 border-b border-base-content/10 bg-base-200">
        <label class="text-xs font-semibold text-base-content/60 mb-1 block">Plugin</label>
        <select
          class="select select-bordered select-sm w-full"
          value={currentPlugin() || ''}
          onChange={(e) => {
            setCurrentPlugin(e.target.value);
          }}
        >
          <option value="" disabled>Select a plugin...</option>
          <For each={plugins()}>
            {(p) => (
              <option value={p.id}>
                {p.name}
                {p.has_backend && ' 🦀'}
                {p.has_frontend && ' ⚛️'}
              </option>
            )}
          </For>
        </select>
      </div>

      {/* File Tree */}
      <div class="flex-1 overflow-hidden">
        <Show
          when={currentPlugin()}
          fallback={
            <div class="flex items-center justify-center h-full p-4 text-center text-base-content/60">
              <p>Select a plugin to view its files</p>
            </div>
          }
        >
          <ProjectTree
            currentPlugin={currentPlugin()}
            onFileSelect={handleFileSelect}
            onCreateFile={handleCreateFile}
            onDeleteFile={handleDeleteFile}
          />
        </Show>
      </div>
    </div>
  );
}
