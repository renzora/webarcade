import { createSignal, createEffect, onCleanup, Show, For } from 'solid-js';
import {
  IconFile,
  IconFolder,
  IconFolderOpen,
  IconFileCode,
  IconBrandRust,
  IconBrandReact,
  IconJson,
  IconMarkdown,
  IconPalette,
  IconSettings,
  IconPlus,
  IconFolderPlus,
  IconRefresh,
  IconTrash,
  IconChevronRight,
  IconChevronDown,
} from '@tabler/icons-solidjs';

export function ProjectTree(props) {
  const [tree, setTree] = createSignal(null);
  const [expanded, setExpanded] = createSignal(new Set([props.currentPlugin]));
  const [contextMenu, setContextMenu] = createSignal(null);

  createEffect(() => {
    if (props.currentPlugin) {
      loadTree();
    }
  });

  const loadTree = async () => {
    try {
      const response = await fetch(`http://localhost:3001/plugin_ide/tree/${props.currentPlugin}`);
      const data = await response.json();
      setTree(data);
      // Auto-expand root
      setExpanded(new Set([data.path]));
    } catch (error) {
      console.error('Failed to load tree:', error);
    }
  };

  const toggleExpanded = (path) => {
    const newExpanded = new Set(expanded());
    if (newExpanded.has(path)) {
      newExpanded.delete(path);
    } else {
      newExpanded.add(path);
    }
    setExpanded(newExpanded);
  };

  const handleContextMenu = (e, node) => {
    e.preventDefault();
    e.stopPropagation();

    setContextMenu({
      x: e.clientX,
      y: e.clientY,
      node,
    });
  };

  const closeContextMenu = () => {
    setContextMenu(null);
  };

  const handleCreateFile = (node, type) => {
    const name = prompt(`Enter ${type} name:`);
    if (name) {
      props.onCreateFile?.(node.path, type, name);
      setTimeout(loadTree, 100);
    }
    closeContextMenu();
  };

  const handleDelete = (node) => {
    if (confirm(`Are you sure you want to delete ${node.name}?`)) {
      props.onDeleteFile?.(node.path);
      setTimeout(loadTree, 100);
    }
    closeContextMenu();
  };

  const getFileIcon = (node) => {
    if (node.type === 'folder') {
      return expanded().has(node.path) ? IconFolderOpen : IconFolder;
    }
    const ext = node.name.split('.').pop().toLowerCase();
    const iconMap = {
      'rs': IconBrandRust,
      'jsx': IconBrandReact,
      'js': IconFileCode,
      'json': IconJson,
      'md': IconMarkdown,
      'css': IconPalette,
      'toml': IconSettings,
    };
    return iconMap[ext] || IconFile;
  };

  const renderNode = (node, depth = 0) => {
    const isExpanded = () => expanded().has(node.path);
    const hasChildren = node.children && node.children.length > 0;
    const FileIcon = getFileIcon(node);

    return (
      <div style={{ "padding-left": `${depth * 12}px` }}>
        <div
          class="flex items-center gap-2 px-2 py-1 hover:bg-base-300 cursor-pointer rounded group"
          onClick={(e) => {
            e.stopPropagation();
            if (node.type === 'folder') {
              toggleExpanded(node.path);
            } else {
              props.onFileSelect?.(node);
            }
          }}
          onContextMenu={(e) => handleContextMenu(e, node)}
        >
          {node.type === 'folder' && (
            <span class="text-base-content/60">
              {isExpanded() ? <IconChevronDown size={14} /> : <IconChevronRight size={14} />}
            </span>
          )}
          {node.type === 'file' && <span class="w-[14px]" />}
          <FileIcon size={16} class="text-base-content/80" />
          <span class="text-sm">{node.name}</span>
        </div>
        {node.type === 'folder' && isExpanded() && hasChildren && (
          <div>
            <For each={node.children}>
              {(child) => renderNode(child, depth + 1)}
            </For>
          </div>
        )}
      </div>
    );
  };

  // Close context menu on click outside
  createEffect(() => {
    if (contextMenu()) {
      const handler = () => closeContextMenu();
      document.addEventListener('click', handler);
      onCleanup(() => document.removeEventListener('click', handler));
    }
  });

  return (
    <div class="flex flex-col h-full bg-base-200">
      <div class="flex gap-1 p-2 bg-base-300 border-b border-base-content/10">
        <button onClick={() => handleCreateFile(tree(), 'file')} class="btn btn-xs btn-ghost gap-1" title="New File">
          <IconPlus size={14} />
          File
        </button>
        <button onClick={() => handleCreateFile(tree(), 'folder')} class="btn btn-xs btn-ghost gap-1" title="New Folder">
          <IconFolderPlus size={14} />
          Folder
        </button>
        <button onClick={loadTree} class="btn btn-xs btn-ghost" title="Refresh">
          <IconRefresh size={14} />
        </button>
      </div>
      <div class="flex-1 overflow-y-auto p-2">
        {tree() ? renderNode(tree()) : (
          <div class="flex items-center justify-center h-full text-base-content/60">
            <span class="loading loading-spinner loading-sm" />
          </div>
        )}
      </div>

      <Show when={contextMenu()}>
        <div
          class="menu bg-base-300 rounded-box shadow-xl border border-base-content/10 w-48"
          style={{
            position: 'fixed',
            left: `${contextMenu().x}px`,
            top: `${contextMenu().y}px`,
            "z-index": 1000,
          }}
        >
          <Show when={contextMenu().node.type === 'folder'}>
            <li>
              <a onClick={() => handleCreateFile(contextMenu().node, 'file')}>
                <IconPlus size={16} />
                New File
              </a>
            </li>
            <li>
              <a onClick={() => handleCreateFile(contextMenu().node, 'folder')}>
                <IconFolderPlus size={16} />
                New Folder
              </a>
            </li>
            <li class="menu-title"><span></span></li>
          </Show>
          <li>
            <a onClick={() => handleDelete(contextMenu().node)} class="text-error">
              <IconTrash size={16} />
              Delete
            </a>
          </li>
        </div>
      </Show>
    </div>
  );
}
