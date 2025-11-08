import { createPlugin } from '@/api/plugin';
import { IconHighlight } from '@tabler/icons-solidjs';
import ChatHighlightViewport from './ChatHighlightViewport.jsx';
import ChatHighlightPanel from './ChatHighlightPanel.jsx';

console.log('[Chat Highlight Plugin] Module loaded');

export default createPlugin({
  id: 'chat-highlight',
  name: 'Chat Highlight',
  version: '1.0.0',
  description: 'Display highlighted chat messages with !highlight command',
  author: 'WebArcade Team',

  async onInit() {
    console.log('[Chat Highlight Plugin] Initializing...');
  },

  async onStart(api) {
    console.log('[Chat Highlight Plugin] Starting...');

    // Register chat highlight viewport
    api.viewport('chat-highlight-viewport', {
      label: 'Chat Highlight',
      component: ChatHighlightViewport,
      icon: IconHighlight,
      description: 'Preview and manage chat highlight overlay'
    });
    console.log('[Chat Highlight Plugin] Viewport registered');

    // Register chat highlight control panel
    api.tab('chat-highlight-panel', {
      title: 'Chat Highlight',
      component: ChatHighlightPanel,
      icon: IconHighlight,
      order: 12
    });
    console.log('[Chat Highlight Plugin] Panel registered');
    console.log('[Chat Highlight Plugin] Started successfully');
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
