import { createPlugin } from '@/api/plugin';
import { IconBrandYoutube } from '@tabler/icons-solidjs';
import YouTubeViewport from './YouTubeViewport.jsx';
import YouTubeChannelsPanel from './YouTubeChannelsPanel.jsx';
import youtubeStore from './YouTubeStore.jsx';

export default createPlugin({
  id: 'youtube-plugin',
  name: 'YouTube Integration',
  version: '1.0.0',
  description: 'YouTube integration with channel analytics and OAuth authentication',
  author: 'WebArcade Team',

  async onInit() {
    console.log('[YouTube Plugin] Initializing...');
    // Initialize store and check auth status
    await youtubeStore.checkAuthStatus();
  },

  async onStart(api) {
    console.log('[YouTube Plugin] Starting...');

    // Register Channels as right panel tab
    api.tab('youtube-channels', {
      title: 'YouTube',
      component: YouTubeChannelsPanel,
      icon: IconBrandYoutube,
      order: 10
    });

    // Register unified YouTube viewport
    api.viewport('youtube', {
      label: 'YouTube',
      component: YouTubeViewport,
      icon: IconBrandYoutube,
      description: 'View YouTube channel analytics and manage settings'
    });

    console.log('[YouTube Plugin] Started successfully');
  },

  onUpdate() {
    // Update logic if needed
  },

  async onStop() {
    console.log('[YouTube Plugin] Stopping...');
  },

  async onDispose() {
    console.log('[YouTube Plugin] Disposing...');
  }
});
