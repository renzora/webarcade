import { createPlugin } from '@/api/plugin';
import { IconMusic } from '@tabler/icons-solidjs';
import SongRequestsViewport from './SongRequestsViewport.jsx';

export default createPlugin({
  id: 'song_requests-plugin',
  name: 'Song Requests',
  version: '1.0.0',
  description: 'Manage song requests queue for streams',
  author: 'WebArcade Team',

  async onStart(api) {
    console.log('[Song Requests Plugin] Starting...');

    // Register Song Requests viewport
    api.viewport('song_requests', {
      label: 'Song Requests',
      component: SongRequestsViewport,
      icon: IconMusic,
      description: 'Manage song requests queue for streams'
    });

    console.log('[Song Requests Plugin] Started successfully');
  },

  async onStop() {
    console.log('[Song Requests Plugin] Stopping...');
  }
});
