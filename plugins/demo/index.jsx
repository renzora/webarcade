import { createPlugin } from '@/api/plugin';
import { IconCheck, IconX, IconBox } from '@tabler/icons-solidjs';
import DemoViewport from './DemoViewport.jsx';
import DemoLeftPanel from './DemoLeftPanel.jsx';
import DemoRightPanel from './DemoRightPanel.jsx';
import DemoBottomPanel from './DemoBottomPanel.jsx';
import DemoFooterButton from './DemoFooterButton.jsx';

export default createPlugin({
  id: 'demo',
  name: 'Demo Plugin',
  version: '1.0.0',
  description: 'Showcases all UI components: viewport, panels, menu, and footer',
  author: 'WebArcade Team',

  async onStart(api) {
    console.log('[Demo Plugin] Starting...');

    // Register viewport type
    api.viewport('demo-viewport', {
      label: 'Demo',
      component: DemoViewport,
      description: 'Demo viewport showcasing all UI features'
    });

    // Register left panel (scoped to demo-viewport)
    api.leftPanel({
      component: DemoLeftPanel,
      viewport: 'demo-viewport'
    });

    // Register right panel (scoped to demo-viewport)
    api.rightPanel({
      component: DemoRightPanel,
      viewport: 'demo-viewport'
    });

    // Register bottom panel tab (scoped to demo-viewport)
    api.bottomTab('demo-console', {
      title: 'Demo Console',
      component: DemoBottomPanel,
      viewport: 'demo-viewport',
      order: 10
    });

    // Register toolbar buttons (simple static icons)
    api.toolbar('demo-action1', {
      icon: IconCheck,
      tooltip: 'Demo Action 1',
      onClick: () => console.log('[Demo] Action 1'),
      group: 'demo',
      order: 10
    });

    api.toolbar('demo-action2', {
      icon: IconBox,
      tooltip: 'Demo Action 2',
      onClick: () => console.log('[Demo] Action 2'),
      group: 'demo',
      order: 20
    });

    api.toolbar('demo-action3', {
      icon: IconX,
      tooltip: 'Demo Action 3',
      onClick: () => console.log('[Demo] Action 3'),
      group: 'demo',
      order: 30
    });

    api.toolbarGroup('demo', {
      label: 'Demo',
      order: 50
    });

    // Register top menu item
    api.menu('demo-menu', {
      label: 'Demo',
      order: 100,
      submenu: [
        {
          label: 'Open Demo Viewport',
          onClick: () => api.open('demo-viewport')
        },
        {
          label: 'Toggle Left Panel',
          onClick: () => api.showLeftPanel(!api.getPropertiesPanelVisible())
        },
        {
          label: 'Toggle Right Panel',
          onClick: () => api.showProps(!api.getPropertiesPanelVisible())
        },
        {
          label: 'Toggle Bottom Panel',
          onClick: () => api.toggleBottomPanel()
        }
      ]
    });

    // Register footer button
    api.footer('demo-status', {
      component: DemoFooterButton,
      order: 50,
      section: 'status'
    });

    // Open the demo viewport and show panels
    api.open('demo-viewport');

    // Show bottom panel after a short delay to ensure viewport is active
    setTimeout(() => {
      api.showBottomPanel(true);
    }, 100);

    console.log('[Demo Plugin] All components registered successfully');
  },

  async onStop() {
    console.log('[Demo Plugin] Stopping...');
  }
});
