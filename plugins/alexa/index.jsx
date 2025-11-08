import { createPlugin } from '@/api/plugin';
import { IconBrandAmazon } from '@tabler/icons-solidjs';
import AlexaViewport from './AlexaViewport.jsx';

export default createPlugin({
  id: 'twitch-alexa-plugin',
  name: 'Twitch Alexa Control',
  version: '1.0.0',
  description: 'Control OBS scenes and stream settings with Amazon Alexa voice commands',
  author: 'WebArcade Team',

  async onStart(api) {
    console.log('[Twitch Alexa Plugin] Starting...');

    // Register Alexa viewport
    api.viewport('twitch-alexa', {
      label: 'Alexa Control',
      component: AlexaViewport,
      icon: IconBrandAmazon,
      description: 'Control OBS scenes and stream settings with Amazon Alexa voice commands'
    });

    console.log('[Twitch Alexa Plugin] Started successfully');
  },

  async onStop() {
    console.log('[Twitch Alexa Plugin] Stopping...');
  }
});
