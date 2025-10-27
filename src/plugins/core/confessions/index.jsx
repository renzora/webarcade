import { createPlugin } from '@/api/plugin';
import { IconMessageCircle } from '@tabler/icons-solidjs';
import ConfessionsViewport from './ConfessionsViewport.jsx';

export default createPlugin({
  id: 'twitch-confessions-plugin',
  name: 'Twitch Confessions',
  version: '1.0.0',
  description: 'View anonymous confessions sent via whispers',
  author: 'WebArcade Team',

  async onStart(api) {
    console.log('[Twitch Confessions Plugin] Starting...');

    // Register Confessions viewport
    api.viewport('twitch-confessions', {
      label: 'Confessions',
      component: ConfessionsViewport,
      icon: IconMessageCircle,
      description: 'View anonymous confessions sent via whispers'
    });

    console.log('[Twitch Confessions Plugin] Started successfully');
  },

  async onStop() {
    console.log('[Twitch Confessions Plugin] Stopping...');
  }
});
