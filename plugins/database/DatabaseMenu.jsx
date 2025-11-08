import { onMount, For, Show } from 'solid-js';
import { IconDatabase, IconTable } from '@tabler/icons-solidjs';
import { databaseStore } from './databaseStore';

export default function DatabaseMenu() {
  onMount(async () => {
    await databaseStore.loadTables();
  });

  return (
    <div class="h-full bg-base-100 flex flex-col">
      <div class="p-4 border-b border-base-300">
        <div class="flex items-center gap-2">
          <IconDatabase size={20} class="text-primary" />
          <h3 class="font-semibold">Database Tables</h3>
        </div>
      </div>

      <div class="flex-1 overflow-y-auto p-2">
        <Show when={databaseStore.tables().length > 0} fallback={
          <div class="text-center p-4 text-sm text-base-content/60">
            No tables found
          </div>
        }>
          <For each={databaseStore.tables()}>
            {(table) => (
              <button
                class={`w-full text-left px-3 py-2 rounded text-sm hover:bg-base-200 transition-colors flex items-center gap-2 ${
                  databaseStore.selectedTable() === table ? 'bg-primary text-primary-content' : ''
                }`}
                onClick={() => databaseStore.handleTableSelect(table)}
              >
                <IconTable size={16} />
                {table}
              </button>
            )}
          </For>
        </Show>
      </div>
    </div>
  );
}
