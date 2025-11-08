import { createPlugin } from '@/api/plugin';
import { IconWheel } from '@tabler/icons-solidjs';
import RouletteViewport from './RouletteViewport.jsx';

export default createPlugin({
  id: 'roulette-plugin',
  name: 'Roulette',
  version: '1.0.0',
  description: 'Multiplayer roulette wheel game with coin betting',
  author: 'WebArcade Team',

  async onStart(api) {
    console.log('[Roulette Plugin] Starting...');

    // Register Roulette viewport
    api.viewport('roulette-manager', {
      label: 'Roulette',
      component: RouletteViewport,
      icon: IconWheel,
      description: 'Manage roulette games'
    });

    console.log('[Roulette Plugin] Started successfully');
  },

  async onStop() {
    console.log('[Roulette Plugin] Stopping...');
  }
});
