import { createSignal, createEffect, onCleanup, For, Show } from 'solid-js';

const BRIDGE_URL = 'http://localhost:3001';

export default function AuctionViewport() {
  // Auction state
  const [items, setItems] = createSignal([]);
  const [currentAuction, setCurrentAuction] = createSignal(null);
  const [isAuctionActive, setIsAuctionActive] = createSignal(false);
  const [timeRemaining, setTimeRemaining] = createSignal(0);
  const [bids, setBids] = createSignal([]);

  // Form state for adding new items
  const [newItem, setNewItem] = createSignal({
    name: '',
    description: '',
    image: '',
    rarity: 'common',
    startingBid: 10
  });

  // Form state for manual bidding (for testing)
  const [testBid, setTestBid] = createSignal({
    bidder: '',
    amount: 0
  });

  let timerInterval = null;

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

  const broadcastAuctionState = async (auction = null, ended = false) => {
    try {
      const state = auction || currentAuction();
      await fetch(`${BRIDGE_URL}/auction/broadcast`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          type: ended ? 'auction_ended' : 'auction_state',
          auction: state
        })
      });
    } catch (error) {
      console.error('Failed to broadcast auction state:', error);
    }
  };

  const broadcastBid = async (bidder, amount) => {
    try {
      await fetch(`${BRIDGE_URL}/auction/broadcast`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          type: 'auction_bid',
          bidder,
          amount
        })
      });
    } catch (error) {
      console.error('Failed to broadcast bid:', error);
    }
  };

  const addItem = () => {
    const item = newItem();
    if (!item.name || item.startingBid <= 0) {
      alert('Please fill in all required fields');
      return;
    }

    const newItemWithId = {
      id: Date.now(),
      ...item,
      addedAt: new Date().toISOString()
    };

    setItems(prev => [...prev, newItemWithId]);

    // Reset form
    setNewItem({
      name: '',
      description: '',
      image: '',
      rarity: 'common',
      startingBid: 10
    });
  };

  const removeItem = (id) => {
    setItems(prev => prev.filter(item => item.id !== id));
  };

  const startAuction = (item, duration = 60) => {
    if (isAuctionActive()) {
      alert('An auction is already active');
      return;
    }

    const auction = {
      item,
      startingBid: item.startingBid,
      currentBid: null,
      currentBidder: null,
      duration,
      startedAt: Date.now(),
      active: true,
      timeRemaining: duration
    };

    setCurrentAuction(auction);
    setIsAuctionActive(true);
    setTimeRemaining(duration);
    setBids([]);

    // Broadcast initial state
    broadcastAuctionState(auction);

    // Start countdown
    timerInterval = setInterval(() => {
      setTimeRemaining(prev => {
        const newTime = prev - 1;

        if (newTime <= 0) {
          endAuction();
          return 0;
        }

        // Update auction object with new time
        setCurrentAuction(prev => ({
          ...prev,
          timeRemaining: newTime
        }));

        // Broadcast updated time every 5 seconds
        if (newTime % 5 === 0) {
          broadcastAuctionState();
        }

        return newTime;
      });
    }, 1000);

    // Remove item from available items
    removeItem(item.id);
  };

  const placeBid = (bidder, amount) => {
    const auction = currentAuction();
    if (!auction || !auction.active) {
      alert('No active auction');
      return;
    }

    const currentBidAmount = auction.currentBid || auction.startingBid;
    if (amount <= currentBidAmount) {
      alert(`Bid must be higher than ${formatCurrency(currentBidAmount)}`);
      return;
    }

    // Update auction with new bid
    const updatedAuction = {
      ...auction,
      currentBid: amount,
      currentBidder: bidder
    };

    setCurrentAuction(updatedAuction);

    // Add to bid history
    setBids(prev => [{
      bidder,
      amount,
      timestamp: new Date().toISOString()
    }, ...prev]);

    // Broadcast bid
    broadcastBid(bidder, amount);
    broadcastAuctionState(updatedAuction);
  };

  const testPlaceBid = () => {
    const bid = testBid();
    if (!bid.bidder || bid.amount <= 0) {
      alert('Please enter valid bidder name and amount');
      return;
    }

    placeBid(bid.bidder, bid.amount);

    // Reset test form
    setTestBid({ bidder: '', amount: 0 });
  };

  const endAuction = () => {
    if (timerInterval) {
      clearInterval(timerInterval);
      timerInterval = null;
    }

    const auction = currentAuction();
    if (auction) {
      // Mark as ended
      const endedAuction = {
        ...auction,
        active: false,
        endedAt: Date.now()
      };

      setCurrentAuction(endedAuction);
      broadcastAuctionState(endedAuction, true);

      // Show results
      if (auction.currentBidder) {
        alert(`Auction ended! Winner: ${auction.currentBidder} with ${formatCurrency(auction.currentBid)}`);
      } else {
        alert('Auction ended with no bids');
      }
    }

    setTimeout(() => {
      setIsAuctionActive(false);
      setCurrentAuction(null);
      setTimeRemaining(0);
    }, 2000);
  };

  const cancelAuction = () => {
    if (confirm('Are you sure you want to cancel this auction?')) {
      const auction = currentAuction();

      // Return item to list
      if (auction && auction.item) {
        setItems(prev => [...prev, auction.item]);
      }

      endAuction();
    }
  };

  // Cleanup on unmount
  onCleanup(() => {
    if (timerInterval) {
      clearInterval(timerInterval);
    }
  });

  return (
    <div class="h-full flex flex-col bg-base-200 p-6 overflow-y-auto">
      <div class="flex-1 max-w-6xl mx-auto w-full">
        {/* Header */}
        <div class="mb-6">
          <h2 class="text-2xl font-bold mb-2">Auction System</h2>
          <p class="text-base-content/60">Manage live auctions for collectible items</p>
        </div>

        {/* Active Auction */}
        <Show when={isAuctionActive() && currentAuction()}>
          <div class="card bg-gradient-to-br from-yellow-500 to-orange-500 text-white shadow-xl mb-6">
            <div class="card-body">
              <div class="flex items-center justify-between mb-4">
                <h3 class="card-title text-2xl">Active Auction</h3>
                <div class="badge badge-lg badge-error font-mono text-xl">
                  {formatTime(timeRemaining())}
                </div>
              </div>

              <div class="bg-white/10 rounded-lg p-4 mb-4">
                <div class="flex gap-4 items-start">
                  <Show when={currentAuction().item?.image}>
                    <img
                      src={currentAuction().item.image}
                      alt={currentAuction().item.name}
                      class="w-24 h-24 object-cover rounded-lg"
                    />
                  </Show>
                  <div class="flex-1">
                    <h4 class="text-xl font-bold mb-1">{currentAuction().item?.name}</h4>
                    <p class="text-sm opacity-90 mb-2">{currentAuction().item?.description}</p>
                    <span class="badge badge-sm">{currentAuction().item?.rarity?.toUpperCase()}</span>
                  </div>
                </div>
              </div>

              <div class="stats shadow bg-white/10">
                <div class="stat">
                  <div class="stat-title text-white/70">Current Bid</div>
                  <div class="stat-value text-white">
                    {formatCurrency(currentAuction().currentBid || currentAuction().startingBid)}
                  </div>
                  <div class="stat-desc text-white/70">
                    {currentAuction().currentBidder || 'No bids yet'}
                  </div>
                </div>
                <div class="stat">
                  <div class="stat-title text-white/70">Total Bids</div>
                  <div class="stat-value text-white">{bids().length}</div>
                  <div class="stat-desc text-white/70">
                    {bids().length > 0 ? 'Bidding active' : 'Awaiting bids'}
                  </div>
                </div>
              </div>

              <div class="flex gap-2 mt-4">
                <button class="btn btn-error flex-1" onClick={endAuction}>
                  End Now
                </button>
                <button class="btn btn-ghost flex-1" onClick={cancelAuction}>
                  Cancel
                </button>
              </div>
            </div>
          </div>

          {/* Test Bidding */}
          <div class="card bg-base-100 shadow-xl mb-6">
            <div class="card-body">
              <h3 class="card-title">Test Bidding</h3>
              <p class="text-sm text-base-content/60">Place test bids to see how they appear on the overlay</p>

              <div class="grid grid-cols-2 gap-4">
                <div class="form-control">
                  <label class="label">
                    <span class="label-text">Bidder Name</span>
                  </label>
                  <input
                    type="text"
                    placeholder="e.g., User123"
                    class="input input-bordered"
                    value={testBid().bidder}
                    onInput={(e) => setTestBid(prev => ({ ...prev, bidder: e.target.value }))}
                  />
                </div>
                <div class="form-control">
                  <label class="label">
                    <span class="label-text">Bid Amount ($)</span>
                  </label>
                  <input
                    type="number"
                    min={(currentAuction().currentBid || currentAuction().startingBid) + 1}
                    step="0.01"
                    class="input input-bordered"
                    value={testBid().amount}
                    onInput={(e) => setTestBid(prev => ({ ...prev, amount: parseFloat(e.target.value) || 0 }))}
                  />
                </div>
              </div>

              <button class="btn btn-primary" onClick={testPlaceBid}>
                Place Test Bid
              </button>
            </div>
          </div>

          {/* Bid History */}
          <Show when={bids().length > 0}>
            <div class="card bg-base-100 shadow-xl mb-6">
              <div class="card-body">
                <h3 class="card-title">Bid History</h3>
                <div class="overflow-x-auto">
                  <table class="table table-zebra">
                    <thead>
                      <tr>
                        <th>Time</th>
                        <th>Bidder</th>
                        <th>Amount</th>
                      </tr>
                    </thead>
                    <tbody>
                      <For each={bids()}>
                        {(bid) => (
                          <tr>
                            <td>{new Date(bid.timestamp).toLocaleTimeString()}</td>
                            <td>{bid.bidder}</td>
                            <td class="font-bold">{formatCurrency(bid.amount)}</td>
                          </tr>
                        )}
                      </For>
                    </tbody>
                  </table>
                </div>
              </div>
            </div>
          </Show>
        </Show>

        {/* Add New Item */}
        <Show when={!isAuctionActive()}>
          <div class="card bg-base-100 shadow-xl mb-6">
            <div class="card-body">
              <h3 class="card-title">Add Auction Item</h3>

              <div class="grid grid-cols-2 gap-4">
                <div class="form-control col-span-2">
                  <label class="label">
                    <span class="label-text">Item Name</span>
                  </label>
                  <input
                    type="text"
                    placeholder="e.g., Rare Trading Card"
                    class="input input-bordered"
                    value={newItem().name}
                    onInput={(e) => setNewItem(prev => ({ ...prev, name: e.target.value }))}
                  />
                </div>

                <div class="form-control col-span-2">
                  <label class="label">
                    <span class="label-text">Description</span>
                  </label>
                  <textarea
                    placeholder="Item description"
                    class="textarea textarea-bordered"
                    value={newItem().description}
                    onInput={(e) => setNewItem(prev => ({ ...prev, description: e.target.value }))}
                  />
                </div>

                <div class="form-control col-span-2">
                  <label class="label">
                    <span class="label-text">Image URL (optional)</span>
                  </label>
                  <input
                    type="text"
                    placeholder="https://example.com/image.jpg"
                    class="input input-bordered"
                    value={newItem().image}
                    onInput={(e) => setNewItem(prev => ({ ...prev, image: e.target.value }))}
                  />
                </div>

                <div class="form-control">
                  <label class="label">
                    <span class="label-text">Rarity</span>
                  </label>
                  <select
                    class="select select-bordered"
                    value={newItem().rarity}
                    onChange={(e) => setNewItem(prev => ({ ...prev, rarity: e.target.value }))}
                  >
                    <option value="common">Common</option>
                    <option value="uncommon">Uncommon</option>
                    <option value="rare">Rare</option>
                    <option value="epic">Epic</option>
                    <option value="legendary">Legendary</option>
                  </select>
                </div>

                <div class="form-control">
                  <label class="label">
                    <span class="label-text">Starting Bid ($)</span>
                  </label>
                  <input
                    type="number"
                    min="0.01"
                    step="0.01"
                    class="input input-bordered"
                    value={newItem().startingBid}
                    onInput={(e) => setNewItem(prev => ({ ...prev, startingBid: parseFloat(e.target.value) || 0 }))}
                  />
                </div>
              </div>

              <button class="btn btn-primary" onClick={addItem}>
                Add Item
              </button>
            </div>
          </div>

          {/* Available Items */}
          <div class="card bg-base-100 shadow-xl">
            <div class="card-body">
              <h3 class="card-title">Available Items ({items().length})</h3>

              <Show
                when={items().length > 0}
                fallback={
                  <div class="text-center py-8 text-base-content/60">
                    No items available. Add items above to start an auction.
                  </div>
                }
              >
                <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                  <For each={items()}>
                    {(item) => (
                      <div class="card bg-base-200 shadow">
                        <div class="card-body">
                          <div class="flex gap-3">
                            <Show when={item.image}>
                              <img
                                src={item.image}
                                alt={item.name}
                                class="w-16 h-16 object-cover rounded"
                              />
                            </Show>
                            <div class="flex-1 min-w-0">
                              <h4 class="font-bold truncate">{item.name}</h4>
                              <p class="text-sm text-base-content/60 line-clamp-2">{item.description}</p>
                              <div class="flex gap-2 mt-2">
                                <span class="badge badge-sm">{item.rarity}</span>
                                <span class="badge badge-sm badge-primary">{formatCurrency(item.startingBid)}</span>
                              </div>
                            </div>
                          </div>
                          <div class="card-actions justify-end mt-2">
                            <button
                              class="btn btn-sm btn-primary"
                              onClick={() => startAuction(item, 60)}
                            >
                              Start (60s)
                            </button>
                            <button
                              class="btn btn-sm btn-secondary"
                              onClick={() => startAuction(item, 120)}
                            >
                              Start (2min)
                            </button>
                            <button
                              class="btn btn-sm btn-ghost"
                              onClick={() => removeItem(item.id)}
                            >
                              Remove
                            </button>
                          </div>
                        </div>
                      </div>
                    )}
                  </For>
                </div>
              </Show>
            </div>
          </div>
        </Show>

        {/* Instructions */}
        <div class="alert alert-info mt-6">
          <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" class="stroke-current shrink-0 w-6 h-6">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path>
          </svg>
          <div>
            <h3 class="font-bold">Auction Overlay</h3>
            <div class="text-xs">
              Add auction overlay in OBS: <code class="bg-base-300 px-2 py-0.5 rounded">http://localhost:3001/overlay/auction</code>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
