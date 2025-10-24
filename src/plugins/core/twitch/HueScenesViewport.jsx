import { createSignal, onMount, For, Show } from 'solid-js';
import { bridgeFetch } from '@/api/bridge.js';
import { IconPlus, IconTrash, IconPlayerPlay, IconGripVertical, IconEdit, IconCheck, IconX } from '@tabler/icons-solidjs';

export default function HueScenesViewport() {
  const [scenes, setScenes] = createSignal([]);
  const [loading, setLoading] = createSignal(true);
  const [newSceneName, setNewSceneName] = createSignal('');
  const [newSceneTag, setNewSceneTag] = createSignal('');
  const [editingScene, setEditingScene] = createSignal(null);
  const [draggedStep, setDraggedStep] = createSignal(null);

  onMount(async () => {
    await loadScenes();
    setLoading(false);
  });

  const loadScenes = async () => {
    try {
      const response = await bridgeFetch('/hue/animated-scenes');
      const data = await response.json();
      setScenes(data);
    } catch (e) {
      console.error('Failed to load scenes:', e);
    }
  };

  const createScene = async () => {
    const name = newSceneName().trim();
    const tag = newSceneTag().trim().toLowerCase();

    if (!name || !tag) {
      alert('Please enter both name and tag');
      return;
    }

    try {
      const response = await bridgeFetch('/hue/animated-scenes', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ name, tag }),
      });

      if (response.ok) {
        setNewSceneName('');
        setNewSceneTag('');
        await loadScenes();
      }
    } catch (e) {
      console.error('Failed to create scene:', e);
      alert('Failed to create scene. Tag may already exist.');
    }
  };

  const deleteScene = async (sceneId, sceneName) => {
    if (!confirm(`Delete scene '${sceneName}'? This will remove all color steps.`)) return;

    try {
      await bridgeFetch('/hue/animated-scenes', {
        method: 'DELETE',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ scene_id: sceneId }),
      });
      await loadScenes();
    } catch (e) {
      console.error('Failed to delete scene:', e);
      alert('Failed to delete scene');
    }
  };

  const addColorStep = async (sceneId) => {
    const currentScene = scenes().find(s => s.id === sceneId);
    const order = currentScene.steps.length;

    try {
      await bridgeFetch('/hue/animated-scenes/steps', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          scene_id: sceneId,
          order,
          red: 255,
          green: 0,
          blue: 0,
          transition: 10, // 1 second
          duration: 20,   // 2 seconds
        }),
      });
      await loadScenes();
    } catch (e) {
      console.error('Failed to add step:', e);
    }
  };

  const updateStep = async (stepId, field, value) => {
    const scene = scenes().find(s => s.steps.some(step => step.id === stepId));
    const step = scene.steps.find(s => s.id === stepId);

    // Optimistic update
    setScenes(scenes().map(s => {
      if (s.id !== scene.id) return s;
      return {
        ...s,
        steps: s.steps.map(st => {
          if (st.id !== stepId) return st;
          return { ...st, [field]: value };
        })
      };
    }));

    const updated = {
      step_id: stepId,
      red: step.red,
      green: step.green,
      blue: step.blue,
      transition: step.transition,
      duration: step.duration,
      [field]: value,
    };

    try {
      await bridgeFetch('/hue/animated-scenes/steps', {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(updated),
      });
    } catch (e) {
      console.error('Failed to update step:', e);
      // Revert on error
      await loadScenes();
    }
  };

  const updateStepColor = async (stepId, hexColor) => {
    const hex = hexColor.replace('#', '');
    const r = parseInt(hex.substring(0, 2), 16);
    const g = parseInt(hex.substring(2, 4), 16);
    const b = parseInt(hex.substring(4, 6), 16);

    const scene = scenes().find(s => s.steps.some(step => step.id === stepId));
    const step = scene.steps.find(s => s.id === stepId);

    // Optimistic update
    setScenes(scenes().map(s => {
      if (s.id !== scene.id) return s;
      return {
        ...s,
        steps: s.steps.map(st => {
          if (st.id !== stepId) return st;
          return { ...st, red: r, green: g, blue: b };
        })
      };
    }));

    try {
      await bridgeFetch('/hue/animated-scenes/steps', {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          step_id: stepId,
          red: r,
          green: g,
          blue: b,
          transition: step.transition,
          duration: step.duration,
        }),
      });
    } catch (e) {
      console.error('Failed to update color:', e);
      // Revert on error
      await loadScenes();
    }
  };

  const deleteStep = async (stepId) => {
    try {
      await bridgeFetch('/hue/animated-scenes/steps', {
        method: 'DELETE',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ step_id: stepId }),
      });
      await loadScenes();
    } catch (e) {
      console.error('Failed to delete step:', e);
    }
  };

  const playScene = async (sceneId, sceneName) => {
    try {
      await bridgeFetch('/hue/animated-scenes/play', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ scene_id: sceneId }),
      });
    } catch (e) {
      console.error('Failed to play scene:', e);
      alert('Failed to play scene');
    }
  };

  const handleDragStart = (e, sceneId, stepId) => {
    setDraggedStep({ sceneId, stepId });
    e.dataTransfer.effectAllowed = 'move';
  };

  const handleDragOver = (e) => {
    e.preventDefault();
    e.dataTransfer.dropEffect = 'move';
  };

  const handleDrop = async (e, sceneId, targetStepId) => {
    e.preventDefault();
    const dragged = draggedStep();

    if (!dragged || dragged.sceneId !== sceneId || dragged.stepId === targetStepId) {
      setDraggedStep(null);
      return;
    }

    // Get the scene and reorder steps
    const scene = scenes().find(s => s.id === sceneId);
    const steps = [...scene.steps];
    const draggedIndex = steps.findIndex(s => s.id === dragged.stepId);
    const targetIndex = steps.findIndex(s => s.id === targetStepId);

    // Reorder array
    const [removed] = steps.splice(draggedIndex, 1);
    steps.splice(targetIndex, 0, removed);

    // Send reordered IDs to backend
    const stepIds = steps.map(s => s.id);

    try {
      await bridgeFetch('/hue/animated-scenes/reorder', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ scene_id: sceneId, step_ids: stepIds }),
      });
      await loadScenes();
    } catch (e) {
      console.error('Failed to reorder steps:', e);
    }

    setDraggedStep(null);
  };

  const rgbToHex = (r, g, b) => {
    return '#' + [r, g, b].map(x => {
      const hex = x.toString(16);
      return hex.length === 1 ? '0' + hex : hex;
    }).join('');
  };

  return (
    <div class="h-full flex flex-col bg-base-200">
      {/* Header */}
      <div class="flex items-center justify-between bg-base-100 border-b border-base-300 px-4 py-3">
        <div>
          <h2 class="text-lg font-semibold">Animated Scenes</h2>
          <p class="text-xs text-base-content/60">Create multi-color light sequences</p>
        </div>
      </div>

      <div class="flex-1 overflow-y-auto p-4">
        <Show
          when={!loading()}
          fallback={
            <div class="flex items-center justify-center h-full">
              <span class="loading loading-spinner loading-lg"></span>
            </div>
          }
        >
          {/* Create New Scene */}
          <div class="card bg-base-100 shadow-sm mb-4">
            <div class="card-body p-4">
              <h3 class="font-semibold mb-3">Create New Scene</h3>
              <div class="grid grid-cols-1 md:grid-cols-3 gap-3">
                <input
                  type="text"
                  placeholder="Scene name (e.g., Sunset Vibe)"
                  class="input input-bordered input-sm"
                  value={newSceneName()}
                  onInput={(e) => setNewSceneName(e.target.value)}
                />
                <input
                  type="text"
                  placeholder="Tag for chat (e.g., sunset)"
                  class="input input-bordered input-sm"
                  value={newSceneTag()}
                  onInput={(e) => setNewSceneTag(e.target.value)}
                />
                <button class="btn btn-sm btn-primary gap-2" onClick={createScene}>
                  <IconPlus size={16} />
                  Create Scene
                </button>
              </div>
            </div>
          </div>

          {/* Scenes List */}
          <Show
            when={scenes().length > 0}
            fallback={
              <div class="text-center py-12">
                <p class="text-base-content/60">No scenes yet. Create your first animated scene above!</p>
              </div>
            }
          >
            <div class="space-y-4">
              <For each={scenes()}>
                {(scene) => (
                  <div class="card bg-base-100 shadow-md">
                    <div class="card-body p-4">
                      {/* Scene Header */}
                      <div class="flex items-center justify-between mb-3">
                        <div>
                          <h3 class="font-bold text-lg">{scene.name}</h3>
                          <p class="text-sm text-base-content/60">
                            Chat command: <span class="font-mono bg-base-200 px-2 py-0.5 rounded">!lights {scene.tag}</span>
                          </p>
                        </div>
                        <div class="flex gap-2">
                          <button
                            class="btn btn-sm btn-success gap-2"
                            onClick={() => playScene(scene.id, scene.name)}
                            disabled={scene.steps.length === 0}
                          >
                            <IconPlayerPlay size={16} />
                            Play
                          </button>
                          <button
                            class="btn btn-sm btn-error gap-2"
                            onClick={() => deleteScene(scene.id, scene.name)}
                          >
                            <IconTrash size={16} />
                          </button>
                        </div>
                      </div>

                      {/* Color Steps */}
                      <div class="space-y-2">
                        <div class="flex items-center justify-between">
                          <h4 class="font-semibold text-sm">Color Sequence</h4>
                          <button
                            class="btn btn-xs btn-ghost gap-1"
                            onClick={() => addColorStep(scene.id)}
                          >
                            <IconPlus size={14} />
                            Add Color
                          </button>
                        </div>

                        <Show
                          when={scene.steps.length > 0}
                          fallback={
                            <p class="text-sm text-base-content/60 text-center py-4">
                              No colors yet. Add your first color step above.
                            </p>
                          }
                        >
                          <div class="space-y-2">
                            <For each={scene.steps}>
                              {(step, index) => (
                                <div
                                  class="bg-base-200 rounded-lg p-3 flex items-center gap-3"
                                  draggable={true}
                                  onDragStart={(e) => handleDragStart(e, scene.id, step.id)}
                                  onDragOver={handleDragOver}
                                  onDrop={(e) => handleDrop(e, scene.id, step.id)}
                                >
                                  {/* Drag Handle */}
                                  <div class="cursor-move text-base-content/40">
                                    <IconGripVertical size={20} />
                                  </div>

                                  {/* Step Number */}
                                  <div class="font-bold text-sm w-6">
                                    {index() + 1}
                                  </div>

                                  {/* Color Picker */}
                                  <input
                                    type="color"
                                    class="w-12 h-10 rounded cursor-pointer border-2 border-base-300"
                                    value={rgbToHex(step.red, step.green, step.blue)}
                                    onChange={(e) => updateStepColor(step.id, e.target.value)}
                                  />

                                  {/* Transition Time */}
                                  <div class="flex-1">
                                    <label class="text-xs text-base-content/60">
                                      Transition: {step.transition / 10}s
                                    </label>
                                    <input
                                      type="range"
                                      min="0"
                                      max="50"
                                      value={step.transition}
                                      class="range range-xs range-primary"
                                      onInput={(e) => updateStep(step.id, 'transition', parseInt(e.target.value))}
                                    />
                                  </div>

                                  {/* Duration */}
                                  <div class="flex-1">
                                    <label class="text-xs text-base-content/60">
                                      Hold: {step.duration / 10}s
                                    </label>
                                    <input
                                      type="range"
                                      min="0"
                                      max="100"
                                      value={step.duration}
                                      class="range range-xs range-secondary"
                                      onInput={(e) => updateStep(step.id, 'duration', parseInt(e.target.value))}
                                    />
                                  </div>

                                  {/* Delete Step */}
                                  <button
                                    class="btn btn-xs btn-ghost btn-circle"
                                    onClick={() => deleteStep(step.id)}
                                  >
                                    <IconTrash size={14} />
                                  </button>
                                </div>
                              )}
                            </For>
                          </div>
                        </Show>
                      </div>
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
