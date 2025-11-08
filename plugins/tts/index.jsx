import { createPlugin } from '@/api/plugin';
import { IconVolume } from '@tabler/icons-solidjs';
import TTSWhitelistViewport from './TTSWhitelistViewport.jsx';

export default createPlugin({
  id: 'twitch-tts-plugin',
  name: 'Twitch TTS Settings',
  version: '1.0.0',
  description: 'Configure text-to-speech settings and whitelist',
  author: 'WebArcade Team',

  async onStart(api) {
    console.log('[Twitch TTS Plugin] Starting...');

    // Register TTS Whitelist viewport
    api.viewport('twitch-tts', {
      label: 'TTS Settings',
      component: TTSWhitelistViewport,
      icon: IconVolume,
      description: 'Configure text-to-speech settings and whitelist'
    });

    console.log('[Twitch TTS Plugin] Started successfully');
  },

  async onStop() {
    console.log('[Twitch TTS Plugin] Stopping...');
  }
});
