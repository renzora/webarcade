import { createPlugin } from '@/api/plugin';
import { IconDeviceGamepad2 } from '@tabler/icons-solidjs';
import SnakeGameViewport from './viewport.jsx';
import SnakeGamePanel from './panel.jsx';

export default createPlugin({
  id: 'webarcade-snake-game-plugin',
  name: 'Snake Game',
  version: '1.0.0',
  description: 'Classic snake game - eat food, grow longer, avoid walls and yourself!',
  author: 'WebArcade Team',
  icon: IconDeviceGamepad2,

  async onStart(api) {
    console.log('[Webarcade Snake Game Plugin] Starting...');

    api.viewport('webarcade-snake-game', {
      label: 'Snake Game',
      component: SnakeGameViewport,
      icon: IconDeviceGamepad2,
      description: 'Play the classic snake game'
    });

    api.tab('webarcade-snake-game-menu', {
      title: 'Snake',
      component: SnakeGamePanel,
      icon: IconDeviceGamepad2,
      order: 60,
      viewport: 'webarcade-snake-game'
    });

    console.log('[Webarcade Snake Game Plugin] Started successfully');
  },

  async onStop() {
    console.log('[Webarcade Snake Game Plugin] Stopping...');
  }
});
