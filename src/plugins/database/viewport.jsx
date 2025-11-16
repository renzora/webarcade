import { For, Show, createMemo, createSignal } from 'solid-js';
import { IconDatabase, IconPlayerPlay, IconAlertCircle, IconCheck, IconX, IconDownload, IconChevronLeft, IconChevronRight, IconEdit, IconTrash, IconPlus, IconColumns, IconDatabaseExport, IconDatabaseImport, IconUpload, IconCode } from '@tabler/icons-solidjs';
import { databaseStore } from './store';
import { api } from '@/api/bridge';

export default function DatabaseViewport() {
  const [editingRow, setEditingRow] = createSignal(null);
  const [editFormData, setEditFormData] = createSignal({});
  const [showAddModal, setShowAddModal] = createSignal(false);
  const [showAddColumnModal, setShowAddColumnModal] = createSignal(false);
  const [newColumnData, setNewColumnData] = createSignal({
    name: '',
    type: 'TEXT',
    notNull: false,
    defaultValue: ''
  });
  const [showImportModal, setShowImportModal] = createSignal(false);
  const [importFile, setImportFile] = createSignal(null);
  const [importing, setImporting] = createSignal(false);
  const [exporting, setExporting] = createSignal(false);
  const [showQueryInput, setShowQueryInput] = createSignal(false);

  const exportToCSV = () => {
    const data = databaseStore.results();
    if (!data || data.length === 0) return;

    const columns = Object.keys(data[0]);

    let csv = columns.join(',') + '\n';
    data.forEach(row => {
      const values = columns.map(col => {
        const val = row[col];
        if (typeof val === 'string' && (val.includes(',') || val.includes('"'))) {
          return `"${val.replace(/"/g, '""')}"`;
        }
        return val;
      });
      csv += values.join(',') + '\n';
    });

    const blob = new Blob([csv], { type: 'text/csv' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `query_results_${Date.now()}.csv`;
    a.click();
    URL.revokeObjectURL(url);
  };

  const resultColumns = createMemo(() => {
    const data = databaseStore.results();
    if (!data || data.length === 0) return [];
    return Object.keys(data[0]);
  });

  const totalPages = createMemo(() => {
    const data = databaseStore.results();
    if (!data || data.length === 0) return 0;
    return Math.ceil(data.length / databaseStore.itemsPerPage());
  });

  const paginatedResults = createMemo(() => {
    const data = databaseStore.results();
    if (!data || data.length === 0) return [];
    const start = (databaseStore.currentPage() - 1) * databaseStore.itemsPerPage();
    const end = start + databaseStore.itemsPerPage();
    return data.slice(start, end);
  });

  const goToNextPage = () => {
    if (databaseStore.currentPage() < totalPages()) {
      databaseStore.setCurrentPage(databaseStore.currentPage() + 1);
    }
  };

  const goToPreviousPage = () => {
    if (databaseStore.currentPage() > 1) {
      databaseStore.setCurrentPage(databaseStore.currentPage() - 1);
    }
  };

  const handleEditRow = (row) => {
    setEditFormData(JSON.parse(JSON.stringify(row)));
    setEditingRow(row);
  };

  const handleDeleteRow = async (row) => {
    if (!databaseStore.selectedTable()) {
      alert('No table selected');
      return;
    }

    const columns = resultColumns();
    const pkColumn = columns[0];

    if (!confirm(`Are you sure you want to delete this row (${pkColumn}: ${row[pkColumn]})?`)) {
      return;
    }

    try {
      const deleteQuery = `DELETE FROM ${databaseStore.selectedTable()} WHERE ${pkColumn} = ${typeof row[pkColumn] === 'string' ? `'${row[pkColumn]}'` : row[pkColumn]}`;
      await databaseStore.executeQueryWithText(deleteQuery);
      const refreshQuery = `SELECT * FROM ${databaseStore.selectedTable()} LIMIT 100`;
      await databaseStore.executeQueryWithText(refreshQuery);
    } catch (error) {
      alert(`Failed to delete row: ${error.message}`);
    }
  };

  const handleSaveEdit = async () => {
    if (!databaseStore.selectedTable() || !editingRow()) {
      return;
    }

    try {
      const columns = resultColumns();
      const pkColumn = columns[0];

      const setClauses = columns
        .filter(col => col !== pkColumn)
        .map(col => {
          const value = editFormData()[col];
          return `${col} = ${typeof value === 'string' ? `'${value.replace(/'/g, "''")}'` : value}`;
        })
        .join(', ');

      const whereValue = editingRow()[pkColumn];
      const updateQuery = `UPDATE ${databaseStore.selectedTable()} SET ${setClauses} WHERE ${pkColumn} = ${typeof whereValue === 'string' ? `'${whereValue}'` : whereValue}`;

      await databaseStore.executeQueryWithText(updateQuery);

      const refreshQuery = `SELECT * FROM ${databaseStore.selectedTable()} LIMIT 100`;
      await databaseStore.executeQueryWithText(refreshQuery);

      setEditingRow(null);
      setEditFormData({});
    } catch (error) {
      alert(`Failed to update row: ${error.message}`);
    }
  };

  const handleAddRow = async () => {
    if (!databaseStore.selectedTable()) {
      alert('No table selected');
      return;
    }

    try {
      const columns = resultColumns();
      const values = columns.map(col => {
        const value = editFormData()[col];
        if (value === null || value === undefined || value === '') {
          return 'NULL';
        }
        return typeof value === 'string' ? `'${value.replace(/'/g, "''")}'` : value;
      });

      const insertQuery = `INSERT INTO ${databaseStore.selectedTable()} (${columns.join(', ')}) VALUES (${values.join(', ')})`;
      await databaseStore.executeQueryWithText(insertQuery);

      const refreshQuery = `SELECT * FROM ${databaseStore.selectedTable()} LIMIT 100`;
      await databaseStore.executeQueryWithText(refreshQuery);

      setShowAddModal(false);
      setEditFormData({});
    } catch (error) {
      alert(`Failed to add row: ${error.message}`);
    }
  };

  const openAddModal = () => {
    const columns = resultColumns();
    const emptyData = {};
    columns.forEach(col => emptyData[col] = '');
    setEditFormData(emptyData);
    setShowAddModal(true);
  };

  const handleAddColumn = async () => {
    if (!databaseStore.selectedTable()) {
      alert('No table selected');
      return;
    }

    const colData = newColumnData();

    if (!colData.name || !colData.name.trim()) {
      alert('Column name is required');
      return;
    }

    if (!/^[a-zA-Z_][a-zA-Z0-9_]*$/.test(colData.name)) {
      alert('Invalid column name. Use only letters, numbers, and underscores. Must start with a letter or underscore.');
      return;
    }

    try {
      let alterQuery = `ALTER TABLE ${databaseStore.selectedTable()} ADD COLUMN ${colData.name} ${colData.type}`;

      if (colData.notNull) {
        alterQuery += ' NOT NULL';
      }

      if (colData.defaultValue && colData.defaultValue.trim()) {
        const defaultVal = colData.type === 'TEXT'
          ? `'${colData.defaultValue.replace(/'/g, "''")}'`
          : colData.defaultValue;
        alterQuery += ` DEFAULT ${defaultVal}`;
      }

      await databaseStore.executeQueryWithText(alterQuery);

      const refreshQuery = `SELECT * FROM ${databaseStore.selectedTable()} LIMIT 100`;
      await databaseStore.executeQueryWithText(refreshQuery);

      setNewColumnData({
        name: '',
        type: 'TEXT',
        notNull: false,
        defaultValue: ''
      });
      setShowAddColumnModal(false);
    } catch (error) {
      alert(`Failed to add column: ${error.message}`);
    }
  };

  const handleExportDatabase = async () => {
    setExporting(true);
    try {
      const response = await api('database/export');

      if (!response.ok) {
        throw new Error('Failed to export database');
      }

      const blob = await response.blob();
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `database_backup_${new Date().toISOString().replace(/[:.]/g, '-')}.db`;
      a.click();
      URL.revokeObjectURL(url);

      alert('Database exported successfully!');
    } catch (error) {
      alert(`Failed to export database: ${error.message}`);
    } finally {
      setExporting(false);
    }
  };

  const handleImportDatabase = async () => {
    const file = importFile();
    if (!file) {
      alert('Please select a file to import');
      return;
    }

    if (!confirm('⚠️ WARNING: This will replace your entire database! All current data will be lost. Are you sure you want to continue?')) {
      return;
    }

    setImporting(true);
    try {
      const formData = new FormData();
      formData.append('database', file);

      const response = await api('database/import', {
        method: 'POST',
        body: formData,
      });

      if (!response.ok) {
        const error = await response.text();
        throw new Error(error || 'Failed to import database');
      }

      alert('Database imported successfully! Reloading...');

      await databaseStore.loadTables();

      databaseStore.setQuery('');

      setShowImportModal(false);
      setImportFile(null);

      window.location.reload();
    } catch (error) {
      alert(`Failed to import database: ${error.message}`);
    } finally {
      setImporting(false);
    }
  };

  const handleFileSelect = (e) => {
    const file = e.target.files?.[0];
    if (file) {
      setImportFile(file);
    }
  };

  return (
    <div class="h-full w-full flex overflow-hidden">
      <div class="flex-1 min-w-0 flex flex-col">
        <div class="px-4 py-3 flex-shrink-0">
          <div class="flex items-center justify-between">
            <div class="flex items-center gap-3">
              <IconDatabase size={20} class="text-primary" />
              <h2 class="text-lg font-semibold">SQL Query Editor</h2>
              <div class="text-sm text-base-content/60">
                (SQLite Database)
              </div>
            </div>
            <div class="flex items-center gap-2">
              <button
                class={`btn btn-sm gap-2 ${showQueryInput() ? 'btn-primary' : 'btn-outline'}`}
                onClick={() => setShowQueryInput(!showQueryInput())}
                title="Toggle SQL query input"
              >
                <IconCode size={16} />
                {showQueryInput() ? 'Hide Query' : 'Show Query'}
              </button>
              <button
                class="btn btn-sm btn-outline gap-2"
                onClick={handleExportDatabase}
                disabled={exporting()}
                title="Export entire database as backup"
              >
                <IconDatabaseExport size={16} />
                {exporting() ? 'Exporting...' : 'Export DB'}
              </button>
              <button
                class="btn btn-sm btn-outline gap-2"
                onClick={() => setShowImportModal(true)}
                title="Import database from backup"
              >
                <IconDatabaseImport size={16} />
                Import DB
              </button>
            </div>
          </div>
        </div>

        <Show when={showQueryInput()}>
          <div class="bg-base-100 border-b border-base-300 p-4 flex-shrink-0">
          <div class="space-y-2">
            <div class="flex items-center justify-between">
              <label class="text-sm font-semibold">SQL Query</label>
              <button
                class="btn btn-primary btn-sm gap-2 flex-shrink-0"
                onClick={databaseStore.executeQuery}
                disabled={databaseStore.loading() || !databaseStore.query().trim()}
              >
                <IconPlayerPlay size={16} />
                {databaseStore.loading() ? 'Executing...' : 'Execute Query'}
              </button>
            </div>
            <textarea
              class="textarea textarea-bordered w-full font-mono text-sm resize-none"
              rows="8"
              placeholder="Enter your SQL query here...&#10;&#10;Examples:&#10;SELECT * FROM counters LIMIT 10&#10;SELECT username, total_minutes FROM watchtime WHERE total_minutes > 100&#10;UPDATE counters SET count = 0 WHERE task = 'deaths'"
              value={databaseStore.query()}
              onInput={(e) => databaseStore.setQuery(e.target.value)}
            />
          </div>

          <Show when={databaseStore.error()}>
            <div class="alert alert-error mt-3">
              <IconX size={20} />
              <span class="text-sm">{databaseStore.error()}</span>
            </div>
          </Show>

          <Show when={databaseStore.success()}>
            <div class="alert alert-success mt-3">
              <IconCheck size={20} />
              <span class="text-sm">{databaseStore.success()}</span>
            </div>
          </Show>
          </div>
        </Show>

        <div class="flex-1 min-h-0 overflow-hidden flex flex-col">
          <Show
            when={databaseStore.results() && databaseStore.results().length > 0}
            fallback={
              <div class="flex items-center justify-center h-full">
                <div class="text-center">
                  <IconAlertCircle size={48} class="mx-auto mb-4 opacity-30" />
                  <p class="text-sm text-base-content/60">
                    {databaseStore.loading() ? 'Executing query...' : 'No results to display. Execute a query to see results.'}
                  </p>
                </div>
              </div>
            }
          >
            <div class="bg-base-100 border-b border-base-300 px-4 py-2 flex items-center justify-between flex-shrink-0">
              <div class="flex items-center gap-3 min-w-0 flex-1">
                <div class="text-sm font-semibold flex-shrink-0">
                  Results ({databaseStore.results().length} total rows)
                </div>
                <Show when={totalPages() > 1}>
                  <div class="flex items-center gap-2 flex-shrink-0">
                    <button
                      class="btn btn-xs btn-outline"
                      onClick={goToPreviousPage}
                      disabled={databaseStore.currentPage() === 1}
                    >
                      <IconChevronLeft size={14} />
                    </button>
                    <span class="text-xs whitespace-nowrap">
                      Page {databaseStore.currentPage()} of {totalPages()}
                    </span>
                    <button
                      class="btn btn-xs btn-outline"
                      onClick={goToNextPage}
                      disabled={databaseStore.currentPage() === totalPages()}
                    >
                      <IconChevronRight size={14} />
                    </button>
                  </div>
                </Show>
              </div>
              <div class="flex items-center gap-2 flex-shrink-0 ml-2">
                <button
                  class="btn btn-sm btn-info gap-2"
                  onClick={() => setShowAddColumnModal(true)}
                  disabled={!databaseStore.selectedTable()}
                  title="Add a new column to the table"
                >
                  <IconColumns size={16} />
                  Add Column
                </button>
                <button
                  class="btn btn-sm btn-success gap-2"
                  onClick={openAddModal}
                  disabled={!databaseStore.selectedTable()}
                >
                  <IconPlus size={16} />
                  Add Row
                </button>
                <button
                  class="btn btn-sm btn-outline gap-2"
                  onClick={exportToCSV}
                >
                  <IconDownload size={16} />
                  Export CSV
                </button>
              </div>
            </div>

            <div class="flex-1 min-h-0 overflow-hidden p-4">
              <div class="w-full h-full overflow-auto">
                <table class="table table-zebra table-sm table-pin-rows">
                  <thead>
                    <tr>
                      <th class="bg-base-200 font-bold whitespace-nowrap">Actions</th>
                      <For each={resultColumns()}>
                        {(column) => (
                          <th class="bg-base-200 font-bold whitespace-nowrap">{column}</th>
                        )}
                      </For>
                    </tr>
                  </thead>
                  <tbody>
                    <For each={paginatedResults()}>
                      {(row) => (
                        <tr>
                          <td class="whitespace-nowrap">
                            <div class="flex items-center gap-1">
                              <button
                                class="btn btn-xs btn-ghost text-primary hover:text-primary-focus"
                                onClick={() => handleEditRow(row)}
                                title="Edit row"
                              >
                                <IconEdit size={14} />
                              </button>
                              <button
                                class="btn btn-xs btn-ghost text-error hover:text-error-focus"
                                onClick={() => handleDeleteRow(row)}
                                title="Delete row"
                              >
                                <IconTrash size={14} />
                              </button>
                            </div>
                          </td>
                          <For each={resultColumns()}>
                            {(column) => (
                              <td class="font-mono text-xs whitespace-nowrap">
                                <div class="max-w-xs overflow-hidden text-ellipsis">
                                  {row[column] !== null && row[column] !== undefined
                                    ? String(row[column])
                                    : <span class="text-base-content/40">NULL</span>
                                  }
                                </div>
                              </td>
                            )}
                          </For>
                        </tr>
                      )}
                    </For>
                  </tbody>
                </table>
              </div>
            </div>
          </Show>
        </div>
      </div>

      <Show when={editingRow()}>
        <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" onClick={() => setEditingRow(null)}>
          <div class="bg-base-100 rounded-lg shadow-xl max-w-2xl w-full mx-4 max-h-[80vh] overflow-hidden flex flex-col" onClick={(e) => e.stopPropagation()}>
            <div class="p-4 border-b border-base-300 flex items-center justify-between">
              <h3 class="text-lg font-semibold flex items-center gap-2">
                <IconEdit size={20} />
                Edit Row
              </h3>
              <button class="btn btn-ghost btn-sm btn-circle" onClick={() => setEditingRow(null)}>
                <IconX size={20} />
              </button>
            </div>

            <div class="flex-1 overflow-y-auto p-4">
              <div class="space-y-3">
                <For each={resultColumns()}>
                  {(column) => (
                    <div class="form-control">
                      <label class="label">
                        <span class="label-text font-semibold">{column}</span>
                      </label>
                      <input
                        type="text"
                        class="input input-bordered w-full"
                        value={editFormData()[column] ?? ''}
                        onInput={(e) => setEditFormData({ ...editFormData(), [column]: e.target.value })}
                      />
                    </div>
                  )}
                </For>
              </div>
            </div>

            <div class="p-4 border-t border-base-300 flex justify-end gap-2">
              <button class="btn btn-ghost" onClick={() => setEditingRow(null)}>
                Cancel
              </button>
              <button class="btn btn-primary" onClick={handleSaveEdit}>
                Save Changes
              </button>
            </div>
          </div>
        </div>
      </Show>

      <Show when={showAddModal()}>
        <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" onClick={() => setShowAddModal(false)}>
          <div class="bg-base-100 rounded-lg shadow-xl max-w-2xl w-full mx-4 max-h-[80vh] overflow-hidden flex flex-col" onClick={(e) => e.stopPropagation()}>
            <div class="p-4 border-b border-base-300 flex items-center justify-between">
              <h3 class="text-lg font-semibold flex items-center gap-2">
                <IconPlus size={20} />
                Add New Row
              </h3>
              <button class="btn btn-ghost btn-sm btn-circle" onClick={() => setShowAddModal(false)}>
                <IconX size={20} />
              </button>
            </div>

            <div class="flex-1 overflow-y-auto p-4">
              <div class="space-y-3">
                <For each={resultColumns()}>
                  {(column) => (
                    <div class="form-control">
                      <label class="label">
                        <span class="label-text font-semibold">{column}</span>
                      </label>
                      <input
                        type="text"
                        class="input input-bordered w-full"
                        value={editFormData()[column] ?? ''}
                        onInput={(e) => setEditFormData({ ...editFormData(), [column]: e.target.value })}
                        placeholder="Leave empty for NULL"
                      />
                    </div>
                  )}
                </For>
              </div>
            </div>

            <div class="p-4 border-t border-base-300 flex justify-end gap-2">
              <button class="btn btn-ghost" onClick={() => setShowAddModal(false)}>
                Cancel
              </button>
              <button class="btn btn-success" onClick={handleAddRow}>
                Add Row
              </button>
            </div>
          </div>
        </div>
      </Show>

      <Show when={showImportModal()}>
        <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" onClick={() => setShowImportModal(false)}>
          <div class="bg-base-100 rounded-lg shadow-xl max-w-xl w-full mx-4" onClick={(e) => e.stopPropagation()}>
            <div class="p-4 border-b border-base-300 flex items-center justify-between">
              <h3 class="text-lg font-semibold flex items-center gap-2">
                <IconDatabaseImport size={20} />
                Import Database
              </h3>
              <button class="btn btn-ghost btn-sm btn-circle" onClick={() => setShowImportModal(false)}>
                <IconX size={20} />
              </button>
            </div>

            <div class="p-4">
              <div class="space-y-4">
                <div class="alert alert-error">
                  <IconAlertCircle size={20} />
                  <div>
                    <div class="font-bold">⚠️ Destructive Operation</div>
                    <div class="text-sm">This will completely replace your current database. All existing data will be permanently lost!</div>
                  </div>
                </div>

                <div class="form-control">
                  <label class="label">
                    <span class="label-text font-semibold">Select Database File (.db)</span>
                  </label>
                  <input
                    type="file"
                    accept=".db,.sqlite,.sqlite3"
                    class="file-input file-input-bordered w-full"
                    onChange={handleFileSelect}
                  />
                  <label class="label">
                    <span class="label-text-alt text-base-content/60">
                      {importFile() ? `Selected: ${importFile().name}` : 'No file selected'}
                    </span>
                  </label>
                </div>

                <div class="bg-base-200 p-3 rounded-lg">
                  <h4 class="font-semibold text-sm mb-2">Before importing:</h4>
                  <ul class="text-xs space-y-1 list-disc list-inside text-base-content/70">
                    <li>Make sure you have a backup of your current database</li>
                    <li>The file should be a valid SQLite database (.db file)</li>
                    <li>The application will reload after import</li>
                  </ul>
                </div>
              </div>
            </div>

            <div class="p-4 border-t border-base-300 flex justify-end gap-2">
              <button class="btn btn-ghost" onClick={() => setShowImportModal(false)}>
                Cancel
              </button>
              <button
                class="btn btn-error gap-2"
                onClick={handleImportDatabase}
                disabled={!importFile() || importing()}
              >
                <IconUpload size={16} />
                {importing() ? 'Importing...' : 'Import & Replace Database'}
              </button>
            </div>
          </div>
        </div>
      </Show>

      <Show when={showAddColumnModal()}>
        <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" onClick={() => setShowAddColumnModal(false)}>
          <div class="bg-base-100 rounded-lg shadow-xl max-w-xl w-full mx-4" onClick={(e) => e.stopPropagation()}>
            <div class="p-4 border-b border-base-300 flex items-center justify-between">
              <h3 class="text-lg font-semibold flex items-center gap-2">
                <IconColumns size={20} />
                Add New Column to {databaseStore.selectedTable()}
              </h3>
              <button class="btn btn-ghost btn-sm btn-circle" onClick={() => setShowAddColumnModal(false)}>
                <IconX size={20} />
              </button>
            </div>

            <div class="p-4">
              <div class="space-y-4">
                <div class="form-control">
                  <label class="label">
                    <span class="label-text font-semibold">Column Name *</span>
                  </label>
                  <input
                    type="text"
                    class="input input-bordered w-full"
                    value={newColumnData().name}
                    onInput={(e) => setNewColumnData({ ...newColumnData(), name: e.target.value })}
                    placeholder="e.g., email, age, created_at"
                  />
                  <label class="label">
                    <span class="label-text-alt text-base-content/60">Use letters, numbers, and underscores only</span>
                  </label>
                </div>

                <div class="form-control">
                  <label class="label">
                    <span class="label-text font-semibold">Data Type *</span>
                  </label>
                  <select
                    class="select select-bordered w-full"
                    value={newColumnData().type}
                    onChange={(e) => setNewColumnData({ ...newColumnData(), type: e.target.value })}
                  >
                    <option value="TEXT">TEXT</option>
                    <option value="INTEGER">INTEGER</option>
                    <option value="REAL">REAL</option>
                    <option value="BLOB">BLOB</option>
                    <option value="NUMERIC">NUMERIC</option>
                  </select>
                  <label class="label">
                    <span class="label-text-alt text-base-content/60">
                      TEXT: strings | INTEGER: whole numbers | REAL: decimals
                    </span>
                  </label>
                </div>

                <div class="form-control">
                  <label class="label">
                    <span class="label-text font-semibold">Default Value (Optional)</span>
                  </label>
                  <input
                    type="text"
                    class="input input-bordered w-full"
                    value={newColumnData().defaultValue}
                    onInput={(e) => setNewColumnData({ ...newColumnData(), defaultValue: e.target.value })}
                    placeholder="Leave empty for NULL"
                  />
                  <label class="label">
                    <span class="label-text-alt text-base-content/60">
                      Default value for new rows
                    </span>
                  </label>
                </div>

                <div class="form-control">
                  <label class="label cursor-pointer justify-start gap-3">
                    <input
                      type="checkbox"
                      class="checkbox"
                      checked={newColumnData().notNull}
                      onChange={(e) => setNewColumnData({ ...newColumnData(), notNull: e.target.checked })}
                    />
                    <div>
                      <span class="label-text font-semibold">NOT NULL</span>
                      <div class="label-text-alt text-base-content/60">
                        Require a value for this column (cannot be empty)
                      </div>
                    </div>
                  </label>
                </div>

                <div class="alert alert-warning">
                  <IconAlertCircle size={20} />
                  <div class="text-sm">
                    <strong>Warning:</strong> Adding a column with NOT NULL requires a default value for existing rows.
                  </div>
                </div>
              </div>
            </div>

            <div class="p-4 border-t border-base-300 flex justify-end gap-2">
              <button class="btn btn-ghost" onClick={() => setShowAddColumnModal(false)}>
                Cancel
              </button>
              <button class="btn btn-info" onClick={handleAddColumn}>
                <IconPlus size={16} />
                Add Column
              </button>
            </div>
          </div>
        </div>
      </Show>
    </div>
  );
}
