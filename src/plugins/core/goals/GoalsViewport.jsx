import { createSignal, onMount, For, Show } from 'solid-js';
import twitchStore from '../twitch/TwitchStore.jsx';
import { bridgeFetch } from '@/api/bridge.js';
import { IconTarget, IconPlus, IconEdit, IconTrash, IconRefresh, IconAlertCircle, IconTrophy, IconUsers, IconStar } from '@tabler/icons-solidjs';

export default function GoalsViewport() {
  const [goals, setGoals] = createSignal([]);
  const [loading, setLoading] = createSignal(true);
  const [selectedChannel, setSelectedChannel] = createSignal('');
  const [status, setStatus] = createSignal({ status: 'disconnected', connected_channels: [] });
  const [showAddModal, setShowAddModal] = createSignal(false);
  const [editingGoal, setEditingGoal] = createSignal(null);

  // Form state
  const [formTitle, setFormTitle] = createSignal('');
  const [formDescription, setFormDescription] = createSignal('');
  const [formType, setFormType] = createSignal('custom');
  const [formTarget, setFormTarget] = createSignal(100);
  const [formCurrent, setFormCurrent] = createSignal(0);
  const [formIsSubGoal, setFormIsSubGoal] = createSignal(false);

  onMount(async () => {
    const currentStatus = await twitchStore.fetchStatus();
    if (currentStatus) {
      setStatus({ ...currentStatus, connected_channels: currentStatus.connected_channels || [] });
      if (currentStatus.connected_channels && currentStatus.connected_channels.length > 0) {
        setSelectedChannel(currentStatus.connected_channels[0]);
        await loadGoals(currentStatus.connected_channels[0]);
      }
    }
    setLoading(false);
  });

  const loadGoals = async (channel) => {
    if (!channel) return;

    try {
      setLoading(true);
      const response = await bridgeFetch(`/goals/list?channel=${channel}`);
      const data = await response.json();
      setGoals(data);
    } catch (e) {
      console.error('Failed to load goals:', e);
    } finally {
      setLoading(false);
    }
  };

  const handleChannelChange = async (channel) => {
    setSelectedChannel(channel);
    await loadGoals(channel);
  };

  const resetForm = () => {
    setFormTitle('');
    setFormDescription('');
    setFormType('custom');
    setFormTarget(100);
    setFormCurrent(0);
    setFormIsSubGoal(false);
    setEditingGoal(null);
  };

  const openAddModal = () => {
    resetForm();
    setShowAddModal(true);
  };

  const openEditModal = (goal) => {
    setFormTitle(goal.title);
    setFormDescription(goal.description || '');
    setFormType(goal.type);
    setFormTarget(goal.target);
    setFormCurrent(goal.current);
    setFormIsSubGoal(goal.is_sub_goal || false);
    setEditingGoal(goal);
    setShowAddModal(true);
  };

  const closeModal = () => {
    setShowAddModal(false);
    resetForm();
  };

  const saveGoal = async () => {
    if (!formTitle().trim() || formTarget() <= 0) {
      alert('Please provide a title and valid target');
      return;
    }

    try {
      // Auto-fetch current count from Twitch for follower/subscriber goals
      let currentValue = formCurrent();
      if (!editingGoal() && (formType() === 'follower' || formType() === 'subscriber')) {
        try {
          // Create a temporary goal to get the current value from Twitch
          const tempGoalData = {
            channel: selectedChannel(),
            title: formTitle().trim(),
            description: formDescription().trim(),
            type: formType(),
            target: formTarget(),
            current: 0,
            is_sub_goal: formIsSubGoal()
          };

          // Create the goal first
          const createResponse = await bridgeFetch('/goals/create', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(tempGoalData),
          });

          if (createResponse.ok) {
            const createData = await createResponse.json();
            const goalId = createData.id;

            // Now sync it with Twitch to get the current value
            const syncResponse = await bridgeFetch(`/goals/${goalId}/sync-twitch`, {
              method: 'POST',
            });

            if (syncResponse.ok) {
              await loadGoals(selectedChannel());
              closeModal();
              return;
            } else {
              // If sync failed, still keep the goal but with 0 current
              await loadGoals(selectedChannel());
              closeModal();
              alert('Goal created but failed to sync with Twitch. You can manually sync later.');
              return;
            }
          }
        } catch (e) {
          console.error('Failed to auto-fetch from Twitch:', e);
          // Fall through to manual creation
        }
      }

      const goalData = {
        channel: selectedChannel(),
        title: formTitle().trim(),
        description: formDescription().trim(),
        type: formType(),
        target: formTarget(),
        current: currentValue,
        is_sub_goal: formIsSubGoal()
      };

      let response;
      if (editingGoal()) {
        response = await bridgeFetch(`/goals/${editingGoal().id}`, {
          method: 'PUT',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            ...goalData,
            channel: selectedChannel()  // Include channel for broadcasting
          }),
        });
      } else {
        response = await bridgeFetch('/goals/create', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify(goalData),
        });
      }

      if (response.ok) {
        await loadGoals(selectedChannel());
        closeModal();
      }
    } catch (e) {
      console.error('Failed to save goal:', e);
      alert('Failed to save goal');
    }
  };

  const deleteGoal = async (goalId) => {
    if (!confirm('Are you sure you want to delete this goal?')) return;

    try {
      const response = await bridgeFetch(`/goals/${goalId}`, {
        method: 'DELETE',
      });

      if (response.ok) {
        await loadGoals(selectedChannel());
      }
    } catch (e) {
      console.error('Failed to delete goal:', e);
    }
  };

  const updateGoalProgress = async (goalId, newCurrent) => {
    try {
      const response = await bridgeFetch(`/goals/${goalId}/progress`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ current: newCurrent }),
      });

      if (response.ok) {
        await loadGoals(selectedChannel());
      }
    } catch (e) {
      console.error('Failed to update progress:', e);
    }
  };

  const syncWithTwitch = async (goalId) => {
    try {
      const response = await bridgeFetch(`/goals/${goalId}/sync-twitch`, {
        method: 'POST',
      });

      if (response.ok) {
        await loadGoals(selectedChannel());
      }
    } catch (e) {
      console.error('Failed to sync with Twitch:', e);
      alert('Failed to sync with Twitch. Make sure the bot is connected.');
    }
  };

  const getPercentage = (current, target) => {
    if (target === 0) return 0;
    return Math.min(100, Math.max(0, (current / target) * 100));
  };

  const getGoalIcon = (goalType) => {
    switch (goalType) {
      case 'subscriber':
        return <IconStar size={20} class="text-purple-400" />;
      case 'follower':
        return <IconUsers size={20} class="text-green-400" />;
      default:
        return <IconTrophy size={20} class="text-blue-400" />;
    }
  };

  return (
    <div class="h-full flex flex-col bg-base-200">
      {/* Header */}
      <div class="flex items-center justify-between bg-base-100 border-b border-base-300 px-4 py-3">
        <div class="flex items-center gap-3 flex-1">
          <IconTarget size={20} class="text-primary" />
          <h2 class="text-lg font-semibold">Goals Tracker</h2>
        </div>

        <div class="flex items-center gap-2">
          <Show when={status().connected_channels?.length > 0}>
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
          <button
            class="btn btn-primary btn-sm"
            onClick={openAddModal}
            disabled={!selectedChannel()}
          >
            <IconPlus size={16} />
            Add Goal
          </button>
        </div>
      </div>

      {/* Goals List */}
      <div class="flex-1 overflow-y-auto p-4">
        <Show
          when={!loading() && selectedChannel()}
          fallback={
            <div class="flex items-center justify-center h-full">
              <div class="text-center">
                <IconAlertCircle size={48} class="mx-auto mb-4 opacity-30" />
                <p class="text-sm text-base-content/60">
                  {loading() ? 'Loading goals...' : 'Select a channel to view goals'}
                </p>
              </div>
            </div>
          }
        >
          <Show
            when={goals().length > 0}
            fallback={
              <div class="text-center py-8">
                <IconTarget size={48} class="mx-auto mb-4 opacity-30" />
                <p class="text-sm font-semibold mb-2">No goals yet</p>
                <p class="text-xs text-base-content/60">Create your first goal to start tracking progress</p>
              </div>
            }
          >
            <div class="grid gap-4">
              <For each={goals()}>
                {(goal) => (
                  <div class="card bg-base-100 shadow-md">
                    <div class="card-body p-4">
                      <div class="flex items-start justify-between mb-3">
                        <div class="flex items-start gap-3 flex-1">
                          {getGoalIcon(goal.type)}
                          <div class="flex-1">
                            <h3 class="font-bold text-lg">{goal.title}</h3>
                            <div class="flex items-center gap-2 mt-1">
                              <div class="badge badge-sm badge-primary">{goal.type}</div>
                              <Show when={goal.is_sub_goal}>
                                <div class="badge badge-sm badge-warning">Sub Goal</div>
                              </Show>
                            </div>
                            <Show when={goal.description}>
                              <p class="text-sm text-base-content/70 mt-2">{goal.description}</p>
                            </Show>
                          </div>
                        </div>
                        <div class="flex gap-1">
                          <Show when={goal.type === 'subscriber' || goal.type === 'follower'}>
                            <button
                              class="btn btn-circle btn-xs btn-ghost"
                              onClick={() => syncWithTwitch(goal.id)}
                              title="Sync with Twitch"
                            >
                              <IconRefresh size={14} />
                            </button>
                          </Show>
                          <button
                            class="btn btn-circle btn-xs btn-ghost"
                            onClick={() => openEditModal(goal)}
                            title="Edit"
                          >
                            <IconEdit size={14} />
                          </button>
                          <button
                            class="btn btn-circle btn-xs btn-ghost text-error"
                            onClick={() => deleteGoal(goal.id)}
                            title="Delete"
                          >
                            <IconTrash size={14} />
                          </button>
                        </div>
                      </div>

                      {/* Progress */}
                      <div class="space-y-2">
                        <div class="flex items-center justify-between">
                          <div class="flex items-baseline gap-2">
                            <span class="text-2xl font-bold font-mono">{goal.current.toLocaleString()}</span>
                            <span class="text-sm text-base-content/60">/ {goal.target.toLocaleString()}</span>
                          </div>
                          <span class="text-sm font-semibold text-primary">
                            {getPercentage(goal.current, goal.target).toFixed(1)}%
                          </span>
                        </div>

                        <progress
                          class="progress progress-primary w-full"
                          value={goal.current}
                          max={goal.target}
                        />

                        {/* Manual Progress Controls */}
                        <div class="flex gap-2 mt-2">
                          <input
                            type="number"
                            class="input input-bordered input-sm flex-1"
                            value={goal.current}
                            min="0"
                            max={goal.target}
                            onChange={(e) => {
                              const val = parseInt(e.target.value) || 0;
                              updateGoalProgress(goal.id, val);
                            }}
                          />
                          <button
                            class="btn btn-sm btn-outline"
                            onClick={() => updateGoalProgress(goal.id, goal.current + 1)}
                          >
                            +1
                          </button>
                          <button
                            class="btn btn-sm btn-outline"
                            onClick={() => updateGoalProgress(goal.id, Math.max(0, goal.current - 1))}
                          >
                            -1
                          </button>
                        </div>
                      </div>
                    </div>
                  </div>
                )}
              </For>
            </div>
          </Show>
        </Show>
      </div>

      {/* Add/Edit Modal */}
      <Show when={showAddModal()}>
        <div class="modal modal-open">
          <div class="modal-box">
            <h3 class="font-bold text-lg mb-4">
              {editingGoal() ? 'Edit Goal' : 'Add New Goal'}
            </h3>

            <div class="space-y-4">
              {/* Title */}
              <div class="form-control">
                <label class="label">
                  <span class="label-text">Title</span>
                </label>
                <input
                  type="text"
                  placeholder="Goal title"
                  class="input input-bordered"
                  value={formTitle()}
                  onInput={(e) => setFormTitle(e.target.value)}
                />
              </div>

              {/* Description */}
              <div class="form-control">
                <label class="label">
                  <span class="label-text">Description (optional)</span>
                </label>
                <textarea
                  placeholder="Goal description"
                  class="textarea textarea-bordered"
                  value={formDescription()}
                  onInput={(e) => setFormDescription(e.target.value)}
                />
              </div>

              {/* Type */}
              <div class="form-control">
                <label class="label">
                  <span class="label-text">Goal Type</span>
                </label>
                <select
                  class="select select-bordered"
                  value={formType()}
                  onChange={(e) => setFormType(e.target.value)}
                >
                  <option value="custom">Custom</option>
                  <option value="subscriber">Subscriber Goal</option>
                  <option value="follower">Follower Goal</option>
                </select>
              </div>

              {/* Target */}
              <div class="form-control">
                <label class="label">
                  <span class="label-text">Target</span>
                </label>
                <input
                  type="number"
                  placeholder="100"
                  class="input input-bordered"
                  value={formTarget()}
                  min="1"
                  onInput={(e) => setFormTarget(parseInt(e.target.value) || 1)}
                />
              </div>

              {/* Current */}
              <div class="form-control">
                <label class="label">
                  <span class="label-text">Current Progress</span>
                  <Show when={formType() === 'follower' || formType() === 'subscriber'}>
                    <span class="label-text-alt text-info">Auto-fetched from Twitch</span>
                  </Show>
                </label>
                <input
                  type="number"
                  placeholder={formType() === 'follower' || formType() === 'subscriber' ? 'Auto-fetched from Twitch...' : '0'}
                  class="input input-bordered"
                  value={formCurrent()}
                  min="0"
                  disabled={!editingGoal() && (formType() === 'follower' || formType() === 'subscriber')}
                  onInput={(e) => setFormCurrent(parseInt(e.target.value) || 0)}
                />
              </div>

              {/* Sub Goal */}
              <div class="form-control">
                <label class="label cursor-pointer">
                  <span class="label-text">Mark as Sub Goal</span>
                  <input
                    type="checkbox"
                    class="checkbox checkbox-primary"
                    checked={formIsSubGoal()}
                    onChange={(e) => setFormIsSubGoal(e.target.checked)}
                  />
                </label>
                <label class="label">
                  <span class="label-text-alt text-base-content/60">
                    Sub goals are smaller milestones within a larger goal
                  </span>
                </label>
              </div>
            </div>

            <div class="modal-action">
              <button class="btn btn-ghost" onClick={closeModal}>
                Cancel
              </button>
              <button class="btn btn-primary" onClick={saveGoal}>
                {editingGoal() ? 'Update' : 'Create'}
              </button>
            </div>
          </div>
          <div class="modal-backdrop" onClick={closeModal}></div>
        </div>
      </Show>
    </div>
  );
}
