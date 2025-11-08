import { createPlugin } from '@/api/plugin';
import { IconSparkles } from '@tabler/icons-solidjs';
import EmojiWallViewport from './EmojiWallViewport.jsx';
import EmojiWallPanel from './EmojiWallPanel.jsx';

console.log('[Emoji Wall Plugin] Module loaded');

export default createPlugin({
  id: 'emoji-wall',
  name: 'Emote Wall',
  version: '1.0.0',
  description: 'Classic Twitch emote wall overlay with bouncing animated emotes',
  author: 'WebArcade Team',

  async onInit() {
    console.log('[Emoji Wall Plugin] Initializing...');
  },

  async onStart(api) {
    console.log('[Emoji Wall Plugin] Starting...');

    // Register emote wall viewport
    api.viewport('emoji-wall-viewport', {
      label: 'Emote Wall',
      component: EmojiWallViewport,
      icon: IconSparkles,
      description: 'Preview and manage Twitch emote wall overlay'
    });
    console.log('[Emoji Wall Plugin] Viewport registered');

    // Register emote wall control panel
    api.tab('emoji-wall-panel', {
      title: 'Emote Wall',
      component: EmojiWallPanel,
      icon: IconSparkles,
      order: 11
    });
    console.log('[Emoji Wall Plugin] Panel registered');
    console.log('[Emoji Wall Plugin] Started successfully');
  },

  onUpdate() {
    // Update logic if needed
  },

  async onStop() {
    // Cleanup
  },

  async onDispose() {
    // Cleanup
  }
});
