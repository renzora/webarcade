import { createPlugin } from '@/api/plugin';
import { IconSettings, IconChartLine } from '@tabler/icons-solidjs';
import DashboardViewport from './DashboardViewport.jsx';
import PropertiesPanel from './PropertiesPanel.jsx';

export default createPlugin({
  id: 'default-plugin',
  name: 'Default Plugin',
  version: '1.0.0',
  description: 'Sets up default viewport and panels on startup',
  author: 'WebArcade Team',

  async onInit() {
    // Initialization
  },

  async onStart(api) {
    // Register dashboard viewport
    api.viewport('dashboard', {
      label: 'Dashboard',
      component: DashboardViewport,
      icon: IconChartLine,
      description: 'System performance dashboard'
    });

    // Register default properties panel tab
    api.tab('properties', {
      title: 'Properties',
      component: PropertiesPanel,
      icon: IconSettings,
      order: 1
    });

    // Show the right panel by default
    api.showProps(true);

    // Show menu
    api.showMenu(true);

    // Show footer
    api.showFooter(true);

    // Show viewport tabs
    api.showTabs(true);

    // Open the dashboard viewport by default
    setTimeout(() => {
      api.open('dashboard', {
        title: 'Dashboard',
        closable: true
      });
    }, 100);
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
