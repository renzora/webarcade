import { createSignal, Show, For } from 'solid-js';
import { IconX, IconCheck, IconFile, IconChartBar, IconBrandRust, IconRocket } from '@tabler/icons-solidjs';
import { bridge } from '@/api/bridge';

export function NewPluginWizard(props) {
  const [step, setStep] = createSignal(1);
  const [config, setConfig] = createSignal({
    id: '',
    name: '',
    description: '',
    author: '',
    template: 'basic',
  });
  const [creating, setCreating] = createSignal(false);

  const templates = [
    {
      id: 'basic',
      name: 'Basic Plugin',
      description: 'Simple frontend-only plugin with viewport and menu',
      icon: IconFile,
      files: ['index.jsx', 'viewport.jsx', 'package.json'],
    },
    {
      id: 'widget',
      name: 'Widget Plugin',
      description: 'Plugin with dashboard widget properly registered',
      icon: IconChartBar,
      files: ['index.jsx', 'viewport.jsx', 'widgets/MainWidget.jsx', 'package.json'],
    },
    {
      id: 'backend',
      name: 'Backend Plugin',
      description: 'Rust backend with HTTP routes and database',
      icon: IconBrandRust,
      files: ['mod.rs', 'router.rs', 'index.jsx', 'viewport.jsx', 'package.json'],
    },
    {
      id: 'fullstack',
      name: 'Full-Stack Plugin',
      description: 'Complete plugin with Rust backend, frontend, and dashboard widget',
      icon: IconRocket,
      files: ['mod.rs', 'router.rs', 'index.jsx', 'viewport.jsx', 'widgets/MainWidget.jsx', 'api.js', 'package.json'],
    },
  ];

  const validateStep1 = () => {
    if (!config().id || !config().name) {
      alert('Plugin ID and Name are required');
      return false;
    }

    // Validate plugin ID format (lowercase, hyphens only)
    if (!/^[a-z0-9-]+$/.test(config().id)) {
      alert('Plugin ID must contain only lowercase letters, numbers, and hyphens');
      return false;
    }

    return true;
  };

  const createPlugin = async () => {
    setCreating(true);
    try {
      const response = await bridge('/developer/create', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(config()),
      });

      if (!response.ok) {
        const error = await response.text();
        throw new Error(error);
      }

      const { plugin } = await response.json();

      // Notify the file panel to refresh and load index.jsx
      window.dispatchEvent(new CustomEvent('plugin-ide:plugin-created', {
        detail: { pluginId: config().id }
      }));

      props.onCreate?.(plugin);
      props.onClose();
    } catch (error) {
      console.error('Failed to create plugin:', error);
      alert('Failed to create plugin: ' + error.message);
    } finally {
      setCreating(false);
    }
  };

  return (
    <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" onClick={props.onClose}>
      <div class="bg-base-200 rounded-lg w-full max-w-3xl max-h-[80vh] flex flex-col shadow-2xl" onClick={(e) => e.stopPropagation()}>
        <div class="flex items-center justify-between p-4 border-b border-base-content/10">
          <h2 class="text-xl font-bold">Create New Plugin</h2>
          <button onClick={props.onClose} class="btn btn-sm btn-circle btn-ghost">
            <IconX size={20} />
          </button>
        </div>

        <div class="flex items-center justify-center gap-4 p-6">
          <div class="flex items-center gap-2">
            <div class={`w-8 h-8 rounded-full flex items-center justify-center ${step() >= 1 ? 'bg-primary text-primary-content' : 'bg-base-300'}`}>
              1
            </div>
            <span class={step() >= 1 ? 'font-medium' : 'text-base-content/60'}>Details</span>
          </div>
          <div class="w-16 h-0.5 bg-base-300" />
          <div class="flex items-center gap-2">
            <div class={`w-8 h-8 rounded-full flex items-center justify-center ${step() >= 2 ? 'bg-primary text-primary-content' : 'bg-base-300'}`}>
              2
            </div>
            <span class={step() >= 2 ? 'font-medium' : 'text-base-content/60'}>Template</span>
          </div>
        </div>

        <div class="flex-1 overflow-y-auto p-6">
          <Show when={step() === 1}>
            <div class="space-y-4">
              <h3 class="text-lg font-bold mb-4">Plugin Details</h3>
              <div class="form-control">
                <label class="label">
                  <span class="label-text">Plugin ID *</span>
                </label>
                <input
                  type="text"
                  placeholder="my-awesome-plugin"
                  class="input input-bordered"
                  value={config().id}
                  onInput={(e) => setConfig({ ...config(), id: e.target.value.toLowerCase() })}
                  autofocus
                />
                <label class="label">
                  <span class="label-text-alt">Lowercase letters, numbers, and hyphens only</span>
                </label>
              </div>

              <div class="form-control">
                <label class="label">
                  <span class="label-text">Plugin Name *</span>
                </label>
                <input
                  type="text"
                  placeholder="My Awesome Plugin"
                  class="input input-bordered"
                  value={config().name}
                  onInput={(e) => setConfig({ ...config(), name: e.target.value })}
                />
              </div>

              <div class="form-control">
                <label class="label">
                  <span class="label-text">Description</span>
                </label>
                <textarea
                  placeholder="What does this plugin do?"
                  class="textarea textarea-bordered"
                  rows={3}
                  value={config().description}
                  onInput={(e) => setConfig({ ...config(), description: e.target.value })}
                />
              </div>

              <div class="form-control">
                <label class="label">
                  <span class="label-text">Author</span>
                </label>
                <input
                  type="text"
                  placeholder="Your Name"
                  class="input input-bordered"
                  value={config().author}
                  onInput={(e) => setConfig({ ...config(), author: e.target.value })}
                />
              </div>
            </div>
          </Show>

          <Show when={step() === 2}>
            <div class="space-y-4">
              <h3 class="text-lg font-bold mb-4">Choose Template</h3>
              <div class="grid grid-cols-2 gap-4">
                <For each={templates}>
                  {(template) => {
                    const Icon = template.icon;
                    return (
                      <div
                        class={`card bg-base-300 cursor-pointer transition-all hover:shadow-lg ${
                          config().template === template.id ? 'ring-2 ring-primary' : ''
                        }`}
                        onClick={() => setConfig({ ...config(), template: template.id })}
                      >
                        <div class="card-body p-4">
                          <div class="flex items-center gap-3 mb-2">
                            <Icon size={32} class="text-primary" />
                            <div class="flex-1">
                              <h4 class="font-bold">{template.name}</h4>
                            </div>
                            <Show when={config().template === template.id}>
                              <IconCheck size={20} class="text-primary" />
                            </Show>
                          </div>
                          <p class="text-sm text-base-content/70 mb-3">{template.description}</p>
                          <div class="flex flex-wrap gap-1">
                            <For each={template.files}>
                              {(file) => <span class="badge badge-sm">{file}</span>}
                            </For>
                          </div>
                        </div>
                      </div>
                    );
                  }}
                </For>
              </div>
            </div>
          </Show>
        </div>

        <div class="flex items-center justify-between p-4 border-t border-base-content/10">
          <Show when={step() === 2}>
            <button onClick={() => setStep(1)} class="btn btn-ghost">
              Back
            </button>
          </Show>
          <div class="flex-1" />
          <Show when={step() === 1}>
            <button
              onClick={() => {
                if (validateStep1()) {
                  setStep(2);
                }
              }}
              class="btn btn-primary"
            >
              Next
            </button>
          </Show>
          <Show when={step() === 2}>
            <button
              onClick={createPlugin}
              disabled={creating()}
              class="btn btn-primary"
            >
              {creating() ? <span class="loading loading-spinner loading-sm" /> : 'Create Plugin'}
            </button>
          </Show>
        </div>
      </div>
    </div>
  );
}
