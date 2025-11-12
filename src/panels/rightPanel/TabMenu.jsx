import { createSignal, createEffect, createMemo, For } from 'solid-js';
import { editorStore, editorActions } from '@/layout/stores/EditorStore';
import { viewportStore } from '@/panels/viewport/store';
import { propertyTabs, viewportTypes } from '@/api/plugin';

function TabMenu(props) {
  const [tools, setTools] = createSignal(() => getOrderedTools());

  const settings = createMemo(() => editorStore.settings);
  const panelPosition = createMemo(() => settings().editor.panelPosition || 'right');
  const isPanelOnLeft = createMemo(() => panelPosition() === 'left');
  const shouldTooltipGoRight = createMemo(() => isPanelOnLeft());

  const ui = createMemo(() => editorStore.ui);

  function getOrderedTools() {
    const pluginTabs = Array.from(propertyTabs().values())
      .filter(tab => {
        // Check if tab has a condition function
        if (tab.condition && typeof tab.condition === 'function') {
          return tab.condition();
        }
        return true; // Show tab if no condition
      })
      .sort((a, b) => (a.order || 0) - (b.order || 0))
      .map(tab => ({
        id: tab.id,
        icon: tab.icon,
        title: tab.title
      }));

    const toolsMap = pluginTabs.reduce((map, tool) => {
      map[tool.id] = tool;
      return map;
    }, {});

    let currentTabOrder = ui().toolbarTabOrder || [];
    const missingTools = pluginTabs
      .filter(tool => !currentTabOrder.includes(tool.id))
      .map(tool => tool.id);

    if (missingTools.length > 0) {
      currentTabOrder = [...currentTabOrder, ...missingTools];
      editorActions.setToolbarTabOrder(currentTabOrder);
    }

    if (!currentTabOrder || !Array.isArray(currentTabOrder)) {
      return pluginTabs;
    }

    const orderedTools = currentTabOrder
      .map(id => toolsMap[id])
      .filter(Boolean);

    return orderedTools;
  }
  
  createEffect(() => {
    propertyTabs();
    editorStore.selection.entity; // React to selection changes
    const orderedTools = getOrderedTools();
    setTools(orderedTools);
  });

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
    if (tool.isPluginButton && tool.onClick) {
      tool.onClick();
      return;
    }

    // Try to open associated viewport (if any)
    await findAssociatedViewport(tool.id);

    // If viewport was opened/focused, we might still want to show the tab
    // This allows the property panel to be shown alongside the viewport
    if (!props.scenePanelOpen) {
      props.onScenePanelToggle();
    }
    props.onToolSelect(tool.id);
  };


  return (
    <div class="relative w-10 h-full bg-base-300 border-l border-r border-black/15 flex flex-col pointer-events-auto no-select">
      {/* Panel toggle button */}
      <div class="flex-shrink-0 w-full py-1">
        <button
          onClick={() => props.onScenePanelToggle()}
          class="btn btn-ghost h-7 w-full min-h-0 p-0 rounded-none transition-all duration-200 group relative select-none flex items-center justify-center border-none text-base-content/60 hover:bg-base-300 hover:text-base-content"
          title={props.isCollapsed ? "Expand panel" : "Collapse panel"}
        >
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="w-4 h-4">
            {props.isCollapsed ? (
              <path d="m15 18-6-6 6-6"/>
            ) : (
              <path d="m9 18 6-6-6-6"/>
            )}
          </svg>

          <div class={`absolute ${shouldTooltipGoRight() ? 'left-full ml-1' : 'right-full mr-1'} top-1/2 -translate-y-1/2 bg-base-300/95 backdrop-blur-sm border border-base-300 text-base-content text-[11px] px-2 py-1 rounded-md opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none whitespace-nowrap shadow-2xl`}
               style={{ 'z-index': 999999 }}>
            {props.isCollapsed ? "Expand panel" : "Collapse panel"}
            <div class={`absolute ${shouldTooltipGoRight() ? 'right-full' : 'left-full'} top-1/2 -translate-y-1/2 w-0 h-0 ${shouldTooltipGoRight() ? 'border-r-4 border-r-base-300' : 'border-l-4 border-l-base-300'} border-t-4 border-t-transparent border-b-4 border-b-transparent`}></div>
          </div>
        </button>
      </div>

      <div class="flex-1 overflow-hidden h-full flex flex-col gap-0.5 py-1">
        <For each={tools()}>
          {(tool) => (
            <button
              onClick={() => handleToolClick(tool)}
              class={`btn btn-ghost h-7 w-full min-h-0 p-0 rounded-none transition-all duration-200 group relative select-none flex items-center justify-center border-none ${
                props.selectedTool === tool.id
                  ? 'bg-primary/20 text-primary'
                  : 'text-base-content/60 hover:bg-base-300 hover:text-base-content'
              }`}
              title={tool.title}
            >
              <tool.icon class="w-4 h-4" />

              <div class={`absolute ${shouldTooltipGoRight() ? 'left-full ml-1' : 'right-full mr-1'} top-1/2 -translate-y-1/2 bg-base-300/95 backdrop-blur-sm border border-base-300 text-base-content text-[11px] px-2 py-1 rounded-md opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none whitespace-nowrap shadow-2xl`}
                   style={{ 'z-index': 999999 }}>
                {tool.title}
                <div class={`absolute ${shouldTooltipGoRight() ? 'right-full' : 'left-full'} top-1/2 -translate-y-1/2 w-0 h-0 ${shouldTooltipGoRight() ? 'border-r-4 border-r-base-300' : 'border-l-4 border-l-base-300'} border-t-4 border-t-transparent border-b-4 border-b-transparent`}></div>
              </div>
            </button>
          )}
        </For>
      </div>
    </div>
  );
}

export default TabMenu;
