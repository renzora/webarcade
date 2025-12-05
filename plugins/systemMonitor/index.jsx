import { plugin } from '@/api/plugin';
import { IconActivity } from '@tabler/icons-solidjs';
import SystemMonitorFooterButton from './SystemMonitorFooterButton.jsx';

export default plugin({
  id: 'systemMonitor',
  name: 'System Monitor',
  version: '1.0.0',
  description: 'Displays CPU and RAM usage in the footer',
  author: 'WebArcade Team',
  icon: IconActivity,

  start(api) {
    console.log('[System Monitor Plugin] Starting...');

    // Register system monitor button in footer
    api.footer('systemMonitor', {
      component: SystemMonitorFooterButton,
      order: 1
    });

    console.log('[System Monitor Plugin] Started successfully');
  },

  stop(api) {
    console.log('[System Monitor Plugin] Stopping...');
  }
});
