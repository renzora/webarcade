import { viewportStore } from "./store";
import { viewportTypes } from "@/api/plugin";
import { For, Show } from 'solid-js';

const Viewport = () => {
  const renderViewportPanel = (tab) => {
    if (!tab) return null;

    // Check if this is a plugin viewport type
    const pluginViewportType = viewportTypes().get(tab.type);
    if (pluginViewportType && pluginViewportType.component) {
      const PluginComponent = pluginViewportType.component;

      // Keep mounted but hide when inactive - use Show to make it reactive
      return (
        <div
          className="absolute inset-0 bg-base-100"
          style={{ display: viewportStore.activeTabId === tab.id ? 'block' : 'none' }}
        >
          <PluginComponent tab={tab} />
        </div>
      );
    }

    return (
      <div
        className="absolute inset-0 bg-base-100 flex items-center justify-center"
        style={{ display: viewportStore.activeTabId === tab.id ? 'flex' : 'none' }}
      >
        <div className="text-center">
          <div className="text-lg text-base-content/60 mb-2">Unknown Viewport</div>
          <div className="text-sm text-base-content/50">Viewport type "{tab.type}" not found</div>
        </div>
      </div>
    );
  };

  return (
    <div
      class="relative w-full h-full pointer-events-auto viewport-container"
    >
      <div className="w-full h-full flex flex-col gap-0">
        <div className="flex-1 relative overflow-hidden">
          <div className="w-full bg-base-100 h-full overflow-hidden">
            {/* Render all viewports, hide inactive ones */}
            <For each={viewportStore.tabs}>
              {(tab) => renderViewportPanel(tab)}
            </For>
          </div>
        </div>
      </div>
    </div>
  );
};

export default Viewport;