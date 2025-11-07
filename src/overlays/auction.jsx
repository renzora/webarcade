import { createSignal, createEffect, onCleanup, Show, For } from 'solid-js';
import { WEBARCADE_WS } from '@/api/bridge';
import '@/index.css';

function Auction() {
  const [auctionState, setAuctionState] = createSignal(null);
  const [timeRemaining, setTimeRemaining] = createSignal(0);
  const [recentBids, setRecentBids] = createSignal([]);
  const [isVisible, setIsVisible] = createSignal(false);
  let ws = null;
  let timerInterval = null;

  const connectWebSocket = () => {
    ws = new WebSocket(WEBARCADE_WS);

    ws.onopen = () => {
      console.log('[Auction Overlay] Connected to WebSocket');
    };

    ws.onmessage = (event) => {
      const data = JSON.parse(event.data);

      if (data.type === 'auction_state') {
        console.log('[Auction Overlay] Received auction state:', data);
        setAuctionState(data.auction);
        setIsVisible(data.auction?.active || false);

        if (data.auction?.active) {
          setTimeRemaining(data.auction.timeRemaining || 0);
        }
      } else if (data.type === 'auction_bid') {
        console.log('[Auction Overlay] New bid:', data);

        // Update auction state with new highest bid
        const current = auctionState();
        if (current) {
          setAuctionState({
            ...current,
            currentBid: data.amount,
            currentBidder: data.bidder
          });
        }

        // Add to recent bids list with animation
        const bid = {
          id: Date.now(),
          bidder: data.bidder,
          amount: data.amount,
          timestamp: Date.now()
        };

        setRecentBids(prev => [bid, ...prev.slice(0, 4)]);

        // Remove bid animation after 3 seconds
        setTimeout(() => {
          setRecentBids(prev => prev.filter(b => b.id !== bid.id));
        }, 3000);
      } else if (data.type === 'auction_ended') {
        console.log('[Auction Overlay] Auction ended');
        setTimeout(() => {
          setIsVisible(false);
          setAuctionState(null);
          setRecentBids([]);
        }, 5000); // Show final state for 5 seconds
      }
    };

    ws.onerror = (error) => {
      console.error('[Auction Overlay] WebSocket error:', error);
    };

    ws.onclose = () => {
      console.log('[Auction Overlay] Disconnected, reconnecting...');
      setTimeout(connectWebSocket, 3000);
    };
  };

  createEffect(() => {
    connectWebSocket();

    // Timer countdown
    timerInterval = setInterval(() => {
      const current = auctionState();
      if (current?.active && timeRemaining() > 0) {
        setTimeRemaining(prev => Math.max(0, prev - 1));
      }
    }, 1000);

    onCleanup(() => {
      if (ws) ws.close();
      if (timerInterval) clearInterval(timerInterval);
    });
  });

  const formatTime = (seconds) => {
    const mins = Math.floor(seconds / 60);
    const secs = seconds % 60;
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  };

  const formatCurrency = (amount) => {
    return new Intl.NumberFormat('en-US', {
      style: 'currency',
      currency: 'USD'
    }).format(amount);
  };

  const getRarityColor = (rarity) => {
    const colors = {
      common: 'bg-gray-500',
      uncommon: 'bg-green-500',
      rare: 'bg-blue-500',
      epic: 'bg-purple-500',
      legendary: 'bg-orange-500'
    };
    return colors[rarity?.toLowerCase()] || 'bg-gray-500';
  };

  return (
    <div class="fixed inset-0 w-screen h-screen pointer-events-none overflow-hidden bg-transparent">
      <Show when={isVisible() && auctionState()}>
        <div class="absolute bottom-20 right-20 w-[500px] animate-fade-in">
          {/* Main Auction Display */}
          <div class="bg-gradient-to-br from-gray-900 via-gray-800 to-gray-900 rounded-2xl shadow-2xl border-2 border-yellow-500/50 overflow-hidden backdrop-blur-lg">
            {/* Header */}
            <div class="bg-gradient-to-r from-yellow-600 via-yellow-500 to-yellow-600 px-6 py-3 flex items-center justify-between">
              <h2 class="text-2xl font-bold text-white drop-shadow-lg flex items-center gap-2">
                <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                  <path d="M12 2L2 7l10 5 10-5-10-5z"/>
                  <path d="M2 17l10 5 10-5"/>
                  <path d="M2 12l10 5 10-5"/>
                </svg>
                Live Auction
              </h2>
              <div
                class="text-xl font-mono font-bold px-4 py-1 rounded-full"
                classList={{
                  'bg-red-500 text-white animate-pulse': timeRemaining() < 30,
                  'bg-white/20 text-white': timeRemaining() >= 30
                }}
              >
                {formatTime(timeRemaining())}
              </div>
            </div>

            {/* Item Display */}
            <div class="p-6 space-y-4">
              <div class="flex gap-4 items-start">
                {/* Item Image */}
                <div class="flex-shrink-0">
                  <Show
                    when={auctionState().item?.image}
                    fallback={
                      <div class="w-32 h-32 bg-gray-700/50 rounded-lg flex items-center justify-center border-2 border-gray-600">
                        <svg xmlns="http://www.w3.org/2000/svg" width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="text-gray-500">
                          <rect x="2" y="2" width="20" height="20" rx="5" ry="5"></rect>
                          <path d="M16 11.37A4 4 0 1 1 12.63 8 4 4 0 0 1 16 11.37z"></path>
                          <line x1="17.5" y1="6.5" x2="17.51" y2="6.5"></line>
                        </svg>
                      </div>
                    }
                  >
                    <img
                      src={auctionState().item?.image}
                      alt={auctionState().item?.name}
                      class="w-32 h-32 object-cover rounded-lg border-2 border-gray-600 shadow-lg"
                    />
                  </Show>
                </div>

                {/* Item Details */}
                <div class="flex-1 min-w-0">
                  <h3 class="text-2xl font-bold text-white mb-1 truncate">{auctionState().item?.name}</h3>
                  <p class="text-sm text-gray-300 mb-2 line-clamp-2">{auctionState().item?.description}</p>
                  <span
                    class={`inline-block px-3 py-1 rounded-full text-xs font-bold text-white uppercase ${getRarityColor(auctionState().item?.rarity)}`}
                  >
                    {auctionState().item?.rarity || 'Common'}
                  </span>
                </div>
              </div>

              {/* Bid Information */}
              <div class="bg-gray-800/50 rounded-lg p-4 border border-gray-700">
                <div class="flex items-baseline justify-between mb-2">
                  <span class="text-sm text-gray-400 uppercase tracking-wide">Current Bid</span>
                  <span class="text-3xl font-bold text-yellow-400 drop-shadow-lg">
                    {formatCurrency(auctionState().currentBid || auctionState().startingBid)}
                  </span>
                </div>

                <Show when={auctionState().currentBidder}>
                  <div class="flex items-center justify-between pt-2 border-t border-gray-700">
                    <span class="text-xs text-gray-400 uppercase">Leading Bidder</span>
                    <span class="text-lg font-semibold text-green-400">{auctionState().currentBidder}</span>
                  </div>
                </Show>

                <Show when={!auctionState().currentBidder}>
                  <div class="text-center pt-2 border-t border-gray-700">
                    <span class="text-sm text-gray-400">No bids yet • Starting at {formatCurrency(auctionState().startingBid)}</span>
                  </div>
                </Show>
              </div>
            </div>
          </div>

          {/* Recent Bids Feed */}
          <Show when={recentBids().length > 0}>
            <div class="mt-4 space-y-2">
              <For each={recentBids()}>
                {(bid) => (
                  <div class="bg-green-500/90 backdrop-blur-sm rounded-lg px-4 py-2 border-2 border-green-400 shadow-lg animate-slide-in-right flex items-center justify-between">
                    <span class="font-bold text-white flex items-center gap-2">
                      <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <polyline points="20 6 9 17 4 12"/>
                      </svg>
                      {bid.bidder}
                    </span>
                    <span class="text-xl font-bold text-white">{formatCurrency(bid.amount)}</span>
                  </div>
                )}
              </For>
            </div>
          </Show>
        </div>
      </Show>

      <style>{`
        @keyframes fade-in {
          from { opacity: 0; transform: translateY(20px); }
          to { opacity: 1; transform: translateY(0); }
        }
        @keyframes slide-in-right {
          from { opacity: 0; transform: translateX(100px); }
          to { opacity: 1; transform: translateX(0); }
        }
        .animate-fade-in {
          animation: fade-in 0.5s ease-out;
        }
        .animate-slide-in-right {
          animation: slide-in-right 0.3s ease-out;
        }
        .line-clamp-2 {
          display: -webkit-box;
          -webkit-line-clamp: 2;
          -webkit-box-orient: vertical;
          overflow: hidden;
        }
      `}</style>
    </div>
  );
}

export default Auction;
