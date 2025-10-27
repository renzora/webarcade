import { createPlugin } from '@/api/plugin';
import { IconDeviceTv } from '@tabler/icons-solidjs';
import OverlayManagerViewport from './OverlayManagerViewport.jsx';
import OverlayManagerPanel from './OverlayManagerPanel.jsx';

export default createPlugin({
  id: 'twitch-overlay-manager-plugin',
  name: 'Twitch Overlay Manager',
  version: '1.0.0',
  description: 'Create and manage OBS browser source overlays',
  author: 'WebArcade Team',

  async onStart(api) {
    console.log('[Twitch Overlay Manager Plugin] Starting...');

    // Import viewport store to check active viewport
    const { viewportStore } = await import('@/panels/viewport/store');

    // Register Overlay Manager panel (only visible when Overlay Manager viewport is active)
    api.tab('overlay-manager-panel', {
      title: 'Overlays',
      component: OverlayManagerPanel,
      icon: IconDeviceTv,
      order: 6,
      condition: () => {
        const activeTab = viewportStore.tabs.find(tab => tab.id === viewportStore.activeTabId);
        return activeTab?.type === 'overlay-manager';
      }
    });

    // Overlay Manager viewport
    api.viewport('overlay-manager', {
      label: 'Overlay Manager',
      component: OverlayManagerViewport,
      icon: IconDeviceTv,
      description: 'Create and manage OBS browser source overlays'
    });

    console.log('[Twitch Overlay Manager Plugin] Started successfully');
  },

  async onStop() {
    console.log('[Twitch Overlay Manager Plugin] Stopping...');
  }
});
