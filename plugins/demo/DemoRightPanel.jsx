import { createSignal } from 'solid-js';

export default function DemoRightPanel() {
  const [activeTab, setActiveTab] = createSignal('properties');
  const [color, setColor] = createSignal('#3b82f6');
  const [fontSize, setFontSize] = createSignal(16);
  const [padding, setPadding] = createSignal(16);
  const [borderRadius, setBorderRadius] = createSignal(8);
  const [opacity, setOpacity] = createSignal(100);

  return (
    <div class="h-full flex flex-col bg-base-100">
      <div class="border-b border-base-300">
        <div class="flex">
          <button
            class={`flex-1 px-3 py-2 text-xs font-medium transition-colors ${activeTab() === 'properties' ? 'bg-base-200 text-primary border-b-2 border-primary' : 'text-base-content/70 hover:bg-base-200'}`}
            onClick={() => setActiveTab('properties')}
          >
            Properties
          </button>
          <button
            class={`flex-1 px-3 py-2 text-xs font-medium transition-colors ${activeTab() === 'styles' ? 'bg-base-200 text-primary border-b-2 border-primary' : 'text-base-content/70 hover:bg-base-200'}`}
            onClick={() => setActiveTab('styles')}
          >
            Styles
          </button>
        </div>
      </div>

      <div class="flex-1 overflow-auto p-3">
        {activeTab() === 'properties' ? (
          <div class="space-y-4">
            <div>
              <h4 class="text-xs font-semibold text-base-content/50 uppercase mb-2">General</h4>
              <div class="space-y-2">
                <div class="form-control">
                  <label class="label py-1">
                    <span class="label-text text-xs">Name</span>
                  </label>
                  <input type="text" value="Demo Element" class="input input-bordered input-sm w-full" />
                </div>
                <div class="form-control">
                  <label class="label py-1">
                    <span class="label-text text-xs">Type</span>
                  </label>
                  <select class="select select-bordered select-sm w-full">
                    <option>Container</option>
                    <option>Button</option>
                    <option>Text</option>
                    <option>Image</option>
                  </select>
                </div>
                <div class="form-control">
                  <label class="label py-1 cursor-pointer">
                    <span class="label-text text-xs">Visible</span>
                    <input type="checkbox" class="toggle toggle-primary toggle-sm" checked />
                  </label>
                </div>
              </div>
            </div>

            <div>
              <h4 class="text-xs font-semibold text-base-content/50 uppercase mb-2">Transform</h4>
              <div class="grid grid-cols-2 gap-2">
                <div class="form-control">
                  <label class="label py-0">
                    <span class="label-text text-xs">X</span>
                  </label>
                  <input type="number" value="0" class="input input-bordered input-sm w-full" />
                </div>
                <div class="form-control">
                  <label class="label py-0">
                    <span class="label-text text-xs">Y</span>
                  </label>
                  <input type="number" value="0" class="input input-bordered input-sm w-full" />
                </div>
                <div class="form-control">
                  <label class="label py-0">
                    <span class="label-text text-xs">Width</span>
                  </label>
                  <input type="number" value="200" class="input input-bordered input-sm w-full" />
                </div>
                <div class="form-control">
                  <label class="label py-0">
                    <span class="label-text text-xs">Height</span>
                  </label>
                  <input type="number" value="100" class="input input-bordered input-sm w-full" />
                </div>
              </div>
            </div>
          </div>
        ) : (
          <div class="space-y-4">
            <div>
              <h4 class="text-xs font-semibold text-base-content/50 uppercase mb-2">Color</h4>
              <div class="flex items-center gap-2">
                <input
                  type="color"
                  value={color()}
                  onInput={(e) => setColor(e.target.value)}
                  class="w-8 h-8 rounded cursor-pointer"
                />
                <input
                  type="text"
                  value={color()}
                  onInput={(e) => setColor(e.target.value)}
                  class="input input-bordered input-sm flex-1"
                />
              </div>
            </div>

            <div>
              <h4 class="text-xs font-semibold text-base-content/50 uppercase mb-2">Typography</h4>
              <div class="form-control">
                <label class="label py-1">
                  <span class="label-text text-xs">Font Size</span>
                  <span class="label-text-alt">{fontSize()}px</span>
                </label>
                <input
                  type="range"
                  min="8"
                  max="72"
                  value={fontSize()}
                  onInput={(e) => setFontSize(parseInt(e.target.value))}
                  class="range range-primary range-xs"
                />
              </div>
            </div>

            <div>
              <h4 class="text-xs font-semibold text-base-content/50 uppercase mb-2">Spacing</h4>
              <div class="space-y-2">
                <div class="form-control">
                  <label class="label py-1">
                    <span class="label-text text-xs">Padding</span>
                    <span class="label-text-alt">{padding()}px</span>
                  </label>
                  <input
                    type="range"
                    min="0"
                    max="64"
                    value={padding()}
                    onInput={(e) => setPadding(parseInt(e.target.value))}
                    class="range range-secondary range-xs"
                  />
                </div>
                <div class="form-control">
                  <label class="label py-1">
                    <span class="label-text text-xs">Border Radius</span>
                    <span class="label-text-alt">{borderRadius()}px</span>
                  </label>
                  <input
                    type="range"
                    min="0"
                    max="32"
                    value={borderRadius()}
                    onInput={(e) => setBorderRadius(parseInt(e.target.value))}
                    class="range range-accent range-xs"
                  />
                </div>
                <div class="form-control">
                  <label class="label py-1">
                    <span class="label-text text-xs">Opacity</span>
                    <span class="label-text-alt">{opacity()}%</span>
                  </label>
                  <input
                    type="range"
                    min="0"
                    max="100"
                    value={opacity()}
                    onInput={(e) => setOpacity(parseInt(e.target.value))}
                    class="range range-info range-xs"
                  />
                </div>
              </div>
            </div>

            <div>
              <h4 class="text-xs font-semibold text-base-content/50 uppercase mb-2">Preview</h4>
              <div
                class="w-full h-20 flex items-center justify-center text-white"
                style={{
                  "background-color": color(),
                  "font-size": `${fontSize()}px`,
                  "padding": `${padding()}px`,
                  "border-radius": `${borderRadius()}px`,
                  "opacity": opacity() / 100
                }}
              >
                Demo
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
