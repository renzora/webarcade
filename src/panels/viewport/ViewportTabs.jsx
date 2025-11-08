import { createSignal, createMemo, For, Show, onMount, onCleanup, createEffect } from 'solid-js';
import { IconFileText, IconX, IconStar, IconCopy, IconPlayerPlay, IconPlayerPause, IconChairDirector, IconChevronDown } from '@tabler/icons-solidjs';
import { viewportStore, viewportActions } from "./store";
import { viewportTypes } from "@/api/plugin";

const ViewportTabs = () => {
  const [contextMenu, setContextMenu] = createSignal(null);
  const [editingTab, setEditingTab] = createSignal(null);
  const [editingName, setEditingName] = createSignal('');
  const [overflowDropdown, setOverflowDropdown] = createSignal(false);
  const [dropdownPosition, setDropdownPosition] = createSignal({ top: 0, left: 0 });
  const [visibleTabCount, setVisibleTabCount] = createSignal(99);
  let containerRef;
  let tabsContainerRef;
  let dropdownButtonRef;
  const tabs = () => viewportStore.tabs;
  const activeTabId = () => viewportStore.activeTabId;
  const suspendedTabs = () => viewportStore.suspendedTabs;
  const availableViewportTypes = createMemo(() => {
    const pluginTypes = Array.from(viewportTypes().values());
    return pluginTypes;
  });

  const getViewportIcon = (type) => {
    try {
      const viewportType = availableViewportTypes().find(v => v.id === type);
      if (viewportType && viewportType.icon && typeof viewportType.icon === 'function') {
        return viewportType.icon;
      }
      return IconFileText;
    } catch (error) {
      return IconFileText;
    }
  };

  const handleTabClick = (tabId) => {
    viewportActions.setActiveViewportTab(tabId);
  };

  const handleTabClose = (e, tabId) => {
    e.stopPropagation();
    viewportActions.removeViewportTab(tabId);
  };

  const handleStartRename = (tab) => {
    setEditingTab(tab.id);
    setEditingName(tab.name);
    setContextMenu(null);
  };

  const handleFinishRename = () => {
    if (editingTab() && editingName().trim()) {
      viewportActions.renameViewportTab(editingTab(), editingName().trim());
    }
    setEditingTab(null);
    setEditingName('');
  };

  const handleCancelRename = () => {
    setEditingTab(null);
    setEditingName('');
  };

  const handleRenameKeyDown = (e) => {
    if (e.key === 'Enter') {
      handleFinishRename();
    } else if (e.key === 'Escape') {
      handleCancelRename();
    }
  };

  const handleTabContextMenu = (e, tab) => {
    e.preventDefault();
    e.stopPropagation();
    
    setContextMenu({
      position: { x: e.clientX, y: e.clientY },
      tab: tab,
      items: [
        {
          label: 'Rename Tab',
          icon: IconFileText,
          action: () => handleStartRename(tab)
        },
        {
          label: tab.isPinned ? 'Unpin Tab' : 'Pin Tab',
          icon: tab.isPinned ? IconX : IconStar,
          action: () => viewportActions.pinViewportTab(tab.id)
        },
        {
          label: 'Duplicate Tab',
          icon: IconCopy,
          action: () => viewportActions.duplicateViewportTab(tab.id)
        },
        {
          label: (suspendedTabs() || []).includes(tab.id) ? 'Resume Tab' : 'Suspend Tab',
          icon: (suspendedTabs() || []).includes(tab.id) ? IconPlayerPlay : IconPlayerPause,
          action: () => {

          },
          disabled: tab.id === activeTabId()
        },
        { divider: true },
        {
          label: `New ${tab.type.replace('-', ' ').replace(/\b\w/g, l => l.toUpperCase())}`,
          iconComponent: getViewportIcon(tab.type),
          action: () => {
            const newTabId = `viewport-${Date.now()}`;
            const viewportType = availableViewportTypes().find(v => v.id === tab.type);
            const newTab = {
              id: newTabId,
              type: tab.type,
              name: viewportType ? (viewportType.label || viewportType.name || 'New Viewport') : 'New Viewport',
              isPinned: false,
              hasUnsavedChanges: false
            };
            viewportActions.addViewportTab(newTab);
            viewportActions.setActiveViewportTab(newTabId);
          }
        },
        { divider: true },
        {
          label: 'Close Tab',
          icon: IconX,
          action: () => viewportActions.removeViewportTab(tab.id),
          disabled: tabs().length === 1
        },
        {
          label: 'Close Other Tabs',
          icon: IconX,
          action: () => {
            tabs().forEach(t => {
              if (t.id !== tab.id && !t.isPinned) {
                viewportActions.removeViewportTab(t.id);
              }
            });
          },
          disabled: tabs().length === 1
        },
        {
          label: 'Close All Tabs',
          icon: IconX,
          action: () => {
            tabs().forEach(t => {
              if (!t.isPinned) {
                viewportActions.removeViewportTab(t.id);
              }
            });
          },
          disabled: tabs().length === 1 || tabs().every(t => t.isPinned)
        }
      ]
    });
  };

  const handleMiddleClick = (e, tabId) => {
    if (e.button === 1) {
      e.preventDefault();
      viewportActions.removeViewportTab(tabId);
    }
  };

  // Calculate how many tabs can fit in the available space
  const calculateVisibleTabs = () => {
    if (!tabsContainerRef || !containerRef) return;

    const children = Array.from(tabsContainerRef.children);
    if (children.length === 0) return;

    const containerWidth = containerRef.offsetWidth;
    const dropdownButtonWidth = 40;
    const maxVisibleTabs = 5; // Maximum tabs to show before dropdown

    let totalWidth = 0;
    let count = 0;

    // Calculate total width of all tabs
    for (const child of children) {
      totalWidth += child.offsetWidth;
    }

    // If all tabs fit and under the limit, show them all
    if (totalWidth <= containerWidth && tabs().length <= maxVisibleTabs) {
      setVisibleTabCount(tabs().length);
      return;
    }

    // Otherwise, calculate with dropdown button space and max limit
    const availableWidth = containerWidth - dropdownButtonWidth;
    totalWidth = 0;

    for (let i = 0; i < children.length && i < maxVisibleTabs; i++) {
      totalWidth += children[i].offsetWidth;
      if (totalWidth <= availableWidth) {
        count = i + 1;
      } else {
        break;
      }
    }

    setVisibleTabCount(Math.max(1, count));
  };

  // Set up resize listener
  onMount(() => {
    const handleResize = () => {
      calculateVisibleTabs();
      // Close dropdown on resize to avoid positioning issues
      if (overflowDropdown()) {
        setOverflowDropdown(false);
      }
    };

    window.addEventListener('resize', handleResize);
    onCleanup(() => window.removeEventListener('resize', handleResize));
  });

  // Recalculate when tabs change
  createEffect(() => {
    tabs();
    calculateVisibleTabs();
  });

  const visibleTabs = () => tabs().slice(0, visibleTabCount());
  const overflowTabs = () => tabs().slice(visibleTabCount());

  return (
    <>
      <div ref={containerRef} className="flex items-stretch h-8 overflow-hidden relative bg-base-200">
        <div ref={tabsContainerRef} className="flex items-center min-w-0 flex-1 overflow-x-hidden">
          <For each={tabs()}>
            {(tab, index) => {
              const Icon = getViewportIcon(tab.type);
              const isActive = () => tab.id === activeTabId();
              const isVisible = () => index() < visibleTabCount();

              return (
                <div
                  classList={{
                    'group flex items-center gap-2 px-3 py-1 border-r border-neutral cursor-pointer transition-all select-none min-w-0 max-w-48 flex-shrink-0 h-full relative border-t border-b border-neutral': true,
                    'bg-primary/15 text-primary': isActive(),
                    'text-base-content/60 hover:text-base-content': !isActive(),
                    'hidden': !isVisible()
                  }}
                  onClick={() => handleTabClick(tab.id)}
                  onContextMenu={(e) => handleTabContextMenu(e, tab)}
                  onMouseDown={(e) => handleMiddleClick(e, tab.id)}
                  title={tab.name}
                >
                  <div className="absolute inset-x-0 top-0 h-px bg-gradient-to-r from-transparent via-base-content/15 to-transparent"></div>
                  <Icon className="w-4 h-4 flex-shrink-0" />

                  <Show
                    when={editingTab() === tab.id}
                    fallback={
                      <span className="text-sm font-medium truncate min-w-0">
                        {tab.name}{tab.hasUnsavedChanges ? ' •' : ''}
                      </span>
                    }
                  >
                    <input
                      type="text"
                      value={editingName()}
                      onChange={(e) => setEditingName(e.target.value)}
                      onBlur={handleFinishRename}
                      onKeyDown={handleRenameKeyDown}
                      className="text-sm font-medium bg-base-300 border border-base-content/20 rounded px-1 py-0 min-w-0 max-w-32"
                      autofocus
                      onClick={(e) => e.stopPropagation()}
                    />
                  </Show>
                  
                  <Show when={(suspendedTabs() || []).includes(tab.id)}>
                    <IconPlayerPause className="w-3 h-3 text-gray-500 flex-shrink-0" title="Tab Suspended" />
                  </Show>
                  
                  <Show when={tab.isPinned}>
                    <IconStar className="w-3 h-3 text-yellow-500 flex-shrink-0" />
                  </Show>
                  
                  <Show when={tab.hasUnsavedChanges}>
                    <div className="w-2 h-2 bg-orange-500 rounded-full flex-shrink-0" />
                  </Show>
                  
                  <Show when={tabs().length > 1}>
                    <button
                      onClick={(e) => handleTabClose(e, tab.id)}
                      className="w-4 h-4 flex items-center justify-center rounded hover:bg-base-300 transition-colors opacity-0 group-hover:opacity-100 flex-shrink-0"
                      title="Close Tab"
                    >
                      <IconX className="w-3 h-3" />
                    </button>
                  </Show>
                </div>
              );
            }}
          </For>
        </div>

        {/* Dropdown button for overflow tabs */}
        <Show when={overflowTabs().length > 0}>
          <div className="relative flex items-center">
            <button
              ref={dropdownButtonRef}
              onClick={() => {
                if (dropdownButtonRef) {
                  const rect = dropdownButtonRef.getBoundingClientRect();
                  setDropdownPosition({ top: rect.bottom + 4, left: rect.left });
                }
                setOverflowDropdown(!overflowDropdown());
              }}
              className="flex items-center justify-center px-2 h-full border-r border-t border-b border-neutral bg-base-300 hover:bg-base-200 text-base-content/60 hover:text-base-content transition-colors"
              title={`${overflowTabs().length} more tab${overflowTabs().length > 1 ? 's' : ''}`}
            >
              <IconChevronDown className="w-4 h-4" />
              <span className="text-xs ml-1">{overflowTabs().length}</span>
            </button>
          </div>
        </Show>

      </div>

      {/* Overflow dropdown menu - rendered outside to avoid z-index issues */}
      <Show when={overflowDropdown()}>
        <>
          <div
            className="fixed inset-0 bg-black/10"
            onClick={() => setOverflowDropdown(false)}
            onContextMenu={(e) => {
              e.preventDefault();
              setOverflowDropdown(false);
            }}
            style={{ 'z-index': 9999 }}
          />

          <div
            className="fixed bg-base-200 backdrop-blur-sm border-l border-r border-b border-base-300 shadow-xl min-w-48 max-h-96 overflow-y-auto"
            style={{
              top: `${dropdownPosition().top - 1}px`,
              right: `${window.innerWidth - dropdownPosition().left - (dropdownButtonRef?.offsetWidth || 40)}px`,
              'z-index': 10000,
              'border-top-left-radius': '0',
              'border-top-right-radius': '0',
              'border-bottom-left-radius': '0.5rem',
              'border-bottom-right-radius': '0.5rem'
            }}
          >
                  <div className="p-2">
                    <For each={overflowTabs()}>
                      {(tab) => {
                        const Icon = getViewportIcon(tab.type);
                        const isActive = () => tab.id === activeTabId();

                        return (
                          <button
                            onClick={() => {
                              handleTabClick(tab.id);
                              setOverflowDropdown(false);
                            }}
                            onContextMenu={(e) => {
                              handleTabContextMenu(e, tab);
                              setOverflowDropdown(false);
                            }}
                            classList={{
                              'w-full flex items-center gap-2 px-3 py-2 text-sm rounded-md transition-colors cursor-pointer': true,
                              'bg-primary/15 text-primary': isActive(),
                              'text-base-content hover:bg-base-300': !isActive()
                            }}
                          >
                            <Icon className="w-4 h-4 flex-shrink-0" />
                            <span className="truncate flex-1 text-left">
                              {tab.name}{tab.hasUnsavedChanges ? ' •' : ''}
                            </span>

                            <div className="flex items-center gap-1 flex-shrink-0">
                              <Show when={(suspendedTabs() || []).includes(tab.id)}>
                                <IconPlayerPause className="w-3 h-3 text-gray-500" title="Tab Suspended" />
                              </Show>

                              <Show when={tab.isPinned}>
                                <IconStar className="w-3 h-3 text-yellow-500" />
                              </Show>

                              <Show when={tab.hasUnsavedChanges}>
                                <div className="w-2 h-2 bg-orange-500 rounded-full" />
                              </Show>
                            </div>

                            <Show when={tabs().length > 1}>
                              <button
                                onClick={(e) => {
                                  handleTabClose(e, tab.id);
                                  setOverflowDropdown(false);
                                }}
                                className="w-4 h-4 flex items-center justify-center rounded hover:bg-base-300 transition-colors"
                                title="Close Tab"
                              >
                                <IconX className="w-3 h-3" />
                              </button>
                            </Show>
                          </button>
                        );
                      }}
                    </For>
                  </div>
                </div>
              </>
            </Show>

      <Show when={contextMenu()}>
        <>
          <div 
            className="fixed inset-0 z-50"
            onClick={() => setContextMenu(null)}
          />
          
          <div
            className="fixed z-60 bg-base-200 backdrop-blur-sm border border-base-300 rounded-lg shadow-xl min-w-48"
            style={{ 
              left: contextMenu().position.x + 'px', 
              top: contextMenu().position.y + 'px'
            }}
          >
            <div className="p-2">
              <For each={contextMenu().items}>
                {(item, _index) => (
                  <Show
                    when={item.divider}
                    fallback={
                      <button
                        onClick={() => {
                          if (!item.disabled) {
                            item.action();
                          }
                          setContextMenu(null);
                        }}
                        disabled={item.disabled}
                        className={`w-full flex items-center px-3 py-2 text-sm rounded-md transition-colors ${
                          item.disabled 
                            ? 'text-base-content/30 cursor-not-allowed'
                            : 'text-base-content hover:bg-base-300 hover:text-base-content'
                        }`}
                      >
                        <Show when={item.icon || item.iconComponent}>
                          <Show when={item.iconComponent} fallback={
                            <item.icon className={`w-4 h-4 mr-3 ${
                              item.disabled ? 'text-base-content/30' : 'text-base-content/60'
                            }`} />
                          }>
                            <div className={`w-4 h-4 mr-3 ${
                              item.disabled ? 'text-base-content/30' : 'text-base-content/60'
                            }`}>
                              <item.iconComponent className="w-4 h-4" />
                            </div>
                          </Show>
                        </Show>
                        {item.label}
                      </button>
                    }
                  >
                    <div className="border-t border-base-300 my-2" />
                  </Show>
                )}
              </For>
            </div>
          </div>
        </>
      </Show>

    </>
  );
};

export default ViewportTabs;