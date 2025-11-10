import { createPlugin } from '@/api/plugin';
import { IconMessageCircle } from '@tabler/icons-solidjs';

export default createPlugin({
  id: 'twitch-chat-plugin',
  name: 'Twitch Chat',
  version: '1.0.0',
  description: 'Display Twitch chat messages in real-time',
  author: 'WebArcade Team',
  icon: IconMessageCircle,

  async onStart(api) {
    console.log('[Twitch Chat Plugin] Starting...');
    console.log('[Twitch Chat Plugin] Started successfully');
  },

  async onStop() {
    console.log('[Twitch Chat Plugin] Stopping...');
  }
});
