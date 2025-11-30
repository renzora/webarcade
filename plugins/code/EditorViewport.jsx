import { createSignal, onMount, onCleanup, Show, For } from 'solid-js';
import { IconX, IconFile } from '@tabler/icons-solidjs';
import loader from '@monaco-editor/loader';
import { api } from '@/api/bridge';

// Map file extensions to Monaco language IDs
const getLanguageId = (filename) => {
  const ext = filename.split('.').pop()?.toLowerCase();
  const map = {
    js: 'javascript',
    jsx: 'javascript',
    ts: 'typescript',
    tsx: 'typescript',
    rs: 'rust',
    py: 'python',
    json: 'json',
    md: 'markdown',
    css: 'css',
    scss: 'scss',
    html: 'html',
    toml: 'toml',
    yml: 'yaml',
    yaml: 'yaml',
    xml: 'xml',
    sql: 'sql',
    sh: 'shell',
    bash: 'shell',
    ps1: 'powershell',
    c: 'c',
    cpp: 'cpp',
    h: 'cpp',
    hpp: 'cpp',
    java: 'java',
    go: 'go',
    rb: 'ruby',
    php: 'php',
    swift: 'swift',
    kt: 'kotlin',
    lua: 'lua',
    r: 'r',
  };
  return map[ext] || 'plaintext';
};

export default function EditorViewport() {
  let containerRef;
  let editorInstance = null;
  let monacoInstance = null;

  const [tabs, setTabs] = createSignal([]);
  const [activeTab, setActiveTab] = createSignal(null);
  const [loading, setLoading] = createSignal(false);
  const [saving, setSaving] = createSignal(false);
  const [modified, setModified] = createSignal(new Set());

  // Initialize Monaco editor
  onMount(async () => {
    try {
      // Configure Monaco loader
      loader.config({
        paths: {
          vs: 'https://cdn.jsdelivr.net/npm/monaco-editor@0.45.0/min/vs'
        }
      });

      monacoInstance = await loader.init();

      // Create editor instance
      editorInstance = monacoInstance.editor.create(containerRef, {
        value: '',
        language: 'plaintext',
        theme: 'vs-dark',
        automaticLayout: true,
        minimap: { enabled: true },
        fontSize: 14,
        lineNumbers: 'on',
        renderWhitespace: 'selection',
        scrollBeyondLastLine: false,
        wordWrap: 'off',
        tabSize: 2,
        insertSpaces: true,
        folding: true,
        glyphMargin: true,
        lineDecorationsWidth: 10,
        lineNumbersMinChars: 3,
        padding: { top: 10 },
      });

      // Track modifications
      editorInstance.onDidChangeModelContent(() => {
        const tab = activeTab();
        if (tab) {
          setModified(prev => {
            const newSet = new Set(prev);
            newSet.add(tab.path);
            return newSet;
          });
        }
      });

      // Save on Ctrl+S
      editorInstance.addCommand(monacoInstance.KeyMod.CtrlCmd | monacoInstance.KeyCode.KeyS, () => {
        saveCurrentFile();
      });

    } catch (error) {
      console.error('Failed to initialize Monaco editor:', error);
    }
  });

  // Listen for file selection events
  onMount(() => {
    const fileSelectedHandler = async (e) => {
      const { path, name } = e.detail;
      await openFile(path, name);
    };

    const newFileHandler = () => {
      createNewFile();
    };

    document.addEventListener('code:file-selected', fileSelectedHandler);
    document.addEventListener('code:new-file', newFileHandler);

    onCleanup(() => {
      document.removeEventListener('code:file-selected', fileSelectedHandler);
      document.removeEventListener('code:new-file', newFileHandler);
      if (editorInstance) {
        editorInstance.dispose();
      }
    });
  });

  const openFile = async (path, name) => {
    // Check if already open
    const existingTab = tabs().find(t => t.path === path);
    if (existingTab) {
      switchToTab(existingTab);
      return;
    }

    setLoading(true);
    try {
      const response = await api('code/files/read', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ path })
      });
      const data = await response.json();

      if (data.error) {
        console.error('Failed to read file:', data.error);
        return;
      }

      const newTab = {
        id: `tab_${Date.now()}`,
        path,
        name,
        content: data.content || '',
        language: getLanguageId(name)
      };

      setTabs(prev => [...prev, newTab]);
      switchToTab(newTab);

    } catch (e) {
      console.error('Failed to open file:', e);
    }
    setLoading(false);
  };

  const createNewFile = () => {
    const newTab = {
      id: `tab_${Date.now()}`,
      path: null,
      name: 'Untitled',
      content: '',
      language: 'plaintext'
    };

    setTabs(prev => [...prev, newTab]);
    switchToTab(newTab);
  };

  const switchToTab = (tab) => {
    // Save current tab content
    const current = activeTab();
    if (current && editorInstance) {
      const currentTabIndex = tabs().findIndex(t => t.id === current.id);
      if (currentTabIndex !== -1) {
        setTabs(prev => {
          const updated = [...prev];
          updated[currentTabIndex] = {
            ...updated[currentTabIndex],
            content: editorInstance.getValue()
          };
          return updated;
        });
      }
    }

    setActiveTab(tab);

    if (editorInstance && monacoInstance) {
      // Set the model with proper language
      const model = monacoInstance.editor.createModel(
        tab.content,
        tab.language
      );
      editorInstance.setModel(model);
    }
  };

  const closeTab = (tabToClose, e) => {
    e?.stopPropagation();

    const tabIndex = tabs().findIndex(t => t.id === tabToClose.id);
    const newTabs = tabs().filter(t => t.id !== tabToClose.id);
    setTabs(newTabs);

    // Remove from modified set
    if (tabToClose.path) {
      setModified(prev => {
        const newSet = new Set(prev);
        newSet.delete(tabToClose.path);
        return newSet;
      });
    }

    // Switch to another tab if closing active
    if (activeTab()?.id === tabToClose.id) {
      if (newTabs.length > 0) {
        const newIndex = Math.min(tabIndex, newTabs.length - 1);
        switchToTab(newTabs[newIndex]);
      } else {
        setActiveTab(null);
        if (editorInstance) {
          editorInstance.setValue('');
        }
      }
    }
  };

  const saveCurrentFile = async () => {
    const tab = activeTab();
    if (!tab || !editorInstance) return;

    const content = editorInstance.getValue();

    // If no path, we need to save as
    if (!tab.path) {
      // For now, just show a message
      console.log('Save As not implemented yet');
      return;
    }

    setSaving(true);
    try {
      const response = await api('code/files/write', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          path: tab.path,
          content
        })
      });
      const data = await response.json();

      if (data.success) {
        // Remove from modified set
        setModified(prev => {
          const newSet = new Set(prev);
          newSet.delete(tab.path);
          return newSet;
        });
      } else {
        console.error('Failed to save file:', data.error);
      }
    } catch (e) {
      console.error('Failed to save file:', e);
    }
    setSaving(false);
  };

  const isModified = (path) => path && modified().has(path);

  return (
    <div class="w-full h-full flex flex-col bg-base-300">
      {/* Tab bar */}
      <div class="flex items-center bg-base-200 border-b border-base-300 min-h-[36px]">
        <div class="flex-1 flex items-center overflow-x-auto scrollbar-thin">
          <For each={tabs()}>
            {(tab) => (
              <div
                class={`group flex items-center gap-1 px-3 py-1.5 cursor-pointer border-r border-base-300 text-sm whitespace-nowrap ${
                  activeTab()?.id === tab.id
                    ? 'bg-base-300 text-base-content'
                    : 'bg-base-200 text-base-content/60 hover:bg-base-300/50'
                }`}
                onClick={() => switchToTab(tab)}
              >
                <IconFile class="w-4 h-4 shrink-0" />
                <span class="max-w-[150px] truncate">
                  {tab.name}
                  {isModified(tab.path) && <span class="text-warning ml-0.5">*</span>}
                </span>
                <button
                  class="ml-1 p-0.5 rounded hover:bg-base-content/10 opacity-0 group-hover:opacity-100 transition-opacity"
                  onClick={(e) => closeTab(tab, e)}
                >
                  <IconX class="w-3 h-3" />
                </button>
              </div>
            )}
          </For>
        </div>

        <Show when={saving()}>
          <div class="px-2 text-xs text-base-content/60">
            Saving...
          </div>
        </Show>
      </div>

      {/* Editor container */}
      <div class="flex-1 relative">
        <Show when={tabs().length === 0}>
          <div class="absolute inset-0 flex flex-col items-center justify-center text-base-content/40">
            <IconFile class="w-16 h-16 mb-4 opacity-30" />
            <p class="text-lg">No file open</p>
            <p class="text-sm mt-2">Open a folder from the Explorer panel</p>
            <p class="text-sm">or press Ctrl+N for a new file</p>
          </div>
        </Show>

        <Show when={loading()}>
          <div class="absolute inset-0 flex items-center justify-center bg-base-300/80 z-10">
            <span class="loading loading-spinner loading-lg"></span>
          </div>
        </Show>

        <div
          ref={containerRef}
          class="w-full h-full"
          style={{ display: tabs().length > 0 ? 'block' : 'none' }}
        />
      </div>
    </div>
  );
}
