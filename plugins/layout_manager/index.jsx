import { createPlugin } from '@/api/plugin';
import { IconList } from '@tabler/icons-solidjs';
import LayoutManagerViewport from './LayoutManagerViewport.jsx';
import LayoutManagerPanel from './LayoutManagerPanel.jsx';

export default createPlugin({
  id: 'twitch-layout-manager-plugin',
  name: 'Twitch Layout Manager',
  version: '1.0.0',
  description: 'Arrange multiple overlays in a visual layout for OBS',
  author: 'WebArcade Team',

  async onStart(api) {
    console.log('[Twitch Layout Manager Plugin] Starting...');

    // Import viewport store to check active viewport
    const { viewportStore } = await import('@/panels/viewport/store');

    // Register Layout Manager panel (only visible when Layout Manager viewport is active)
    api.tab('layout-manager-panel', {
      title: 'Layouts',
      component: LayoutManagerPanel,
      icon: IconList,
      order: 5,
      condition: () => {
        const activeTab = viewportStore.tabs.find(tab => tab.id === viewportStore.activeTabId);
        return activeTab?.type === 'layout-manager';
      }
    });

    // Layout Manager viewport
    api.viewport('layout-manager', {
      label: 'Layout Manager',
      component: LayoutManagerViewport,
      icon: IconList,
      description: 'Arrange multiple overlays in a visual layout for OBS'
    });

    console.log('[Twitch Layout Manager Plugin] Started successfully');
  },

  async onStop() {
    console.log('[Twitch Layout Manager Plugin] Stopping...');
  }
});
