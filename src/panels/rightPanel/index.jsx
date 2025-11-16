import PanelResizer from '@/ui/PanelResizer.jsx';
import { editorStore, editorActions } from '@/layout/stores/EditorStore';
import { propertyTabs, propertiesPanelVisible } from '@/api/plugin';
import { useViewportContextMenu } from '@/ui/ViewportContextMenu.jsx';
import { Show, createMemo, createSignal, createEffect } from 'solid-js';
import { IconBox } from '@tabler/icons-solidjs';

const RightPanel = () => {
  const { showContextMenu } = useViewportContextMenu();

  // Get reactive store values
  const selection = () => editorStore.selection;
  const ui = () => editorStore.ui;
  const settings = () => editorStore.settings;
  const selectedObjectId = () => selection().entity;
  const selectedRightTool = () => ui().selectedTool;
  const rightPanelWidth = () => editorStore.ui.rightPanelWidth;
  
  const isScenePanelOpen = () => {
    return propertiesPanelVisible() && editorStore.panels.isScenePanelOpen;
  };
  
  const panelPosition = () => settings().editor.panelPosition || 'right';
  const isLeftPanel = () => panelPosition() === 'left';

  const {
    setSelectedTool: setSelectedRightTool,
    setScenePanelOpen
  } = editorActions;

  // Panel resize functionality
  const [isResizingRight, setIsResizingRight] = createSignal(false);
  const [rightDragOffset, setRightDragOffset] = createSignal(0);
  
  const handleRightResizeStart = (e) => {
    setIsResizingRight(true);
    // The actual panel left edge (where content starts, not including toolbar)
    const currentPanelLeft = window.innerWidth - rightPanelWidth();
    const offset = e?.clientX ? e.clientX - currentPanelLeft : 0;
    setRightDragOffset(offset);
  };
  
  const handleRightResizeEnd = () => {
    setIsResizingRight(false);
  };
  
  const handleRightResizeMove = (e) => {
    if (!isResizingRight()) return;
    
    const minPanelWidth = 180;
    const maxPanelWidth = 500;
    
    let newWidth;
    if (isLeftPanel()) {
      newWidth = e.clientX - rightDragOffset();
    } else {
      // Apply the drag offset so panel edge follows mouse cursor (same logic as bottom panel)
      newWidth = window.innerWidth - (e.clientX - rightDragOffset());
      
      // If the calculated width would be less than minimum (cursor too far right)
      // Just set to minimum width
      if (newWidth < minPanelWidth) {
        newWidth = minPanelWidth;
      }
      
      // If cursor is beyond window bounds, also set to minimum
      if (e.clientX >= window.innerWidth) {
        newWidth = minPanelWidth;
      }
    }
    
    const clampedWidth = Math.max(minPanelWidth, Math.min(newWidth, maxPanelWidth, window.innerWidth));
    editorActions.setRightPanelWidth(clampedWidth);
  };

  const handleObjectSelect = (objectId) => {
    if (objectId) {
      // Switch to scripts tab when an object is selected
      setSelectedRightTool('scripts');
    }
  };

  const handleContextMenu = (e, item, context = 'scene') => {
    if (!e) return;
    
    e.preventDefault();
    e.stopPropagation();

    // Use the reactive context menu system
    showContextMenu(e, item, context);
  };

  const handleRightPanelToggle = () => {
    if (!isScenePanelOpen()) {
      setScenePanelOpen(true);

      if (!selectedRightTool() || selectedRightTool() === 'select') {
        // Get the first available tab from property tabs
        const availableTabs = Array.from(propertyTabs().values())
          .filter(tab => !tab.condition || tab.condition())
          .sort((a, b) => (a.order || 0) - (b.order || 0));

        const firstTabId = availableTabs.length > 0 ? availableTabs[0].id : null;
        setSelectedRightTool(firstTabId);
      }
    } else {
      setScenePanelOpen(false);
    }
  };

  // Effect to ensure first tab is selected when panel opens and no tab is selected
  createEffect(() => {
    if (isScenePanelOpen() && propertyTabs().size > 0) {
      const currentTool = selectedRightTool();
      if (!currentTool || currentTool === 'select' || !propertyTabs().has(currentTool)) {
        const availableTabs = Array.from(propertyTabs().values())
          .filter(tab => !tab.condition || tab.condition())
          .sort((a, b) => (a.order || 0) - (b.order || 0));

        if (availableTabs.length > 0) {
          setSelectedRightTool(availableTabs[0].id);
        }
      }
    }
  });

  const renderTabContent = () => {
    const pluginTab = propertyTabs().get(selectedRightTool());
    if (pluginTab && pluginTab.component) {
      const PluginComponent = pluginTab.component;
      return <PluginComponent 
        onObjectSelect={handleObjectSelect}
        onContextMenu={handleContextMenu}
      />;
    }
    
    switch (selectedRightTool()) {
      
      default:
        return (
          <div class="h-full flex flex-col items-center justify-center text-center text-base-content/60 p-4">
            <IconBox class="w-8 h-8 mb-2 opacity-40" />
            <p class="text-xs">No properties panel available</p>
          </div>
        );
    }
  };

  return (
    <Show when={propertiesPanelVisible()}>
      <div
        className={`relative no-select flex-shrink-0 h-full ${!isResizingRight() ? 'transition-all duration-300' : ''}`}
        style={{
          width: isScenePanelOpen() ? `${rightPanelWidth()}px` : '0px'
        }}
      >
        <Show when={isScenePanelOpen()}>
          <div className="relative h-full flex">
            {/* Resize handle */}
            <PanelResizer
              type="right"
              isResizing={isResizingRight}
              onResizeStart={handleRightResizeStart}
              onResizeEnd={handleRightResizeEnd}
              onResize={handleRightResizeMove}
              isLeftPanel={isLeftPanel()}
              position={{
                left: '-4px',
                top: 0,
                bottom: 0,
                width: '8px',
                zIndex: 30
              }}
              className="!bg-transparent hover:!bg-transparent"
            />

            <div className="flex-1 min-w-0 overflow-hidden">
              <div className="flex flex-col h-full">
                {/* Panel content */}
                <div className="h-full bg-base-300 border-l border-base-300 shadow-lg overflow-hidden">
                  {/* Tab content */}
                  <div className="bg-base-200 w-full h-full overflow-y-auto scrollbar-thin">
                    {renderTabContent()}
                  </div>
                </div>
              </div>
            </div>
          </div>
        </Show>

        <Show when={!isScenePanelOpen()}>
          <div className="relative h-full flex items-center justify-center">
            <button
              onClick={() => setScenePanelOpen(true)}
              className="w-6 h-12 bg-base-300 border border-base-300 rounded-l-lg flex items-center justify-center text-base-content/60 hover:text-primary hover:bg-base-200 transition-colors group"
              title="Open panel"
            >
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" className="w-3 h-3">
                <path d="m9 18-6-6 6-6"/>
              </svg>
            </button>
          </div>
        </Show>
      </div>
    </Show>
  );
};

export default RightPanel;