import LeftPanelMenu from './LeftPanelMenu.jsx';
import PanelResizer from '@/ui/PanelResizer.jsx';
import { Show, createSignal, For } from 'solid-js';
import { leftPanelMenuItems } from '@/api/plugin';

const LeftPanel = () => {
  const [isOpen, setIsOpen] = createSignal(true);
  const [panelWidth, setPanelWidth] = createSignal(240);
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

    const minPanelWidth = 150;
    const maxPanelWidth = 400;

    const newWidth = e.clientX - dragOffset();

    if (newWidth < minPanelWidth) {
      setPanelWidth(minPanelWidth);
      return;
    }

    const clampedWidth = Math.max(minPanelWidth, Math.min(newWidth, maxPanelWidth, window.innerWidth / 2));
    setPanelWidth(clampedWidth);
  };

  const handleToggle = () => {
    setIsOpen(!isOpen());
  };

  return (
    <div
      className={`relative no-select flex-shrink-0 h-full ${!isResizing() ? 'transition-all duration-300' : ''}`}
      style={{
        width: isOpen() ? `${panelWidth()}px` : '0px'
      }}
    >
      {/* Toggle button - always visible, positioned on outer edge, near the top */}
      <div className="absolute right-0 top-2 translate-x-full z-40">
        <button
          onClick={(e) => {
            e.stopPropagation();
            handleToggle();
          }}
          className="w-4 h-8 bg-base-200 border border-base-300 rounded-r-lg flex items-center justify-center text-base-content/60 hover:text-primary hover:bg-base-200 transition-colors group"
          title={isOpen() ? "Close panel" : "Open panel"}
        >
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" className="w-2.5 h-2.5">
            <path d={isOpen() ? "m15 18-6-6 6-6" : "m9 18 6-6-6-6"}/>
          </svg>
        </button>
      </div>

      <Show when={isOpen()}>
        <div className="relative h-full flex">
          <div className="flex-1 min-w-0 overflow-hidden">
            <div className="flex flex-col h-full">
              {/* Panel content */}
              <div className="h-full bg-base-200 shadow-lg overflow-hidden">
                <LeftPanelMenu />
              </div>
            </div>
          </div>

          {/* Resize handle */}
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
            className="!bg-transparent hover:!bg-transparent"
          />
        </div>
      </Show>
    </div>
  );
};

export default LeftPanel;
