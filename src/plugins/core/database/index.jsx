import { createPlugin } from '@/api/plugin';
import { IconDatabase } from '@tabler/icons-solidjs';
import DatabaseViewport from './DatabaseViewport.jsx';

export default createPlugin({
  id: 'twitch-database-plugin',
  name: 'Twitch Database Manager',
  version: '1.0.0',
  description: 'Execute SQL queries and manage the SQLite database',
  author: 'WebArcade Team',

  async onStart(api) {
    console.log('[Twitch Database Plugin] Starting...');

    // Register Database viewport
    api.viewport('twitch-database', {
      label: 'Database Manager',
      component: DatabaseViewport,
      icon: IconDatabase,
      description: 'Execute SQL queries and manage the SQLite database'
    });

    console.log('[Twitch Database Plugin] Started successfully');
  },

  async onStop() {
    console.log('[Twitch Database Plugin] Stopping...');
  }
});
