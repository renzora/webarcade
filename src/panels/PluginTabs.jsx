import { Show, For, createMemo } from 'solid-js';
import { Dynamic } from 'solid-js/web';
import panelStore, { panels, activePlugin, PANELS } from '@/api/plugin/panels';
import { pluginStore } from '@/api/plugin/store';
import { IconBox } from '@tabler/icons-solidjs';

/**
 * Plugin tabs - the main tab bar showing all registered plugins
 */
const PluginTabs = () => {
  // Get all plugin tabs
  const getTabs = createMemo(() => {
    const allPanels = panels();
    const tabs = [];

    allPanels.forEach((item) => {
      if (item.panel === PANELS.TAB) {
        tabs.push(item);
      }
    });

    return tabs.sort((a, b) => a.order - b.order);
  });

  // Handle tab click
  const handleTabClick = (tab) => {
    const currentActive = activePlugin();

    if (currentActive === tab.pluginId) return;

    // Get plugin instance and call lifecycle methods
    const pluginData = pluginStore.getPluginInstance(tab.pluginId);
    if (pluginData?.instance) {
      // Call onActive on new plugin
      if (typeof pluginData.instance.onActive === 'function') {
        pluginData.instance.onActive();
      }
    } else {
      // Fallback: just set active plugin directly
      panelStore.setActivePlugin(tab.pluginId);
    }
  };

  return (
    <Show when={getTabs().length > 0}>
      <div class="flex items-center gap-1 bg-base-200 border-b border-base-300 px-2 py-1">
        <For each={getTabs()}>
          {(tab) => (
            <button
              class={`flex items-center gap-2 px-3 py-1 text-sm font-medium transition-colors rounded ${
                activePlugin() === tab.pluginId
                  ? 'bg-primary text-primary-content'
                  : 'text-base-content/60 hover:text-base-content hover:bg-base-300'
              }`}
              onClick={() => handleTabClick(tab)}
              title={tab.label}
            >
              <Show
                when={tab.icon}
                fallback={<IconBox class="w-4 h-4" />}
              >
                <Dynamic component={tab.icon} class="w-4 h-4" />
              </Show>
              <span>{tab.label}</span>
            </button>
          )}
        </For>
      </div>
    </Show>
  );
};

export default PluginTabs;
