import { createPlugin } from '@/api/plugin';
import { IconClock } from '@tabler/icons-solidjs';
import TimerViewport from './TimerViewport.jsx';

export default createPlugin({
  id: 'twitch-timer-plugin',
  name: 'Twitch Timer',
  version: '1.0.0',
  description: 'Manage timers and Pomodoro sessions',
  author: 'WebArcade Team',

  async onStart(api) {
    console.log('[Twitch Timer Plugin] Starting...');

    // Register Timer viewport
    api.viewport('twitch-timer', {
      label: 'Timer',
      component: TimerViewport,
      icon: IconClock,
      description: 'Manage timers and Pomodoro sessions'
    });

    console.log('[Twitch Timer Plugin] Started successfully');
  },

  async onStop() {
    console.log('[Twitch Timer Plugin] Stopping...');
  }
});
