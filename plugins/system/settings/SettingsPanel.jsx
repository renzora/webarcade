import { createSignal, onMount, For } from 'solid-js';
import { allThemes } from '@/themes';
import { editorStore, editorActions } from '@/layout/stores/EditorStore.jsx';

const SettingsPanel = () => {
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


  const handleThemeChange = (themeName) => {
    // Use editor store to manage theme
    editorActions.setTheme(themeName);
  };

  return (
    <div class="h-full flex flex-col bg-base-100 p-4 overflow-y-auto">
      <h2 class="text-lg font-semibold text-base-content mb-4">Application Settings</h2>

      {/* Theme Selection */}
      <div class="mb-6">
        <h3 class="text-sm font-semibold text-base-content mb-2">Theme</h3>
        <div class="space-y-1">
          <For each={Object.entries(themesByCategory())}>
            {([category, themes]) => (
              <div class="mb-3">
                <div class="text-xs font-semibold text-base-content/60 uppercase tracking-wide mb-1">
                  {category}
                </div>
                <div class="space-y-0.5">
                  <For each={themes}>
                    {(theme) => (
                      <button
                        onClick={() => handleThemeChange(theme.name)}
                        class={`w-full px-3 py-2 text-left text-sm transition-colors rounded ${
                          editorStore.theme === theme.name
                            ? 'bg-primary text-primary-content'
                            : 'bg-base-200 text-base-content hover:bg-base-300'
                        }`}
                      >
                        <span class="flex items-center justify-between">
                          <span>{theme.label}</span>
                          {editorStore.theme === theme.name && <span class="text-sm">✓</span>}
                        </span>
                      </button>
                    )}
                  </For>
                </div>
              </div>
            )}
          </For>
        </div>
      </div>

      {/* Additional Settings can be added here */}
    </div>
  );
};

export default SettingsPanel;
