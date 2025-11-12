import { createPlugin } from '@/api/plugin';
import { IconSettings } from '@tabler/icons-solidjs';
import SettingsPanel from './SettingsPanel.jsx';

export default createPlugin({
  id: 'settings-plugin',
  name: 'Settings',
  version: '1.0.0',
  description: 'Application settings and preferences',
  author: 'WebArcade Team',
  icon: IconSettings,

  async onStart(api) {
    console.log('[Settings Plugin] Starting...');

    // Register Settings panel in right panel
    api.tab('settings-panel', {
      title: 'Settings',
      component: SettingsPanel,
      icon: IconSettings,
      order: 999 // Put at the end
    });

    // Widgets are auto-loaded from ./widgets/ directory

    console.log('[Settings Plugin] Started successfully');
  },

  async onStop() {
    console.log('[Settings Plugin] Stopping...');
  }
});
