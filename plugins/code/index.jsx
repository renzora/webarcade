import { createPlugin } from '@/api/plugin';
import { IconCode } from '@tabler/icons-solidjs';
import { createEffect } from 'solid-js';
import { viewportStore } from '@/panels/viewport/store';
import CodeEditorViewport from './CodeEditorViewport.jsx';

export default createPlugin({
  id: 'code-editor-viewport-plugin',
  name: 'Code Editor Viewport Plugin',
  version: '1.0.0',
  description: 'Code editor viewport for editing scripts and text files',
  author: 'Renzora Engine Team',

  async onInit(_api) {
  },

  async onStart(api) {
    
    api.viewport('code-editor', {
      label: 'Code Editor',
      component: CodeEditorViewport,
      icon: IconCode,
      description: 'Code editor for scripts and text files'
    });
    
    // Toolbar visibility is now managed elsewhere
    // const effect = createEffect(() => {
    //   const activeTabId = viewportStore.activeTabId;
    //   const tabs = viewportStore.tabs;
    //   const activeTab = tabs.find(tab => tab.id === activeTabId);
    // });

    // api.toolbarEffect = effect;
    
  },

  onUpdate() {

  },

  async onStop(api) {
    // Cleanup if needed
  },

  async onDispose() {
    // Cleanup if needed
  }
});