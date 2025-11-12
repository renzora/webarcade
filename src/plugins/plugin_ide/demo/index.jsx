import { createPlugin } from '@/api/plugin';
import { IconChartBar } from '@tabler/icons-solidjs';
import MyViewport from './viewport.jsx';

export default createPlugin({
  id: 'demo',
  name: 'demo',
  version: '1.0.0',
  description: 'demo',
  author: 'demo',

  async onStart(api) {
    console.log('[demo] Starting...');

    api.viewport('demo-viewport', {
      label: 'demo',
      component: MyViewport,
      icon: IconChartBar,
      description: 'demo'
    });

    api.menu('demo-menu', {
      label: 'demo',
      icon: IconChartBar,
      onClick: () => {
        api.open('demo-viewport', { label: 'demo' });
      }
    });
  }
});
