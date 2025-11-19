import { createPlugin } from '@/api/plugin';
import { IconPalette } from '@tabler/icons-solidjs';
import ThemeFooterButton from './ThemeFooterButton.jsx';

export default createPlugin({
  id: 'theme-plugin',
  name: 'Theme Selector',
  version: '1.0.0',
  description: 'Quick theme switcher in the footer',
  author: 'WebArcade Team',
  icon: IconPalette,

  async onStart(api) {
    // Register footer button for theme selection
    api.footer('theme-selector', {
      component: ThemeFooterButton,
      order: 50, // Place it early in the footer
      section: 'status'
    });
  },

  async onStop() {
    // Cleanup if needed
  }
});
