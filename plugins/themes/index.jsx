import { createPlugin } from '@/api/plugin';
import { IconPalette } from '@tabler/icons-solidjs';
import ThemeFooterButton from './ThemeFooterButton.jsx';

export { DAISYUI_THEMES } from './themes.jsx';

export default createPlugin({
  id: 'theme-plugin',
  name: 'Theme System',
  version: '3.0.0',
  description: 'Theme system using DaisyUI built-in themes',
  author: 'WebArcade Team',
  icon: IconPalette,

  async onStart(api) {

    api.footer('theme-selector', {
      component: ThemeFooterButton,
      order: 50,
      section: 'status'
    });

    console.log('[Theme Plugin] Registered theme selector');
  },

  async onStop(api) {
    console.log('[Theme Plugin] Stopping...');
  }
});
