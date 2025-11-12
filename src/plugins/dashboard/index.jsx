import { createPlugin } from '@/api/plugin';
import { IconChartLine, IconPlus } from '@tabler/icons-solidjs';
import DashboardViewport from './viewport.jsx';
import { dashboardAPI } from './api.js';

// Store current dashboard ID globally
let currentDashboardId = 'default';
let pluginApi = null;

export default createPlugin({
  id: 'webarcade-dashboard-plugin',
  name: 'Dashboard Plugin',
  version: '1.0.0',
  description: 'Dashboard with plugin widgets',
  author: 'WebArcade Team',
  icon: IconChartLine,

  async onInit() {

  },

  async onStart(api) {
    pluginApi = api;

    // Register main viewport
    api.viewport('webarcade-dashboard', {
      label: 'Dashboard',
      component: DashboardViewport,
      icon: IconChartLine,
      description: 'Dashboard with plugin widgets',
      props: () => ({
        currentDashboardId: () => currentDashboardId,
        setCurrentDashboardId: (id) => {
          currentDashboardId = id;
          // Refresh the viewport to load new dashboard
          api.emit('dashboard:changed', { dashboardId: id });
        }
      })
    });

    api.showProps(true);
    api.showMenu(true);
    api.showFooter(true);
    api.showTabs(true);

    // Load and register dashboard menu items
    await loadDashboardMenuItems(api);

    // Open dashboard
    setTimeout(() => {
      api.open('webarcade-dashboard', {
        title: 'Dashboard',
        closable: true
      });
    }, 100);
  },

  onUpdate() {

  },

  async onStop() {

  },

  async onDispose() {

  }
});

async function loadDashboardMenuItems(api) {
  try {
    const dashboards = await dashboardAPI.getDashboards();

    // Add each dashboard as a menu item
    dashboards.forEach((dashboard, index) => {
      api.registerLeftPanelMenuItem(`dashboard-${dashboard.id}`, {
        label: dashboard.name,
        icon: IconChartLine,
        category: 'Dashboards',
        order: 20 + index,
        onClick: () => {
          console.log('[Dashboard] Switching to dashboard:', dashboard.id);
          currentDashboardId = dashboard.id;

          // Emit event to notify WidgetGrid
          api.emit('dashboard:changed', { dashboardId: dashboard.id });

          // Update the viewport title
          api.open('webarcade-dashboard', {
            title: dashboard.name,
            closable: true
          });
        }
      });
    });

    // Add "Create Dashboard" button
    api.registerLeftPanelMenuItem('dashboard-create', {
      label: 'New Dashboard',
      icon: IconPlus,
      category: 'Dashboards',
      order: 1000,
      onClick: async () => {
        const name = prompt('Enter dashboard name:');
        if (!name) return;

        try {
          await dashboardAPI.createDashboard(name);
          // Reload menu items
          await loadDashboardMenuItems(api);
        } catch (error) {
          alert('Failed to create dashboard: ' + error.message);
        }
      }
    });
  } catch (error) {
    console.error('[Dashboard] Failed to load dashboards:', error);
  }
}

// Export function to reload menu items
export async function reloadDashboardMenu() {
  if (pluginApi) {
    await loadDashboardMenuItems(pluginApi);
  }
}

export function getCurrentDashboardId() {
  return currentDashboardId;
}

export function setCurrentDashboardId(id) {
  currentDashboardId = id;
  if (pluginApi) {
    pluginApi.emit('dashboard:changed', { dashboardId: id });
  }
}
