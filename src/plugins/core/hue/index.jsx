import { createPlugin } from '@/api/plugin';
import { IconBulb, IconSparkles } from '@tabler/icons-solidjs';
import HueViewport from './HueViewport.jsx';
import HueScenesViewport from './HueScenesViewport.jsx';
import HuePanel from './HuePanel.jsx';

export default createPlugin({
  id: 'twitch-hue-plugin',
  name: 'Twitch Hue Integration',
  version: '1.0.0',
  description: 'Control Philips Hue smart lights from Twitch',
  author: 'WebArcade Team',

  async onStart(api) {
    console.log('[Twitch Hue Plugin] Starting...');

    // Import viewport store to check active viewport
    const { viewportStore } = await import('@/panels/viewport/store');

    // Register Hue quick control panel (only visible when Hue viewport is active)
    api.tab('hue-control', {
      title: 'Hue',
      component: HuePanel,
      icon: IconBulb,
      order: 3,
      condition: () => {
        const activeTab = viewportStore.tabs.find(tab => tab.id === viewportStore.activeTabId);
        return activeTab?.type === 'twitch-hue';
      }
    });

    // Register Hue viewport
    api.viewport('twitch-hue', {
      label: 'Hue Lights',
      component: HueViewport,
      icon: IconBulb,
      description: 'Control Philips Hue smart lights'
    });

    // Register Hue Scenes viewport
    api.viewport('twitch-hue-scenes', {
      label: 'Hue Scenes',
      component: HueScenesViewport,
      icon: IconSparkles,
      description: 'Create animated multi-color light sequences'
    });

    console.log('[Twitch Hue Plugin] Started successfully');
  },

  async onStop() {
    console.log('[Twitch Hue Plugin] Stopping...');
  }
});
