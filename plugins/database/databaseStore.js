import { createSignal } from 'solid-js';
import { bridgeFetch } from '@/api/bridge.js';

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

export const databaseStore = {
  tables,
  selectedTable,
  schema,
  query,
  setQuery,
  results,
  loading,
  error,
  success,
  currentPage,
  setCurrentPage,
  itemsPerPage,
  loadTables,
  handleTableSelect,
  executeQuery,
  executeQueryWithText,
  setCommonQuery,
};
