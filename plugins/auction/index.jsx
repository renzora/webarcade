import { createPlugin } from '@/api/plugin';
import { IconGavel } from '@tabler/icons-solidjs';
import AuctionViewport from './AuctionViewport.jsx';

export default createPlugin({
  id: 'auction-plugin',
  name: 'Auction System',
  version: '1.0.0',
  description: 'Live auction system for collectible items with WebSocket integration',
  author: 'WebArcade Team',

  async onStart(api) {
    console.log('[Auction Plugin] Starting...');

    // Register Auction viewport
    api.viewport('auction', {
      label: 'Auction',
      component: AuctionViewport,
      icon: IconGavel,
      description: 'Manage live auctions for collectible items'
    });

    console.log('[Auction Plugin] Started successfully');
  },

  async onStop() {
    console.log('[Auction Plugin] Stopping...');
  }
});
