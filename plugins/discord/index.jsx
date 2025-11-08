import { createPlugin } from '@/api/plugin';
import { IconBrandDiscord } from '@tabler/icons-solidjs';
import DiscordViewport from './DiscordViewport.jsx';

export default createPlugin({
  id: 'discord',
  name: 'Discord Bot',
  version: '1.0.0',
  description: 'General-purpose Discord bot with custom commands',
  author: 'WebArcade Team',

  async onStart(api) {
    console.log('[Discord Bot] Starting...');

    // Register Discord Bot viewport
    api.viewport('discord-bot', {
      label: 'Discord Bot',
      component: DiscordViewport,
      icon: IconBrandDiscord,
      description: 'Configure and manage your Discord bot'
    });

    console.log('[Discord Bot] Started successfully');
  },

  async onStop() {
    console.log('[Discord Bot] Stopping...');
  }
});
