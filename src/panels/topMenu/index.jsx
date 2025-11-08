import { createSignal, createEffect, onCleanup, createMemo, For, Show } from 'solid-js';
import { IconChevronRight, IconMinus, IconSquare, IconCopy, IconX, IconBell } from '@tabler/icons-solidjs';
import { editorStore, editorActions } from '@/layout/stores/EditorStore';
import { topMenuItems, horizontalMenuButtonsEnabled } from '@/api/plugin';
import { getCurrentWindow } from '@tauri-apps/api/window';
import ViewportTabs from '@/panels/viewport/ViewportTabs.jsx';

function TopMenu() {
  const [activeMenu, setActiveMenu] = createSignal(null);
  const [isSaving, setIsSaving] = createSignal(false);
  const [lastSync, _setLastSync] = createSignal(null);
  const [hasUnsavedChanges, _setHasUnsavedChanges] = createSignal(false);
  const [_showSyncTooltip, _setShowSyncTooltip] = createSignal(false);
  const [_showUpdateTooltip, _setShowUpdateTooltip] = createSignal(false);
  const [showProjectManager, setShowProjectManager] = createSignal(false);
  const [menuPosition, setMenuPosition] = createSignal(null);
  const [isMaximized, setIsMaximized] = createSignal(false);
  const [showNotifications, setShowNotifications] = createSignal(false);
  const [notificationCount, setNotificationCount] = createSignal(3);

  // Check initial maximize state and listen for window changes
  createEffect(() => {
    if (typeof window !== 'undefined' && window.__TAURI_INTERNALS__) {
      const checkMaximizeState = async () => {
        try {
          const currentWindow = getCurrentWindow();
          const maximized = await currentWindow.isMaximized();
          setIsMaximized(maximized);
        } catch (error) {
        }
      };

      // Check initial state
      checkMaximizeState();

      // Listen for window resize events to update maximize state
      const handleResize = () => {
        checkMaximizeState();
      };

      window.addEventListener('resize', handleResize);
      
      // Also listen for custom engine events
      const handleProjectSelected = () => {
        // Delay check to allow window maximize to complete
        setTimeout(checkMaximizeState, 100);
      };

      document.addEventListener('engine:project-selected', handleProjectSelected);

      onCleanup(() => {
        window.removeEventListener('resize', handleResize);
        document.removeEventListener('engine:project-selected', handleProjectSelected);
      });
    }
  });

  // Tauri window control functions
  const handleMinimize = async () => {
    try {
      const window = getCurrentWindow();
      await window.minimize();
    } catch (error) {
    }
  };

  const handleMaximize = async () => {
    try {
      const window = getCurrentWindow();
      const currentMaximized = await window.isMaximized();
      if (currentMaximized) {
        await window.unmaximize();
        setIsMaximized(false);
      } else {
        await window.maximize();
        setIsMaximized(true);
      }
    } catch (error) {
    }
  };

  const handleClose = async () => {
    try {
      
      // Helper function to actually close the application
      const proceedWithClose = async () => {
        try {
          // Emit event to backend to approve the close
          const { emit } = await import('@tauri-apps/api/event');
          await emit('proceed-with-close');
        } catch (closeError) {
          // Fallback to direct window close
          try {
            const window = getCurrentWindow();
            await window.close();
          } catch (windowError) {
            alert('Unable to close the application. Please close manually.');
          }
        }
      };

      // No unsaved changes system - just close directly
      
      // No unsaved changes, close immediately
      await proceedWithClose();
      
    } catch (error) {
      // If there's an error, ask user if they want to close anyway using browser confirm as fallback
      const closeAnyway = confirm('An error occurred while checking for unsaved changes. Do you want to close anyway?');
      if (closeAnyway) {
        try {
          const { emit } = await import('@tauri-apps/api/event');
          await emit('proceed-with-close');
        } catch (exitError) {
        }
      }
    }
  };

  // Click outside detection for dropdowns
  createEffect(() => {
    const handleClickOutside = (event) => {
      const target = event.target;
      const isMenuButton = target.closest('.menu-button');
      const isDropdownContent = target.closest('.dropdown-content');
      
      if (!isMenuButton && !isDropdownContent) {
        // Close all dropdowns and menus
        setActiveMenu(null);
        setMenuPosition(null);
      }
    };

    if (activeMenu()) {
      document.addEventListener('mousedown', handleClickOutside);
      onCleanup(() => {
        document.removeEventListener('mousedown', handleClickOutside);
      });
    }
  });

  const calculateDropdownPosition = (buttonRect, dropdownWidth = 192) => {
    const viewportWidth = window.innerWidth;
    const margin = 8;
    
    // Try to center the dropdown under the button
    let left = buttonRect.left + (buttonRect.width / 2) - (dropdownWidth / 2);
    
    // Check if dropdown would go off the right edge
    if (left + dropdownWidth + margin > viewportWidth) {
      left = viewportWidth - dropdownWidth - margin;
    }
    
    // Check if dropdown would go off the left edge
    if (left < margin) {
      left = margin;
    }
    
    return {
      left,
      top: buttonRect.bottom + 4
    };
  };

  const _settings = createMemo(() => editorStore.settings);

  const _handleSave = async () => {
    if (isSaving()) return;
    
    try {
      setIsSaving(true);
      // TODO: Implement actual save functionality
      editorActions.addConsoleMessage('Save not implemented', 'warning');
    } catch (error) {
      editorActions.addConsoleMessage('Failed to save project', 'error');
    } finally {
      setIsSaving(false);
    }
  };

  const formatLastSync = (date) => {
    if (!date) return 'Never synced';
    
    const now = new Date();
    const diff = now - date;
    const minutes = Math.floor(diff / 60000);
    const hours = Math.floor(diff / 3600000);
    const days = Math.floor(diff / 86400000);
    
    if (minutes < 1) return 'Just now';
    if (minutes < 60) return `${minutes}m ago`;
    if (hours < 24) return `${hours}h ago`;
    return `${days}d ago`;
  };

  const _getSyncStatusInfo = createMemo(() => {
    if (hasUnsavedChanges()) {
      return {
        color: 'bg-yellow-500',
        tooltip: 'Unsaved changes - will auto-save soon'
      };
    }
    return {
      color: 'bg-green-500',
      tooltip: `Last sync: ${formatLastSync(lastSync())}`
    };
  });


  // Create dynamic menu structure from plugin extensions only
  const menuStructure = createMemo(() => {
    const pluginMenuItems = topMenuItems();
    const pluginMenuArray = Array.from(pluginMenuItems.values())
      .sort((a, b) => (a.order || 0) - (b.order || 0));
    
    const menuStructure = {};
    
    // Add plugin menu items as top-level menus
    pluginMenuArray.forEach(item => {
      menuStructure[item.label] = item.submenu || [
        { 
          id: item.id, 
          label: item.label, 
          icon: item.icon,
          action: item.onClick 
        }
      ];
    });

    return menuStructure;
  });

  const handleMenuClick = (menuName, event) => {
    if (activeMenu() === menuName) {
      setActiveMenu(null);
      setMenuPosition(null);
    } else {
      
      const rect = event.currentTarget.getBoundingClientRect();
      const position = calculateDropdownPosition(rect, 224); // Menu width is 224px (w-56)
      setMenuPosition({
        left: position.left,
        top: rect.bottom + 1
      });
      setActiveMenu(menuName);
    }
  };

  const handleItemClick = (item) => {
    setActiveMenu(null);
    setMenuPosition(null);
    if (item.action) {
      item.action();
    } else if (['new', 'open', 'export'].includes(item.id)) {
      setShowProjectManager(true);
    } else {
      editorActions.addConsoleMessage(`Menu action: ${item.label}`, 'info');
    }
  };

  return (
    <>
      <div
        class="relative w-full h-8 bg-base-200 backdrop-blur-md shadow-sm border-b border-black/30 flex items-center px-2"
        data-tauri-drag-region
      >
        {/* Viewport Tabs */}
        <div class="flex items-center overflow-hidden">
          <ViewportTabs />
        </div>

        {/* Draggable spacer - provides area to drag window */}
        <div class="flex-1 min-w-16" data-tauri-drag-region />

        {/* Show old menu buttons if horizontalMenuButtonsEnabled */}
        <Show when={horizontalMenuButtonsEnabled()}>
          <div
            style={{
              '-webkit-app-region': 'no-drag'
            }}
            class="flex items-center"
          >
            <For each={Object.entries(menuStructure())}>
              {([menuName, _items]) => (
                <div class="relative inline-block">
                  <button
                    onClick={(e) => handleMenuClick(menuName, e)}
                    onMouseEnter={(e) => {
                      if (activeMenu()) {

                        const rect = e.currentTarget.getBoundingClientRect();
                        const position = calculateDropdownPosition(rect, 224);
                        setMenuPosition({
                          left: position.left,
                          top: rect.bottom + 1
                        });
                        setActiveMenu(menuName);
                      } else {
                      }
                    }}
                    class={`menu-button px-3 py-1 text-sm text-base-content hover:bg-base-300 rounded transition-colors cursor-pointer ${
                      activeMenu() === menuName ? 'bg-base-300' : ''
                    }`}
                  >
                    {menuName}
                  </button>
                </div>
              )}
            </For>
          </div>
        </Show>

        {/* Notifications Button */}
        <div
          class="flex items-center"
          style={{
            '-webkit-app-region': 'no-drag'
          }}
        >
          <button
            onClick={() => setShowNotifications(!showNotifications())}
            class="relative w-8 h-8 flex items-center justify-center text-base-content/60 hover:text-base-content hover:bg-base-300 rounded transition-colors cursor-pointer"
            title="Notifications"
            style={{ '-webkit-app-region': 'no-drag' }}
          >
            <IconBell class="w-4 h-4" />
            <Show when={notificationCount() > 0}>
              <span class="absolute top-1 right-1 w-2 h-2 bg-error rounded-full"></span>
            </Show>
          </button>
        </div>

        {/* Tauri Window Controls - Only show in desktop app */}
        {typeof window !== 'undefined' && window.__TAURI_INTERNALS__ && (
          <div
            class="flex items-center gap-3 text-xs text-gray-500"
            style={{
              '-webkit-app-region': 'no-drag'
            }}
          >
            <div class="flex items-center">
              <button
                onClick={(e) => {
                  e.preventDefault();
                  e.stopPropagation();
                  handleMinimize();
                }}
                class="w-8 h-8 flex items-center justify-center text-base-content/60 hover:text-base-content hover:bg-base-300 rounded transition-colors cursor-pointer"
                title="Minimize"
                style={{ '-webkit-app-region': 'no-drag' }}
              >
                <IconMinus class="w-4 h-4" />
              </button>
              <button
                onClick={handleMaximize}
                class="w-8 h-8 flex items-center justify-center text-base-content/60 hover:text-base-content hover:bg-base-300 rounded transition-colors cursor-pointer"
                title={isMaximized() ? "Restore" : "Maximize"}
                style={{ '-webkit-app-region': 'no-drag' }}
              >
                {isMaximized() ? <IconCopy class="w-4 h-4" /> : <IconSquare class="w-4 h-4" />}
              </button>
              <button
                onClick={handleClose}
                class="w-8 h-8 flex items-center justify-center text-base-content/60 hover:text-error hover:bg-error/10 rounded transition-colors cursor-pointer"
                title="Close"
                style={{ '-webkit-app-region': 'no-drag' }}
              >
                <IconX class="w-4 h-4" />
              </button>
            </div>
          </div>
        )}
      </div>
      
      <Show when={horizontalMenuButtonsEnabled() && activeMenu() && menuPosition()}>
        <div 
          class="dropdown-content fixed w-56 bg-base-200 backdrop-blur-sm rounded-lg shadow-xl z-[110] border border-base-300"
          style={{
            left: menuPosition().left + 'px',
            top: menuPosition().top + 'px'
          }}
        >
          <div class="p-1">
            <For each={menuStructure()[activeMenu()]}>
              {(item, _index) => (
                <>
                  <Show when={item.divider}>
                    <div class="border-t border-base-300 my-1 mx-2" />
                  </Show>
                  <Show when={!item.divider}>
                    <div class="relative group/item">
                      <button
                        class="w-full px-3 py-1.5 text-left text-sm text-base-content hover:bg-primary hover:text-primary-content flex items-center justify-between transition-all duration-150 relative rounded-md cursor-pointer"
                        onClick={() => item.submenu ? null : handleItemClick(item)}
                      >
                        <div class="flex items-center gap-2">
                          <Show when={item.icon}>
                            <span class="w-4 h-4 flex items-center justify-center text-base-content/60 group-hover/item:text-primary-content">
                              <Show when={item.id === 'save' && isSaving()} fallback={<item.icon class="w-3.5 h-3.5" />}>
                                <div class="w-3 h-3 border-2 border-base-content/60 border-t-transparent rounded-full animate-spin" />
                              </Show>
                            </span>
                          </Show>
                          <span class="font-normal">
                            {item.id === 'save' && isSaving() ? 'Saving...' : item.label}
                          </span>
                        </div>
                        <Show when={item.shortcut}>
                          <span class="ml-auto text-xs text-base-content/40 group-hover/item:text-primary-content/70">{item.shortcut}</span>
                        </Show>
                        <Show when={item.submenu}>
                          <IconChevronRight class="w-3 h-3 text-base-content/60 group-hover/item:text-primary-content ml-auto" />
                        </Show>
                      </button>
                      
                      {/* Nested submenu - CSS hover activated */}
                      <Show when={item.submenu}>
                        <div class="absolute left-full top-0 -ml-1 w-56 bg-base-200 backdrop-blur-sm rounded-lg shadow-xl border border-base-300 opacity-0 invisible group-hover/item:opacity-100 group-hover/item:visible transition-all duration-200 z-[120] before:absolute before:inset-y-0 before:-left-1 before:w-2 before:content-['']">
                          <div class="p-1">
                            <For each={item.submenu}>
                              {(subItem) => (
                                <>
                                  <Show when={subItem.divider}>
                                    <div class="border-t border-base-300 my-1 mx-2" />
                                  </Show>
                                  <Show when={!subItem.divider}>
                                    <div class="relative group/subitem">
                                      <button
                                        class="w-full px-3 py-1.5 text-left text-sm text-base-content hover:bg-primary hover:text-primary-content flex items-center justify-between transition-all duration-150 rounded-md cursor-pointer"
                                        onClick={() => subItem.submenu ? null : handleItemClick(subItem)}
                                      >
                                        <div class="flex items-center gap-2">
                                          <Show when={subItem.icon}>
                                            <span class="w-4 h-4 flex items-center justify-center text-base-content/60 group-hover/subitem:text-primary-content">
                                              <subItem.icon class="w-3.5 h-3.5" />
                                            </span>
                                          </Show>
                                          <span class="font-normal">{subItem.label}</span>
                                        </div>
                                        <Show when={subItem.shortcut}>
                                          <span class="ml-auto text-xs text-base-content/40 group-hover/subitem:text-primary-content/70">{subItem.shortcut}</span>
                                        </Show>
                                        <Show when={subItem.submenu}>
                                          <IconChevronRight class="w-3 h-3 text-base-content/60 group-hover/subitem:text-primary-content ml-auto" />
                                        </Show>
                                      </button>
                                      
                                      {/* Third level submenu */}
                                      <Show when={subItem.submenu}>
                                        <div class="absolute left-full top-0 -ml-1 w-56 bg-base-200 backdrop-blur-sm rounded-lg shadow-xl border border-base-300 opacity-0 invisible group-hover/subitem:opacity-100 group-hover/subitem:visible transition-all duration-200 z-[130] before:absolute before:inset-y-0 before:-left-1 before:w-2 before:content-['']">
                                          <div class="p-1">
                                            <For each={subItem.submenu}>
                                              {(thirdItem) => (
                                                <button
                                                  class="w-full px-3 py-1.5 text-left text-sm text-base-content hover:bg-primary hover:text-primary-content flex items-center gap-2 transition-all duration-150 rounded-md cursor-pointer"
                                                  onClick={() => handleItemClick(thirdItem)}
                                                >
                                                  <Show when={thirdItem.icon}>
                                                    <span class="w-4 h-4 flex items-center justify-center text-base-content/60">
                                                      <thirdItem.icon class="w-3.5 h-3.5" />
                                                    </span>
                                                  </Show>
                                                  <span class="font-normal">{thirdItem.label}</span>
                                                  <Show when={thirdItem.shortcut}>
                                                    <span class="ml-auto text-xs text-base-content/40">{thirdItem.shortcut}</span>
                                                  </Show>
                                                </button>
                                              )}
                                            </For>
                                          </div>
                                        </div>
                                      </Show>
                                    </div>
                                  </Show>
                                </>
                              )}
                            </For>
                          </div>
                        </div>
                      </Show>
                    </div>
                  </Show>
                </>
              )}
            </For>
          </div>
        </div>
      </Show>
      

      <Show when={showProjectManager()}>
        <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div class="bg-base-200 p-6 rounded-xl">
            <h2 class="text-base-content mb-4">Project Manager</h2>
            <p class="text-base-content/80 mb-4">Project manager coming soon...</p>
            <button 
              onClick={() => setShowProjectManager(false)}
              class="px-4 py-2 bg-primary text-primary-content rounded"
            >
              Close
            </button>
          </div>
        </div>
      </Show>
    </>
  );
}

export default TopMenu;