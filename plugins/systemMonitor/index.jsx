import { createPlugin } from '@/api/plugin';
import { IconActivity } from '@tabler/icons-solidjs';
import SystemMonitorFooterButton from './SystemMonitorFooterButton.jsx';

export default createPlugin({
  id: 'systemMonitor',
  name: 'System Monitor',
  version: '1.0.0',
  description: 'Displays CPU and RAM usage in the footer',
  author: 'WebArcade Team',
  icon: IconActivity,

  async onStart(api) {
    console.log('[System Monitor Plugin] Starting...');

    // Register system monitor button in footer
    api.footer('systemMonitor', {
      component: SystemMonitorFooterButton,
      order: 1
    });

    console.log('[System Monitor Plugin] Started successfully');
  },

  async onStop() {
    console.log('[System Monitor Plugin] Stopping...');
  }
});
