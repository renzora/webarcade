import { createPlugin } from '@/api/plugin';
import { IconWheel } from '@tabler/icons-solidjs';
import WheelViewport from './WheelViewport.jsx';

export default createPlugin({
  id: 'twitch-wheel-plugin',
  name: 'Twitch Spin Wheel',
  version: '1.0.0',
  description: 'Create and manage wheel spin options',
  author: 'WebArcade Team',

  async onStart(api) {
    console.log('[Twitch Wheel Plugin] Starting...');

    // Register Wheel viewport
    api.viewport('twitch-wheel', {
      label: 'Spin Wheel',
      component: WheelViewport,
      icon: IconWheel,
      description: 'Create and manage wheel spin options'
    });

    console.log('[Twitch Wheel Plugin] Started successfully');
  },

  async onStop() {
    console.log('[Twitch Wheel Plugin] Stopping...');
  }
});
