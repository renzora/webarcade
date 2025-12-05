import { createSignal, For, Show } from 'solid-js';
import { panelVisibility, panelStore, horizontalMenuButtonsEnabled, footerVisible, pluginTabsVisible, bottomPanelVisible, toolbarVisible, fullscreenMode } from '@/api/plugin';

// Shared state for cross-component communication
export const [greetings, setGreetings] = createSignal([]);
export const [greetingCount, setGreetingCount] = createSignal(0);

// Get pluginAPI at runtime to avoid import timing issues
const getPluginAPI = () => window.WebArcadeAPI?.pluginAPI;

export default function Viewport() {
    return (
        <div class="flex flex-col items-center justify-center h-full gap-4">
            <h1 class="text-4xl font-bold">Hello World</h1>
            <p class="text-base-content/60">Greetings received: {greetingCount()}</p>
            <div class="max-w-md w-full max-h-48 overflow-auto bg-base-200 rounded-lg p-4">
                <For each={greetings()}>
                    {(greeting) => (
                        <div class="text-sm py-1 border-b border-base-300 last:border-0">
                            <span class="text-primary">{greeting.from}:</span> {greeting.message}
                        </div>
                    )}
                </For>
                <Show when={greetings().length === 0}>
                    <p class="text-base-content/40 text-sm">No greetings yet. Click "Send to Hello World" in Greeter!</p>
                </Show>
            </div>

            <div class="flex flex-wrap gap-4 mt-4 justify-center">
                <label class="flex items-center gap-2 cursor-pointer">
                    <input
                        type="checkbox"
                        class="toggle toggle-sm toggle-primary"
                        checked={panelVisibility.left}
                        onChange={(e) => panelStore.setPanelVisible('left', e.target.checked)}
                    />
                    <span class="text-sm">Left</span>
                </label>
                <label class="flex items-center gap-2 cursor-pointer">
                    <input
                        type="checkbox"
                        class="toggle toggle-sm toggle-primary"
                        checked={panelVisibility.right}
                        onChange={(e) => panelStore.setPanelVisible('right', e.target.checked)}
                    />
                    <span class="text-sm">Right</span>
                </label>
                <label class="flex items-center gap-2 cursor-pointer">
                    <input
                        type="checkbox"
                        class="toggle toggle-sm toggle-primary"
                        checked={bottomPanelVisible()}
                        onChange={(e) => getPluginAPI()?.showBottomPanel(e.target.checked)}
                    />
                    <span class="text-sm">Bottom</span>
                </label>
                <label class="flex items-center gap-2 cursor-pointer">
                    <input
                        type="checkbox"
                        class="toggle toggle-sm toggle-secondary"
                        checked={horizontalMenuButtonsEnabled()}
                        onChange={(e) => getPluginAPI()?.showMenu(e.target.checked)}
                    />
                    <span class="text-sm">Menu</span>
                </label>
                <label class="flex items-center gap-2 cursor-pointer">
                    <input
                        type="checkbox"
                        class="toggle toggle-sm toggle-secondary"
                        checked={footerVisible()}
                        onChange={(e) => getPluginAPI()?.showFooter(e.target.checked)}
                    />
                    <span class="text-sm">Footer</span>
                </label>
                <label class="flex items-center gap-2 cursor-pointer">
                    <input
                        type="checkbox"
                        class="toggle toggle-sm toggle-secondary"
                        checked={pluginTabsVisible()}
                        onChange={(e) => getPluginAPI()?.showPluginTabs(e.target.checked)}
                    />
                    <span class="text-sm">Tabs</span>
                </label>
                <label class="flex items-center gap-2 cursor-pointer">
                    <input
                        type="checkbox"
                        class="toggle toggle-sm toggle-secondary"
                        checked={toolbarVisible()}
                        onChange={(e) => getPluginAPI()?.showToolbar(e.target.checked)}
                    />
                    <span class="text-sm">Toolbar</span>
                </label>
            </div>

            <button
                class="btn btn-primary mt-4"
                onClick={() => getPluginAPI()?.toggleFullscreen()}
            >
                {fullscreenMode() ? 'Exit Fullscreen' : 'Fullscreen'}
            </button>
        </div>
    );
}
