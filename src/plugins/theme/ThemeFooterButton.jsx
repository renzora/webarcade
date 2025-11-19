import { createSignal, For, Show } from 'solid-js';
import { allThemes } from '@/themes';
import { editorStore, editorActions } from '@/layout/stores/EditorStore.jsx';
import { IconPalette, IconChevronUp } from '@tabler/icons-solidjs';

const ThemeFooterButton = () => {
  const [isOpen, setIsOpen] = createSignal(false);

  // Group themes by category
  const themesByCategory = () => {
    const grouped = {};
    allThemes.forEach(theme => {
      if (!grouped[theme.category]) {
        grouped[theme.category] = [];
      }
      grouped[theme.category].push(theme);
    });
    return grouped;
  };

  const handleThemeSelect = (themeName) => {
    editorActions.setTheme(themeName);
    setIsOpen(false);
  };

  const currentTheme = () => {
    const theme = allThemes.find(t => t.name === editorStore.theme);
    return theme ? theme.label : 'Unknown';
  };

  return (
    <div class="relative">
      {/* Dropdown Menu */}
      <Show when={isOpen()}>
        <div
          class="absolute bottom-full right-0 mb-2 w-64 bg-base-100 rounded-lg shadow-xl border border-base-300 max-h-96 overflow-y-auto"
          style={{ "z-index": "9999" }}
        >
          <div class="sticky top-0 bg-base-100 border-b border-base-300 px-3 py-2">
            <div class="flex items-center gap-2">
              <IconPalette size={16} class="text-base-content/70" />
              <span class="text-sm font-semibold text-base-content">Select Theme</span>
            </div>
          </div>

          <div class="p-2">
            <For each={Object.entries(themesByCategory())}>
              {([category, themes]) => (
                <div class="mb-3 last:mb-0">
                  <div class="px-2 py-1 text-xs font-semibold text-base-content/50 uppercase tracking-wide">
                    {category}
                  </div>
                  <div class="space-y-1">
                    <For each={themes}>
                      {(theme) => (
                        <button
                          class={`w-full text-left px-3 py-2 rounded text-sm transition-colors ${
                            editorStore.theme === theme.name
                              ? 'bg-primary text-primary-content font-medium'
                              : 'hover:bg-base-200 text-base-content'
                          }`}
                          onClick={() => handleThemeSelect(theme.name)}
                        >
                          {theme.label}
                        </button>
                      )}
                    </For>
                  </div>
                </div>
              )}
            </For>
          </div>
        </div>
      </Show>

      {/* Click outside to close */}
      <Show when={isOpen()}>
        <div
          class="fixed inset-0"
          style={{ "z-index": "9998" }}
          onClick={() => setIsOpen(false)}
        />
      </Show>

      {/* Footer Button */}
      <button
        class={`flex items-center gap-2 px-3 py-1 rounded transition-colors ${
          isOpen() ? 'bg-base-300' : 'hover:bg-base-200'
        }`}
        onClick={() => setIsOpen(!isOpen())}
        title="Change theme"
      >
        <IconPalette size={16} class="text-base-content/70" />
        <span class="text-xs text-base-content/80">{currentTheme()}</span>
        <IconChevronUp
          size={14}
          class={`text-base-content/50 transition-transform ${isOpen() ? 'rotate-180' : ''}`}
        />
      </button>
    </div>
  );
};

export default ThemeFooterButton;
