import { Show, For, createSignal, createMemo, onMount, onCleanup } from 'solid-js';
import { Dynamic } from 'solid-js/web';
import PanelResizer from '@/ui/PanelResizer.jsx';
import panelStore, { panels, activePlugin, panelVisibility, PANELS } from '@/api/plugin/panels';
import { IconBox, IconX, IconChevronLeft, IconChevronRight, IconChevronUp, IconChevronDown } from '@tabler/icons-solidjs';

/**
 * Unified Panel component that renders tabs for any panel location
 * @param {Object} props
 * @param {'left' | 'right' | 'bottom' | 'viewport'} props.position - Panel position
 * @param {string} [props.className] - Additional CSS classes
 */
const Panel = (props) => {
  const position = () => props.position;

  // Get tabs for this panel position
  const getTabs = createMemo(() => {
    const pos = position();
    switch (pos) {
      case 'left': return panelStore.getLeftTabs();
      case 'right': return panelStore.getRightTabs();
      case 'bottom': return panelStore.getBottomTabs();
      case 'viewport': return panelStore.getViewports();
      default: return [];
    }
  });

  // Active tab state per position
  const storageKey = () => `panel_${position()}_activeTab`;
  const [activeTab, setActiveTab] = createSignal(localStorage.getItem(storageKey()) || null);

  // Auto-select first tab when tabs change
  createMemo(() => {
    const tabs = getTabs();
    const current = activeTab();
    if (tabs.length > 0) {
      if (!current || !tabs.find(t => t.fullId === current)) {
        setActiveTab(tabs[0].fullId);
      }
    } else {
      setActiveTab(null);
    }
  });

  // Save active tab to localStorage
  createMemo(() => {
    const tab = activeTab();
    if (tab) {
      localStorage.setItem(storageKey(), tab);
    }
  });

  // Get active tab's component
  const getActiveComponent = createMemo(() => {
    const tabs = getTabs();
    const active = tabs.find(t => t.fullId === activeTab());
    return active?.component || null;
  });

  // Get active tab data
  const getActiveTabData = createMemo(() => {
    const tabs = getTabs();
    return tabs.find(t => t.fullId === activeTab()) || null;
  });

  // Panel sizing
  const getSizeKey = () => `panel_${position()}_size`;
  const getDefaultSize = () => {
    switch (position()) {
      case 'left': return 240;
      case 'right': return 300;
      case 'bottom': return 200;
      default: return 200;
    }
  };

  const [size, setSize] = createSignal(
    parseInt(localStorage.getItem(getSizeKey()) || getDefaultSize(), 10)
  );
  const [isResizing, setIsResizing] = createSignal(false);
  const [isCollapsed, setIsCollapsed] = createSignal(false);

  // Collapse/expand
  const getCollapseKey = () => `panel_${position()}_collapsed`;
  onMount(() => {
    const collapsed = localStorage.getItem(getCollapseKey());
    setIsCollapsed(collapsed === 'true');
  });

  const toggleCollapse = () => {
    const newState = !isCollapsed();
    setIsCollapsed(newState);
    localStorage.setItem(getCollapseKey(), String(newState));
  };

  // Resizing handlers
  const handleResizeStart = () => {
    setIsResizing(true);
  };

  const handleResizeEnd = () => {
    setIsResizing(false);
    localStorage.setItem(getSizeKey(), String(size()));
  };

  const handleResizeMove = (e) => {
    if (!isResizing()) return;

    const pos = position();
    let newSize;

    if (pos === 'left') {
      newSize = Math.max(180, Math.min(e.clientX, 500));
    } else if (pos === 'right') {
      newSize = Math.max(180, Math.min(window.innerWidth - e.clientX, 500));
    } else if (pos === 'bottom') {
      newSize = Math.max(100, Math.min(window.innerHeight - e.clientY, window.innerHeight * 0.6));
    }

    if (newSize) {
      setSize(newSize);
      window.dispatchEvent(new Event('viewport-resize'));
    }
  };

  // Check if panel should be visible
  const isVisible = createMemo(() => {
    const pos = position();
    if (pos === 'viewport') return true; // Viewport always visible
    return panelVisibility[pos] && getTabs().length > 0;
  });

  // Close tab handler
  const handleCloseTab = (fullId, e) => {
    e?.stopPropagation();
    panelStore.unregister(fullId);
  };

  // Render tab bar
  const renderTabBar = () => (
    <div class="flex items-center justify-between bg-base-300/50 border-b border-base-300 px-1 flex-shrink-0">
      <div class="flex items-center gap-0.5 overflow-x-auto flex-1">
        <For each={getTabs()}>
          {(tab) => (
            <button
              class={`flex items-center gap-1.5 px-2 py-1 text-xs font-medium transition-colors rounded-t ${
                activeTab() === tab.fullId
                  ? 'bg-base-200 text-base-content border-t border-l border-r border-base-300'
                  : 'text-base-content/60 hover:text-base-content hover:bg-base-200/50'
              }`}
              onClick={() => setActiveTab(tab.fullId)}
              title={tab.label}
            >
              <Show when={tab.icon}>
                <Dynamic component={tab.icon} class="w-3.5 h-3.5" />
              </Show>
              <span class="truncate max-w-24">{tab.label}</span>
              <Show when={tab.closable !== false}>
                <button
                  class="ml-1 p-0.5 rounded hover:bg-base-300 text-base-content/40 hover:text-base-content"
                  onClick={(e) => handleCloseTab(tab.fullId, e)}
                  title="Close tab"
                >
                  <IconX class="w-3 h-3" />
                </button>
              </Show>
            </button>
          )}
        </For>
      </div>

      {/* Panel controls */}
      <div class="flex items-center gap-1 ml-2">
        <button
          class="p-1 rounded hover:bg-base-300 text-base-content/60 hover:text-base-content transition-colors"
          onClick={toggleCollapse}
          title={isCollapsed() ? 'Expand' : 'Collapse'}
        >
          <Show when={position() === 'left'}>
            {isCollapsed() ? <IconChevronRight class="w-4 h-4" /> : <IconChevronLeft class="w-4 h-4" />}
          </Show>
          <Show when={position() === 'right'}>
            {isCollapsed() ? <IconChevronLeft class="w-4 h-4" /> : <IconChevronRight class="w-4 h-4" />}
          </Show>
          <Show when={position() === 'bottom'}>
            {isCollapsed() ? <IconChevronUp class="w-4 h-4" /> : <IconChevronDown class="w-4 h-4" />}
          </Show>
        </button>
      </div>
    </div>
  );

  // Render content area
  const renderContent = () => (
    <div class="flex-1 overflow-auto">
      <Show
        when={getActiveComponent()}
        fallback={
          <div class="h-full flex flex-col items-center justify-center text-center text-base-content/60 p-4">
            <IconBox class="w-8 h-8 mb-2 opacity-40" />
            <p class="text-xs">No content</p>
          </div>
        }
      >
        <Dynamic component={getActiveComponent()} tab={getActiveTabData()} />
      </Show>
    </div>
  );

  // Get resizer position config
  const getResizerPosition = () => {
    switch (position()) {
      case 'left':
        return { right: '-4px', top: 0, bottom: 0, width: '8px', zIndex: 30 };
      case 'right':
        return { left: '-4px', top: 0, bottom: 0, width: '8px', zIndex: 30 };
      case 'bottom':
        return { left: 0, right: 0, top: '-4px', height: '8px', zIndex: 30 };
      default:
        return {};
    }
  };

  // Render based on position
  const renderPanel = () => {
    const pos = position();

    if (pos === 'viewport') {
      // Viewport is special - no tab bar, just content
      return (
        <div class="w-full h-full relative">
          <For each={getTabs()}>
            {(tab) => (
              <div
                class="absolute inset-0 bg-base-100"
                style={{ display: activeTab() === tab.fullId ? 'block' : 'none' }}
              >
                <Show when={tab.component}>
                  <Dynamic component={tab.component} tab={tab} />
                </Show>
              </div>
            )}
          </For>
        </div>
      );
    }

    // Side and bottom panels with tabs
    const isHorizontal = pos === 'left' || pos === 'right';
    const borderClass = pos === 'left' ? 'border-r' : pos === 'right' ? 'border-l' : 'border-t';

    return (
      <Show when={isVisible()}>
        <div
          class={`relative flex-shrink-0 bg-base-200 ${borderClass} border-base-300 flex ${isHorizontal ? 'flex-col h-full' : 'flex-col'} ${props.className || ''}`}
          style={{
            [isHorizontal ? 'width' : 'height']: isCollapsed() ? '0px' : `${size()}px`,
            transition: isResizing() ? 'none' : 'all 0.3s'
          }}
        >
          <Show when={!isCollapsed()}>
            <PanelResizer
              type={pos}
              isResizing={isResizing}
              onResizeStart={handleResizeStart}
              onResizeEnd={handleResizeEnd}
              onResize={handleResizeMove}
              position={getResizerPosition()}
            />

            {renderTabBar()}
            {renderContent()}
          </Show>

          {/* Collapsed state button */}
          <Show when={isCollapsed()}>
            <div class={`flex items-center justify-center ${isHorizontal ? 'h-full' : 'w-full'}`}>
              <button
                onClick={toggleCollapse}
                class={`${isHorizontal ? 'w-6 h-12' : 'h-6 w-12'} bg-base-300 border border-base-300 ${
                  pos === 'left' ? 'rounded-r-lg' : pos === 'right' ? 'rounded-l-lg' : 'rounded-b-lg'
                } flex items-center justify-center text-base-content/60 hover:text-primary hover:bg-base-200 transition-colors`}
                title="Expand panel"
              >
                <Show when={pos === 'left'}><IconChevronRight class="w-3 h-3" /></Show>
                <Show when={pos === 'right'}><IconChevronLeft class="w-3 h-3" /></Show>
                <Show when={pos === 'bottom'}><IconChevronUp class="w-3 h-3" /></Show>
              </button>
            </div>
          </Show>
        </div>
      </Show>
    );
  };

  return renderPanel();
};

export default Panel;

// Named exports for convenience
export const LeftPanel = () => <Panel position="left" />;
export const RightPanel = () => <Panel position="right" />;
export const BottomPanel = () => <Panel position="bottom" />;
export const ViewportPanel = () => <Panel position="viewport" />;
