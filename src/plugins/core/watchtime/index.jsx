import { createPlugin } from '@/api/plugin';
import { IconClock } from '@tabler/icons-solidjs';
import WatchtimeViewport from './WatchtimeViewport.jsx';

export default createPlugin({
  id: 'twitch-watchtime-plugin',
  name: 'Twitch Watchtime',
  version: '1.0.0',
  description: 'View and search viewer watchtime with pagination',
  author: 'WebArcade Team',

  async onStart(api) {
    console.log('[Twitch Watchtime Plugin] Starting...');

    // Register Watchtime viewport
    api.viewport('twitch-watchtime', {
      label: 'Watchtime',
      component: WatchtimeViewport,
      icon: IconClock,
      description: 'View and search viewer watchtime with pagination'
    });

    console.log('[Twitch Watchtime Plugin] Started successfully');
  },

  async onStop() {
    console.log('[Twitch Watchtime Plugin] Stopping...');
  }
});
