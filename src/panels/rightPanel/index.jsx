import TabMenu from './TabMenu.jsx';
import PanelResizer from '@/ui/PanelResizer.jsx';
import { editorStore, editorActions } from '@/layout/stores/EditorStore';
import { propertyTabs, propertiesPanelVisible } from '@/api/plugin';
import { useViewportContextMenu } from '@/ui/ViewportContextMenu.jsx';
import { Show, createMemo, createSignal, createEffect } from 'solid-js';
import { IconBox } from '@tabler/icons-solidjs';

const RightPanel = () => {
  const { showContextMenu } = useViewportContextMenu();
  const [_contextMenu, _setContextMenu] = createSignal(null);
  const [isCollapsed, setIsCollapsed] = createSignal(false);

  // Get reactive store values
  const selection = () => editorStore.selection;
  const ui = () => editorStore.ui;
  const settings = () => editorStore.settings;
  const selectedObjectId = () => selection().entity;
  const selectedRightTool = () => ui().selectedTool;
  const rightPanelWidth = () => editorStore.ui.rightPanelWidth;
  const bottomPanelHeight = () => editorStore.ui.bottomPanelHeight;
  
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
  const [isResizingTabs, setIsResizingTabs] = createSignal(false);
  const [tabContainerHeight, setTabContainerHeight] = createSignal(680);
  
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
    
    const minPanelWidth = 250;
    const maxPanelWidth = 800;
    
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

  const handleTabResizeStart = (e) => {
    e.preventDefault();
    e.stopPropagation();
    setIsResizingTabs(true);
    const startY = e.clientY;
    const startHeight = tabContainerHeight();
    
    const handleMouseMove = (e) => {
      e.preventDefault();
      e.stopPropagation();
      const deltaY = startY - e.clientY;
      
      // Calculate available space dynamically
      const rightPanelElement = document.querySelector(`[style*="width: ${rightPanelWidth()}px"]`);
      const sceneElement = rightPanelElement?.querySelector('.flex-1.min-h-0');
      
      let maxHeight = window.innerHeight - 200; // Default fallback
      
      if (rightPanelElement && sceneElement) {
        const panelRect = rightPanelElement.getBoundingClientRect();
        const _sceneRect = sceneElement.getBoundingClientRect();
        
        // Reserve minimum 150px for Scene.jsx (header + some content + bottom panel)
        const minSceneHeight = 150;
        const availableSpace = panelRect.height - minSceneHeight - 24; // 24px for resize bar + padding
        maxHeight = Math.max(200, availableSpace);
      }
      
      const newHeight = Math.max(200, Math.min(maxHeight, startHeight + deltaY));
      setTabContainerHeight(newHeight);
    };
    
    const handleMouseUp = (e) => {
      e.preventDefault();
      e.stopPropagation();
      setIsResizingTabs(false);
      document.removeEventListener('mousemove', handleMouseMove);
      document.removeEventListener('mouseup', handleMouseUp);
    };
    
    document.addEventListener('mousemove', handleMouseMove);
    document.addEventListener('mouseup', handleMouseUp);
  };

  const handleContextMenu = (e, item, context = 'scene') => {
    if (!e) return;
    
    e.preventDefault();
    e.stopPropagation();

    // Use the reactive context menu system
    showContextMenu(e, item, context);
  };

  const handleRightPanelToggle = () => {
    const currentState = isScenePanelOpen();

    if (currentState && !isCollapsed()) {
      // Currently expanded -> collapse to icons
      setIsCollapsed(true);
    } else if (currentState && isCollapsed()) {
      // Currently collapsed -> hide completely
      setScenePanelOpen(false);
      setIsCollapsed(false);
    } else {
      // Currently hidden -> show expanded
      setScenePanelOpen(true);
      setIsCollapsed(false);

      if (!selectedRightTool() || selectedRightTool() === 'select') {
        // Get the first available tab from property tabs
        const availableTabs = Array.from(propertyTabs().values())
          .filter(tab => !tab.condition || tab.condition())
          .sort((a, b) => (a.order || 0) - (b.order || 0));

        const firstTabId = availableTabs.length > 0 ? availableTabs[0].id : null;
        setSelectedRightTool(firstTabId);
      }
    }
  };
  
  // Always show tabs
  const shouldShowTabs = createMemo(() => true);

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

  const _getTabTitle = createMemo(() => {
    const pluginTab = propertyTabs().get(selectedRightTool());
    if (pluginTab) {
      return pluginTab.title;
    }
    
    switch (selectedRightTool()) {
      default: return 'Properties';
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
          width: isScenePanelOpen() ? (isCollapsed() ? '48px' : `${rightPanelWidth()}px`) : '0px'
        }}
      >
        <Show when={isScenePanelOpen()}>
          <div className="relative h-full flex">
            {/* Resize handle - hide when collapsed */}
            <Show when={!isCollapsed()}>
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
                className="!bg-transparent !opacity-0 hover:!bg-primary/20 hover:!opacity-100"
              />
            </Show>

            <div className="flex-1 min-w-0 overflow-hidden">
              <div className="flex flex-col h-full">
                {/* Collapse/Expand button - positioned inside panel */}
                <div className="absolute top-2 right-2 z-10">
                  <button
                    onClick={(e) => {
                      e.stopPropagation();
                      handleRightPanelToggle();
                    }}
                    className="w-6 h-6 text-base-content/60 hover:text-primary transition-colors flex items-center justify-center group relative"
                    style={{
                      'background-color': 'oklch(var(--b2))',
                      'border-left': '1px solid oklch(var(--b3))',
                      'border-top': '1px solid oklch(var(--b3))',
                      'border-bottom': '1px solid oklch(var(--b3))',
                      'border-top-left-radius': '6px',
                      'border-bottom-left-radius': '6px'
                    }}
                    title={isCollapsed() ? "Hide panel" : "Collapse panel"}
                  >
                    <div className="w-3 h-3 flex items-center justify-center">
                      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" className="w-3 h-3">
                        <path d="m9 18 6-6-6-6"/>
                      </svg>
                    </div>

                    <div className="absolute right-full mr-1 top-1/2 -translate-y-1/2 bg-base-300 backdrop-blur-sm border border-base-300 text-base-content text-xs px-3 py-1.5 rounded-md opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none whitespace-nowrap shadow-2xl"
                         style={{ 'z-index': 50 }}>
                      {isCollapsed() ? "Hide panel" : "Collapse panel"}
                      <div className="absolute left-full top-1/2 -translate-y-1/2 w-0 h-0 border-l-4 border-l-base-300 border-t-4 border-t-transparent border-b-4 border-b-transparent"></div>
                    </div>
                  </button>
                </div>

                {/* Panel content */}
                <div className="h-full bg-base-300 border-l border-base-300 shadow-lg overflow-hidden flex">
                  {/* Tab system for properties */}
                  <div className="flex bg-base-200 w-full h-full">
                    {/* Vertical toolbar inside right panel */}
                    <div className="w-auto flex-shrink-0 h-full">
                      <TabMenu
                        selectedTool={selectedRightTool()}
                        onToolSelect={(tool) => {
                          setSelectedRightTool(tool);
                          if (isCollapsed()) {
                            setIsCollapsed(false);
                          }
                        }}
                        scenePanelOpen={isScenePanelOpen()}
                        onScenePanelToggle={handleRightPanelToggle}
                        isLeftPanel={isLeftPanel()}
                        panelResize={{
                          handleRightResizeStart,
                          handleRightResizeMove,
                          handleRightResizeEnd
                        }}
                      />
                    </div>

                    {/* Tab content - hide when collapsed */}
                    <Show when={!isCollapsed()}>
                      <div className="flex-1 min-w-0 overflow-y-auto scrollbar-thin">
                        {renderTabContent()}
                      </div>
                    </Show>
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