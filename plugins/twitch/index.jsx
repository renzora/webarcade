import { createPlugin } from '@/api/plugin';
import { IconBrandTwitch } from '@tabler/icons-solidjs';

export default createPlugin({
  id: 'twitch-plugin',
  name: 'Twitch Integration',
  version: '1.0.0',
  description: 'Twitch IRC chat and EventSub integration - use Twitch Setup plugin to configure',
  author: 'WebArcade Team',
  icon: IconBrandTwitch,

  async onStart(api) {
    console.log('[Twitch Plugin] Starting...');

    // Widgets are auto-loaded from ./widgets/ directory
    // - TwitchChat.jsx - IRC chat viewer and sender
    // - TwitchEvents.jsx - EventSub event viewer
    //
    // Note: Use the "Twitch Setup" plugin to configure credentials and connect accounts

    console.log('[Twitch Plugin] Started successfully');
  },

  async onStop() {
    console.log('[Twitch Plugin] Stopping...');
  }
});
