import { plugin } from '@/api/plugin';
import Viewport, { setGreetings, setGreetingCount } from './viewport';
import LeftPanel from './left-panel';
import RightPanel from './right-panel';
import BottomPanel from './bottom-panel';
import { IconFile, IconFolderOpen, IconDeviceFloppy, IconArrowBackUp, IconArrowForwardUp, IconZoomIn, IconZoomOut, IconMaximize } from '@tabler/icons-solidjs';

export default plugin({
    id: 'hello-world',
    name: 'Hello World',
    version: '1.0.0',
    description: 'Hello World plugin',
    author: 'WebArcade',

    start(api) {
        console.log('[Hello World] start() called');

        // Register the plugin tab (shows in main tab bar)
        api.add({
            panel: 'tab',
            label: 'Hello World',
        });

        // Register the main viewport
        api.add({
            panel: 'viewport',
            id: 'main',
            label: 'Main View',
            component: Viewport,
        });

        // Register left panel tab
        api.add({
            panel: 'left',
            id: 'explorer',
            label: 'Explorer',
            component: LeftPanel,
        });

        // Register right panel tab
        api.add({
            panel: 'right',
            id: 'properties',
            label: 'Properties',
            component: RightPanel,
        });

        // Register bottom panel tab
        api.add({
            panel: 'bottom',
            id: 'console',
            label: 'Console',
            component: BottomPanel,
        });

        // Register top menu items
        console.log('[Hello World] Registering menus...');
        api.menu('file-menu', {
            label: 'File',
            submenu: [
                { label: 'New', action: () => alert('New file') },
                { label: 'Open', action: () => alert('Open file') },
                { label: 'Save', action: () => alert('Save file') },
                { divider: true },
                { label: 'Exit', action: () => window.close() }
            ]
        });

        api.menu('edit-menu', {
            label: 'Edit',
            submenu: [
                { label: 'Undo', action: () => alert('Undo') },
                { label: 'Redo', action: () => alert('Redo') },
                { divider: true },
                { label: 'Cut', action: () => alert('Cut') },
                { label: 'Copy', action: () => alert('Copy') },
                { label: 'Paste', action: () => alert('Paste') }
            ]
        });

        api.menu('view-menu', {
            label: 'View',
            submenu: [
                { label: 'Toggle Left Panel', action: () => window.WebArcadeAPI?.pluginAPI?.togglePanel?.('left') || window.WebArcadeAPI?.panelStore?.togglePanel('left') },
                { label: 'Toggle Right Panel', action: () => window.WebArcadeAPI?.panelStore?.togglePanel('right') },
                { label: 'Toggle Bottom Panel', action: () => window.WebArcadeAPI?.pluginAPI?.toggleBottomPanel() },
                { divider: true },
                { label: 'Toggle Footer', action: () => { const api = window.WebArcadeAPI?.pluginAPI; api?.showFooter(!api?.getFooterVisible?.()); } },
                { label: 'Toggle Menu', action: () => { const api = window.WebArcadeAPI?.pluginAPI; api?.showMenu(!api?.getHorizontalMenuButtonsEnabled?.()); } }
            ]
        });

        api.menu('help-menu', {
            label: 'Help',
            submenu: [
                { label: 'Documentation', action: () => alert('Documentation coming soon!') },
                { label: 'About', action: () => alert('Hello World Plugin v1.0.0\n\nA demo plugin for WebArcade.') }
            ]
        });

        // Register toolbar groups
        api.toolbarGroup('file-group', {
            order: 1
        });

        api.toolbarGroup('edit-group', {
            order: 2
        });

        api.toolbarGroup('view-group', {
            order: 3
        });

        // Register toolbar items
        api.toolbar('new-file', {
            icon: IconFile,
            label: 'New',
            tooltip: 'Create new file',
            group: 'file-group',
            order: 1,
            onClick: () => alert('New file')
        });

        api.toolbar('open-file', {
            icon: IconFolderOpen,
            label: 'Open',
            tooltip: 'Open file',
            group: 'file-group',
            order: 2,
            onClick: () => alert('Open file')
        });

        api.toolbar('save-file', {
            icon: IconDeviceFloppy,
            label: 'Save',
            tooltip: 'Save file',
            group: 'file-group',
            order: 3,
            separator: true,
            onClick: () => alert('Save file')
        });

        api.toolbar('undo', {
            icon: IconArrowBackUp,
            label: 'Undo',
            tooltip: 'Undo last action',
            group: 'edit-group',
            order: 1,
            onClick: () => alert('Undo')
        });

        api.toolbar('redo', {
            icon: IconArrowForwardUp,
            label: 'Redo',
            tooltip: 'Redo last action',
            group: 'edit-group',
            order: 2,
            separator: true,
            onClick: () => alert('Redo')
        });

        api.toolbar('zoom-in', {
            icon: IconZoomIn,
            label: 'Zoom In',
            tooltip: 'Zoom in',
            group: 'view-group',
            order: 1,
            onClick: () => alert('Zoom in')
        });

        api.toolbar('zoom-out', {
            icon: IconZoomOut,
            label: 'Zoom Out',
            tooltip: 'Zoom out',
            group: 'view-group',
            order: 2,
            onClick: () => alert('Zoom out')
        });

        api.toolbar('fullscreen', {
            icon: IconMaximize,
            label: 'Fullscreen',
            tooltip: 'Toggle fullscreen',
            group: 'view-group',
            order: 3,
            onClick: () => window.WebArcadeAPI?.pluginAPI?.toggleFullscreen()
        });

        // Listen for events from greeter plugin
        const handleGreeterMessage = (event) => {
            const data = event.detail;
            console.log('[Hello World] Received from Greeter:', data);
            setGreetings(prev => [...prev, data]);
            setGreetingCount(prev => prev + 1);
        };

        document.addEventListener('greeter:greeting-sent', handleGreeterMessage);

        // Store cleanup function
        api._cleanup = () => {
            document.removeEventListener('greeter:greeting-sent', handleGreeterMessage);
        };

        console.log('[Hello World] All panels registered');
    },

    active(api) {
        console.log('[Hello World] Plugin activated');
    },

    inactive(api) {
        console.log('[Hello World] Plugin deactivated');
    },

    stop(api) {
        console.log('[Hello World] Plugin stopped');
        if (api._cleanup) api._cleanup();
    }
});
