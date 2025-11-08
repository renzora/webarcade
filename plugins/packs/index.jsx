import { createPlugin } from '@/api/plugin';
import { IconPackage } from '@tabler/icons-solidjs';
import PacksViewport from './PacksViewport.jsx';

export default createPlugin({
  id: 'packs-plugin',
  name: 'Packs System',
  version: '1.0.0',
  description: 'Create packs with collectible items of varying rarities',
  author: 'WebArcade Team',

  async onStart(api) {
    console.log('[Packs Plugin] Starting...');

    // Register Packs viewport
    api.viewport('packs-manager', {
      label: 'Packs',
      component: PacksViewport,
      icon: IconPackage,
      description: 'Manage packs and items'
    });

    console.log('[Packs Plugin] Started successfully');
  },

  async onStop() {
    console.log('[Packs Plugin] Stopping...');
  }
});
