import { IconSettings, IconInfoCircle } from '@tabler/icons-solidjs';

export default function PropertiesPanel() {
  return (
    <div class="h-full bg-base-200 p-4 space-y-4">
      <div class="flex items-center gap-2 mb-4">
        <IconSettings size={20} />
        <h3 class="text-lg font-bold">Properties</h3>
      </div>

      <div class="card bg-base-100 shadow-sm">
        <div class="card-body p-4">
          <div class="flex items-center gap-2 mb-3">
            <IconInfoCircle size={18} class="text-info" />
            <h4 class="font-semibold">About</h4>
          </div>
          <div class="space-y-2 text-sm">
            <div class="flex justify-between">
              <span class="text-base-content/70">Version:</span>
              <span class="font-mono">1.0.0</span>
            </div>
            <div class="flex justify-between">
              <span class="text-base-content/70">Platform:</span>
              <span class="font-mono">WebArcade</span>
            </div>
            <div class="flex justify-between">
              <span class="text-base-content/70">Framework:</span>
              <span class="font-mono">Solid.js</span>
            </div>
          </div>
        </div>
      </div>

      <div class="card bg-base-100 shadow-sm">
        <div class="card-body p-4">
          <h4 class="font-semibold mb-3">Quick Actions</h4>
          <div class="space-y-2">
            <button class="btn btn-sm btn-block btn-outline">
              New File
            </button>
            <button class="btn btn-sm btn-block btn-outline">
              Open Settings
            </button>
            <button class="btn btn-sm btn-block btn-outline">
              View Documentation
            </button>
          </div>
        </div>
      </div>

      <div class="text-xs text-base-content/50 text-center mt-auto">
        Select an item to view properties
      </div>
    </div>
  );
}
