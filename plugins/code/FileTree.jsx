import { createSignal, onMount, onCleanup, For, Show } from 'solid-js';
import { IconFolder, IconFolderOpen, IconFile, IconChevronRight, IconChevronDown, IconRefresh } from '@tabler/icons-solidjs';
import { api } from '@/api/bridge';

// File type to icon/color mapping
const getFileInfo = (filename) => {
  const ext = filename.split('.').pop()?.toLowerCase();
  const map = {
    js: { color: 'text-yellow-400' },
    jsx: { color: 'text-yellow-400' },
    ts: { color: 'text-blue-400' },
    tsx: { color: 'text-blue-400' },
    rs: { color: 'text-orange-400' },
    py: { color: 'text-green-400' },
    json: { color: 'text-yellow-300' },
    md: { color: 'text-gray-400' },
    css: { color: 'text-pink-400' },
    scss: { color: 'text-pink-400' },
    html: { color: 'text-orange-300' },
    toml: { color: 'text-gray-300' },
    yml: { color: 'text-purple-400' },
    yaml: { color: 'text-purple-400' },
  };
  return map[ext] || { color: 'text-base-content/70' };
};

function TreeNode(props) {
  const [expanded, setExpanded] = createSignal(false);
  const [children, setChildren] = createSignal([]);
  const [loading, setLoading] = createSignal(false);

  const isDirectory = () => props.node.is_dir;

  const loadChildren = async () => {
    if (!isDirectory() || children().length > 0) return;

    setLoading(true);
    try {
      const response = await api('code/files/list', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ path: props.node.path })
      });
      const data = await response.json();
      if (data.files) {
        // Sort: directories first, then files, alphabetically
        const sorted = data.files.sort((a, b) => {
          if (a.is_dir && !b.is_dir) return -1;
          if (!a.is_dir && b.is_dir) return 1;
          return a.name.localeCompare(b.name);
        });
        setChildren(sorted);
      }
    } catch (e) {
      console.error('Failed to load directory:', e);
    }
    setLoading(false);
  };

  const handleClick = async () => {
    if (isDirectory()) {
      const newExpanded = !expanded();
      setExpanded(newExpanded);
      if (newExpanded) {
        await loadChildren();
      }
    } else {
      // Open file in editor
      props.onFileSelect?.(props.node);
    }
  };

  const fileInfo = getFileInfo(props.node.name);

  return (
    <div class="select-none">
      <div
        class={`flex items-center gap-1 px-2 py-0.5 cursor-pointer hover:bg-base-300 rounded text-sm ${
          props.selectedPath === props.node.path ? 'bg-primary/20 text-primary' : ''
        }`}
        style={{ "padding-left": `${props.depth * 12 + 4}px` }}
        onClick={handleClick}
      >
        <Show when={isDirectory()}>
          <span class="w-4 h-4 flex items-center justify-center text-base-content/50">
            {loading() ? (
              <span class="loading loading-spinner loading-xs"></span>
            ) : expanded() ? (
              <IconChevronDown class="w-3 h-3" />
            ) : (
              <IconChevronRight class="w-3 h-3" />
            )}
          </span>
          {expanded() ? (
            <IconFolderOpen class="w-4 h-4 text-yellow-500" />
          ) : (
            <IconFolder class="w-4 h-4 text-yellow-500" />
          )}
        </Show>
        <Show when={!isDirectory()}>
          <span class="w-4 h-4"></span>
          <IconFile class={`w-4 h-4 ${fileInfo.color}`} />
        </Show>
        <span class="truncate">{props.node.name}</span>
      </div>

      <Show when={expanded() && children().length > 0}>
        <For each={children()}>
          {(child) => (
            <TreeNode
              node={child}
              depth={props.depth + 1}
              onFileSelect={props.onFileSelect}
              selectedPath={props.selectedPath}
            />
          )}
        </For>
      </Show>
    </div>
  );
}

export default function FileTree() {
  const [rootPath, setRootPath] = createSignal('');
  const [rootItems, setRootItems] = createSignal([]);
  const [selectedPath, setSelectedPath] = createSignal('');
  const [loading, setLoading] = createSignal(false);

  const loadRoot = async (path) => {
    if (!path) return;

    setLoading(true);
    setRootPath(path);

    try {
      const response = await api('code/files/list', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ path })
      });
      const data = await response.json();
      if (data.files) {
        // Sort: directories first, then files, alphabetically
        const sorted = data.files.sort((a, b) => {
          if (a.is_dir && !b.is_dir) return -1;
          if (!a.is_dir && b.is_dir) return 1;
          return a.name.localeCompare(b.name);
        });
        setRootItems(sorted);
      }
    } catch (e) {
      console.error('Failed to load root:', e);
    }
    setLoading(false);
  };

  const handleOpenFolder = async () => {
    try {
      const response = await api('code/files/pick-folder', {
        method: 'POST'
      });
      const data = await response.json();
      if (data.path) {
        await loadRoot(data.path);
      }
    } catch (e) {
      console.error('Failed to open folder:', e);
    }
  };

  const handleFileSelect = (node) => {
    setSelectedPath(node.path);
    document.dispatchEvent(new CustomEvent('code:file-selected', {
      detail: { path: node.path, name: node.name }
    }));
  };

  const handleRefresh = () => {
    if (rootPath()) {
      loadRoot(rootPath());
    }
  };

  // Listen for open folder event
  onMount(() => {
    const openFolderHandler = () => handleOpenFolder();
    document.addEventListener('code:open-folder', openFolderHandler);

    onCleanup(() => {
      document.removeEventListener('code:open-folder', openFolderHandler);
    });
  });

  return (
    <div class="h-full flex flex-col bg-base-200">
      {/* Header */}
      <div class="flex items-center justify-between px-3 py-2 border-b border-base-300">
        <span class="text-xs font-semibold uppercase text-base-content/60">Explorer</span>
        <button
          class="btn btn-ghost btn-xs btn-square"
          onClick={handleRefresh}
          disabled={!rootPath()}
          title="Refresh"
        >
          <IconRefresh class="w-3.5 h-3.5" />
        </button>
      </div>

      {/* Content */}
      <div class="flex-1 overflow-auto">
        <Show when={!rootPath()}>
          <div class="p-4 text-center">
            <p class="text-sm text-base-content/60 mb-3">No folder opened</p>
            <button
              class="btn btn-sm btn-primary"
              onClick={handleOpenFolder}
            >
              <IconFolder class="w-4 h-4 mr-1" />
              Open Folder
            </button>
          </div>
        </Show>

        <Show when={rootPath()}>
          <div class="py-1">
            {/* Root folder name */}
            <div class="px-3 py-1 text-xs font-medium text-base-content/80 truncate border-b border-base-300 mb-1">
              {rootPath().split(/[/\\]/).pop()}
            </div>

            <Show when={loading()}>
              <div class="flex justify-center py-4">
                <span class="loading loading-spinner loading-sm"></span>
              </div>
            </Show>

            <Show when={!loading()}>
              <For each={rootItems()}>
                {(item) => (
                  <TreeNode
                    node={item}
                    depth={0}
                    onFileSelect={handleFileSelect}
                    selectedPath={selectedPath()}
                  />
                )}
              </For>
            </Show>
          </div>
        </Show>
      </div>
    </div>
  );
}
