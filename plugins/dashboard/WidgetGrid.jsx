import { createSignal, For, Show, onMount, createEffect, onCleanup, untrack } from 'solid-js';
import { usePluginAPI } from '@/api/plugin';
import { IconGripVertical, IconX, IconColumns1, IconColumns2 } from '@tabler/icons-solidjs';
import Packery from 'packery';
import Draggabilly from 'draggabilly';

export default function WidgetGrid() {
  const api = usePluginAPI();
  const [widgetLayout, setWidgetLayout] = createSignal([]);
  const [availableWidgets, setAvailableWidgets] = createSignal([]);
  const [draggedIndex, setDraggedIndex] = createSignal(null);
  const [dropTargetIndex, setDropTargetIndex] = createSignal(null);
  let containerRef;
  let packeryInstance = null;
  let isDragging = false;
  let isInitialized = false;

  onMount(() => {
    // Small delay to ensure plugins are loaded
    setTimeout(() => {
      // Get all available widgets
      const allWidgets = api.getWidgets();
      console.log('[WidgetGrid] All widgets:', allWidgets);
      setAvailableWidgets(allWidgets);

      // Load widget layout from localStorage
      const savedLayout = localStorage.getItem('dashboard-widget-layout');
      let layout = null;

      if (savedLayout) {
        try {
          layout = JSON.parse(savedLayout);
          console.log('[WidgetGrid] Loaded layout from storage:', layout);
        } catch (e) {
          layout = null;
        }
      }

      // Always ensure all widgets are in the layout
      if (allWidgets.length > 0) {
        if (!layout || layout.length === 0) {
          // No saved layout, add all widgets
          layout = allWidgets.map((widget, index) => ({
            id: widget.id,
            order: index,
            columns: 1
          }));
          console.log('[WidgetGrid] Created new layout:', layout);
        } else {
          // Merge saved layout with new widgets
          const existingIds = new Set(layout.map(item => item.id));
          const newWidgets = allWidgets
            .filter(widget => !existingIds.has(widget.id))
            .map((widget, index) => ({
              id: widget.id,
              order: layout.length + index,
              columns: 1
            }));

          if (newWidgets.length > 0) {
            layout = [...layout, ...newWidgets];
            console.log('[WidgetGrid] Added new widgets:', newWidgets);
          }
        }

        setWidgetLayout(layout);
        saveLayout(layout);
        console.log('[WidgetGrid] Final layout:', layout);
        console.log('[WidgetGrid] Widget layout count:', widgetLayout().length);
      } else {
        console.warn('[WidgetGrid] No widgets available from API!');
      }
    }, 200);
  });

  const saveLayout = (layout) => {
    localStorage.setItem('dashboard-widget-layout', JSON.stringify(layout));
  };

  const removeWidget = (widgetId) => {
    const newLayout = widgetLayout().filter(item => item.id !== widgetId);
    const updatedLayout = newLayout.map((item, index) => ({
      ...item,
      order: index
    }));
    setWidgetLayout(updatedLayout);
    saveLayout(updatedLayout);

    // Reinitialize Packery to rebind draggable handlers
    setTimeout(() => {
      isInitialized = false;
      initPackery();
    }, 50);
  };

  const toggleWidgetColumns = (widgetId) => {
    const newLayout = widgetLayout().map(item => {
      if (item.id === widgetId) {
        return {
          ...item,
          columns: item.columns === 2 ? 1 : 2
        };
      }
      return item;
    });
    setWidgetLayout(newLayout);
    saveLayout(newLayout);

    // Reinitialize Packery to update widths and rebind draggable handlers
    setTimeout(() => {
      isInitialized = false;
      initPackery();
    }, 100);
  };

  const getWidgetById = (widgetId) => {
    return availableWidgets().find(w => w.id === widgetId);
  };

  const calculateColumns = () => {
    if (!containerRef) return 6;
    const horizontalPadding = 32; // 16px left + 16px right
    const containerWidth = containerRef.offsetWidth - horizontalPadding;
    const minColumnWidth = 180; // Reduced from 200 to allow more columns
    const gutter = 16;

    const maxColumns = Math.floor((containerWidth + gutter) / (minColumnWidth + gutter));
    return Math.max(1, Math.min(maxColumns, 8)); // Increased max from 6 to 8 columns
  };

  const initPackery = () => {
    if (!containerRef) return;

    // Destroy existing instance
    if (packeryInstance) {
      // Clean up ResizeObservers
      const itemElems = containerRef.querySelectorAll('.widget-item');
      itemElems.forEach((itemElem) => {
        if (itemElem._resizeObserver) {
          itemElem._resizeObserver.disconnect();
          delete itemElem._resizeObserver;
        }
      });

      packeryInstance.destroy();
    }

    // Calculate column width based on container (accounting for horizontal padding)
    const horizontalPadding = 32; // 16px left + 16px right
    const containerWidth = containerRef.offsetWidth - horizontalPadding;
    const gutter = 16;
    const columns = calculateColumns();
    const columnWidth = (containerWidth - (gutter * (columns - 1))) / columns;

    // Initialize Packery
    packeryInstance = new Packery(containerRef, {
      itemSelector: '.widget-item',
      columnWidth: columnWidth,
      gutter: gutter,
      transitionDuration: '0.2s',
      stagger: 0,
      isInitLayout: true
    });

    // Force Packery to reload and recognize all items
    packeryInstance.reloadItems();
    packeryInstance.layout();

    // Make all widget items draggable using Draggabilly
    const itemElems = containerRef.querySelectorAll('.widget-item');

    itemElems.forEach((itemElem) => {
      const draggie = new Draggabilly(itemElem, {
        // Entire widget is draggable, but ignore interactive elements
        ignore: 'button, input, textarea, select, a, .no-drag',
        grid: [1, 1] // Smoother dragging with finer grid
      });

      // Store draggabilly instance
      packeryInstance.bindDraggabillyEvents(draggie);

      // Track drag state
      draggie.on('dragStart', () => {
        isDragging = true;
        // Elevate dragged widget above all others
        itemElem.style.zIndex = '1000';
      });

      // Save new order after drag
      draggie.on('dragEnd', () => {
        isDragging = false;
        // Reset z-index after drag
        itemElem.style.zIndex = '';

        const items = packeryInstance.getItemElements();
        const newLayout = items.map((elem, index) => {
          const widgetId = elem.getAttribute('data-widget-id');
          const layoutItem = widgetLayout().find(item => item.id === widgetId);
          return {
            ...layoutItem,
            order: index
          };
        });

        // Only update if order actually changed
        const currentOrder = widgetLayout().map(item => item.id).join(',');
        const newOrder = newLayout.map(item => item.id).join(',');

        if (currentOrder !== newOrder) {
          // Save to localStorage only - don't update state to avoid re-render
          localStorage.setItem('dashboard-widget-layout', JSON.stringify(newLayout));

          // Update the data attributes to keep DOM in sync without re-rendering
          items.forEach((elem, index) => {
            elem.setAttribute('data-order', index);
          });
        }
      });

      // Add ResizeObserver to detect height changes in widgets
      const resizeObserver = new ResizeObserver(() => {
        // Debounce layout calls to avoid excessive relayouts
        if (!isDragging && packeryInstance) {
          requestAnimationFrame(() => {
            packeryInstance.layout();
          });
        }
      });
      resizeObserver.observe(itemElem);

      // Store observer for cleanup
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

  // Initialize packery when widgets are loaded
  createEffect(() => {
    const layout = widgetLayout();
    if (layout.length > 0 && containerRef && !isInitialized) {
      setTimeout(() => {
        initPackery();
        setTimeout(() => layoutPackery(), 100);
      }, 100);
    }
  });

  // Relayout on window resize and panel resize
  onMount(() => {
    let resizeTimeout;
    let lastColumns = calculateColumns();

    const handleResize = () => {
      // Debounce to prevent resize loop
      clearTimeout(resizeTimeout);
      resizeTimeout = setTimeout(() => {
        if (widgetLayout().length > 0 && containerRef) {
          const newColumns = calculateColumns();

          // If column count changed, reinitialize to recalculate everything
          if (newColumns !== lastColumns) {
            lastColumns = newColumns;
            isInitialized = false;
            initPackery();
          } else if (packeryInstance) {
            // Just update width and relayout
            const horizontalPadding = 32; // 16px left + 16px right
            const containerWidth = containerRef.offsetWidth - horizontalPadding;
            const gutter = 16;
            const columnWidth = (containerWidth - (gutter * (newColumns - 1))) / newColumns;

            // Update packery column width and force relayout
            packeryInstance.options.columnWidth = columnWidth;

            // Update widget widths
            const widgetElements = containerRef.querySelectorAll('.widget-item');
            widgetElements.forEach((elem) => {
              const widgetId = elem.getAttribute('data-widget-id');
              const layoutItem = widgetLayout().find(item => item.id === widgetId);
              if (layoutItem) {
                const columns = layoutItem.columns || 1;
                const width = getWidgetWidth(columns);
                elem.style.width = `${width}px`;
              }
            });

            packeryInstance.layout();
          }
        }
      }, 150);
    };

    // Use ResizeObserver to detect panel resize
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

      // Clean up widget ResizeObservers
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

  const getWidgetWidth = (requestedColumns) => {
    if (!containerRef) return 300;
    const horizontalPadding = 32; // 16px left + 16px right
    const containerWidth = containerRef.offsetWidth - horizontalPadding;
    const gutter = 16;
    const numColumns = calculateColumns();
    const columnWidth = (containerWidth - (gutter * (numColumns - 1))) / numColumns;

    // If requesting 2 columns but there's not enough space, use 1 column
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
        {/* Masonry Layout Container */}
        <div ref={containerRef} style={{ padding: '0 16px' }}>
          <For each={widgetLayout()}>
            {(layoutItem, index) => {
              const widget = getWidgetById(layoutItem.id);
              if (!widget) return null;

              const WidgetComponent = widget.component;
              const columns = layoutItem.columns || 1;
              const width = untrack(() => getWidgetWidth(columns));
              const isDragged = () => draggedIndex() === index();
              const isDropTarget = () => dropTargetIndex() === index() && draggedIndex() !== null && draggedIndex() !== index();

              return (
                  <div
                    class="widget-item relative group cursor-grab active:cursor-grabbing"
                    classList={{
                      'ring-4 ring-primary ring-offset-2': isDropTarget(),
                      'opacity-30': isDragged()
                    }}
                    data-widget-id={layoutItem.id}
                    data-order={layoutItem.order}
                    style={{
                      width: `${width}px`
                    }}
                  >
                  {/* Widget Controls - Small buttons in top-left corner */}
                  <div class="absolute top-1 left-1 z-20 flex items-center gap-0.5 opacity-0 group-hover:opacity-100 transition-opacity">
                    <button
                      class="btn btn-ghost btn-xs btn-square bg-base-100/80 backdrop-blur-sm hover:bg-base-200/90 shadow-sm h-6 w-6 min-h-0"
                      onClick={(e) => {
                        e.stopPropagation();
                        toggleWidgetColumns(widget.id);
                      }}
                    >
                      {columns === 1 ? <IconColumns2 size={12} /> : <IconColumns1 size={12} />}
                    </button>
                    <button
                      class="btn btn-ghost btn-xs btn-square bg-base-100/80 backdrop-blur-sm hover:bg-error/90 hover:text-error-content shadow-sm h-6 w-6 min-h-0"
                      onClick={(e) => {
                        e.stopPropagation();
                        removeWidget(widget.id);
                      }}
                    >
                      <IconX size={12} />
                    </button>
                  </div>

                  {/* Widget Content */}
                  <div class="relative">
                    <Show when={isDropTarget()}>
                      <div class="absolute inset-0 flex items-center justify-center bg-primary/20 rounded-lg z-10 pointer-events-none">
                        <span class="text-primary font-semibold text-lg bg-base-100 px-4 py-2 rounded-lg shadow-lg">
                          Drop here
                        </span>
                      </div>
                    </Show>
                    <WidgetComponent />
                  </div>
                </div>
              );
            }}
          </For>
        </div>

        {/* Instructions when empty */}
        <Show when={widgetLayout().length === 0}>
          <div class="text-center py-20 opacity-50">
            <p class="text-lg">No widgets on dashboard</p>
            <p class="text-sm mt-2">Widgets will appear here automatically</p>
          </div>
        </Show>
      </div>
    </div>
  );
}
