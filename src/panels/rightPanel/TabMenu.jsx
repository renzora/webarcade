import { createSignal, createEffect, createMemo, For } from 'solid-js';
import { editorStore, editorActions } from '@/layout/stores/EditorStore';
import { viewportStore } from '@/panels/viewport/store';
import { propertyTabs, viewportTypes } from '@/api/plugin';

const defaultTools = [];

const defaultBottomTools = [];

const workflowTools = {
  'daw-editor': [],
  'material-editor': [],
  'animation-editor': [],
  'text-editor': [],
  'video-editor': [],
  'photo-editor': [],
  'model-preview': [],
  'default': []
};

function TabMenu(props) {
  const [tools, setTools] = createSignal(() => getOrderedTools());
  
  createEffect(() => {
    propertyTabs();
    editorStore.selection.entity; // React to selection changes
    const newTools = getOrderedTools();
    setTools(newTools);
  });
  const [bottomTools, setBottomTools] = createSignal(() => getOrderedBottomTools());
  
  const [dragState, setDragState] = createSignal({
    isDragging: false,
    draggedTool: null,
    dragOverTool: null,
    draggedFromBottom: false
  });

  const settings = createMemo(() => editorStore.settings);
  const panelPosition = createMemo(() => settings().editor.panelPosition || 'right');
  const isPanelOnLeft = createMemo(() => panelPosition() === 'left');
  const shouldTooltipGoRight = createMemo(() => isPanelOnLeft());

  const viewport = createMemo(() => viewportStore);
  const ui = createMemo(() => editorStore.ui);
  const getCurrentWorkflow = () => {
    const viewportData = viewport();
    if (!viewportData.tabs || viewportData.tabs.length === 0) {
      return 'default';
    }
    const activeTabData = viewportData.tabs.find(tab => tab.id === viewportData.activeTabId);
    return activeTabData?.type || 'default';
  };
  
  function getOrderedTools() {
    const currentWorkflow = getCurrentWorkflow();
    const allowedToolIds = workflowTools[currentWorkflow] || workflowTools['default'];
    const _selectedEntityId = editorStore.selection.entity;
    const selectedBabylonObject = null; // Render store removed
    
    const pluginTabs = Array.from(propertyTabs().values())
      .filter(tab => {
        // Check if tab has a condition function
        if (tab.condition && typeof tab.condition === 'function') {
          // Pass the actual Babylon.js object to the condition function
          return tab.condition(selectedBabylonObject);
        }
        return true; // Show tab if no condition
      })
      .sort((a, b) => (a.order || 0) - (b.order || 0))
      .map(tab => ({
        id: tab.id,
        icon: tab.icon,
        title: tab.title
      }));
    
    const allTools = [...defaultTools, ...pluginTabs];
    
    const toolsMap = allTools.reduce((map, tool) => {
      map[tool.id] = tool;
      return map;
    }, {});
    
    let currentTabOrder = ui().toolbarTabOrder || [];
    const missingTools = allTools
      .filter(tool => !currentTabOrder.includes(tool.id))
      .map(tool => tool.id);
    
    if (missingTools.length > 0) {
      currentTabOrder = [...currentTabOrder, ...missingTools];
      editorActions.setToolbarTabOrder(currentTabOrder);
    }
    
    if (!currentTabOrder || !Array.isArray(currentTabOrder)) {
      return allTools.filter(tool => 
        allowedToolIds.includes(tool.id) || 
        propertyTabs().has(tool.id)
      );
    }
    
    const workflowFilteredTools = currentTabOrder
      .filter(id => allowedToolIds.includes(id) || propertyTabs().has(id))
      .map(id => toolsMap[id])
      .filter(Boolean);
    
    return workflowFilteredTools;
  }
  
  function getOrderedBottomTools() {
    const allBottomTools = [...defaultBottomTools];
    
    const toolbarBottomTabOrder = ui().toolbarBottomTabOrder;
    if (!toolbarBottomTabOrder || !Array.isArray(toolbarBottomTabOrder)) {
      return allBottomTools;
    }
    const toolsMap = allBottomTools.reduce((map, tool) => {
      map[tool.id] = tool;
      return map;
    }, {});
    
    const orderedTools = toolbarBottomTabOrder.map(id => toolsMap[id]).filter(Boolean);
    const existingIds = new Set(toolbarBottomTabOrder);
    const newTools = allBottomTools.filter(tool => !existingIds.has(tool.id));
    
    return [...orderedTools, ...newTools];
  }

  createEffect(() => {
    const orderedTools = getOrderedTools();
    setTools(orderedTools);
  });

  
  createEffect(() => {
    const newBottomTools = getOrderedBottomTools();
    setBottomTools(newBottomTools);
  });

  const handleDragStart = (e, tool, isFromBottom = false) => {
    e.dataTransfer.effectAllowed = 'move';
    e.dataTransfer.setData('text/html', '');
    
    const dragElement = e.currentTarget.cloneNode(true);
    dragElement.style.position = 'absolute';
    dragElement.style.top = '-1000px';
    dragElement.style.left = '-1000px';
    dragElement.style.background = 'oklch(var(--b2) / 0.95)';
    dragElement.style.border = '1px solid oklch(var(--b3) / 0.5)';
    dragElement.style.borderRadius = '8px';
    dragElement.style.padding = '8px';
    dragElement.style.boxShadow = '0 25px 50px -12px rgb(0 0 0 / 0.5)';
    dragElement.style.transform = 'scale(1.1)';
    dragElement.style.pointerEvents = 'none';
    dragElement.style.zIndex = '9999';
    
    const icon = dragElement.querySelector('svg');
    if (icon) {
      icon.style.color = 'oklch(var(--bc))';
    }
    
    document.body.appendChild(dragElement);
    e.dataTransfer.setDragImage(dragElement, 24, 24);
    
    setTimeout(() => {
      if (document.body.contains(dragElement)) {
        document.body.removeChild(dragElement);
      }
    }, 100);
    
    setDragState({
      isDragging: true,
      draggedTool: tool,
      dragOverTool: null,
      draggedFromBottom: isFromBottom
    });
  };

  const handleDragOver = (e, tool) => {
    e.preventDefault();
    e.dataTransfer.dropEffect = 'move';
    
    if (dragState().draggedTool && dragState().draggedTool.id !== tool.id) {
      setDragState(prev => ({ ...prev, dragOverTool: tool }));
    }
  };

  const handleDragLeave = (e) => {
    const rect = e.currentTarget.getBoundingClientRect();
    const { clientX, clientY } = e;
    
    if (clientX < rect.left || clientX > rect.right || clientY < rect.top || clientY > rect.bottom) {
      setDragState(prev => ({ ...prev, dragOverTool: null }));
    }
  };

  const handleDrop = (e, dropTool, isBottomArea = false) => {
    e.preventDefault();
    e.stopPropagation();
    
    const currentDragState = dragState();
    if (!currentDragState.draggedTool || currentDragState.draggedTool.id === dropTool.id) {
      setDragState({
        isDragging: false,
        draggedTool: null,
        dragOverTool: null,
        draggedFromBottom: false
      });
      return;
    }

    const sourceArray = currentDragState.draggedFromBottom ? bottomTools() : tools();
    const targetArray = isBottomArea ? bottomTools() : tools();
    const setSourceArray = currentDragState.draggedFromBottom ? setBottomTools : setTools;
    const setTargetArray = isBottomArea ? setBottomTools : setTools;

    if (currentDragState.draggedFromBottom !== isBottomArea) {
      const newSourceArray = sourceArray.filter(tool => tool.id !== currentDragState.draggedTool.id);
      setSourceArray(newSourceArray);
      
      const dropIndex = targetArray.findIndex(tool => tool.id === dropTool.id);
      const newTargetArray = [...targetArray];
      newTargetArray.splice(dropIndex, 0, currentDragState.draggedTool);
      setTargetArray(newTargetArray);
      
      if (currentDragState.draggedFromBottom) {
        editorActions.setToolbarBottomTabOrder(newSourceArray.map(tool => tool.id));
        editorActions.setToolbarTabOrder(newTargetArray.map(tool => tool.id));
      } else {
        editorActions.setToolbarTabOrder(newSourceArray.map(tool => tool.id));
        editorActions.setToolbarBottomTabOrder(newTargetArray.map(tool => tool.id));
      }
    } else {
      const draggedIndex = sourceArray.findIndex(tool => tool.id === currentDragState.draggedTool.id);
      const dropIndex = sourceArray.findIndex(tool => tool.id === dropTool.id);
      
      if (draggedIndex !== -1 && dropIndex !== -1 && draggedIndex !== dropIndex) {
        const newArray = [...sourceArray];
        const [removed] = newArray.splice(draggedIndex, 1);
        newArray.splice(dropIndex, 0, removed);
        setSourceArray(newArray);
        
        const newOrder = newArray.map(tool => tool.id);
        if (isBottomArea) {
          editorActions.setToolbarBottomTabOrder(newOrder);
        } else {
          editorActions.setToolbarTabOrder(newOrder);
        }
      }
    }

    setDragState({
      isDragging: false,
      draggedTool: null,
      dragOverTool: null,
      draggedFromBottom: false
    });
  };

  const handleDragEnd = () => {
    setDragState({
      isDragging: false,
      draggedTool: null,
      dragOverTool: null,
      draggedFromBottom: false
    });
  };

  const findAssociatedViewport = async (tabId) => {
    // Get the tab configuration
    const tab = propertyTabs().get(tabId);
    if (!tab) {
      return null;
    }

    const allViewportTypes = viewportTypes();
    let viewportTypeId = null;

    // Strategy 0: Check if tab explicitly defines a viewport association
    if (tab.viewport) {
      if (allViewportTypes.has(tab.viewport)) {
        viewportTypeId = tab.viewport;
      }
    }

    // If no explicit viewport and no plugin info, can't auto-detect
    if (!viewportTypeId && !tab.plugin) {
      return null;
    }

    // Find all viewports from the same plugin (for auto-detection)
    if (!viewportTypeId) {
      const pluginViewports = Array.from(allViewportTypes.entries())
        .filter(([_, viewport]) => viewport.plugin === tab.plugin)
        .map(([id, _]) => id);

      if (pluginViewports.length === 0) {
        return null;
      }

      // Strategy 1: If plugin only has one viewport, use that
      if (pluginViewports.length === 1) {
        viewportTypeId = pluginViewports[0];
      }
    }

    if (!viewportTypeId) {
      return null;
    }

    // Check if this viewport type actually exists
    const viewportType = viewportTypes().get(viewportTypeId);
    if (!viewportType) {
      return null;
    }

    try {
      const { viewportActions } = await import('@/panels/viewport/store');

      // Check if a tab with this type already exists
      const existingTab = viewportStore.tabs.find(tab => tab.type === viewportTypeId);

      if (existingTab) {
        // Tab already exists, just activate it
        viewportActions.setActiveViewportTab(existingTab.id);
        return true;
      }

      // No existing tab found, create a new one
      const newTabId = `${viewportTypeId}_${Date.now()}`;
      const newTab = {
        id: newTabId,
        name: viewportType.label,
        label: viewportType.label,
        type: viewportTypeId,
        icon: viewportType.icon,
        component: viewportType.component,
        isPinned: false,
        hasUnsavedChanges: false
      };

      viewportActions.addViewportTab(newTab);
      viewportActions.setActiveViewportTab(newTabId);
      return true;
    } catch (error) {
      console.error('[TabMenu] Error opening viewport:', error);
      return null;
    }
  };

  const handleToolClick = async (tool) => {
    if (!dragState().isDragging) {
      if (tool.isPluginButton && tool.onClick) {
        tool.onClick();
        return;
      }

      const _currentWorkflow = getCurrentWorkflow();

      // Try to open associated viewport (if any)
      const viewportOpened = await findAssociatedViewport(tool.id);

      // If viewport was opened/focused, we might still want to show the tab
      // This allows the property panel to be shown alongside the viewport
      if (!props.scenePanelOpen) {
        props.onScenePanelToggle();
      }
      props.onToolSelect(tool.id);
    }
  };


  return (
    <div class="relative w-10 h-full bg-base-300 border-l border-r border-black/15 flex flex-col pointer-events-auto no-select">
      <div class="flex-1 overflow-hidden h-full flex flex-col gap-0.5 py-1">
        <For each={tools()}>
          {(tool) => {
            const isDragged = () => dragState().draggedTool?.id === tool.id;
            const isDragOver = () => dragState().dragOverTool?.id === tool.id;
            
            return (
              <button
                draggable
                onClick={() => handleToolClick(tool)}
                onDragStart={(e) => handleDragStart(e, tool, false)}
                onDragOver={(e) => handleDragOver(e, tool, false)}
                onDragLeave={handleDragLeave}
                onDrop={(e) => handleDrop(e, tool, false)}
                onDragEnd={handleDragEnd}
                class={`btn btn-ghost h-7 w-full min-h-0 p-0 rounded-none transition-all duration-200 group relative select-none flex items-center justify-center border-none ${
                  isDragged()
                    ? 'opacity-50 cursor-grabbing scale-95'
                    : props.selectedTool === tool.id
                      ? 'bg-primary/20 text-primary'
                      : 'text-base-content/60 hover:bg-base-300 hover:text-base-content'
                }`}
                title={tool.title}
              >
                <tool.icon class="w-4 h-4" />
                
                {isDragOver() && (
                  <div class="absolute inset-x-0 top-0 h-0.5 bg-primary rounded-full"></div>
                )}
                
                {!dragState().isDragging && (
                  <div class={`absolute ${shouldTooltipGoRight() ? 'left-full ml-1' : 'right-full mr-1'} top-1/2 -translate-y-1/2 bg-base-300/95 backdrop-blur-sm border border-base-300 text-base-content text-[11px] px-2 py-1 rounded-md opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none whitespace-nowrap shadow-2xl`}
                       style={{ 'z-index': 999999 }}>
                    {tool.title}
                    <div class={`absolute ${shouldTooltipGoRight() ? 'right-full' : 'left-full'} top-1/2 -translate-y-1/2 w-0 h-0 ${shouldTooltipGoRight() ? 'border-r-4 border-r-base-300' : 'border-l-4 border-l-base-300'} border-t-4 border-t-transparent border-b-4 border-b-transparent`}></div>
                  </div>
                )}
              </button>
            );
          }}
        </For>
      </div>
    </div>
  );
}

export default TabMenu;
