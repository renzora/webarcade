import { createSignal, createMemo, For, Show } from 'solid-js';
import { leftPanelMenuItems } from '@/api/plugin';
import { IconSearch } from '@tabler/icons-solidjs';

const LeftPanelMenu = () => {
  const [searchQuery, setSearchQuery] = createSignal('');
  const [showDescriptions, setShowDescriptions] = createSignal(
    localStorage.getItem('leftPanelShowDescriptions') !== 'false'
  );

  // Get all menu items and organize by category
  const menuItems = createMemo(() => {
    const items = Array.from(leftPanelMenuItems().values())
      .sort((a, b) => a.order - b.order);

    return items;
  });

  // Filter menu items based on search query
  const filteredMenuItems = createMemo(() => {
    const query = searchQuery().toLowerCase().trim();

    if (!query) {
      return menuItems();
    }

    return menuItems().filter(item => {
      const labelMatch = item.label?.toLowerCase().includes(query);
      const descriptionMatch = item.description?.toLowerCase().includes(query);
      const categoryMatch = item.category?.toLowerCase().includes(query);

      return labelMatch || descriptionMatch || categoryMatch;
    });
  });

  // Group items by category
  const groupedMenuItems = createMemo(() => {
    const items = filteredMenuItems();
    const groups = new Map();

    items.forEach(item => {
      const category = item.category || 'General';
      if (!groups.has(category)) {
        groups.set(category, []);
      }
      groups.get(category).push(item);
    });

    return Array.from(groups.entries());
  });

  const handleItemClick = (item) => {
    if (item.onClick) {
      item.onClick();
    }
  };

  const handleSearchInput = (e) => {
    setSearchQuery(e.target.value);
  };

  const clearSearch = () => {
    setSearchQuery('');
  };

  const toggleDescriptions = () => {
    const newValue = !showDescriptions();
    setShowDescriptions(newValue);
    localStorage.setItem('leftPanelShowDescriptions', String(newValue));
  };

  return (
    <div className="flex flex-col h-full bg-base-200">
      {/* Header */}
      <div className="flex-shrink-0 p-2 border-b border-base-300">
        {/* Search bar */}
        <div className="relative">
          <div className="absolute left-3 top-1/2 -translate-y-1/2 pointer-events-none">
            <IconSearch className="w-4 h-4 text-base-content/40" />
          </div>
          <input
            type="text"
            value={searchQuery()}
            onInput={handleSearchInput}
            placeholder="Search menu items..."
            className="w-full pl-9 pr-8 py-1.5 text-sm bg-base-300 border border-base-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary/50 focus:border-primary text-base-content placeholder-base-content/40"
          />
          <Show when={searchQuery()}>
            <button
              onClick={clearSearch}
              className="absolute right-2 top-1/2 -translate-y-1/2 w-5 h-5 flex items-center justify-center rounded hover:bg-base-content/10 transition-colors"
              title="Clear search"
            >
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" className="w-3 h-3">
                <path d="M18 6L6 18M6 6l12 12"/>
              </svg>
            </button>
          </Show>
        </div>
      </div>

      {/* Menu items */}
      <div className="flex-1 overflow-y-auto scrollbar-thin p-2">
        <Show
          when={groupedMenuItems().length > 0}
          fallback={
            <div className="flex flex-col items-center justify-center h-full text-center p-4">
              <IconSearch className="w-8 h-8 text-base-content/20 mb-2" />
              <p className="text-sm text-base-content/40">
                {searchQuery() ? 'No items found' : 'No menu items available'}
              </p>
              <Show when={searchQuery()}>
                <p className="text-xs text-base-content/30 mt-1">
                  Try a different search term
                </p>
              </Show>
            </div>
          }
        >
          <For each={groupedMenuItems()}>
            {([category, items], index) => (
              <div className="mb-2">
                {/* Category header */}
                <div className="px-2 py-0.5 mb-0.5 flex items-center justify-between">
                  <h3 className="text-xs font-semibold text-base-content/50 uppercase tracking-wide">
                    {category}
                  </h3>
                  <Show when={index() === 0}>
                    <button
                      onClick={toggleDescriptions}
                      className="flex items-center justify-center w-5 h-5 text-base-content/40 hover:text-base-content hover:bg-base-300 rounded transition-colors"
                      title={showDescriptions() ? "Hide descriptions" : "Show descriptions"}
                    >
                      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" className="w-3 h-3">
                        {showDescriptions() ? (
                          <>
                            <path d="M3 12h18"/>
                            <path d="M3 6h18"/>
                            <path d="M3 18h18"/>
                          </>
                        ) : (
                          <>
                            <path d="M10 6h11"/>
                            <path d="M10 12h11"/>
                            <path d="M10 18h11"/>
                            <rect x="3" y="5" width="2" height="2"/>
                            <rect x="3" y="11" width="2" height="2"/>
                            <rect x="3" y="17" width="2" height="2"/>
                          </>
                        )}
                      </svg>
                    </button>
                  </Show>
                </div>

                {/* Category items */}
                <div>
                  <For each={items}>
                    {(item, itemIndex) => (
                      <button
                        onClick={() => handleItemClick(item)}
                        classList={{
                          "w-full flex items-start gap-2 px-2 py-1.5 text-sm text-base-content transition-all group cursor-pointer border-b border-base-300": true,
                          "bg-base-200 hover:bg-primary/20": itemIndex() % 2 === 0,
                          "bg-base-content/[0.02] hover:bg-primary/20": itemIndex() % 2 === 1,
                        }}
                        title={item.description}
                      >
                        {/* Icon */}
                        <Show when={item.icon}>
                          <div className="w-4 h-4 mt-0.5 text-base-content/60 group-hover:text-base-content flex-shrink-0">
                            <item.icon className="w-4 h-4" />
                          </div>
                        </Show>

                        {/* Label and description */}
                        <div className="flex-1 text-left min-w-0">
                          <div className="font-medium text-base-content group-hover:text-base-content text-xs">
                            {item.label}
                          </div>
                          <Show when={showDescriptions() && item.description}>
                            <div className="text-xs text-base-content/50 group-hover:text-base-content/70 mt-0.5 leading-tight">
                              {item.description}
                            </div>
                          </Show>
                        </div>
                      </button>
                    )}
                  </For>
                </div>
              </div>
            )}
          </For>
        </Show>
      </div>

      {/* Footer with item count */}
      <div className="flex-shrink-0 px-4 py-2 border-t border-base-300 bg-base-300/30">
        <p className="text-xs text-base-content/40 text-center">
          {filteredMenuItems().length} {filteredMenuItems().length === 1 ? 'item' : 'items'}
          {searchQuery() && menuItems().length !== filteredMenuItems().length &&
            ` (of ${menuItems().length} total)`
          }
        </p>
      </div>
    </div>
  );
};

export default LeftPanelMenu;
