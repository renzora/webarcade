import { createPlugin } from '@/api/plugin';
import { IconAlertCircle } from '@tabler/icons-solidjs';
import AlertsViewport from './AlertsViewport.jsx';

export default createPlugin({
  id: 'twitch-alerts-plugin',
  name: 'Twitch Stream Alerts',
  version: '1.0.0',
  description: 'Test and configure stream alerts with 3D Babylon.js animations',
  author: 'WebArcade Team',

  async onStart(api) {
    console.log('[Twitch Alerts Plugin] Starting...');

    // Alerts Overlay viewport
    api.viewport('alerts-overlay', {
      label: 'Stream Alerts',
      component: AlertsViewport,
      icon: IconAlertCircle,
      description: 'Test and configure stream alerts with 3D Babylon.js animations'
    });

    console.log('[Twitch Alerts Plugin] Started successfully');
  },

  async onStop() {
    console.log('[Twitch Alerts Plugin] Stopping...');
  }
});
