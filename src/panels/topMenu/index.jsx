import { createSignal, createEffect, onCleanup, createMemo, For, Show } from 'solid-js';
import { IconChevronRight, IconMinus, IconSquare, IconCopy, IconX } from '@tabler/icons-solidjs';
import { topMenuItems, topMenuButtons, horizontalMenuButtonsEnabled, viewportTabsVisible } from '@/api/plugin';
import { getCurrentWindow } from '@tauri-apps/api/window';
import ViewportTabs from '@/panels/viewport/ViewportTabs.jsx';
import { viewportStore } from '@/panels/viewport/store';

// Signal to track if top menu has items (exported for ViewportTabs to use)
export const [hasTopMenuItems, setHasTopMenuItems] = createSignal(false);

function TopMenu() {
  const [activeMenu, setActiveMenu] = createSignal(null);
  const [isSaving, setIsSaving] = createSignal(false);
  const [menuPosition, setMenuPosition] = createSignal(null);
  const [isMaximized, setIsMaximized] = createSignal(false);

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

      checkMaximizeState();

      const handleResize = () => {
        checkMaximizeState();
      };

      window.addEventListener('resize', handleResize);

      onCleanup(() => {
        window.removeEventListener('resize', handleResize);
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
      const proceedWithClose = async () => {
        try {
          const { emit } = await import('@tauri-apps/api/event');
          await emit('proceed-with-close');
        } catch (closeError) {
          try {
            const window = getCurrentWindow();
            await window.close();
          } catch (windowError) {
            alert('Unable to close the application. Please close manually.');
          }
        }
      };

      await proceedWithClose();

    } catch (error) {
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

    let left = buttonRect.left + (buttonRect.width / 2) - (dropdownWidth / 2);

    if (left + dropdownWidth + margin > viewportWidth) {
      left = viewportWidth - dropdownWidth - margin;
    }

    if (left < margin) {
      left = margin;
    }

    return {
      left,
      top: buttonRect.bottom + 4
    };
  };

  // Get the current viewport type directly from the store
  const currentViewportType = () => {
    const activeTab = viewportStore.tabs.find(t => t.id === viewportStore.activeTabId);
    return activeTab?.type || null;
  };

  // Create dynamic menu structure from plugin extensions only, filtered by viewport
  const menuStructure = createMemo(() => {
    const currentViewport = currentViewportType();
    const pluginMenuItems = topMenuItems();

    // Filter menu items by current viewport
    const pluginMenuArray = Array.from(pluginMenuItems.values())
      .filter(item => item.viewport === currentViewport)
      .sort((a, b) => (a.order || 0) - (b.order || 0));

    const menuStructure = {};

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

    // Update the exported signal
    setHasTopMenuItems(Object.keys(menuStructure).length > 0);

    return menuStructure;
  });

  // Get sorted top menu buttons from plugins
  const sortedTopMenuButtons = createMemo(() => {
    return Array.from(topMenuButtons().values())
      .sort((a, b) => (a.order || 0) - (b.order || 0));
  });

  const handleMenuClick = (menuName, event) => {
    if (activeMenu() === menuName) {
      setActiveMenu(null);
      setMenuPosition(null);
    } else {

      const rect = event.currentTarget.getBoundingClientRect();
      const position = calculateDropdownPosition(rect, 224);
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
    }
  };

  // Check if we have any menus to show
  const hasMenus = () => Object.keys(menuStructure()).length > 0;

  return (
    <>
      {/* Top row: Menu items + window controls - only show if there are menus */}
      <Show when={hasMenus()}>
        <div
          class="relative w-full h-8 bg-base-300 backdrop-blur-md shadow-sm flex items-center px-2"
          data-tauri-drag-region
        >
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

          {/* Draggable spacer - provides area to drag window */}
          <div class="flex-1 min-w-16" data-tauri-drag-region />

          {/* Plugin-registered top menu buttons (notifications, settings, etc.) */}
          <div
            class="flex items-center relative"
            style={{
              '-webkit-app-region': 'no-drag',
              'z-index': 150
            }}
          >
            <For each={sortedTopMenuButtons()}>
              {(button) => {
                const Component = button.component;
                return Component ? <Component /> : null;
              }}
            </For>
          </div>

          {/* Tauri Window Controls - Only show in desktop app and hide in background mode */}
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
      </Show>

      {/* Second row: Viewport Tabs - pass whether to show window controls */}
      <Show when={viewportTabsVisible()}>
        <div class="w-full bg-base-200">
          <ViewportTabs showWindowControls={!hasMenus()} />
        </div>
      </Show>

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
    </>
  );
}

export default TopMenu;
