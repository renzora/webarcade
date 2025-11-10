import { createPlugin } from '@/api/plugin';
import { Activity } from 'lucide-solid';
import WithingsViewport from './viewport.jsx';

export default createPlugin({
  id: 'withings',
  name: 'Withings',
  version: '1.0.0',
  description: 'Withings health data integration for body composition tracking',
  author: 'WebArcade',
  icon: Activity,

  async onStart(api) {
    console.log('Withings plugin started');

    // Register viewport for the main dashboard
    api.viewport('withings-dashboard', {
      label: 'Withings',
      icon: Activity,
      component: WithingsViewport,
      description: 'Body composition tracking and analytics'
    });

    // Register left panel menu item
    api.registerLeftPanelMenuItem('withings', {
      label: 'Withings',
      icon: Activity,
      onClick: () => {
        api.open('withings-dashboard');
      },
    });

    console.log('Withings plugin registered successfully');
  },

  async onStop() {
    console.log('Withings plugin stopped');
  },
});
