import { createPlugin } from '@/api/plugin';
import { IconBrandTwitch, IconMessage, IconSettings, IconDeviceTv, IconList, IconChecklist, IconVolume, IconBulb, IconSparkles, IconTerminal2, IconClock, IconUsers, IconDatabase, IconWheel, IconScale, IconBrandDiscord, IconMusic } from '@tabler/icons-solidjs';
import TwitchSettingsViewport from './TwitchSettingsViewport.jsx';
import TwitchChatViewport from './TwitchChatViewport.jsx';
import OverlayManagerViewport from './OverlayManagerViewport.jsx';
import CountersViewport from './CountersViewport.jsx';
import TasksViewport from './TasksViewport.jsx';
import TTSWhitelistViewport from './TTSWhitelistViewport.jsx';
import HueViewport from './HueViewport.jsx';
import HueScenesViewport from './HueScenesViewport.jsx';
import HuePanel from './HuePanel.jsx';
import TextCommandsViewport from './TextCommandsViewport.jsx';
import TimerViewport from './TimerViewport.jsx';
import WatchtimeViewport from './WatchtimeViewport.jsx';
import ViewerStatsViewport from './ViewerStatsViewport.jsx';
import DatabaseViewport from './DatabaseViewport.jsx';
import WheelViewport from './WheelViewport.jsx';
import WithingsViewport from './WithingsViewport.jsx';
import DiscordViewport from './DiscordViewport.jsx';
import SongRequestsViewport from './SongRequestsViewport.jsx';
import twitchStore from './TwitchStore.jsx';

export default createPlugin({
  id: 'twitch-plugin',
  name: 'Twitch Integration',
  version: '1.0.0',
  description: 'Twitch bot with chat, commands, and overlays',
  author: 'WebArcade Team',

  async onInit() {
    console.log('[Twitch Plugin] Initializing...');
    // Initialize WebSocket connection for Twitch events
    await twitchStore.connect();
  },

  async onStart(api) {
    console.log('[Twitch Plugin] Starting...');

    // Import viewport store to check active viewport
    const { viewportStore } = await import('@/panels/viewport/store');

    // Register Chat as right panel tab
    api.tab('twitch-chat', {
      title: 'Chat',
      component: TwitchChatViewport,
      icon: IconMessage,
      order: 2
    });

    // Register Hue quick control panel (only visible when Hue viewport is active)
    api.tab('hue-control', {
      title: 'Hue',
      component: HuePanel,
      icon: IconBulb,
      order: 3,
      condition: () => {
        const activeTab = viewportStore.tabs.find(tab => tab.id === viewportStore.activeTabId);
        return activeTab?.type === 'twitch-hue';
      }
    });

    // Register Settings as a viewport
    api.viewport('twitch-settings', {
      label: 'Twitch Settings',
      component: TwitchSettingsViewport,
      icon: IconSettings,
      description: 'Configure Twitch bot settings and authentication'
    });

    // Overlay Manager viewport
    api.viewport('overlay-manager', {
      label: 'Overlay Manager',
      component: OverlayManagerViewport,
      icon: IconDeviceTv,
      description: 'Create and manage OBS browser source overlays'
    });

    // Register Counters viewport
    api.viewport('twitch-counters', {
      label: 'Stream Counters',
      component: CountersViewport,
      icon: IconList,
      description: 'Manage and track stream counters'
    });

    // Register Wheel viewport
    api.viewport('twitch-wheel', {
      label: 'Spin Wheel',
      component: WheelViewport,
      icon: IconWheel,
      description: 'Create and manage wheel spin options'
    });

    // Register Tasks viewport
    api.viewport('twitch-tasks', {
      label: 'Channel Tasks',
      component: TasksViewport,
      icon: IconChecklist,
      description: 'View and manage all tasks for the channel'
    });

    // Register TTS Whitelist viewport
    api.viewport('twitch-tts', {
      label: 'TTS Settings',
      component: TTSWhitelistViewport,
      icon: IconVolume,
      description: 'Configure text-to-speech settings and whitelist'
    });

    // Register Hue viewport
    api.viewport('twitch-hue', {
      label: 'Hue Lights',
      component: HueViewport,
      icon: IconBulb,
      description: 'Control Philips Hue smart lights'
    });

    // Register Hue Scenes viewport
    api.viewport('twitch-hue-scenes', {
      label: 'Hue Scenes',
      component: HueScenesViewport,
      icon: IconSparkles,
      description: 'Create animated multi-color light sequences'
    });

    // Register Text Commands viewport
    api.viewport('twitch-text-commands', {
      label: 'Text Commands',
      component: TextCommandsViewport,
      icon: IconTerminal2,
      description: 'Create custom text commands with dynamic variables'
    });

    // Register Timer viewport
    api.viewport('twitch-timer', {
      label: 'Timer',
      component: TimerViewport,
      icon: IconClock,
      description: 'Manage timers and Pomodoro sessions'
    });

    // Register Watchtime viewport
    api.viewport('twitch-watchtime', {
      label: 'Watchtime',
      component: WatchtimeViewport,
      icon: IconClock,
      description: 'View and search viewer watchtime with pagination'
    });

    // Register Viewer Stats viewport
    api.viewport('twitch-viewer-stats', {
      label: 'Viewer Stats',
      component: ViewerStatsViewport,
      icon: IconUsers,
      description: 'View active viewers by day, week, or month'
    });

    // Register Database viewport
    api.viewport('twitch-database', {
      label: 'Database Manager',
      component: DatabaseViewport,
      icon: IconDatabase,
      description: 'Execute SQL queries and manage the SQLite database'
    });

    // Register Withings viewport
    api.viewport('twitch-withings', {
      label: 'Withings Health',
      component: WithingsViewport,
      icon: IconScale,
      description: 'Track weight and health metrics from Withings scale'
    });

    // Register Discord viewport
    api.viewport('twitch-discord', {
      label: 'Discord Bot',
      component: DiscordViewport,
      icon: IconBrandDiscord,
      description: 'Configure Discord bot for song requests'
    });

    // Register Song Requests viewport
    api.viewport('twitch-song-requests', {
      label: 'Song Requests',
      component: SongRequestsViewport,
      icon: IconMusic,
      description: 'Manage Discord song request queue for YouTube Music'
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
