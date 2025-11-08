import { createPlugin } from '@/api/plugin';
import { IconMessage, IconSettings, IconInfoCircle, IconCalendar } from '@tabler/icons-solidjs';
import TwitchSettingsViewport from './TwitchSettingsViewport.jsx';
import TwitchChatViewport from './TwitchChatViewport.jsx';
import TwitchScheduleViewport from './TwitchScheduleViewport.jsx';
import StreamInfoPanel from './StreamInfoPanel.jsx';
import twitchStore from './TwitchStore.jsx';

export default createPlugin({
  id: 'twitch-plugin',
  name: 'Twitch Integration',
  version: '1.0.0',
  description: 'Core Twitch bot with chat and settings',
  author: 'WebArcade Team',

  async onInit() {
    console.log('[Twitch Plugin] Initializing...');
    // Initialize WebSocket connection for Twitch events
    await twitchStore.connect();
  },

  async onStart(api) {
    console.log('[Twitch Plugin] Starting...');

    // Register Chat as right panel tab
    api.tab('twitch-chat', {
      title: 'Chat',
      component: TwitchChatViewport,
      icon: IconMessage,
      order: 2
    });

    // Register Stream Info panel
    api.tab('stream-info', {
      title: 'Stream Info',
      component: StreamInfoPanel,
      icon: IconInfoCircle,
      order: 4,
      viewport: 'twitch-settings'
    });

    // Register Settings as a viewport
    api.viewport('twitch-settings', {
      label: 'Twitch Settings',
      component: TwitchSettingsViewport,
      icon: IconSettings,
      description: 'Configure Twitch bot settings and authentication'
    });

    // Register Schedule as a viewport
    api.viewport('twitch-schedule', {
      label: 'Stream Schedule',
      component: TwitchScheduleViewport,
      icon: IconCalendar,
      description: 'View and manage your Twitch stream schedule'
    });

    console.log('[Twitch Plugin] Started successfully');
  },

  onUpdate() {
    // Update logic if needed
  },

  async onStop() {
    console.log('[Twitch Plugin] Stopping...');
  },

  async onDispose() {
    console.log('[Twitch Plugin] Disposing...');
    twitchStore.disconnect();
  }
});
