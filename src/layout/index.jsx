import { createSignal, onMount, onCleanup, Show } from 'solid-js';
import TopMenu from '@/panels/topMenu';
import Viewport from '@/panels/viewport';
import Footer from '@/panels/footer';
import RightPanel from '@/panels/rightPanel';
import LeftPanel from '@/panels/leftPanel';
import { ViewportContextMenuProvider } from '@/ui/ViewportContextMenu.jsx';
import { keyboardShortcuts } from '@/components/KeyboardShortcuts';
import { editorActions } from './stores/EditorStore.jsx';
import { propertiesPanelVisible, leftPanelVisible, footerVisible } from '@/api/plugin';

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
    <ViewportContextMenuProvider editorActions={editorActions} onAPIReady={setContextMenuAPI}>
      <div class="fixed inset-0 flex flex-col pointer-events-none z-10" onContextMenu={(e) => e.preventDefault()}>
        <div class="flex-shrink-0 pointer-events-auto z-50">
          <TopMenu />
        </div>


        <div
          class="flex-1 flex overflow-hidden pointer-events-auto"
          style={{
            'padding-bottom': footerVisible() ? '24px' : '0px'
          }}
        >
          <Show when={leftPanelVisible()}>
            <LeftPanel />
          </Show>
          <div class="flex-1 relative overflow-hidden min-w-0">
            <Viewport />
          </div>
          <Show when={propertiesPanelVisible()}>
            <RightPanel />
          </Show>
        </div>

        <Show when={footerVisible()}>
          <Footer />
        </Show>
      </div>

      {/* Global Tooltip - appears above everything */}
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