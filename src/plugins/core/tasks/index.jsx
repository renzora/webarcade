import { createPlugin } from '@/api/plugin';
import { IconChecklist } from '@tabler/icons-solidjs';
import TasksViewport from './TasksViewport.jsx';

export default createPlugin({
  id: 'twitch-tasks-plugin',
  name: 'Twitch Channel Tasks',
  version: '1.0.0',
  description: 'View and manage all tasks for the channel',
  author: 'WebArcade Team',

  async onStart(api) {
    console.log('[Twitch Tasks Plugin] Starting...');

    // Register Tasks viewport
    api.viewport('twitch-tasks', {
      label: 'Channel Tasks',
      component: TasksViewport,
      icon: IconChecklist,
      description: 'View and manage all tasks for the channel'
    });

    console.log('[Twitch Tasks Plugin] Started successfully');
  },

  async onStop() {
    console.log('[Twitch Tasks Plugin] Stopping...');
  }
});
