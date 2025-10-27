import { createPlugin } from '@/api/plugin';
import { IconMusic } from '@tabler/icons-solidjs';
import SongRequestsViewport from './SongRequestsViewport.jsx';

export default createPlugin({
  id: 'twitch-song-requests-plugin',
  name: 'Twitch Song Requests',
  version: '1.0.0',
  description: 'Manage Discord song request queue for YouTube Music',
  author: 'WebArcade Team',

  async onStart(api) {
    console.log('[Twitch Song Requests Plugin] Starting...');

    // Register Song Requests viewport
    api.viewport('twitch-song-requests', {
      label: 'Song Requests',
      component: SongRequestsViewport,
      icon: IconMusic,
      description: 'Manage Discord song request queue for YouTube Music'
    });

    console.log('[Twitch Song Requests Plugin] Started successfully');
  },

  async onStop() {
    console.log('[Twitch Song Requests Plugin] Stopping...');
  }
});
