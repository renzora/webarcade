import { createPlugin } from '@/api/plugin';
import { IconCode, IconFolder, IconFile } from '@tabler/icons-solidjs';
import EditorViewport from './EditorViewport.jsx';
import FileTree from './FileTree.jsx';

export default createPlugin({
  id: 'code',
  name: 'Code Editor',
  version: '1.0.0',
  description: 'Monaco-based code editor with file tree navigation',
  author: 'WebArcade Team',

  async onStart(api) {
    console.log('[Code Editor] Starting...');

    // Register viewport type
    api.viewport('code-viewport', {
      label: 'Code',
      component: EditorViewport,
      icon: IconCode,
      description: 'Monaco-based code editor'
    });

    // Register left panel with file tree
    api.leftPanel({
      component: FileTree,
      viewport: 'code-viewport'
    });

    // Register toolbar items
    api.toolbar('code-new-file', {
      icon: IconFile,
      tooltip: 'New File',
      onClick: () => {
        document.dispatchEvent(new CustomEvent('code:new-file'));
      },
      group: 'code',
      order: 10,
      viewport: 'code-viewport'
    });

    api.toolbar('code-open-folder', {
      icon: IconFolder,
      tooltip: 'Open Folder',
      onClick: () => {
        document.dispatchEvent(new CustomEvent('code:open-folder'));
      },
      group: 'code',
      order: 20,
      viewport: 'code-viewport'
    });

    api.toolbarGroup('code', {
      label: 'Editor',
      order: 10,
      viewport: 'code-viewport'
    });

    // Register top menu item
    api.menu('code-menu', {
      label: 'Code',
      order: 50,
      submenu: [
        {
          label: 'Open Code Editor',
          onClick: () => api.open('code-viewport')
        },
        {
          label: 'New File',
          onClick: () => document.dispatchEvent(new CustomEvent('code:new-file'))
        },
        {
          label: 'Open Folder...',
          onClick: () => document.dispatchEvent(new CustomEvent('code:open-folder'))
        }
      ]
    });

    // Open the code editor viewport
    api.open('code-viewport');

    console.log('[Code Editor] Started successfully');
  },

  async onStop() {
    console.log('[Code Editor] Stopping...');
  }
});
