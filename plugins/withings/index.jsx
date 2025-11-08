import { createPlugin } from '@/api/plugin';
import { IconScale } from '@tabler/icons-solidjs';
import WithingsViewport from './WithingsViewport.jsx';

export default createPlugin({
  id: 'twitch-withings-plugin',
  name: 'Twitch Withings Health',
  version: '1.0.0',
  description: 'Track weight and health metrics from Withings scale',
  author: 'WebArcade Team',

  async onStart(api) {
    console.log('[Twitch Withings Plugin] Starting...');

    // Register Withings viewport
    api.viewport('twitch-withings', {
      label: 'Withings Health',
      component: WithingsViewport,
      icon: IconScale,
      description: 'Track weight and health metrics from Withings scale'
    });

    console.log('[Twitch Withings Plugin] Started successfully');
  },

  async onStop() {
    console.log('[Twitch Withings Plugin] Stopping...');
  }
});
