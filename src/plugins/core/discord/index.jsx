import { createPlugin } from '@/api/plugin';
import { IconBrandDiscord, IconRobot } from '@tabler/icons-solidjs';
import DiscordViewport from './DiscordViewport.jsx';
import DiscordCommandsViewport from './DiscordCommandsViewport.jsx';

export default createPlugin({
  id: 'twitch-discord-plugin',
  name: 'Twitch Discord Integration',
  version: '1.0.0',
  description: 'Configure Discord bot for song requests and custom commands',
  author: 'WebArcade Team',

  async onStart(api) {
    console.log('[Twitch Discord Plugin] Starting...');

    // Register Discord viewport
    api.viewport('twitch-discord', {
      label: 'Discord Bot',
      component: DiscordViewport,
      icon: IconBrandDiscord,
      description: 'Configure Discord bot for song requests'
    });

    // Register Discord Commands viewport
    api.viewport('twitch-discord-commands', {
      label: 'Discord Commands',
      component: DiscordCommandsViewport,
      icon: IconRobot,
      description: 'Manage custom Discord bot commands'
    });

    console.log('[Twitch Discord Plugin] Started successfully');
  },

  async onStop() {
    console.log('[Twitch Discord Plugin] Stopping...');
  }
});
