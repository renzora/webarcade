import { createPlugin } from '@/api/plugin';
import { createSignal, createEffect, onCleanup } from 'solid-js';
import { IconVideo } from '@tabler/icons-solidjs';
import AboutOverlay from '../system/AboutOverlay.jsx';
import { viewportTypes } from '@/api/plugin';

const [showAbout, setShowAbout] = createSignal(false);

export { setShowAbout };

export default createPlugin({
  id: 'menu-plugin',
  name: 'Menu Plugin',
  version: '1.0.0',
  description: 'Core application menu items',
  author: 'Renzora Engine Team',

  async onInit() {

  },

  async onStart(api) {

    api.registerLayoutComponent('about-overlay', () => {
      return (
        <AboutOverlay
          isOpen={showAbout}
          onClose={() => setShowAbout(false)}
        />
      );
    });

    const registeredViewportItems = new Set();

    createEffect(() => {
      const types = viewportTypes();

      types.forEach((viewportType, typeId) => {
        const menuId = `viewport-${typeId}`;

        if (registeredViewportItems.has(menuId)) {
          return;
        }

        api.registerLeftPanelMenuItem(menuId, {
          label: viewportType.label || viewportType.name || typeId,
          icon: viewportType.icon || IconVideo,
          description: viewportType.description || `Create a new ${viewportType.label || typeId} viewport`,
          category: 'Plugins',
          order: 10,
          onClick: () => {
            api.createViewportTab(typeId, {
              label: viewportType.label || viewportType.name || 'New Viewport'
            });
          }
        });

        registeredViewportItems.add(menuId);
      });
    });

  }
});