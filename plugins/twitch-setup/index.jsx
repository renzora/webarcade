import { createPlugin } from '@/api/plugin';
import { IconBrandTwitch } from '@tabler/icons-solidjs';
import TwitchSetupViewport from './Viewport.jsx';
import TwitchSetupPanel from './Panel.jsx';

export default createPlugin({
  id: 'twitch-setup',
  name: 'Twitch Setup',
  version: '1.0.0',
  description: 'Configure Twitch integration - manage app credentials and account connections',
  author: 'WebArcade Team',
  icon: IconBrandTwitch,

  async onStart(api) {
    console.log('[Twitch Setup] Starting plugin...');

    // Register viewport
    api.viewport('twitch-setup', {
      label: 'Twitch Setup',
      component: TwitchSetupViewport,
      icon: IconBrandTwitch,
      description: 'Configure Twitch app credentials and connect accounts'
    });

    // Register panel
    api.tab('twitch-setup-panel', {
      title: 'Twitch Setup',
      component: TwitchSetupPanel,
      icon: IconBrandTwitch,
      order: 100,
      viewport: 'twitch-setup'
    });

    // Show UI elements
    api.showProps(true);
    api.showTabs(true);

    console.log('[Twitch Setup] Plugin started successfully');
  },

  async onStop() {
    console.log('[Twitch Setup] Stopping plugin...');
  }
});
