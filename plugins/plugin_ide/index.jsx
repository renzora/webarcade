import { createPlugin } from '@/api/plugin';
import { IconCode, IconFiles } from '@tabler/icons-solidjs';
import PluginIDEViewport from './viewport.jsx';
import FilesPanel from './components/FilesPanel.jsx';

export default createPlugin({
  id: 'plugin-ide',
  name: 'Plugin IDE',
  version: '1.0.0',
  description: 'Integrated development environment for building WebArcade plugins',
  author: 'WebArcade',

  async onInit() {
    console.log('[Plugin IDE] Initializing...');
  },

  async onStart(api) {
    console.log('[Plugin IDE] Starting...');

    // Register the IDE viewport
    api.viewport('plugin-ide-viewport', {
      label: 'Plugin IDE',
      component: PluginIDEViewport,
      icon: IconCode,
      description: 'Develop and manage plugins with Monaco editor'
    });

    // Register Files panel in right panel
    api.tab('plugin-ide-files', {
      title: 'Files',
      component: FilesPanel,
      icon: IconFiles,
      order: 1,
      viewport: 'plugin-ide-viewport'
    });

    // Register menu item to open the IDE
    api.menu('plugin-ide-menu', {
      label: 'Plugin IDE',
      icon: IconCode,
      onClick: () => {
        api.open('plugin-ide-viewport', {
          label: 'Plugin IDE'
        });
      }
    });

    // Show all UI elements
    api.showProps(true);  // Show props panel for file tree
    api.showMenu(true);
    api.showFooter(true);
    api.showTabs(true);

    console.log('[Plugin IDE] Started successfully');
  },

  async onStop() {
    console.log('[Plugin IDE] Stopping...');
  },

  async onDispose() {
    console.log('[Plugin IDE] Disposing...');
  }
});
