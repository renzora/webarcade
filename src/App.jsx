import { onMount, For } from 'solid-js'
import './base.css'
import './themes'
import { Engine, layoutComponents } from '@/api/plugin'
import Layout from './layout'
import DevNotice from './components/DevNotice'
import KeyboardShortcuts from './components/KeyboardShortcuts'
import PluginInstaller from './components/PluginInstaller'
import { usePluginAPI } from '@/api/plugin'
import { editorStore } from '@/layout/stores/EditorStore.jsx'
import {
  IconSettings as SettingsIcon, IconMaximize
} from '@tabler/icons-solidjs'
export default function App() {
  onMount(async () => {
    // Apply theme on startup
    const theme = editorStore.theme || 'dark';
    document.documentElement.setAttribute('data-theme', theme);


    // Wait for plugin API to be initialized
    setTimeout(() => {
      try {
        const api = usePluginAPI();
    
        // Settings helper  
        api.helper('settings-button', {
          title: 'Settings',
          icon: SettingsIcon,
          order: 30,
          onClick: () => {
          }
        });

        api.helper('fullscreen-button', {
          title: 'Toggle Fullscreen',
          icon: IconMaximize,
          order: 40,
          onClick: () => {
            if (!document.fullscreenElement) {
              document.documentElement.requestFullscreen().catch(err => {
              });
            } else {
              document.exitFullscreen();
            }
          }
        });
      } catch (error) {
      }
    }, 100);
    
    // Setup window close handling for Tauri (handles Alt+F4, system close, etc.)
    const setupWindowCloseHandler = async () => {
      try {
        // Check if we're in Tauri environment
        if (window.__TAURI__) {
          const { listen } = await import('@tauri-apps/api/event');
          
          // Listen for window close request from Tauri (triggered by system/OS close)
          await listen('window-close-requested', async () => {
            await handleWindowCloseRequest();
          });
        }
      } catch (error) {
      }
    };
    
    setupWindowCloseHandler();
    
    // Show window immediately
    setTimeout(async () => {
      if (typeof window !== 'undefined' && window.__TAURI_INTERNALS__) {
        try {
          const { getCurrentWindow } = await import('@tauri-apps/api/window');
          const currentWindow = getCurrentWindow();
          await currentWindow.show();
        } catch (error) {
        }
      }
    }, 50);
  })
  
  // Simplified window close handler
  const handleWindowCloseRequest = async () => {
  };

  return (
    <Engine>
      <KeyboardShortcuts />
      <PluginInstaller />
      <div class="w-full h-full">
        <Layout />
        <DevNotice />

        {/* Render layout components from plugins */}
        <For each={Array.from(layoutComponents().values())}>
          {(layoutComponent) => {
            // Handle both old and new structure for backwards compatibility
            const Component = layoutComponent?.component || layoutComponent;
            return <Component />;
          }}
        </For>
      </div>
    </Engine>
  );
}