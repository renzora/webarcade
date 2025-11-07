import { createSignal, onMount, Show, For } from 'solid-js';

const WEBARCADE_API = 'http://localhost:3001';

const PacksViewport = () => {
  const [packs, setPacks] = createSignal([]);
  const [items, setItems] = createSignal([]);
  const [loading, setLoading] = createSignal(false);

  // Pack form state
  const [newPackName, setNewPackName] = createSignal('');
  const [newPackPrice, setNewPackPrice] = createSignal(100);
  const [editingPack, setEditingPack] = createSignal(null);

  // Item form state
  const [newItemName, setNewItemName] = createSignal('');
  const [newItemRarity, setNewItemRarity] = createSignal('common');
  const [newItemValue, setNewItemValue] = createSignal(10);
  const [editingItem, setEditingItem] = createSignal(null);

  const rarities = [
    { value: 'common', label: 'Common', color: 'text-gray-400', weight: 50 },
    { value: 'uncommon', label: 'Uncommon', color: 'text-green-400', weight: 30 },
    { value: 'rare', label: 'Rare', color: 'text-blue-400', weight: 15 },
    { value: 'epic', label: 'Epic', color: 'text-purple-400', weight: 4 },
    { value: 'legendary', label: 'Legendary', color: 'text-orange-400', weight: 0.9 },
    { value: 'mythic', label: 'Mythic', color: 'text-red-400', weight: 0.1 }
  ];

  // Load packs
  const loadPacks = async () => {
    try {
      const response = await fetch(`${WEBARCADE_API}/packs`);
      const data = await response.json();
      setPacks(data);
    } catch (error) {
      console.error('Failed to load packs:', error);
    }
  };

  // Load items
  const loadItems = async () => {
    try {
      const response = await fetch(`${WEBARCADE_API}/packs/items`);
      const data = await response.json();
      setItems(data);
    } catch (error) {
      console.error('Failed to load items:', error);
    }
  };

  // Add pack
  const addPack = async () => {
    const name = newPackName().trim();
    if (!name) return;

    setLoading(true);
    try {
      await fetch(`${WEBARCADE_API}/packs`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          name,
          price: newPackPrice()
        })
      });
      setNewPackName('');
      setNewPackPrice(100);
      await loadPacks();
    } catch (error) {
      console.error('Failed to add pack:', error);
    }
    setLoading(false);
  };

  // Update pack
  const updatePack = async () => {
    const pack = editingPack();
    if (!pack) return;

    setLoading(true);
    try {
      await fetch(`${WEBARCADE_API}/packs`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(pack)
      });
      setEditingPack(null);
      await loadPacks();
    } catch (error) {
      console.error('Failed to update pack:', error);
    }
    setLoading(false);
  };

  // Delete pack
  const deletePack = async (id) => {
    if (!confirm('Delete this pack?')) return;

    setLoading(true);
    try {
      await fetch(`${WEBARCADE_API}/packs`, {
        method: 'DELETE',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ id })
      });
      await loadPacks();
    } catch (error) {
      console.error('Failed to delete pack:', error);
    }
    setLoading(false);
  };

  // Toggle pack enabled
  const togglePack = async (id) => {
    setLoading(true);
    try {
      await fetch(`${WEBARCADE_API}/packs/toggle`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ id })
      });
      await loadPacks();
    } catch (error) {
      console.error('Failed to toggle pack:', error);
    }
    setLoading(false);
  };

  // Add item
  const addItem = async () => {
    const name = newItemName().trim();
    if (!name) return;

    setLoading(true);
    try {
      await fetch(`${WEBARCADE_API}/packs/items`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          name,
          rarity: newItemRarity(),
          value: newItemValue()
        })
      });
      setNewItemName('');
      setNewItemRarity('common');
      setNewItemValue(10);
      await loadItems();
    } catch (error) {
      console.error('Failed to add item:', error);
    }
    setLoading(false);
  };

  // Update item
  const updateItem = async () => {
    const item = editingItem();
    if (!item) return;

    setLoading(true);
    try {
      await fetch(`${WEBARCADE_API}/packs/items`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(item)
      });
      setEditingItem(null);
      await loadItems();
    } catch (error) {
      console.error('Failed to update item:', error);
    }
    setLoading(false);
  };

  // Delete item
  const deleteItem = async (id) => {
    if (!confirm('Delete this item?')) return;

    setLoading(true);
    try {
      await fetch(`${WEBARCADE_API}/packs/items`, {
        method: 'DELETE',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ id })
      });
      await loadItems();
    } catch (error) {
      console.error('Failed to delete item:', error);
    }
    setLoading(false);
  };

  // Toggle item enabled
  const toggleItem = async (id) => {
    setLoading(true);
    try {
      await fetch(`${WEBARCADE_API}/packs/items/toggle`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ id })
      });
      await loadItems();
    } catch (error) {
      console.error('Failed to toggle item:', error);
    }
    setLoading(false);
  };

  // Seed sample packs and items
  const seedPacks = async () => {
    if (!confirm('This will add sample packs and items if none exist. Continue?')) return;

    setLoading(true);
    try {
      const response = await fetch(`${WEBARCADE_API}/packs/seed`, {
        method: 'POST'
      });
      const data = await response.json();
      alert(data.message || 'Seeded successfully!');
      await loadPacks();
      await loadItems();
    } catch (error) {
      console.error('Failed to seed packs:', error);
      alert('Failed to seed packs');
    }
    setLoading(false);
  };

  // Clear all packs and items
  const clearAllPacks = async () => {
    if (!confirm('⚠️ This will DELETE ALL packs, items, and user collections. This cannot be undone. Are you sure?')) return;

    setLoading(true);
    try {
      const response = await fetch(`${WEBARCADE_API}/packs/clear`, {
        method: 'DELETE'
      });
      const data = await response.json();
      alert(data.message || 'Cleared successfully!');
      await loadPacks();
      await loadItems();
    } catch (error) {
      console.error('Failed to clear packs:', error);
      alert('Failed to clear packs');
    }
    setLoading(false);
  };

  onMount(() => {
    loadPacks();
    loadItems();
  });

  const getRarityColor = (rarity) => {
    return rarities.find(r => r.value === rarity)?.color || 'text-gray-400';
  };

  return (
    <div class="p-6 pb-24 space-y-6 overflow-y-auto max-h-screen">
      {/* Packs Section */}
      <div class="bg-base-200 rounded-lg p-4 shadow">
        <h2 class="text-xl font-bold mb-4">📦 Packs</h2>

        {/* Add Pack Form */}
        <div class="mb-4 space-y-3">
          <div class="flex gap-2">
            <input
              type="text"
              value={newPackName()}
              onInput={(e) => setNewPackName(e.target.value)}
              placeholder="Pack name..."
              class="input input-bordered flex-1"
              disabled={loading()}
            />
            <input
              type="number"
              value={newPackPrice()}
              onInput={(e) => setNewPackPrice(parseInt(e.target.value))}
              placeholder="Price"
              class="input input-bordered w-32"
              min="1"
              disabled={loading()}
            />
            <button
              onClick={addPack}
              class="btn btn-primary"
              disabled={loading() || !newPackName().trim()}
            >
              Add Pack
            </button>
            <button
              onClick={seedPacks}
              class="btn btn-success"
              disabled={loading()}
              title="Add sample packs and items"
            >
              🌱 Seed Sample Data
            </button>
            <button
              onClick={clearAllPacks}
              class="btn btn-error"
              disabled={loading()}
              title="Delete all packs and items"
            >
              🗑️ Clear All
            </button>
          </div>
        </div>

        {/* Packs List */}
        <div class="space-y-2">
          <Show when={packs().length === 0}>
            <p class="text-gray-400 text-center py-4">
              No packs yet. Create one above!
            </p>
          </Show>

          <For each={packs()}>
            {(pack) => (
              <div class={`p-3 rounded flex items-center gap-3 ${pack.enabled ? 'bg-base-300' : 'bg-base-100 opacity-50'}`}>
                <input
                  type="checkbox"
                  checked={pack.enabled}
                  onChange={() => togglePack(pack.id)}
                  class="checkbox checkbox-primary"
                  disabled={loading()}
                />

                <Show
                  when={editingPack()?.id === pack.id}
                  fallback={
                    <div class="flex-1">
                      <div class="font-bold">{pack.name}</div>
                      <div class="text-sm text-gray-400">{pack.price} coins</div>
                    </div>
                  }
                >
                  <div class="flex-1 flex gap-2">
                    <input
                      type="text"
                      value={editingPack().name}
                      onInput={(e) => setEditingPack({ ...editingPack(), name: e.target.value })}
                      class="input input-bordered input-sm flex-1"
                      disabled={loading()}
                    />
                    <input
                      type="number"
                      value={editingPack().price}
                      onInput={(e) => setEditingPack({ ...editingPack(), price: parseInt(e.target.value) })}
                      class="input input-bordered input-sm w-24"
                      min="1"
                      disabled={loading()}
                    />
                  </div>
                </Show>

                <div class="flex gap-2">
                  <Show
                    when={editingPack()?.id === pack.id}
                    fallback={
                      <>
                        <button
                          onClick={() => setEditingPack({ ...pack })}
                          class="btn btn-sm btn-ghost"
                          disabled={loading()}
                        >
                          ✏️ Edit
                        </button>
                        <button
                          onClick={() => deletePack(pack.id)}
                          class="btn btn-sm btn-error btn-ghost"
                          disabled={loading()}
                        >
                          🗑️
                        </button>
                      </>
                    }
                  >
                    <button
                      onClick={updatePack}
                      class="btn btn-sm btn-success"
                      disabled={loading()}
                    >
                      💾 Save
                    </button>
                    <button
                      onClick={() => setEditingPack(null)}
                      class="btn btn-sm btn-ghost"
                      disabled={loading()}
                    >
                      ❌
                    </button>
                  </Show>
                </div>
              </div>
            )}
          </For>
        </div>
      </div>

      {/* Items Section */}
      <div class="bg-base-200 rounded-lg p-4 shadow">
        <h2 class="text-xl font-bold mb-4">✨ Collectible Items</h2>

        {/* Add Item Form */}
        <div class="mb-4 space-y-3">
          <div class="flex gap-2">
            <input
              type="text"
              value={newItemName()}
              onInput={(e) => setNewItemName(e.target.value)}
              placeholder="Item name..."
              class="input input-bordered flex-1"
              disabled={loading()}
            />
            <select
              value={newItemRarity()}
              onChange={(e) => setNewItemRarity(e.target.value)}
              class="select select-bordered w-40"
              disabled={loading()}
            >
              <For each={rarities}>
                {(rarity) => (
                  <option value={rarity.value}>{rarity.label}</option>
                )}
              </For>
            </select>
            <input
              type="number"
              value={newItemValue()}
              onInput={(e) => setNewItemValue(parseInt(e.target.value))}
              placeholder="Value"
              class="input input-bordered w-24"
              min="1"
              disabled={loading()}
            />
            <button
              onClick={addItem}
              class="btn btn-primary"
              disabled={loading() || !newItemName().trim()}
            >
              Add Item
            </button>
          </div>
        </div>

        {/* Rarity Info */}
        <div class="mb-4 p-3 bg-info/20 rounded text-sm">
          <div class="font-bold mb-2">Drop Rates:</div>
          <div class="grid grid-cols-2 gap-2">
            <For each={rarities}>
              {(rarity) => (
                <div class="flex justify-between">
                  <span class={rarity.color}>{rarity.label}:</span>
                  <span>{rarity.weight}%</span>
                </div>
              )}
            </For>
          </div>
        </div>

        {/* Items List */}
        <div class="space-y-2">
          <Show when={items().length === 0}>
            <p class="text-gray-400 text-center py-4">
              No items yet. Create one above!
            </p>
          </Show>

          <For each={items()}>
            {(item) => (
              <div class={`p-3 rounded flex items-center gap-3 ${item.enabled ? 'bg-base-300' : 'bg-base-100 opacity-50'}`}>
                <input
                  type="checkbox"
                  checked={item.enabled}
                  onChange={() => toggleItem(item.id)}
                  class="checkbox checkbox-primary"
                  disabled={loading()}
                />

                <Show
                  when={editingItem()?.id === item.id}
                  fallback={
                    <div class="flex-1">
                      <div class="flex items-center gap-2">
                        <span class="font-bold">{item.name}</span>
                        <span class={`text-sm ${getRarityColor(item.rarity)}`}>
                          [{item.rarity}]
                        </span>
                      </div>
                      <div class="text-sm text-gray-400">Value: {item.value} coins</div>
                    </div>
                  }
                >
                  <div class="flex-1 flex gap-2">
                    <input
                      type="text"
                      value={editingItem().name}
                      onInput={(e) => setEditingItem({ ...editingItem(), name: e.target.value })}
                      class="input input-bordered input-sm flex-1"
                      disabled={loading()}
                    />
                    <select
                      value={editingItem().rarity}
                      onChange={(e) => setEditingItem({ ...editingItem(), rarity: e.target.value })}
                      class="select select-bordered select-sm w-32"
                      disabled={loading()}
                    >
                      <For each={rarities}>
                        {(rarity) => (
                          <option value={rarity.value}>{rarity.label}</option>
                        )}
                      </For>
                    </select>
                    <input
                      type="number"
                      value={editingItem().value}
                      onInput={(e) => setEditingItem({ ...editingItem(), value: parseInt(e.target.value) })}
                      class="input input-bordered input-sm w-24"
                      min="1"
                      disabled={loading()}
                    />
                  </div>
                </Show>

                <div class="flex gap-2">
                  <Show
                    when={editingItem()?.id === item.id}
                    fallback={
                      <>
                        <button
                          onClick={() => setEditingItem({ ...item })}
                          class="btn btn-sm btn-ghost"
                          disabled={loading()}
                        >
                          ✏️ Edit
                        </button>
                        <button
                          onClick={() => deleteItem(item.id)}
                          class="btn btn-sm btn-error btn-ghost"
                          disabled={loading()}
                        >
                          🗑️
                        </button>
                      </>
                    }
                  >
                    <button
                      onClick={updateItem}
                      class="btn btn-sm btn-success"
                      disabled={loading()}
                    >
                      💾 Save
                    </button>
                    <button
                      onClick={() => setEditingItem(null)}
                      class="btn btn-sm btn-ghost"
                      disabled={loading()}
                    >
                      ❌
                    </button>
                  </Show>
                </div>
              </div>
            )}
          </For>
        </div>

        <div class="mt-4 p-3 bg-warning/20 rounded text-sm">
          <p><strong>💡 Commands:</strong></p>
          <ul class="list-disc list-inside space-y-1 mt-2">
            <li>!buypack &lt;pack_name&gt; - Purchase a pack</li>
            <li>!packs - View available packs</li>
            <li>!openpack &lt;pack_name&gt; - Open a pack you own</li>
            <li>!items - View your collection</li>
          </ul>
        </div>
      </div>
    </div>
  );
};

export default PacksViewport;
