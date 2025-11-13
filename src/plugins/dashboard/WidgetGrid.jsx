import { createSignal, For, Show, onMount, createEffect, onCleanup, untrack } from 'solid-js';
import { usePluginAPI } from '@/api/plugin';
import { IconX, IconColumns1, IconColumns2, IconPlus, IconLayoutGrid, IconLayoutGridAdd } from '@tabler/icons-solidjs';
import Packery from 'packery';
import Draggabilly from 'draggabilly';
import { dashboardAPI } from './api';

export default function WidgetGrid(props) {
  const api = usePluginAPI();
  const [currentDashboardId, setCurrentDashboardId] = createSignal('default');
  const [widgetInstances, setWidgetInstances] = createSignal([]);
  const [availableWidgets, setAvailableWidgets] = createSignal([]);
  const [showAddWidget, setShowAddWidget] = createSignal(false);
  const [widgetSearchQuery, setWidgetSearchQuery] = createSignal('');
  const [gridSize, setGridSize] = createSignal('normal'); // 'small' or 'large'

  let containerRef;
  let packeryInstance = null;
  let isDragging = false;
  let isInitialized = false;

  // Load widgets on mount and when dashboard changes
  onMount(async () => {
    try {
      // Load available widgets from plugin API
      const allWidgets = api.getWidgets();
      setAvailableWidgets(allWidgets);
      console.log('[WidgetGrid] Available widgets:', allWidgets);

      // Load widgets for current dashboard
      await loadWidgets(currentDashboardId());

      // Listen for dashboard changes
      api.on('dashboard:changed', async (data) => {
        console.log('[WidgetGrid] Dashboard changed to:', data.dashboardId);
        setCurrentDashboardId(data.dashboardId);
        await loadWidgets(data.dashboardId);

        // Reinitialize Packery after loading new widgets
        setTimeout(() => {
          isInitialized = false;
          if (containerRef) {
            initPackery();
          }
        }, 200);
      });
    } catch (error) {
      console.error('[WidgetGrid] Failed to initialize:', error);
    }
  });

  const loadWidgets = async (dashboardId) => {
    try {
      const data = await dashboardAPI.getWidgets(dashboardId);
      setWidgetInstances(data);
      console.log('[WidgetGrid] Loaded widgets:', data);

      // Reinitialize Packery after loading
      setTimeout(() => {
        isInitialized = false;
        if (containerRef) {
          initPackery();
        }
      }, 100);
    } catch (error) {
      console.error('[WidgetGrid] Failed to load widgets:', error);
    }
  };


  const addWidget = async (widgetId) => {
    try {
      const maxOrder = widgetInstances().reduce((max, w) => Math.max(max, w.order_index), -1);
      await dashboardAPI.createWidget(currentDashboardId(), widgetId, maxOrder + 1, 1);
      await loadWidgets(currentDashboardId());
      setShowAddWidget(false);
    } catch (error) {
      console.error('[WidgetGrid] Failed to add widget:', error);
      alert('Failed to add widget: ' + error.message);
    }
  };

  const removeWidget = async (widgetInstanceId) => {
    try {
      await dashboardAPI.deleteWidget(widgetInstanceId);
      await loadWidgets(currentDashboardId());
    } catch (error) {
      console.error('[WidgetGrid] Failed to remove widget:', error);
      alert('Failed to remove widget: ' + error.message);
    }
  };

  const toggleWidgetColumns = async (widgetInstance) => {
    try {
      const newColumns = widgetInstance.columns === 2 ? 1 : 2;
      await dashboardAPI.updateWidget(widgetInstance.id, { columns: newColumns });
      await loadWidgets(currentDashboardId());
    } catch (error) {
      console.error('[WidgetGrid] Failed to toggle columns:', error);
      alert('Failed to update widget: ' + error.message);
    }
  };

  const getWidgetById = (widgetId) => {
    return availableWidgets().find(w => w.id === widgetId);
  };

  const filteredWidgets = () => {
    const query = widgetSearchQuery().toLowerCase();
    if (!query) return availableWidgets();

    return availableWidgets().filter(widget =>
      widget.name?.toLowerCase().includes(query) ||
      widget.id.toLowerCase().includes(query) ||
      widget.description?.toLowerCase().includes(query)
    );
  };

  const toggleGridSize = (size) => {
    setGridSize(size);

    // Force reinitialize Packery with new column size
    isInitialized = false;
    setTimeout(() => {
      if (containerRef) {
        initPackery();
      }
    }, 50);
  };

  const calculateColumns = () => {
    if (!containerRef) return 6;
    const horizontalPadding = 32;
    const containerWidth = containerRef.offsetWidth - horizontalPadding;

    // Adjust minimum column width based on grid size
    const minColumnWidth = gridSize() === 'small' ? 150 : gridSize() === 'large' ? 250 : 180;
    const gutter = 16;

    const maxColumns = Math.floor((containerWidth + gutter) / (minColumnWidth + gutter));
    return Math.max(1, Math.min(maxColumns, 8));
  };

  const initPackery = () => {
    if (!containerRef) return;

    if (packeryInstance) {
      const itemElems = containerRef.querySelectorAll('.widget-item');
      itemElems.forEach((itemElem) => {
        if (itemElem._resizeObserver) {
          itemElem._resizeObserver.disconnect();
          delete itemElem._resizeObserver;
        }
      });

      packeryInstance.destroy();
    }

    const horizontalPadding = 32;
    const containerWidth = containerRef.offsetWidth - horizontalPadding;
    const gutter = 16;
    const columns = calculateColumns();
    const columnWidth = (containerWidth - (gutter * (columns - 1))) / columns;

    // Update all widget widths before initializing Packery
    const widgetElements = containerRef.querySelectorAll('.widget-item');
    widgetElements.forEach((elem) => {
      const widgetInstanceId = elem.getAttribute('data-widget-instance-id');
      const widgetInstance = widgetInstances().find(item => item.id === widgetInstanceId);
      if (widgetInstance) {
        const cols = widgetInstance.columns || 1;
        const width = getWidgetWidth(cols, columnWidth);
        elem.style.width = `${width}px`;
      }
    });

    // Set width for stamped add-widget tile
    const stampElem = containerRef.querySelector('.add-widget-stamp');
    if (stampElem) {
      stampElem.style.width = `${columnWidth}px`;
    }

    packeryInstance = new Packery(containerRef, {
      itemSelector: '.widget-item',
      columnWidth: columnWidth,
      gutter: gutter,
      transitionDuration: '0.2s',
      stagger: 0,
      isInitLayout: true,
      stamp: '.add-widget-stamp'
    });

    packeryInstance.reloadItems();
    packeryInstance.layout();

    const itemElems = containerRef.querySelectorAll('.widget-item');

    itemElems.forEach((itemElem) => {
      const draggie = new Draggabilly(itemElem, {
        ignore: 'button, input, textarea, select, a, .no-drag',
        grid: [1, 1]
      });

      packeryInstance.bindDraggabillyEvents(draggie);

      draggie.on('dragStart', () => {
        isDragging = true;
        itemElem.style.zIndex = '1000';
      });

      draggie.on('dragEnd', async () => {
        isDragging = false;
        itemElem.style.zIndex = '';

        // Force layout update to avoid overlapping with stamped element
        if (packeryInstance) {
          packeryInstance.layout();
        }

        const items = packeryInstance.getItemElements();
        const widgetIds = items.map((elem) => elem.getAttribute('data-widget-instance-id'));

        try {
          await dashboardAPI.reorderWidgets(currentDashboardId(), widgetIds);
        } catch (error) {
          console.error('[WidgetGrid] Failed to save order:', error);
        }
      });

      const resizeObserver = new ResizeObserver(() => {
        if (!isDragging && packeryInstance) {
          requestAnimationFrame(() => {
            packeryInstance.layout();
          });
        }
      });
      resizeObserver.observe(itemElem);

      itemElem._resizeObserver = resizeObserver;
    });

    isInitialized = true;
  };

  const layoutPackery = (reloadItems = false) => {
    if (packeryInstance) {
      if (reloadItems) {
        packeryInstance.reloadItems();
      }
      packeryInstance.layout();
    }
  };

  createEffect(() => {
    const instances = widgetInstances();
    if (instances.length > 0 && containerRef && !isInitialized) {
      setTimeout(() => {
        initPackery();
        setTimeout(() => layoutPackery(), 100);
      }, 100);
    }
  });

  // Watch for grid size changes and reinitialize
  createEffect(() => {
    const size = gridSize();
    if (widgetInstances().length > 0 && containerRef && isInitialized) {
      isInitialized = false;
      setTimeout(() => {
        initPackery();
      }, 50);
    }
  });

  onMount(() => {
    let resizeTimeout;
    let lastColumns = calculateColumns();

    const handleResize = () => {
      clearTimeout(resizeTimeout);
      resizeTimeout = setTimeout(() => {
        if (widgetInstances().length > 0 && containerRef) {
          const newColumns = calculateColumns();

          if (newColumns !== lastColumns) {
            lastColumns = newColumns;
            isInitialized = false;
            initPackery();
          } else if (packeryInstance) {
            const horizontalPadding = 32;
            const containerWidth = containerRef.offsetWidth - horizontalPadding;
            const gutter = 16;
            const columnWidth = (containerWidth - (gutter * (newColumns - 1))) / newColumns;

            packeryInstance.options.columnWidth = columnWidth;

            const widgetElements = containerRef.querySelectorAll('.widget-item');
            widgetElements.forEach((elem) => {
              const widgetInstanceId = elem.getAttribute('data-widget-instance-id');
              const widgetInstance = widgetInstances().find(item => item.id === widgetInstanceId);
              if (widgetInstance) {
                const columns = widgetInstance.columns || 1;
                const width = getWidgetWidth(columns);
                elem.style.width = `${width}px`;
              }
            });

            packeryInstance.layout();
          }
        }
      }, 150);
    };

    const resizeObserver = new ResizeObserver(() => {
      handleResize();
    });

    if (containerRef?.parentElement) {
      resizeObserver.observe(containerRef.parentElement);
    }

    window.addEventListener('resize', handleResize);
    onCleanup(() => {
      clearTimeout(resizeTimeout);
      window.removeEventListener('resize', handleResize);
      resizeObserver.disconnect();

      if (containerRef) {
        const itemElems = containerRef.querySelectorAll('.widget-item');
        itemElems.forEach((itemElem) => {
          if (itemElem._resizeObserver) {
            itemElem._resizeObserver.disconnect();
            delete itemElem._resizeObserver;
          }
        });
      }

      if (packeryInstance) {
        packeryInstance.destroy();
      }
    });
  });

  const getWidgetWidth = (requestedColumns, columnWidthOverride = null) => {
    if (!containerRef) return 300;
    const horizontalPadding = 32;
    const containerWidth = containerRef.offsetWidth - horizontalPadding;
    const gutter = 16;
    const numColumns = calculateColumns();
    const columnWidth = columnWidthOverride !== null ? columnWidthOverride : (containerWidth - (gutter * (numColumns - 1))) / numColumns;

    if (requestedColumns === 2) {
      const twoColWidth = columnWidth * 2 + gutter;
      const minTwoColWidth = 400;

      if (twoColWidth < minTwoColWidth || numColumns < 4) {
        return columnWidth;
      }
      return twoColWidth;
    }

    return columnWidth;
  };

  return (
    <div class="h-full overflow-y-auto bg-gradient-to-br from-base-300 to-base-200 p-4">
      <div class="max-w-7xl mx-auto">
        {/* Header with Controls */}
        <div class="mb-4 flex items-center justify-end gap-2">
          <div class="flex items-center gap-1">
            <button
              class={`btn btn-sm btn-square ${gridSize() === 'normal' ? 'btn-active' : 'btn-ghost'}`}
              onClick={() => toggleGridSize('normal')}
              title="Normal size"
            >
              <IconLayoutGrid size={18} />
            </button>
            <button
              class={`btn btn-sm btn-square ${gridSize() === 'large' ? 'btn-active' : 'btn-ghost'}`}
              onClick={() => toggleGridSize('large')}
              title="Large size"
            >
              <IconLayoutGridAdd size={18} />
            </button>
          </div>
        </div>

        {/* Widget Grid */}
        <div
          ref={containerRef}
          style={{
            padding: '0 16px'
          }}
        >
          {/* Stamped Add Widget Tile */}
          <div
            class="add-widget-stamp relative"
            style={{
              position: 'absolute',
              left: '16px',
              top: '0px',
              'min-height': '200px'
            }}
          >
            <button
              class="card bg-gradient-to-br from-primary/20 to-primary/10 hover:from-primary/30 hover:to-primary/20 border-2 border-dashed border-primary/40 hover:border-primary transition-all duration-200 cursor-pointer h-full w-full"
              onClick={() => setShowAddWidget(true)}
            >
              <div class="card-body items-center justify-center text-center p-8 h-full">
                <div class="w-16 h-16 rounded-full bg-primary/20 flex items-center justify-center mb-4">
                  <IconPlus size={32} class="text-primary" />
                </div>
                <h3 class="text-lg font-semibold text-primary mb-2">Add Widget</h3>
              </div>
            </button>
          </div>

          <For each={widgetInstances()}>
            {(widgetInstance) => {
              const widget = getWidgetById(widgetInstance.widget_id);
              if (!widget) return null;

              const WidgetComponent = widget.component;
              const columns = widgetInstance.columns || 1;
              const width = untrack(() => getWidgetWidth(columns));

              return (
                <div
                  class="widget-item relative group cursor-grab active:cursor-grabbing"
                  data-widget-instance-id={widgetInstance.id}
                  data-order={widgetInstance.order_index}
                  style={{ width: `${width}px` }}
                >
                  {/* Widget Controls */}
                  <div class="absolute top-1 left-1 z-20 flex items-center gap-0.5 opacity-0 group-hover:opacity-100 transition-opacity">
                    <button
                      class="btn btn-ghost btn-xs btn-square bg-base-100/80 backdrop-blur-sm hover:bg-base-200/90 shadow-sm h-6 w-6 min-h-0"
                      onClick={(e) => {
                        e.stopPropagation();
                        toggleWidgetColumns(widgetInstance);
                      }}
                    >
                      {columns === 1 ? <IconColumns2 size={12} /> : <IconColumns1 size={12} />}
                    </button>
                    <button
                      class="btn btn-ghost btn-xs btn-square bg-base-100/80 backdrop-blur-sm hover:bg-error/90 hover:text-error-content shadow-sm h-6 w-6 min-h-0"
                      onClick={(e) => {
                        e.stopPropagation();
                        removeWidget(widgetInstance.id);
                      }}
                    >
                      <IconX size={12} />
                    </button>
                  </div>

                  {/* Widget Content */}
                  <div class="relative">
                    <WidgetComponent instanceId={widgetInstance.id} config={widgetInstance.config} />
                  </div>
                </div>
              );
            }}
          </For>
        </div>

        {/* Empty State */}
        <Show when={widgetInstances().length === 0}>
          <div class="text-center py-20 opacity-50">
            <p class="text-lg">No widgets on this dashboard</p>
            <p class="text-sm mt-2">Click "Add Widget" to get started</p>
          </div>
        </Show>
      </div>

      {/* Add Widget Modal */}
      <Show when={showAddWidget()}>
        <div class="modal modal-open">
          <div class="modal-box max-w-5xl max-h-[90vh] flex flex-col p-0">
            {/* Header */}
            <div class="p-6 pb-4 border-b border-base-300">
              <h3 class="font-bold text-2xl mb-4">Add Widget</h3>

              {/* Search Bar */}
              <div class="relative">
                <input
                  type="text"
                  placeholder="Search widgets..."
                  class="input input-bordered w-full pr-10"
                  value={widgetSearchQuery()}
                  onInput={(e) => setWidgetSearchQuery(e.target.value)}
                  autofocus
                />
                <svg
                  class="absolute right-3 top-1/2 -translate-y-1/2 w-5 h-5 opacity-50"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
                </svg>
              </div>

              <Show when={widgetSearchQuery()}>
                <div class="mt-2 text-sm opacity-70">
                  Found {filteredWidgets().length} widget{filteredWidgets().length !== 1 ? 's' : ''}
                </div>
              </Show>
            </div>

            {/* Widget Grid */}
            <div class="flex-1 overflow-y-auto p-6">
              <Show when={filteredWidgets().length > 0} fallback={
                <div class="text-center py-12 opacity-50">
                  <svg class="w-16 h-16 mx-auto mb-4 opacity-30" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20 13V6a2 2 0 00-2-2H6a2 2 0 00-2 2v7m16 0v5a2 2 0 01-2 2H6a2 2 0 01-2-2v-5m16 0h-2.586a1 1 0 00-.707.293l-2.414 2.414a1 1 0 01-.707.293h-3.172a1 1 0 01-.707-.293l-2.414-2.414A1 1 0 006.586 13H4" />
                  </svg>
                  <p class="text-lg">No widgets found</p>
                  <p class="text-sm mt-2">Try a different search term</p>
                </div>
              }>
                <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                  <For each={filteredWidgets()}>
                    {(widget) => (
                      <button
                        class="card bg-base-200 hover:bg-base-300 transition-all duration-200 hover:shadow-lg group text-left"
                        onClick={() => addWidget(widget.id)}
                      >
                        <div class="card-body p-4">
                          {/* Widget Icon/Preview */}
                          <div class="flex items-center gap-3 mb-3">
                            <div class="w-12 h-12 rounded-lg bg-primary/10 flex items-center justify-center flex-shrink-0">
                              <Show when={widget.icon} fallback={
                                <svg class="w-6 h-6 text-primary" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 5a1 1 0 011-1h4a1 1 0 011 1v7a1 1 0 01-1 1H5a1 1 0 01-1-1V5zM14 5a1 1 0 011-1h4a1 1 0 011 1v7a1 1 0 01-1 1h-4a1 1 0 01-1-1V5zM4 16a1 1 0 011-1h4a1 1 0 011 1v3a1 1 0 01-1 1H5a1 1 0 01-1-1v-3zM14 16a1 1 0 011-1h4a1 1 0 011 1v3a1 1 0 01-1 1h-4a1 1 0 01-1-1v-3z" />
                                </svg>
                              }>
                                {widget.icon && <widget.icon size={24} class="text-primary" />}
                              </Show>
                            </div>

                            <div class="flex-1 min-w-0">
                              <h4 class="font-semibold text-base truncate group-hover:text-primary transition-colors">
                                {widget.name || widget.id}
                              </h4>
                              <Show when={widget.plugin}>
                                <p class="text-xs opacity-50 truncate">{widget.plugin}</p>
                              </Show>
                            </div>

                            <div class="w-8 h-8 rounded-full bg-primary text-primary-content flex items-center justify-center opacity-0 group-hover:opacity-100 transition-opacity flex-shrink-0">
                              <IconPlus size={18} />
                            </div>
                          </div>

                          {/* Widget Description */}
                          <Show when={widget.description}>
                            <p class="text-sm opacity-70 line-clamp-2">
                              {widget.description}
                            </p>
                          </Show>

                          {/* Widget Info */}
                          <div class="flex items-center gap-2 mt-2 text-xs opacity-50">
                            <Show when={widget.defaultSize}>
                              <span class="badge badge-sm badge-ghost">
                                {widget.defaultSize.w}×{widget.defaultSize.h}
                              </span>
                            </Show>
                          </div>
                        </div>
                      </button>
                    )}
                  </For>
                </div>
              </Show>
            </div>

            {/* Footer */}
            <div class="p-4 border-t border-base-300 flex justify-end">
              <button class="btn btn-ghost" onClick={() => {
                setShowAddWidget(false);
                setWidgetSearchQuery('');
              }}>
                Close
              </button>
            </div>
          </div>
          <div class="modal-backdrop bg-black/50" onClick={() => {
            setShowAddWidget(false);
            setWidgetSearchQuery('');
          }}></div>
        </div>
      </Show>
    </div>
  );
}