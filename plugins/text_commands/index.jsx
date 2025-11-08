import { createPlugin } from '@/api/plugin';
import { IconTerminal2 } from '@tabler/icons-solidjs';
import TextCommandsViewport from './TextCommandsViewport.jsx';

export default createPlugin({
  id: 'twitch-text-commands-plugin',
  name: 'Twitch Text Commands',
  version: '1.0.0',
  description: 'Create custom text commands with dynamic variables',
  author: 'WebArcade Team',

  async onStart(api) {
    console.log('[Twitch Text Commands Plugin] Starting...');

    // Register Text Commands viewport
    api.viewport('twitch-text-commands', {
      label: 'Text Commands',
      component: TextCommandsViewport,
      icon: IconTerminal2,
      description: 'Create custom text commands with dynamic variables'
    });

    console.log('[Twitch Text Commands Plugin] Started successfully');
  },

  async onStop() {
    console.log('[Twitch Text Commands Plugin] Stopping...');
  }
});
