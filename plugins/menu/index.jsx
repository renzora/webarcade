import { createPlugin } from '@/api/plugin';
import { createSignal, createEffect, onCleanup } from 'solid-js';
import { IconRefresh, IconVideo, IconEdit, IconArrowLeft, IconArrowRight, IconPlus, IconFolder, IconFile, IconArrowDown, IconScissors, IconCopy, IconClipboard, IconTrash, IconCube, IconWorld, IconBox, IconCircle, IconCylinder, IconSquare, IconChairDirector, IconLink, IconHelp, IconHeadphones, IconBrandYoutube, IconBrandDiscord, IconBook, IconInfoCircle, IconDeviceFloppy, IconMountain, IconSun, IconBulb, IconSphere, IconPackage, IconSettings, IconEye, IconRobot, IconBrain, IconMessage, IconX
} from '@tabler/icons-solidjs';
import AboutOverlay from '../system/AboutOverlay.jsx';
import { editorActions } from '@/layout/stores/EditorStore';
import { viewportTypes } from '@/api/plugin';

const [showAbout, setShowAbout] = createSignal(false);
const [showOpenProject, setShowOpenProject] = createSignal(false);

// Export setShowAbout for use in footer
export { setShowAbout };



const handleNewProject = async () => {
};

const handleOpenProject = async () => {
  setShowOpenProject(true);
};

const handleProjectSelect = async (project) => {
  alert('Project switching is currently unavailable. The project API has been removed.');
};


export default createPlugin({
  id: 'menu-plugin',
  name: 'Menu Plugin',
  version: '1.0.0',
  description: 'Core application menu items',
  author: 'Renzora Engine Team',

  async onInit() {

  },

  async onStart(api) {
    // Menus removed for personal use

    api.registerLayoutComponent('about-overlay', () => {
      return (
        <AboutOverlay
          isOpen={showAbout}
          onClose={() => setShowAbout(false)}
        />
      );
    });

    // Dynamically register all plugin viewport types
    const registeredViewportItems = new Set();

    createEffect(() => {
      const types = viewportTypes();

      // Register each viewport type as a left panel menu item
      types.forEach((viewportType, typeId) => {
        const menuId = `viewport-${typeId}`;

        // Skip if already registered
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

    // Register utility menu items
    api.registerLeftPanelMenuItem('settings', {
      label: 'Settings',
      icon: IconSettings,
      description: 'Open application settings',
      category: 'Tools',
      order: 100,
      onClick: () => {
        console.log('Settings clicked');
      }
    });

  }
});