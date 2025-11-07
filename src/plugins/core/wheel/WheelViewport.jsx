import { createSignal, onMount, For, Show } from 'solid-js';
import twitchStore from '../twitch/TwitchStore.jsx';
import { bridgeFetch } from '@/api/bridge.js';
import { IconWheel, IconPlus, IconTrash, IconEdit, IconAlertCircle } from '@tabler/icons-solidjs';

export default function WheelViewport() {
  const [options, setOptions] = createSignal([]);
  const [loading, setLoading] = createSignal(true);
  const [selectedChannel, setSelectedChannel] = createSignal('');
  const [status, setStatus] = createSignal({ status: 'disconnected', connected_channels: [] });

  // Form state
  const [newOptionText, setNewOptionText] = createSignal('');
  const [newOptionColor, setNewOptionColor] = createSignal('#9146FF');
  const [newOptionWeight, setNewOptionWeight] = createSignal(1);
  const [newOptionPercentage, setNewOptionPercentage] = createSignal('');
  const [usePercentage, setUsePercentage] = createSignal(false);
  const [newPrizeType, setNewPrizeType] = createSignal('');
  const [newPrizeData, setNewPrizeData] = createSignal('');

  // Edit state
  const [editingId, setEditingId] = createSignal(null);
  const [editText, setEditText] = createSignal('');
  const [editColor, setEditColor] = createSignal('#9146FF');
  const [editWeight, setEditWeight] = createSignal(1);
  const [editPercentage, setEditPercentage] = createSignal('');
  const [editUsePercentage, setEditUsePercentage] = createSignal(false);
  const [editPrizeType, setEditPrizeType] = createSignal('');
  const [editPrizeData, setEditPrizeData] = createSignal('');

  // Spin state
  const [isSpinning, setIsSpinning] = createSignal(false);

  const colorPresets = [
    '#FF0000', '#FF7F00', '#FFFF00', '#00FF00', '#0000FF',
    '#4B0082', '#9400D3', '#FF1493', '#00CED1', '#FFD700'
  ];

  onMount(async () => {
    const currentStatus = await twitchStore.fetchStatus();
    if (currentStatus) {
      setStatus({ ...currentStatus, connected_channels: currentStatus.connected_channels || [] });
      if (currentStatus.connected_channels && currentStatus.connected_channels.length > 0) {
        setSelectedChannel(currentStatus.connected_channels[0]);
        await loadOptions(currentStatus.connected_channels[0]);
      }
    }
    setLoading(false);
  });

  const loadOptions = async (channel) => {
    if (!channel) return;

    try {
      setLoading(true);
      const response = await bridgeFetch(`/wheel/options?channel=${channel}`);
      const data = await response.json();
      setOptions(data);
    } catch (e) {
      console.error('Failed to load wheel options:', e);
    } finally {
      setLoading(false);
    }
  };

  const handleChannelChange = async (channel) => {
    setSelectedChannel(channel);
    await loadOptions(channel);
  };

  const addOption = async () => {
    const text = newOptionText().trim();
    if (!text) return;

    try {
      const payload = {
        channel: selectedChannel(),
        option_text: text,
        color: newOptionColor(),
      };

      // Add either percentage or weight
      if (usePercentage() && newOptionPercentage()) {
        payload.chance_percentage = parseFloat(newOptionPercentage());
        payload.weight = 1; // Default weight when using percentage
      } else {
        payload.weight = newOptionWeight();
      }

      // Add prize info if specified
      if (newPrizeType()) {
        payload.prize_type = newPrizeType();
        payload.prize_data = newPrizeData();
      }

      const response = await bridgeFetch('/wheel/options', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(payload),
      });

      if (response.ok) {
        setNewOptionText('');
        setNewOptionColor('#9146FF');
        setNewOptionWeight(1);
        setNewOptionPercentage('');
        setUsePercentage(false);
        setNewPrizeType('');
        setNewPrizeData('');
        await loadOptions(selectedChannel());
      }
    } catch (e) {
      console.error('Failed to add wheel option:', e);
    }
  };

  const deleteOption = async (id) => {
    if (!confirm('Delete this wheel option?')) return;

    try {
      const response = await bridgeFetch(`/wheel/options/${id}`, {
        method: 'DELETE',
      });

      if (response.ok) {
        await loadOptions(selectedChannel());
      }
    } catch (e) {
      console.error('Failed to delete wheel option:', e);
    }
  };

  const startEdit = (option) => {
    setEditingId(option.id);
    setEditText(option.option_text);
    setEditColor(option.color);
    setEditWeight(option.weight);
    setEditPercentage(option.chance_percentage ? option.chance_percentage.toString() : '');
    setEditUsePercentage(!!option.chance_percentage);
    setEditPrizeType(option.prize_type || '');
    setEditPrizeData(option.prize_data || '');
  };

  const cancelEdit = () => {
    setEditingId(null);
    setEditText('');
    setEditColor('#9146FF');
    setEditWeight(1);
    setEditPercentage('');
    setEditUsePercentage(false);
    setEditPrizeType('');
    setEditPrizeData('');
  };

  const saveEdit = async () => {
    const id = editingId();
    if (!id) return;

    const text = editText().trim();
    if (!text) return;

    try {
      const payload = {
        id: id,
        option_text: text,
        color: editColor(),
      };

      // Add either percentage or weight
      if (editUsePercentage() && editPercentage()) {
        payload.chance_percentage = parseFloat(editPercentage());
        payload.weight = 1; // Default weight when using percentage
      } else {
        payload.weight = editWeight();
        payload.chance_percentage = null; // Clear percentage if using weight
      }

      // Add prize info if specified
      if (editPrizeType()) {
        payload.prize_type = editPrizeType();
        payload.prize_data = editPrizeData();
      } else {
        payload.prize_type = null;
        payload.prize_data = null;
      }

      const response = await bridgeFetch(`/wheel/options/${id}`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(payload),
      });

      if (response.ok) {
        cancelEdit();
        await loadOptions(selectedChannel());
      }
    } catch (e) {
      console.error('Failed to update wheel option:', e);
    }
  };

  const toggleEnabled = async (id, currentEnabled) => {
    try {
      const response = await bridgeFetch(`/wheel/options/${id}/toggle`, {
        method: 'POST',
      });

      if (response.ok) {
        await loadOptions(selectedChannel());
      }
    } catch (e) {
      console.error('Failed to toggle wheel option:', e);
    }
  };

  const spinWheel = async () => {
    console.log('Spin button clicked');
    if (isSpinning()) {
      console.log('Already spinning, ignoring click');
      return;
    }

    const enabledOptions = options().filter(opt => opt.enabled === 1);
    console.log('Enabled options:', enabledOptions.length);

    if (enabledOptions.length === 0) {
      alert('Add at least one enabled wheel option first!');
      return;
    }

    try {
      setIsSpinning(true);
      console.log('Sending spin request to:', '/wheel/spin', 'for channel:', selectedChannel());

      const response = await bridgeFetch('/wheel/spin', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          channel: selectedChannel(),
        }),
      });

      console.log('Spin response status:', response.status);

      if (response.ok) {
        const data = await response.json();
        console.log('Wheel spin result:', data);

        let message = `Winner: ${data.winner}! 🎉`;
        if (data.prize_type) {
          message += `\nPrize: ${data.prize_type}`;
          if (data.prize_data) {
            message += ` (${data.prize_data})`;
          }
        }
        alert(message);
      } else {
        const errorText = await response.text();
        console.error('Spin request failed:', response.status, errorText);
        alert(`Spin failed: ${errorText}`);
      }

      // Re-enable button after 5 seconds (spin + winner display time)
      setTimeout(() => {
        setIsSpinning(false);
      }, 5000);
    } catch (e) {
      console.error('Failed to spin wheel:', e);
      alert(`Error: ${e.message}`);
      setIsSpinning(false);
    }
  };

  return (
    <div class="h-full flex flex-col bg-base-200">
      {/* Header */}
      <div class="flex items-center justify-between bg-base-100 border-b border-base-300 px-4 py-3">
        <div class="flex items-center gap-3 flex-1">
          <IconWheel size={20} class="text-primary" />
          <h2 class="text-lg font-semibold">Wheel Options</h2>
        </div>

        <div class="flex items-center gap-2">
          <Show when={status().connected_channels?.length > 0}>
            <button
              class={`btn btn-primary btn-sm ${isSpinning() ? 'loading' : ''}`}
              onClick={spinWheel}
              disabled={isSpinning() || options().filter(opt => opt.enabled === 1).length === 0}
            >
              <IconWheel size={16} class={isSpinning() ? 'animate-spin' : ''} />
              {isSpinning() ? 'Spinning...' : 'Spin Wheel'}
            </button>
            <select
              class="select select-bordered select-sm"
              value={selectedChannel()}
              onChange={(e) => handleChannelChange(e.target.value)}
            >
              {status().connected_channels?.map((channel) => (
                <option value={channel}>#{channel}</option>
              ))}
            </select>
          </Show>
        </div>
      </div>

      {/* Add Option Form */}
      <div class="p-4 bg-base-100 border-b border-base-300">
        <div class="flex flex-col gap-3">
          <div class="flex gap-2">
            <input
              type="text"
              placeholder="Option text..."
              class="input input-bordered input-sm flex-1"
              value={newOptionText()}
              onInput={(e) => setNewOptionText(e.target.value)}
              onKeyPress={(e) => e.key === 'Enter' && addOption()}
            />
            <input
              type="color"
              class="w-12 h-8 rounded cursor-pointer border border-base-300"
              value={newOptionColor()}
              onInput={(e) => setNewOptionColor(e.target.value)}
              title="Pick color"
            />
            <label class="swap swap-flip">
              <input type="checkbox" checked={usePercentage()} onChange={(e) => setUsePercentage(e.target.checked)} />
              <div class="swap-on">
                <input
                  type="number"
                  min="0.01"
                  max="100"
                  step="0.1"
                  class="input input-bordered input-sm w-24"
                  value={newOptionPercentage()}
                  onInput={(e) => setNewOptionPercentage(e.target.value)}
                  placeholder="%"
                  title="Percentage chance (0-100)"
                />
              </div>
              <div class="swap-off">
                <input
                  type="number"
                  min="1"
                  max="100"
                  class="input input-bordered input-sm w-20"
                  value={newOptionWeight()}
                  onInput={(e) => setNewOptionWeight(parseInt(e.target.value) || 1)}
                  title="Weight (probability)"
                  placeholder="Weight"
                />
              </div>
            </label>
            <button
              class="btn btn-xs btn-ghost"
              onClick={() => setUsePercentage(!usePercentage())}
              title={usePercentage() ? "Switch to weight" : "Switch to percentage"}
            >
              {usePercentage() ? '%' : 'W'}
            </button>
            <button
              class="btn btn-primary btn-sm"
              onClick={addOption}
              disabled={!newOptionText().trim() || !selectedChannel()}
            >
              <IconPlus size={16} />
              Add
            </button>
          </div>

          {/* Color Presets */}
          <div class="flex gap-1">
            <span class="text-xs text-base-content/60 mr-2">Quick colors:</span>
            <For each={colorPresets}>
              {(color) => (
                <button
                  class="w-6 h-6 rounded border border-base-300 hover:scale-110 transition-transform"
                  style={{ 'background-color': color }}
                  onClick={() => setNewOptionColor(color)}
                  title={color}
                />
              )}
            </For>
          </div>

          {/* Prize Selection */}
          <div class="flex gap-2 items-center">
            <span class="text-xs text-base-content/60">Prize:</span>
            <select
              class="select select-bordered select-sm flex-1"
              value={newPrizeType()}
              onChange={(e) => setNewPrizeType(e.target.value)}
            >
              <option value="">No Prize</option>
              <option value="xp">XP</option>
              <option value="currency">Currency</option>
              <option value="pack">Pack</option>
              <option value="item">Item</option>
              <option value="tts_voice">TTS Voice</option>
            </select>
            <Show when={newPrizeType()}>
              <input
                type="text"
                placeholder={
                  newPrizeType() === 'xp' ? 'XP amount' :
                  newPrizeType() === 'currency' ? 'Coin amount' :
                  newPrizeType() === 'pack' ? 'Pack ID' :
                  newPrizeType() === 'item' ? '{"item_id":"...","quantity":1}' :
                  newPrizeType() === 'tts_voice' ? 'Voice ID' :
                  'Prize data'
                }
                class="input input-bordered input-sm flex-1"
                value={newPrizeData()}
                onInput={(e) => setNewPrizeData(e.target.value)}
              />
            </Show>
          </div>
        </div>
      </div>

      {/* Options List */}
      <div class="flex-1 overflow-y-auto p-4">
        <Show
          when={!loading() && selectedChannel()}
          fallback={
            <div class="flex items-center justify-center h-full">
              <div class="text-center">
                <IconAlertCircle size={48} class="mx-auto mb-4 opacity-30" />
                <p class="text-sm text-base-content/60">
                  {loading() ? 'Loading wheel options...' : 'Select a channel to view wheel options'}
                </p>
              </div>
            </div>
          }
        >
          <Show
            when={options().length > 0}
            fallback={
              <div class="text-center py-8">
                <IconWheel size={48} class="mx-auto mb-4 opacity-30" />
                <p class="text-sm font-semibold mb-2">No wheel options yet</p>
                <p class="text-xs text-base-content/60">Add your first wheel option above</p>
              </div>
            }
          >
            <div class="grid gap-3">
              <For each={options()}>
                {(option) => (
                  <div class={`card bg-base-100 shadow-sm ${!option.enabled ? 'opacity-50' : ''}`}>
                    <div class="card-body p-4">
                      <Show
                        when={editingId() === option.id}
                        fallback={
                          <div class="flex items-center justify-between gap-3">
                            <div class="flex items-center gap-3 flex-1">
                              <div
                                class="w-8 h-8 rounded-full border-2 border-white shadow-sm flex-shrink-0"
                                style={{ 'background-color': option.color }}
                              />
                              <div class="flex-1">
                                <h3 class="font-semibold text-sm">{option.option_text}</h3>
                                <p class="text-xs text-base-content/60">
                                  {option.chance_percentage
                                    ? `${option.chance_percentage.toFixed(1)}% chance`
                                    : `Weight: ${option.weight}`} | {option.enabled ? 'Enabled' : 'Disabled'}
                                  <Show when={option.prize_type}>
                                    <span class="ml-2 px-2 py-0.5 bg-primary/20 text-primary rounded">
                                      🎁 {option.prize_type}
                                      <Show when={option.prize_data}>
                                        : {option.prize_data}
                                      </Show>
                                    </span>
                                  </Show>
                                </p>
                              </div>
                            </div>

                            <div class="flex gap-2">
                              <button
                                class={`btn btn-circle btn-sm ${option.enabled ? 'btn-warning' : 'btn-success'}`}
                                onClick={() => toggleEnabled(option.id, option.enabled)}
                                title={option.enabled ? 'Disable' : 'Enable'}
                              >
                                {option.enabled ? '⏸' : '▶'}
                              </button>
                              <button
                                class="btn btn-circle btn-sm btn-ghost"
                                onClick={() => startEdit(option)}
                                title="Edit"
                              >
                                <IconEdit size={16} />
                              </button>
                              <button
                                class="btn btn-circle btn-sm btn-error btn-outline"
                                onClick={() => deleteOption(option.id)}
                                title="Delete"
                              >
                                <IconTrash size={16} />
                              </button>
                            </div>
                          </div>
                        }
                      >
                        {/* Edit Mode */}
                        <div class="flex flex-col gap-2">
                          <div class="flex gap-2">
                            <input
                              type="text"
                              class="input input-bordered input-sm flex-1"
                              value={editText()}
                              onInput={(e) => setEditText(e.target.value)}
                            />
                            <input
                              type="color"
                              class="w-12 h-8 rounded cursor-pointer border border-base-300"
                              value={editColor()}
                              onInput={(e) => setEditColor(e.target.value)}
                            />
                            <Show when={editUsePercentage()} fallback={
                              <input
                                type="number"
                                min="1"
                                max="100"
                                class="input input-bordered input-sm w-20"
                                value={editWeight()}
                                onInput={(e) => setEditWeight(parseInt(e.target.value) || 1)}
                                placeholder="Weight"
                              />
                            }>
                              <input
                                type="number"
                                min="0.01"
                                max="100"
                                step="0.1"
                                class="input input-bordered input-sm w-24"
                                value={editPercentage()}
                                onInput={(e) => setEditPercentage(e.target.value)}
                                placeholder="%"
                              />
                            </Show>
                            <button
                              class="btn btn-xs btn-ghost"
                              onClick={() => setEditUsePercentage(!editUsePercentage())}
                              title={editUsePercentage() ? "Switch to weight" : "Switch to percentage"}
                            >
                              {editUsePercentage() ? '%' : 'W'}
                            </button>
                          </div>

                          {/* Prize Selection in Edit Mode */}
                          <div class="flex gap-2 items-center">
                            <span class="text-xs text-base-content/60">Prize:</span>
                            <select
                              class="select select-bordered select-sm flex-1"
                              value={editPrizeType()}
                              onChange={(e) => setEditPrizeType(e.target.value)}
                            >
                              <option value="">No Prize</option>
                              <option value="xp">XP</option>
                              <option value="currency">Currency</option>
                              <option value="pack">Pack</option>
                              <option value="item">Item</option>
                              <option value="tts_voice">TTS Voice</option>
                            </select>
                            <Show when={editPrizeType()}>
                              <input
                                type="text"
                                placeholder={
                                  editPrizeType() === 'xp' ? 'XP amount' :
                                  editPrizeType() === 'currency' ? 'Coin amount' :
                                  editPrizeType() === 'pack' ? 'Pack ID' :
                                  editPrizeType() === 'item' ? '{"item_id":"...","quantity":1}' :
                                  editPrizeType() === 'tts_voice' ? 'Voice ID' :
                                  'Prize data'
                                }
                                class="input input-bordered input-sm flex-1"
                                value={editPrizeData()}
                                onInput={(e) => setEditPrizeData(e.target.value)}
                              />
                            </Show>
                          </div>

                          <div class="flex gap-2 justify-end">
                            <button
                              class="btn btn-sm btn-ghost"
                              onClick={cancelEdit}
                            >
                              Cancel
                            </button>
                            <button
                              class="btn btn-sm btn-primary"
                              onClick={saveEdit}
                            >
                              Save
                            </button>
                          </div>
                        </div>
                      </Show>
                    </div>
                  </div>
                )}
              </For>
            </div>
          </Show>
        </Show>
      </div>
    </div>
  );
}
