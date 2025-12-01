import { createPlugin } from '@/api/plugin';
import { IconBook } from '@tabler/icons-solidjs';
import GuideViewport from './GuideViewport.jsx';

export default createPlugin({
  id: 'demo',
  name: 'Plugin Guide',
  version: '1.0.0',
  description: 'Plugin development documentation',
  author: 'WebArcade Team',

  async onStart(api) {
    api.viewport('guide', {
      label: 'Quick Start',
      icon: IconBook,
      component: GuideViewport,
      onActivate: (api) => {
        api.showTabs(true);
      }
    });

    await api.setWindowSize(800, 700);
    await api.centerWindow();
    api.open('guide');
  }
});
