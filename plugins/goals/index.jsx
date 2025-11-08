import { createPlugin } from '@/api/plugin';
import { IconTarget } from '@tabler/icons-solidjs';
import GoalsViewport from './GoalsViewport.jsx';

export default createPlugin({
  id: 'twitch-goals-plugin',
  name: 'Twitch Goals Tracker',
  version: '1.0.0',
  description: 'Track and manage stream goals with Twitch integration',
  author: 'WebArcade Team',

  async onStart(api) {
    console.log('[Twitch Goals Plugin] Starting...');

    // Register Goals viewport
    api.viewport('twitch-goals', {
      label: 'Goals Tracker',
      component: GoalsViewport,
      icon: IconTarget,
      description: 'Track and manage stream goals with Twitch integration'
    });

    console.log('[Twitch Goals Plugin] Started successfully');
  },

  async onStop() {
    console.log('[Twitch Goals Plugin] Stopping...');
  }
});
