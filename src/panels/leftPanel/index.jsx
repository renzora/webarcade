import LeftPanelMenu from './LeftPanelMenu.jsx';
import PanelResizer from '@/ui/PanelResizer.jsx';
import { Show, createSignal, For } from 'solid-js';
import { leftPanelMenuItems } from '@/api/plugin';

const LeftPanel = () => {
  const [isOpen, setIsOpen] = createSignal(true);
  const [isCollapsed, setIsCollapsed] = createSignal(false);
  const [panelWidth, setPanelWidth] = createSignal(270);
  const [isResizing, setIsResizing] = createSignal(false);
  const [dragOffset, setDragOffset] = createSignal(0);

  const handleResizeStart = (e) => {
    setIsResizing(true);
    const currentPanelRight = panelWidth();
    const offset = e?.clientX ? e.clientX - currentPanelRight : 0;
    setDragOffset(offset);
  };

  const handleResizeEnd = () => {
    setIsResizing(false);
  };

  const handleResizeMove = (e) => {
    if (!isResizing()) return;

    const minPanelWidth = 200;
    const maxPanelWidth = 600;

    const newWidth = e.clientX - dragOffset();

    if (newWidth < minPanelWidth) {
      setPanelWidth(minPanelWidth);
      return;
    }

    const clampedWidth = Math.max(minPanelWidth, Math.min(newWidth, maxPanelWidth, window.innerWidth / 2));
    setPanelWidth(clampedWidth);
  };

  const handleToggle = () => {
    if (isOpen() && !isCollapsed()) {
      // Currently expanded -> collapse to icons
      setIsCollapsed(true);
    } else if (isOpen() && isCollapsed()) {
      // Currently collapsed -> hide completely
      setIsOpen(false);
      setIsCollapsed(false);
    } else {
      // Currently hidden -> show expanded
      setIsOpen(true);
      setIsCollapsed(false);
    }
  };

  return (
    <div
      className={`relative no-select flex-shrink-0 h-full ${!isResizing() ? 'transition-all duration-300' : ''}`}
      style={{
        width: isOpen() ? (isCollapsed() ? '48px' : `${panelWidth()}px`) : '0px'
      }}
    >
      <Show when={isOpen()}>
        <div className="relative h-full flex">
          <div className="flex-1 min-w-0 overflow-hidden">
            <div className="flex flex-col h-full">
              {/* Collapse/Expand button - positioned inside panel */}
              <div className="absolute top-2 left-2 z-10">
                <button
                  onClick={(e) => {
                    e.stopPropagation();
                    handleToggle();
                  }}
                  className="w-6 h-6 text-base-content/60 hover:text-primary transition-colors flex items-center justify-center group relative"
                  style={{
                    'background-color': 'oklch(var(--b2))',
                    'border-right': '1px solid oklch(var(--b3))',
                    'border-top': '1px solid oklch(var(--b3))',
                    'border-bottom': '1px solid oklch(var(--b3))',
                    'border-top-right-radius': '6px',
                    'border-bottom-right-radius': '6px'
                  }}
                  title={isCollapsed() ? "Hide panel" : "Collapse panel"}
                >
                  <div className="w-3 h-3 flex items-center justify-center">
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" className="w-3 h-3">
                      <path d="m15 18-6-6 6-6"/>
                    </svg>
                  </div>

                  <div className="absolute left-full ml-1 top-1/2 -translate-y-1/2 bg-base-300 backdrop-blur-sm border border-base-300 text-base-content text-xs px-3 py-1.5 rounded-md opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none whitespace-nowrap shadow-2xl"
                       style={{ 'z-index': 50 }}>
                    {isCollapsed() ? "Hide panel" : "Collapse panel"}
                    <div className="absolute right-full top-1/2 -translate-y-1/2 w-0 h-0 border-r-4 border-r-base-300 border-t-4 border-t-transparent border-b-4 border-b-transparent"></div>
                  </div>
                </button>
              </div>

              {/* Panel content */}
              <div className="h-full bg-base-300 border-r border-base-300 shadow-lg overflow-hidden">
                <Show
                  when={!isCollapsed()}
                  fallback={
                    <div className="h-full flex flex-col items-center py-2 overflow-y-auto scrollbar-thin">
                      <For each={Array.from(leftPanelMenuItems().values()).sort((a, b) => a.order - b.order)}>
                        {(item) => (
                          <button
                            onClick={() => {
                              if (item.onClick) {
                                item.onClick();
                              }
                              setIsCollapsed(false);
                            }}
                            className="w-10 h-10 flex items-center justify-center text-base-content/60 hover:text-primary hover:bg-base-200 rounded-lg transition-colors mb-1 group relative"
                            title={item.label}
                          >
                            <Show when={item.icon}>
                              <item.icon className="w-5 h-5" />
                            </Show>

                            <div className="absolute left-full ml-2 top-1/2 -translate-y-1/2 bg-base-300/95 backdrop-blur-sm border border-base-300 text-base-content text-xs px-3 py-1.5 rounded-md opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none whitespace-nowrap shadow-2xl z-50">
                              {item.label}
                              <div className="absolute right-full top-1/2 -translate-y-1/2 w-0 h-0 border-r-4 border-r-base-300 border-t-4 border-t-transparent border-b-4 border-b-transparent"></div>
                            </div>
                          </button>
                        )}
                      </For>
                    </div>
                  }
                >
                  <LeftPanelMenu />
                </Show>
              </div>
            </div>
          </div>

          {/* Resize handle - hide when collapsed */}
          <Show when={!isCollapsed()}>
            <PanelResizer
              type="left"
              isResizing={isResizing}
              onResizeStart={handleResizeStart}
              onResizeEnd={handleResizeEnd}
              onResize={handleResizeMove}
              position={{
                right: '-4px',
                top: 0,
                bottom: 0,
                width: '8px',
                zIndex: 30
              }}
              className="!bg-transparent !opacity-0 hover:!bg-primary/20 hover:!opacity-100"
            />
          </Show>
        </div>
      </Show>

      <Show when={!isOpen()}>
        <div className="relative h-full flex items-center justify-center">
          <button
            onClick={() => setIsOpen(true)}
            className="w-6 h-12 bg-base-300 border border-base-300 rounded-r-lg flex items-center justify-center text-base-content/60 hover:text-primary hover:bg-base-200 transition-colors group"
            title="Open panel"
          >
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" className="w-3 h-3">
              <path d="m15 18 6-6-6-6"/>
            </svg>
          </button>
        </div>
      </Show>
    </div>
  );
};

export default LeftPanel;
