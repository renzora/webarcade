import { createSignal, onMount, For, Show, createMemo } from 'solid-js';
import { bridgeFetch } from '@/api/bridge.js';
import { IconDatabase, IconTable, IconPlayerPlay, IconAlertCircle, IconCheck, IconX, IconDownload, IconChevronLeft, IconChevronRight } from '@tabler/icons-solidjs';

export default function DatabaseViewport() {
  const [tables, setTables] = createSignal([]);
  const [selectedTable, setSelectedTable] = createSignal('');
  const [schema, setSchema] = createSignal('');
  const [query, setQuery] = createSignal('');
  const [results, setResults] = createSignal(null);
  const [loading, setLoading] = createSignal(false);
  const [error, setError] = createSignal('');
  const [success, setSuccess] = createSignal('');
  const [currentPage, setCurrentPage] = createSignal(1);
  const [itemsPerPage] = createSignal(50);

  // Common queries
  const commonQueries = [
    { label: 'All Counters', query: 'SELECT * FROM counters ORDER BY count DESC LIMIT 100' },
    { label: 'All Watchtime', query: 'SELECT * FROM watchtime ORDER BY total_minutes DESC LIMIT 100' },
    { label: 'All User Levels', query: 'SELECT * FROM user_levels ORDER BY level DESC, xp DESC LIMIT 100' },
    { label: 'All Todos', query: 'SELECT * FROM todos WHERE completed = 0 ORDER BY created_at DESC LIMIT 100' },
    { label: 'TTS Settings', query: 'SELECT * FROM tts_settings' },
    { label: 'Stream Uptime', query: 'SELECT * FROM stream_uptime' },
    { label: 'All Tables', query: "SELECT name FROM sqlite_master WHERE type='table' ORDER BY name" },
  ];

  onMount(async () => {
    await loadTables();
  });

  const loadTables = async () => {
    try {
      const response = await bridgeFetch('/database/tables');
      const data = await response.json();
      setTables(data || []);
    } catch (e) {
      console.error('Failed to load tables:', e);
      setError('Failed to load database tables');
    }
  };

  const loadTableSchema = async (tableName) => {
    if (!tableName) return;

    try {
      const response = await bridgeFetch(`/database/schema?table=${tableName}`);
      const data = await response.json();
      setSchema(data.schema || '');
      // Auto-fill query with SELECT * from table
      const queryText = `SELECT * FROM ${tableName} LIMIT 100`;
      setQuery(queryText);
      // Auto-execute the query to show the table fields
      await executeQueryWithText(queryText);
    } catch (e) {
      console.error('Failed to load schema:', e);
      setError(`Failed to load schema for ${tableName}`);
    }
  };

  const handleTableSelect = (tableName) => {
    setSelectedTable(tableName);
    setResults(null);
    setError('');
    setSuccess('');
    loadTableSchema(tableName);
  };

  const executeQueryWithText = async (queryText) => {
    if (!queryText || !queryText.trim()) {
      setError('Please enter a query');
      return;
    }

    setLoading(true);
    setError('');
    setSuccess('');
    setResults(null);

    try {
      const response = await bridgeFetch('/database/query', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ query: queryText.trim() }),
      });

      const data = await response.json();

      if (data.success) {
        if (data.data) {
          // SELECT query
          setResults(data.data);
          setCurrentPage(1); // Reset to page 1 on new results
          setSuccess(`Query executed successfully. ${data.count} row(s) returned.`);
        } else {
          // Write query (INSERT/UPDATE/DELETE)
          setSuccess(data.message || `Query executed successfully. ${data.rows_affected} row(s) affected.`);
          // Reload tables in case schema changed
          await loadTables();
        }
      } else {
        setError(data.error || 'Query execution failed');
      }
    } catch (e) {
      console.error('Failed to execute query:', e);
      setError(`Failed to execute query: ${e.message}`);
    } finally {
      setLoading(false);
    }
  };

  const executeQuery = async () => {
    await executeQueryWithText(query());
  };

  const setCommonQuery = (queryText) => {
    setQuery(queryText);
    setResults(null);
    setError('');
    setSuccess('');
  };

  const exportToCSV = () => {
    const data = results();
    if (!data || data.length === 0) return;

    // Get column names from first row
    const columns = Object.keys(data[0]);

    // Create CSV content
    let csv = columns.join(',') + '\n';
    data.forEach(row => {
      const values = columns.map(col => {
        const val = row[col];
        // Escape values containing commas or quotes
        if (typeof val === 'string' && (val.includes(',') || val.includes('"'))) {
          return `"${val.replace(/"/g, '""')}"`;
        }
        return val;
      });
      csv += values.join(',') + '\n';
    });

    // Download
    const blob = new Blob([csv], { type: 'text/csv' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `query_results_${Date.now()}.csv`;
    a.click();
    URL.revokeObjectURL(url);
  };

  const resultColumns = createMemo(() => {
    const data = results();
    if (!data || data.length === 0) return [];
    return Object.keys(data[0]);
  });

  // Pagination computed values
  const totalPages = createMemo(() => {
    const data = results();
    if (!data || data.length === 0) return 0;
    return Math.ceil(data.length / itemsPerPage());
  });

  const paginatedResults = createMemo(() => {
    const data = results();
    if (!data || data.length === 0) return [];
    const start = (currentPage() - 1) * itemsPerPage();
    const end = start + itemsPerPage();
    return data.slice(start, end);
  });

  const goToNextPage = () => {
    if (currentPage() < totalPages()) {
      setCurrentPage(currentPage() + 1);
    }
  };

  const goToPreviousPage = () => {
    if (currentPage() > 1) {
      setCurrentPage(currentPage() - 1);
    }
  };

  return (
    <div class="h-full flex bg-base-200">
      {/* Sidebar - Tables List */}
      <div class="w-64 bg-base-100 border-r border-base-300 flex flex-col">
        <div class="p-4 border-b border-base-300">
          <div class="flex items-center gap-2">
            <IconDatabase size={20} class="text-primary" />
            <h3 class="font-semibold">Database Tables</h3>
          </div>
        </div>

        <div class="flex-1 overflow-y-auto p-2">
          <Show when={tables().length > 0} fallback={
            <div class="text-center p-4 text-sm text-base-content/60">
              No tables found
            </div>
          }>
            <For each={tables()}>
              {(table) => (
                <button
                  class={`w-full text-left px-3 py-2 rounded text-sm hover:bg-base-200 transition-colors flex items-center gap-2 ${
                    selectedTable() === table ? 'bg-primary text-primary-content' : ''
                  }`}
                  onClick={() => handleTableSelect(table)}
                >
                  <IconTable size={16} />
                  {table}
                </button>
              )}
            </For>
          </Show>
        </div>

        {/* Common Queries */}
        <div class="border-t border-base-300 p-2">
          <div class="text-xs font-semibold text-base-content/60 px-3 py-2">Quick Queries</div>
          <For each={commonQueries}>
            {(cq) => (
              <button
                class="w-full text-left px-3 py-1.5 rounded text-xs hover:bg-base-200 transition-colors"
                onClick={() => setCommonQuery(cq.query)}
              >
                {cq.label}
              </button>
            )}
          </For>
        </div>
      </div>

      {/* Main Content */}
      <div class="flex-1 flex flex-col">
        {/* Header */}
        <div class="bg-base-100 border-b border-base-300 px-4 py-3">
          <div class="flex items-center gap-3">
            <IconDatabase size={20} class="text-primary" />
            <h2 class="text-lg font-semibold">SQL Query Editor</h2>
            <div class="text-sm text-base-content/60">
              (SQLite Database)
            </div>
          </div>
        </div>

        {/* Schema Display */}
        <Show when={selectedTable() && schema()}>
          <div class="bg-base-100 border-b border-base-300 px-4 py-2">
            <details class="collapse collapse-arrow bg-base-200 rounded">
              <summary class="collapse-title text-sm font-medium min-h-0 py-2">
                Table Schema: {selectedTable()}
              </summary>
              <div class="collapse-content">
                <pre class="text-xs bg-base-300 p-3 rounded overflow-x-auto">{schema()}</pre>
              </div>
            </details>
          </div>
        </Show>

        {/* Query Editor */}
        <div class="bg-base-100 border-b border-base-300 p-4">
          <div class="space-y-2">
            <div class="flex items-center justify-between">
              <label class="text-sm font-semibold">SQL Query</label>
              <button
                class="btn btn-primary btn-sm gap-2"
                onClick={executeQuery}
                disabled={loading() || !query().trim()}
              >
                <IconPlayerPlay size={16} />
                {loading() ? 'Executing...' : 'Execute Query'}
              </button>
            </div>
            <textarea
              class="textarea textarea-bordered w-full font-mono text-sm"
              rows="8"
              placeholder="Enter your SQL query here...&#10;&#10;Examples:&#10;SELECT * FROM counters LIMIT 10&#10;SELECT username, total_minutes FROM watchtime WHERE total_minutes > 100&#10;UPDATE counters SET count = 0 WHERE task = 'deaths'"
              value={query()}
              onInput={(e) => setQuery(e.target.value)}
            />
          </div>

          {/* Status Messages */}
          <Show when={error()}>
            <div class="alert alert-error mt-3">
              <IconX size={20} />
              <span class="text-sm">{error()}</span>
            </div>
          </Show>

          <Show when={success()}>
            <div class="alert alert-success mt-3">
              <IconCheck size={20} />
              <span class="text-sm">{success()}</span>
            </div>
          </Show>
        </div>

        {/* Results */}
        <div class="flex-1 overflow-hidden flex flex-col">
          <Show
            when={results() && results().length > 0}
            fallback={
              <div class="flex items-center justify-center h-full">
                <div class="text-center">
                  <IconAlertCircle size={48} class="mx-auto mb-4 opacity-30" />
                  <p class="text-sm text-base-content/60">
                    {loading() ? 'Executing query...' : 'No results to display. Execute a query to see results.'}
                  </p>
                </div>
              </div>
            }
          >
            <div class="bg-base-100 border-b border-base-300 px-4 py-2 flex items-center justify-between">
              <div class="flex items-center gap-3">
                <div class="text-sm font-semibold">
                  Results ({results().length} total rows)
                </div>
                <Show when={totalPages() > 1}>
                  <div class="flex items-center gap-2">
                    <button
                      class="btn btn-xs btn-outline"
                      onClick={goToPreviousPage}
                      disabled={currentPage() === 1}
                    >
                      <IconChevronLeft size={14} />
                    </button>
                    <span class="text-xs">
                      Page {currentPage()} of {totalPages()}
                    </span>
                    <button
                      class="btn btn-xs btn-outline"
                      onClick={goToNextPage}
                      disabled={currentPage() === totalPages()}
                    >
                      <IconChevronRight size={14} />
                    </button>
                  </div>
                </Show>
              </div>
              <button
                class="btn btn-sm btn-outline gap-2"
                onClick={exportToCSV}
              >
                <IconDownload size={16} />
                Export CSV
              </button>
            </div>

            <div class="flex-1 overflow-hidden p-4">
              <div class="overflow-x-auto overflow-y-auto h-full">
                <table class="table table-zebra table-sm table-pin-rows">
                  <thead>
                    <tr>
                      <For each={resultColumns()}>
                        {(column) => (
                          <th class="bg-base-200 font-bold">{column}</th>
                        )}
                      </For>
                    </tr>
                  </thead>
                  <tbody>
                    <For each={paginatedResults()}>
                      {(row) => (
                        <tr>
                          <For each={resultColumns()}>
                            {(column) => (
                              <td class="font-mono text-xs max-w-xs truncate">
                                {row[column] !== null && row[column] !== undefined
                                  ? String(row[column])
                                  : <span class="text-base-content/40">NULL</span>
                                }
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
    </div>
  );
}
