import { createPlugin } from '@/api/plugin';
import { IconChartBar } from '@tabler/icons-solidjs';
import MyViewport from './viewport.jsx';

export default createPlugin({
  id: 'test',
  name: 'test',
  version: '1.0.0',
  description: 'nothing',
  author: 'james',

  async onStart(api) {
    console.log('[test] Starting...');

    api.viewport('test-viewport', {
      label: 'test',
      component: MyViewport,
      icon: IconChartBar,
      description: 'nothing'
    });

    api.menu('test-menu', {
      label: 'test',
      icon: IconChartBar,
      onClick: () => {
        api.open('test-viewport', { label: 'test' });
      }
    });
  }
});
