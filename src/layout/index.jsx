import { createSignal, onMount, onCleanup, Show } from 'solid-js';
import TopMenu from '@/panels/topMenu';
import Toolbar from '@/panels/toolbar';
import Footer from '@/panels/footer';
import Panel, { LeftPanel, RightPanel, BottomPanel, ViewportPanel } from '@/panels/Panel';
import PluginTabs from '@/panels/PluginTabs';
import { ViewportContextMenuProvider } from '@/ui/ViewportContextMenu.jsx';
import { keyboardShortcuts } from '@/components/KeyboardShortcuts';
import { panelVisibility } from '@/api/plugin/panels';
import { footerVisible, horizontalMenuButtonsEnabled, bottomPanelVisible, pluginTabsVisible } from '@/api/plugin';

const Layout = () => {
  const [globalTooltip, setGlobalTooltip] = createSignal(null);
  const [contextMenuAPI, setContextMenuAPI] = createSignal(null);

  onMount(() => {
    // Listen for global tooltip events
    const handleTooltipShow = (e) => setGlobalTooltip(e.detail);
    const handleTooltipHide = () => setGlobalTooltip(null);

    document.addEventListener('global:tooltip-show', handleTooltipShow);
    document.addEventListener('global:tooltip-hide', handleTooltipHide);

    // Setup context menu shortcuts once the context menu API is available
    setTimeout(() => {
      const api = contextMenuAPI();
      if (api && api.showContextMenu) {
        keyboardShortcuts.enableContextMenu(api.showContextMenu);
      }
    }, 100);

    onCleanup(() => {
      document.removeEventListener('global:tooltip-show', handleTooltipShow);
      document.removeEventListener('global:tooltip-hide', handleTooltipHide);
      keyboardShortcuts.disableContextMenu();
    });
  });

  return (
    <ViewportContextMenuProvider onAPIReady={setContextMenuAPI}>
      <div
        class="fixed bg-base-100 inset-0 flex flex-col pointer-events-none z-10 transition-opacity duration-300"
        onContextMenu={(e) => e.preventDefault()}
      >
        {/* Top Menu */}
        <Show when={horizontalMenuButtonsEnabled()}>
          <div class="flex-shrink-0 pointer-events-auto z-50">
            <TopMenu />
          </div>
        </Show>

        {/* Toolbar */}
        <div class="pointer-events-auto">
          <Toolbar />
        </div>

        {/* Plugin Tabs */}
        <Show when={pluginTabsVisible()}>
          <div class="pointer-events-auto">
            <PluginTabs />
          </div>
        </Show>

        {/* Main Content Area */}
        <div
          class="flex-1 flex flex-col overflow-hidden pointer-events-auto"
          style={{
            'padding-bottom': footerVisible() ? '24px' : '0px'
          }}
        >
          {/* Horizontal layout: Left | Viewport | Right */}
          <div class="flex-1 flex overflow-hidden">
            {/* Left Panel */}
            <Show when={panelVisibility.left}>
              <LeftPanel />
            </Show>

            {/* Main Viewport */}
            <div class="flex-1 relative overflow-hidden min-w-0">
              <ViewportPanel />
            </div>

            {/* Right Panel */}
            <Show when={panelVisibility.right}>
              <RightPanel />
            </Show>
          </div>

          {/* Bottom Panel */}
          <Show when={bottomPanelVisible()}>
            <BottomPanel />
          </Show>
        </div>

        {/* Footer */}
        <Show when={footerVisible()}>
          <Footer />
        </Show>
      </div>

      {/* Global Tooltip */}
      <Show when={globalTooltip()}>
        <div class="fixed z-[99999] bg-black text-white text-xs p-3 pointer-events-none shadow-xl border border-gray-600 max-w-xs"
             style={`left: ${globalTooltip().x}px; top: ${globalTooltip().y}px;`}>
          <div class="font-semibold mb-2 text-white truncate">{globalTooltip().asset.name}</div>
          <div class="space-y-1 text-gray-300">
            <div class="truncate">Type: {globalTooltip().asset.extension?.toUpperCase() || 'Unknown'}</div>
            <div class="truncate">Size: {globalTooltip().asset.size ? `${Math.round(globalTooltip().asset.size / 1024)} KB` : 'Unknown'}</div>
            <Show when={globalTooltip().asset.path}>
              <div class="text-gray-400 truncate">Path: {globalTooltip().asset.path}</div>
            </Show>
          </div>
        </div>
      </Show>
    </ViewportContextMenuProvider>
  );
};

export default Layout;
