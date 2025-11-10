import { createPlugin } from '@/api/plugin';
import { IconBulb } from '@tabler/icons-solidjs';

export default createPlugin({
  id: 'philips-hue-plugin',
  name: 'Philips Hue',
  version: '1.0.0',
  description: 'Control your Philips Hue smart lights',
  author: 'WebArcade Team',
  icon: IconBulb,

  async onStart(api) {
    console.log('[Philips Hue Plugin] Starting...');

    // Widgets are auto-loaded from ./widgets/ directory
    // - HueBridgeSetup.jsx - Configure Hue Bridge connection
    // - LightControl.jsx - Control individual lights
    // - RoomControl.jsx - Control groups/rooms

    console.log('[Philips Hue Plugin] Started successfully');
  },

  async onStop() {
    console.log('[Philips Hue Plugin] Stopping...');
  }
});
