import { createSignal, createEffect, Show } from 'solid-js';
import { IconDeviceFloppy } from '@tabler/icons-solidjs';
import MonacoEditor from './MonacoEditor';
import { bridge } from '@/api/bridge';

export function CodeEditor(props) {
  const [content, setContent] = createSignal('');
  const [modified, setModified] = createSignal(false);
  const [saving, setSaving] = createSignal(false);
  const [isLoading, setIsLoading] = createSignal(false);
  let editorInstance = null;

  createEffect(() => {
    console.log('[Editor] props.file:', props.file);
    console.log('[Editor] props.currentPlugin:', props.currentPlugin);
    if (props.file && props.currentPlugin) {
      console.log('[Editor] Loading file:', props.file.name);
      setIsLoading(true);
      loadFile();
    } else {
      // Clear content when no file is selected
      setContent('');
      setModified(false);
    }
  });

  const loadFile = async () => {
    if (!props.file || !props.currentPlugin) return;

    try {
      // Extract relative path from full path
      // The path from ProjectTree is an absolute path, we need to make it relative to the plugin directory
      let relativePath = props.file.path;

      // Replace backslashes with forward slashes for consistency
      relativePath = relativePath.replace(/\\/g, '/');

      // Find the plugin directory in the path and get everything after it
      const pluginPattern = new RegExp(`plugins/${props.currentPlugin}/(.+)$`);
      const match = relativePath.match(pluginPattern);

      if (match && match[1]) {
        relativePath = match[1];
      } else {
        // If no match, just use the filename
        relativePath = props.file.name;
      }

      const url = `/plugin_ide/file/${props.currentPlugin}/${relativePath}`;
      console.log('[Editor] Fetching file from:', url);
      console.log('[Editor] Full file path:', props.file.path);
      console.log('[Editor] Relative path:', relativePath);

      const response = await bridge(url);
      console.log('[Editor] Response status:', response.status);
      console.log('[Editor] Response headers:', response.headers.get('content-type'));

      const text = await response.text();
      console.log('[Editor] Response text length:', text.length);
      console.log('[Editor] Response preview:', text.substring(0, 100));

      setContent(text);
      setModified(false);
      setIsLoading(false);
      console.log('[Editor] Content set successfully, content length:', content().length);
    } catch (error) {
      console.error('Failed to load file:', error);
      setContent('// Error loading file');
      setIsLoading(false);
    }
  };

  const handleChange = (value) => {
    setContent(value);
    setModified(true);
  };

  const handleSave = async () => {
    if (!props.file || !props.currentPlugin || saving()) return;

    setSaving(true);
    try {
      // Extract relative path from full path
      let relativePath = props.file.path;

      // Replace backslashes with forward slashes for consistency
      relativePath = relativePath.replace(/\\/g, '/');

      // Find the plugin directory in the path and get everything after it
      const pluginPattern = new RegExp(`plugins/${props.currentPlugin}/(.+)$`);
      const match = relativePath.match(pluginPattern);

      if (match && match[1]) {
        relativePath = match[1];
      } else {
        // If no match, just use the filename
        relativePath = props.file.name;
      }

      await bridge(`/plugin_ide/file/${props.currentPlugin}/${relativePath}`, {
        method: 'PUT',
        headers: {
          'Content-Type': 'text/plain',
        },
        body: content(),
      });

      setModified(false);
      props.onSave?.(props.file);
    } catch (error) {
      console.error('Failed to save file:', error);
      alert('Failed to save file: ' + error.message);
    } finally {
      setSaving(false);
    }
  };

  // Keyboard shortcut for save
  createEffect(() => {
    const handler = (e) => {
      if ((e.ctrlKey || e.metaKey) && e.key === 's') {
        e.preventDefault();
        handleSave();
      }
    };
    window.addEventListener('keydown', handler);
    return () => window.removeEventListener('keydown', handler);
  });

  const getLanguage = (filename) => {
    if (!filename) return 'javascript';
    const ext = filename.split('.').pop().toLowerCase();
    const languageMap = {
      'rs': 'rust',
      'jsx': 'javascript',
      'js': 'javascript',
      'ts': 'typescript',
      'tsx': 'typescript',
      'json': 'json',
      'md': 'markdown',
      'css': 'css',
      'scss': 'scss',
      'less': 'less',
      'toml': 'ini',  // Monaco doesn't have TOML, INI is close
      'html': 'html',
      'xml': 'xml',
      'yaml': 'yaml',
      'yml': 'yaml',
      'txt': 'plaintext',
      'sh': 'shell',
      'bash': 'shell',
      'sql': 'sql',
      'py': 'python',
    };
    const lang = languageMap[ext] || 'javascript';
    console.log(`[Editor] File: ${filename}, Extension: ${ext}, Language: ${lang}`);
    return lang;
  };

  const handleEditorMount = (editor, monaco) => {
    editorInstance = editor;
    console.log('[Editor] Monaco editor mounted');
  };

  return (
    <Show
      when={props.file}
      fallback={
        <div class="flex flex-col items-center justify-center h-full bg-base-200">
          <div class="text-center">
            <h2 class="text-2xl font-bold mb-2">No file selected</h2>
            <p class="text-base-content/60">Select a file from the Files panel to start editing</p>
          </div>
        </div>
      }
    >
      <div class="flex flex-col h-full">
        <div class="flex items-center justify-between px-4 py-2 bg-base-300 border-b border-base-content/10">
          <span class="text-sm font-medium flex items-center gap-2">
            {props.file?.name}
            {modified() && <span class="text-primary text-xl">●</span>}
          </span>
          <div class="flex items-center gap-2">
            <button
              onClick={handleSave}
              disabled={!modified() || saving()}
              class="btn btn-sm btn-primary gap-2"
            >
              <IconDeviceFloppy size={16} />
              {saving() ? 'Saving...' : 'Save'}
            </button>
            <span class="text-xs text-base-content/60">Ctrl+S</span>
          </div>
        </div>

        <div class="flex-1">
          <Show
            when={!isLoading()}
            fallback={
              <div class="flex items-center justify-center h-full bg-base-200">
                <div class="flex flex-col items-center gap-2">
                  <span class="loading loading-spinner loading-lg"></span>
                  <p class="text-base-content/60">Loading file...</p>
                </div>
              </div>
            }
          >
            <MonacoEditor
              value={content()}
              onChange={handleChange}
              language={getLanguage(props.file?.name)}
              theme="daisyui-theme"
              height="100%"
              onMount={handleEditorMount}
            />
          </Show>
        </div>
      </div>
    </Show>
  );
}
