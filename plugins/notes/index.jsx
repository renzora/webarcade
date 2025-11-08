import { createPlugin } from '@/api/plugin';
import { IconNotes, IconNotebook } from '@tabler/icons-solidjs';
import NotesViewport from './NotesViewport.jsx';
import NotesPanel from './NotesPanel.jsx';

export default createPlugin({
  id: 'notes-plugin',
  name: 'Notes & Journal',
  version: '1.0.0',
  description: 'Take notes, write journal entries, and organize your thoughts',
  author: 'WebArcade Team',

  async onStart(api) {
    console.log('[Notes Plugin] Starting...');

    // Import viewport store to check active viewport
    const { viewportStore } = await import('@/panels/viewport/store');

    // Register Notes quick panel (only visible when Notes viewport is active)
    api.tab('notes-panel', {
      title: 'Quick Notes',
      component: NotesPanel,
      icon: IconNotebook,
      order: 5,
      condition: () => {
        const activeTab = viewportStore.tabs.find(tab => tab.id === viewportStore.activeTabId);
        return activeTab?.type === 'notes-manager';
      }
    });

    // Register Notes viewport
    api.viewport('notes-manager', {
      label: 'Notes',
      component: NotesViewport,
      icon: IconNotes,
      description: 'Manage your notes, journal entries, and ideas'
    });

    console.log('[Notes Plugin] Started successfully');
  },

  async onStop() {
    console.log('[Notes Plugin] Stopping...');
  }
});
