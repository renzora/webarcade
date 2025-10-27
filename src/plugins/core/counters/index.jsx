import { createPlugin } from '@/api/plugin';
import { IconList } from '@tabler/icons-solidjs';
import CountersViewport from './CountersViewport.jsx';

export default createPlugin({
  id: 'twitch-counters-plugin',
  name: 'Twitch Stream Counters',
  version: '1.0.0',
  description: 'Manage and track stream counters',
  author: 'WebArcade Team',

  async onStart(api) {
    console.log('[Twitch Counters Plugin] Starting...');

    // Register Counters viewport
    api.viewport('twitch-counters', {
      label: 'Stream Counters',
      component: CountersViewport,
      icon: IconList,
      description: 'Manage and track stream counters'
    });

    console.log('[Twitch Counters Plugin] Started successfully');
  },

  async onStop() {
    console.log('[Twitch Counters Plugin] Stopping...');
  }
});
