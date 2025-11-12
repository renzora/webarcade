import { onMount, For, Show, createSignal } from 'solid-js';
import { IconDatabase, IconTable, IconEdit, IconTrash, IconX, IconCheck } from '@tabler/icons-solidjs';
import { databaseStore } from './store';

export default function DatabaseMenu() {
  const [editingTable, setEditingTable] = createSignal(null);
  const [newTableName, setNewTableName] = createSignal('');

  onMount(async () => {
    await databaseStore.loadTables();
  });

  const handleRenameTable = async (oldName) => {
    const newName = newTableName().trim();

    if (!newName) {
      alert('Please enter a new table name');
      return;
    }

    if (!/^[a-zA-Z_][a-zA-Z0-9_]*$/.test(newName)) {
      alert('Invalid table name. Use only letters, numbers, and underscores. Must start with a letter or underscore.');
      return;
    }

    if (!confirm(`Rename table "${oldName}" to "${newName}"?`)) {
      return;
    }

    try {
      await databaseStore.executeQueryWithText(`ALTER TABLE ${oldName} RENAME TO ${newName}`);
      await databaseStore.loadTables();
      setEditingTable(null);
      setNewTableName('');
    } catch (error) {
      alert(`Failed to rename table: ${error.message}`);
    }
  };

  const handleDeleteTable = async (tableName) => {
    if (!confirm(`Are you sure you want to delete the table "${tableName}"? This action cannot be undone!`)) {
      return;
    }

    try {
      await databaseStore.executeQueryWithText(`DROP TABLE ${tableName}`);
      await databaseStore.loadTables();
      if (databaseStore.selectedTable() === tableName) {
        databaseStore.handleTableSelect('');
      }
    } catch (error) {
      alert(`Failed to delete table: ${error.message}`);
    }
  };

  const startRename = (tableName) => {
    setEditingTable(tableName);
    setNewTableName(tableName);
  };

  const cancelRename = () => {
    setEditingTable(null);
    setNewTableName('');
  };

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
              <Show
                when={editingTable() === table}
                fallback={
                  <div
                    class={`w-full px-2 py-1.5 rounded text-sm hover:bg-base-200 transition-colors flex items-center gap-1 group ${
                      databaseStore.selectedTable() === table ? 'bg-primary text-primary-content' : ''
                    }`}
                  >
                    <button
                      class="flex items-center gap-2 flex-1 min-w-0 text-left"
                      onClick={() => databaseStore.handleTableSelect(table)}
                    >
                      <IconTable size={16} class="flex-shrink-0" />
                      <span class="truncate">{table}</span>
                    </button>
                    <div class="flex items-center gap-0.5 opacity-0 group-hover:opacity-100 transition-opacity flex-shrink-0">
                      <button
                        class="btn btn-xs btn-ghost text-info hover:text-info-focus p-1 min-h-0 h-6 w-6"
                        onClick={(e) => {
                          e.stopPropagation();
                          startRename(table);
                        }}
                        title="Rename table"
                      >
                        <IconEdit size={14} />
                      </button>
                      <button
                        class="btn btn-xs btn-ghost text-error hover:text-error-focus p-1 min-h-0 h-6 w-6"
                        onClick={(e) => {
                          e.stopPropagation();
                          handleDeleteTable(table);
                        }}
                        title="Delete table"
                      >
                        <IconTrash size={14} />
                      </button>
                    </div>
                  </div>
                }
              >
                <div class="w-full px-2 py-1.5 rounded bg-base-200 flex items-center gap-1">
                  <input
                    type="text"
                    class="input input-xs flex-1 min-w-0 h-7 px-2 text-xs"
                    value={newTableName()}
                    onInput={(e) => setNewTableName(e.target.value)}
                    onClick={(e) => e.stopPropagation()}
                    onKeyDown={(e) => {
                      if (e.key === 'Enter') {
                        handleRenameTable(table);
                      } else if (e.key === 'Escape') {
                        cancelRename();
                      }
                    }}
                    autofocus
                  />
                  <button
                    class="btn btn-xs btn-success p-1 min-h-0 h-6 w-6"
                    onClick={(e) => {
                      e.stopPropagation();
                      handleRenameTable(table);
                    }}
                    title="Confirm rename"
                  >
                    <IconCheck size={14} />
                  </button>
                  <button
                    class="btn btn-xs btn-ghost p-1 min-h-0 h-6 w-6"
                    onClick={(e) => {
                      e.stopPropagation();
                      cancelRename();
                    }}
                    title="Cancel"
                  >
                    <IconX size={14} />
                  </button>
                </div>
              </Show>
            )}
          </For>
        </Show>
      </div>
    </div>
  );
}
