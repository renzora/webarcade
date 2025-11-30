import { createSignal, For } from 'solid-js';

export default function DemoLeftPanel() {
  const [expandedFolders, setExpandedFolders] = createSignal(new Set(['src', 'components']));
  const [selectedItem, setSelectedItem] = createSignal(null);

  const fileTree = [
    {
      name: 'src',
      type: 'folder',
      children: [
        {
          name: 'components',
          type: 'folder',
          children: [
            { name: 'Button.jsx', type: 'file' },
            { name: 'Input.jsx', type: 'file' },
            { name: 'Modal.jsx', type: 'file' }
          ]
        },
        {
          name: 'hooks',
          type: 'folder',
          children: [
            { name: 'useAuth.js', type: 'file' },
            { name: 'useStore.js', type: 'file' }
          ]
        },
        { name: 'App.jsx', type: 'file' },
        { name: 'index.js', type: 'file' }
      ]
    },
    {
      name: 'public',
      type: 'folder',
      children: [
        { name: 'index.html', type: 'file' },
        { name: 'favicon.ico', type: 'file' }
      ]
    },
    { name: 'package.json', type: 'file' },
    { name: 'README.md', type: 'file' }
  ];

  const toggleFolder = (name) => {
    setExpandedFolders(prev => {
      const newSet = new Set(prev);
      if (newSet.has(name)) {
        newSet.delete(name);
      } else {
        newSet.add(name);
      }
      return newSet;
    });
  };

  const TreeItem = (props) => {
    const { item, depth = 0 } = props;
    const isExpanded = () => expandedFolders().has(item.name);
    const isSelected = () => selectedItem() === item.name;

    return (
      <div>
        <div
          class={`flex items-center gap-1 px-2 py-1 cursor-pointer hover:bg-base-300 rounded text-sm ${isSelected() ? 'bg-primary/20 text-primary' : ''}`}
          style={{ "padding-left": `${depth * 12 + 8}px` }}
          onClick={() => {
            if (item.type === 'folder') {
              toggleFolder(item.name);
            }
            setSelectedItem(item.name);
          }}
        >
          {item.type === 'folder' ? (
            <>
              <span class="text-xs">{isExpanded() ? '▼' : '▶'}</span>
              <span class="text-warning">{isExpanded() ? '📂' : '📁'}</span>
            </>
          ) : (
            <>
              <span class="w-3" />
              <span>📄</span>
            </>
          )}
          <span class="truncate">{item.name}</span>
        </div>
        {item.type === 'folder' && isExpanded() && item.children && (
          <For each={item.children}>
            {(child) => <TreeItem item={child} depth={depth + 1} />}
          </For>
        )}
      </div>
    );
  };

  return (
    <div class="h-full flex flex-col bg-base-100">
      <div class="px-3 py-2 border-b border-base-300">
        <h3 class="font-semibold text-sm text-base-content">Explorer</h3>
        <p class="text-xs text-base-content/50">Demo file tree</p>
      </div>

      <div class="flex-1 overflow-auto py-2">
        <For each={fileTree}>
          {(item) => <TreeItem item={item} />}
        </For>
      </div>

      <div class="px-3 py-2 border-t border-base-300 text-xs text-base-content/50">
        {selectedItem() ? (
          <span>Selected: {selectedItem()}</span>
        ) : (
          <span>Click to select</span>
        )}
      </div>
    </div>
  );
}
