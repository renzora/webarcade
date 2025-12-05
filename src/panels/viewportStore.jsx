import { createStore } from 'solid-js/store'

const [viewportStore, setViewportStore] = createStore({
  tabs: [],
  activeTabId: null,
  suspendedTabs: []
})

export const viewportActions = {

  setActiveViewportTab: (tabId) => {
    const previousTabId = viewportStore.activeTabId;

    // Emit deactivation event for previous tab
    if (previousTabId && previousTabId !== tabId) {
      const event = new CustomEvent('viewport:tab-deactivated', {
        detail: { tabId: previousTabId }
      });
      document.dispatchEvent(event);
    }

    setViewportStore('activeTabId', tabId);

    // Emit activation event for new tab
    if (tabId) {
      const event = new CustomEvent('viewport:tab-activated', {
        detail: { tabId }
      });
      document.dispatchEvent(event);
    }
  },
  
  addViewportTab: (tab) => {
    const currentTabs = viewportStore.tabs;
    setViewportStore('tabs', (tabs) => [...tabs, tab]);

    if (currentTabs.length === 0 || !viewportStore.activeTabId) {
      // Use setActiveViewportTab to ensure event is dispatched
      viewportActions.setActiveViewportTab(tab.id);
    }
  },
  
  removeViewportTab: (tabId) => {
    const tabs = viewportStore.tabs;
    const index = tabs.findIndex(tab => tab.id === tabId);
    
    if (index !== -1 && tabs.length > 1) {
      if (viewportStore.activeTabId === tabId) {
        const newActiveTab = tabs[index === 0 ? 1 : index - 1];
        setViewportStore('activeTabId', newActiveTab.id);
      }
      
      setViewportStore('tabs', tabs => tabs.filter(tab => tab.id !== tabId));
    }
  },
  
  pinViewportTab: (tabId) => {
    const index = viewportStore.tabs.findIndex(tab => tab.id === tabId);
    if (index !== -1) {
      setViewportStore('tabs', index, 'isPinned', (pinned) => !pinned);
    }
  },
  
  duplicateViewportTab: (tabId) => {
    const tab = viewportStore.tabs.find(t => t.id === tabId);
    if (tab) {
      const newTabId = `viewport-${Date.now()}`;
      const newTab = {
        ...tab,
        id: newTabId,
        name: `${tab.name} (Copy)`,
        isPinned: false
      };
      viewportActions.addViewportTab(newTab);
      viewportActions.setActiveViewportTab(newTabId);
    }
  },
  
  renameViewportTab: (tabId, newName) => {
    const index = viewportStore.tabs.findIndex(tab => tab.id === tabId);
    if (index !== -1) {
      setViewportStore('tabs', index, 'name', newName);
    }
  }
}

export { viewportStore }

if (typeof window !== 'undefined') {
  window.viewportStore = viewportStore
  window.viewportActions = viewportActions
}
