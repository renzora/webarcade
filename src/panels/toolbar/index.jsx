import { Show, For, createMemo } from 'solid-js';
import { toolbarItems, toolbarGroups, toolbarVisible, activePlugin } from '@/api/plugin';

const Toolbar = () => {
  // Get sorted toolbar items grouped by their group, filtered by active plugin
  const groupedItems = createMemo(() => {
    const currentPlugin = activePlugin();
    const allItems = Array.from(toolbarItems().values());
    const allGroups = Array.from(toolbarGroups().values());

    // Filter items by active plugin
    const items = allItems
      .filter(item => item.plugin === currentPlugin)
      .sort((a, b) => a.order - b.order);

    // Filter groups by active plugin
    const groups = allGroups
      .filter(group => group.plugin === currentPlugin)
      .sort((a, b) => a.order - b.order);

    // Create a map of group id to items
    const groupMap = new Map();

    // Initialize with registered groups
    groups.forEach(group => {
      groupMap.set(group.id, { ...group, items: [] });
    });

    // Add default group if not registered
    if (!groupMap.has('default')) {
      groupMap.set('default', { id: 'default', label: '', order: 0, items: [] });
    }

    // Assign items to their groups
    items.forEach(item => {
      const groupId = item.group || 'default';
      if (!groupMap.has(groupId)) {
        groupMap.set(groupId, { id: groupId, label: '', order: 100, items: [] });
      }
      const visible = typeof item.visible === 'function' ? item.visible() : item.visible;
      if (visible !== false) {
        groupMap.get(groupId).items.push(item);
      }
    });

    // Convert to array and filter out empty groups
    return Array.from(groupMap.values())
      .filter(group => group.items.length > 0)
      .sort((a, b) => a.order - b.order);
  });

  const hasItems = createMemo(() => {
    return groupedItems().some(group => group.items.length > 0);
  });

  return (
    <Show when={toolbarVisible() && hasItems()}>
      <div class="flex-shrink-0 flex items-stretch bg-base-200 border-b border-base-300 px-2 py-1" data-tauri-drag-region>
        <div class="flex items-center gap-1" style={{ '-webkit-app-region': 'no-drag' }}>
          <For each={groupedItems()}>
            {(group, groupIndex) => (
              <>
                {/* Group separator (except for first group) */}
                <Show when={groupIndex() > 0}>
                  <div class="w-px h-6 bg-base-300 mx-1" />
                </Show>

                {/* Group label if present */}
                <Show when={group.label}>
                  <span class="text-xs text-base-content/50 px-1">{group.label}</span>
                </Show>

                {/* Group items */}
                <div class="flex items-center gap-0.5">
                  <For each={group.items}>
                    {(item) => (
                      <>
                        <Show when={item.component} fallback={
                          <ToolbarButton item={item} />
                        }>
                          {(() => {
                            const Component = item.component;
                            return <Component />;
                          })()}
                        </Show>

                        {/* Item separator */}
                        <Show when={item.separator}>
                          <div class="w-px h-5 bg-base-300 mx-1" />
                        </Show>
                      </>
                    )}
                  </For>
                </div>
              </>
            )}
          </For>
        </div>
        {/* Spacer for window dragging */}
        <div class="flex-1" data-tauri-drag-region />
      </div>
    </Show>
  );
};

const ToolbarButton = ({ item }) => {
  const isDisabled = () => typeof item.disabled === 'function' ? item.disabled() : item.disabled;
  const isActive = () => typeof item.active === 'function' ? item.active() : item.active;

  const handleClick = (e) => {
    e.preventDefault();
    e.stopPropagation();
    if (!isDisabled() && item.onClick) {
      item.onClick();
    }
  };

  return (
    <button
      class={`
        flex items-center justify-center w-7 h-7 rounded transition-colors
        ${isActive()
          ? 'bg-primary text-primary-content'
          : 'text-base-content/70 hover:text-base-content hover:bg-base-300'
        }
        ${isDisabled() ? 'opacity-40 cursor-not-allowed' : 'cursor-pointer'}
      `}
      onClick={handleClick}
      disabled={isDisabled()}
      title={item.tooltip}
    >
      <Show when={item.icon}>
        {(() => {
          const Icon = item.icon;
          return <Icon class="w-4 h-4" />;
        })()}
      </Show>
      <Show when={!item.icon && item.label}>
        <span class="text-xs font-medium px-1">{item.label}</span>
      </Show>
    </button>
  );
};

export default Toolbar;
