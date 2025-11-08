import { createPlugin } from '@/api/plugin';
import { IconSnowflake } from '@tabler/icons-solidjs';
import SnowViewport from './SnowViewport.jsx';
import SnowPanel from './SnowPanel.jsx';

console.log('[Snow Plugin] Module loaded');

export default createPlugin({
  id: 'snow',
  name: 'Snow Overlay',
  version: '1.0.0',
  description: 'Falling snow overlay effect with customizable settings',
  author: 'WebArcade Team',

  async onInit() {
    console.log('[Snow Plugin] Initializing...');
  },

  async onStart(api) {
    console.log('[Snow Plugin] Starting...');

    // Register snow viewport
    api.viewport('snow-viewport', {
      label: 'Snow',
      component: SnowViewport,
      icon: IconSnowflake,
      description: 'Preview and manage snow overlay'
    });
    console.log('[Snow Plugin] Viewport registered');

    // Register snow control panel
    api.tab('snow-panel', {
      title: 'Snow',
      component: SnowPanel,
      icon: IconSnowflake,
      order: 12
    });
    console.log('[Snow Plugin] Panel registered');
    console.log('[Snow Plugin] Started successfully');
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
