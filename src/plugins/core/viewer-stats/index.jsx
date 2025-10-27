import { createPlugin } from '@/api/plugin';
import { IconUsers } from '@tabler/icons-solidjs';
import ViewerStatsViewport from './ViewerStatsViewport.jsx';

export default createPlugin({
  id: 'twitch-viewer-stats-plugin',
  name: 'Twitch Viewer Stats',
  version: '1.0.0',
  description: 'View active viewers by day, week, or month',
  author: 'WebArcade Team',

  async onStart(api) {
    console.log('[Twitch Viewer Stats Plugin] Starting...');

    // Register Viewer Stats viewport
    api.viewport('twitch-viewer-stats', {
      label: 'Viewer Stats',
      component: ViewerStatsViewport,
      icon: IconUsers,
      description: 'View active viewers by day, week, or month'
    });

    console.log('[Twitch Viewer Stats Plugin] Started successfully');
  },

  async onStop() {
    console.log('[Twitch Viewer Stats Plugin] Stopping...');
  }
});
