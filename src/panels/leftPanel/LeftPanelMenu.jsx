import { createSignal, createMemo, For, Show } from 'solid-js';
import { leftPanelMenuItems, registeredPlugins, viewportTypes } from '@/api/plugin';
import pluginStore from '../../../plugins/plugins/PluginStore.jsx';
import { IconSearch } from '@tabler/icons-solidjs';

const LeftPanelMenu = () => {
  const [searchQuery, setSearchQuery] = createSignal('');
  const [showDescriptions, setShowDescriptions] = createSignal(
    localStorage.getItem('leftPanelShowDescriptions') !== 'false'
  );
  const [uploading, setUploading] = createSignal(false);
  const [isDragging, setIsDragging] = createSignal(false);
  let fileInputRef;

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

  const handleAddPlugin = () => {
    fileInputRef?.click();
  };

  const handleFileSelect = async (e) => {
    const file = e.target.files?.[0];
    if (!file) return;

    if (!file.name.endsWith('.zip')) {
      alert('Please select a .zip file');
      return;
    }

    await uploadPlugin(file);

    // Reset file input
    if (fileInputRef) {
      fileInputRef.value = '';
    }
  };

  const uploadPlugin = async (file) => {
    setUploading(true);

    try {
      const formData = new FormData();
      formData.append('plugin', file);

      const response = await fetch('http://localhost:3001/system/plugins/upload', {
        method: 'POST',
        body: formData,
      });

      if (!response.ok) {
        const error = await response.text();
        throw new Error(error || 'Failed to upload plugin');
      }

      const result = await response.json();
      alert(`Plugin "${result.pluginName}" installed successfully! Please restart the application.`);
    } catch (error) {
      console.error('Error uploading plugin:', error);
      alert(`Error uploading plugin: ${error.message}`);
    } finally {
      setUploading(false);
    }
  };

  const deletePlugin = async (menuItemId, e) => {
    e.stopPropagation(); // Prevent triggering the menu item click

    // Extract viewport typeId from menu item ID (e.g., "viewport-test-plugin" -> "test-plugin")
    let viewportId = menuItemId;
    if (menuItemId.startsWith('viewport-')) {
      viewportId = menuItemId.substring('viewport-'.length);
    }

    // Convert viewport ID to potential plugin ID (test-plugin -> test_plugin)
    const potentialPluginId = viewportId.replace(/-/g, '_');
    let pluginFolderName = null;

    // Get plugin configs from store
    const pluginConfigs = pluginStore.getPluginConfigs();

    for (const [configId, pluginConfig] of pluginConfigs) {
      // Extract folder name from path
      const pathParts = pluginConfig.path.split('/').filter(p => p);
      const folderName = pathParts[pathParts.length - 1];

      // Check if this plugin matches by:
      // 1. Plugin ID matches (test_plugin === test_plugin)
      // 2. Plugin ID normalized matches viewport ID (test_plugin -> test-plugin === test-plugin)
      // 3. Folder name matches potential plugin ID (test_plugin === test_plugin)
      const normalizedId = pluginConfig.id.replace(/_/g, '-');

      if (pluginConfig.id === potentialPluginId ||
          normalizedId === viewportId ||
          folderName === potentialPluginId) {
        pluginFolderName = folderName;
        console.log(`[Delete Plugin] Found plugin folder: ${folderName} for viewport: ${viewportId}`);
        break;
      }
    }

    if (!pluginFolderName) {
      alert(`Could not find plugin folder for "${viewportId}"\n\nSearched for: ${potentialPluginId}`);
      console.error('[Delete Plugin] Could not find folder for viewport:', viewportId);
      return;
    }

    if (!confirm(`Are you sure you want to delete the plugin "${pluginFolderName}"?\n\nThis will remove all plugin files and cannot be undone.`)) {
      return;
    }

    try {
      console.log(`[Delete Plugin] Deleting plugin: ${pluginFolderName}`);
      const response = await fetch(`http://localhost:3001/system/plugins/${encodeURIComponent(pluginFolderName)}`, {
        method: 'DELETE',
      });

      if (!response.ok) {
        const error = await response.json();
        throw new Error(error.error || 'Failed to delete plugin');
      }

      const result = await response.json();
      alert(`Plugin "${result.pluginName}" deleted successfully! The application will reload.`);

      // Reload the page to refresh the plugin list
      window.location.reload();
    } catch (error) {
      console.error('Error deleting plugin:', error);
      alert(`Error deleting plugin: ${error.message}`);
    }
  };

  const handleDragOver = (e) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragging(true);
  };

  const handleDragLeave = (e) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragging(false);
  };

  const handleDrop = async (e) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragging(false);

    const files = Array.from(e.dataTransfer.files);
    const zipFile = files.find(file => file.name.endsWith('.zip'));

    if (!zipFile) {
      alert('Please drop a .zip file');
      return;
    }

    await uploadPlugin(zipFile);
  };

  return (
    <div
      className="flex flex-col h-full bg-base-200 relative"
      onDragOver={handleDragOver}
      onDragLeave={handleDragLeave}
      onDrop={handleDrop}
    >
      {/* Drag overlay */}
      <Show when={isDragging()}>
        <div className="absolute inset-0 bg-primary/10 border-2 border-dashed border-primary z-50 flex items-center justify-center">
          <div className="text-center">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" className="w-12 h-12 mx-auto mb-2 text-primary">
              <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
              <polyline points="17 8 12 3 7 8"/>
              <line x1="12" y1="3" x2="12" y2="15"/>
            </svg>
            <p className="text-sm font-medium text-primary">Drop plugin .zip file here</p>
          </div>
        </div>
      </Show>

      {/* Hidden file input for plugin upload */}
      <input
        ref={fileInputRef}
        type="file"
        accept=".zip"
        onChange={handleFileSelect}
        className="hidden"
      />

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
                    <div className="flex items-center gap-1">
                      {/* Add Plugin button */}
                      <button
                        onClick={handleAddPlugin}
                        disabled={uploading()}
                        className="flex items-center justify-center w-5 h-5 text-base-content/40 hover:text-base-content hover:bg-base-300 rounded transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                        title="Add plugin"
                      >
                        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" className="w-3 h-3">
                          <path d="M12 5v14M5 12h14"/>
                        </svg>
                      </button>

                      {/* Description toggle button */}
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
                    </div>
                  </Show>
                </div>

                {/* Category items */}
                <div>
                  <For each={items}>
                    {(item, itemIndex) => {
                      // Check if this is a deletable plugin (not a core plugin)
                      const isDeletable = item.id && !item.id.startsWith('core-');

                      return (
                        <div className="relative group/item">
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

                            {/* Delete button - only for non-core plugins */}
                            <Show when={isDeletable}>
                              <div className="opacity-0 group-hover/item:opacity-100 transition-opacity">
                                <button
                                  onClick={(e) => deletePlugin(item.id, e)}
                                  className="w-6 h-6 flex items-center justify-center text-error/60 hover:text-error hover:bg-error/10 rounded transition-colors"
                                  title="Delete plugin"
                                >
                                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" className="w-3.5 h-3.5">
                                    <path d="M3 6h18M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6"/>
                                    <path d="M10 11v6M14 11v6"/>
                                  </svg>
                                </button>
                              </div>
                            </Show>
                          </button>
                        </div>
                      );
                    }}
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
