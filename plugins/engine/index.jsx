import { createPlugin } from '@/api/plugin';
import Viewport from './viewport';
import LeftPanel from './LeftPanel';
import RightPanel from './RightPanel';

export default createPlugin({
    id: 'engine',
    name: 'Engine',
    version: '1.0.0',
    description: '3D engine with viewport rendering',
    author: 'WebArcade',

    async onStart(api) {
        console.log('[Engine] Starting...');

        api.viewport('engine-viewport', {
            label: 'Engine',
            component: Viewport,
            description: '3D viewport with cube and grid'
        });

        // Register panels for this viewport type
        api.leftPanel({ component: LeftPanel, viewport: 'engine-viewport' });
        api.rightPanel({ component: RightPanel, viewport: 'engine-viewport' });

        api.open('engine-viewport');
    },

    async onStop() {
        console.log('[Engine] Stopping...');
    }
});
