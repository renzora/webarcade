import { createSignal, onMount, For, Show } from 'solid-js';

const WEBARCADE_API = 'http://localhost:3001';

export default function NotesPanel() {
  const [notes, setNotes] = createSignal([]);
  const [loading, setLoading] = createSignal(true);
  const [quickNote, setQuickNote] = createSignal('');

  onMount(async () => {
    await loadNotes();
    setLoading(false);
  });

  const loadNotes = async () => {
    try {
      const response = await fetch(`${WEBARCADE_API}/notes`);
      const data = await response.json();
      // Show only the 5 most recent notes
      setNotes(data.slice(0, 5));
    } catch (e) {
      console.error('Failed to load notes:', e);
    }
  };

  const createQuickNote = async () => {
    const content = quickNote().trim();
    if (!content) return;

    setLoading(true);
    try {
      await fetch(`${WEBARCADE_API}/notes`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          title: 'Quick Note',
          content,
          category: 'General',
          tags: []
        })
      });
      setQuickNote('');
      await loadNotes();
    } catch (e) {
      console.error('Failed to create quick note:', e);
    }
    setLoading(false);
  };

  const formatDate = (timestamp) => {
    const date = new Date(timestamp * 1000);
    const now = new Date();
    const diff = now - date;
    const days = Math.floor(diff / (1000 * 60 * 60 * 24));

    if (days === 0) return 'Today';
    if (days === 1) return 'Yesterday';
    if (days < 7) return `${days} days ago`;
    return date.toLocaleDateString();
  };

  return (
    <div class="flex flex-col h-full bg-base-100">
      {/* Quick Note Input */}
      <div class="p-4 border-b border-base-300">
        <h3 class="text-sm font-semibold mb-2 text-gray-400">Quick Note</h3>
        <textarea
          value={quickNote()}
          onInput={(e) => setQuickNote(e.target.value)}
          placeholder="Jot down a quick thought..."
          class="textarea textarea-bordered w-full h-20 text-sm resize-none"
          disabled={loading()}
        />
        <button
          onClick={createQuickNote}
          class="btn btn-primary btn-sm w-full mt-2"
          disabled={loading() || !quickNote().trim()}
        >
          Save Note
        </button>
      </div>

      {/* Recent Notes */}
      <div class="flex-1 overflow-y-auto p-4">
        <h3 class="text-sm font-semibold mb-3 text-gray-400">Recent Notes</h3>

        <Show when={!loading()} fallback={
          <div class="flex items-center justify-center py-8">
            <span class="loading loading-spinner loading-md"></span>
          </div>
        }>
          <Show when={notes().length > 0} fallback={
            <div class="text-center py-8 text-gray-500 text-sm">
              No notes yet
            </div>
          }>
            <div class="space-y-2">
              <For each={notes()}>
                {(note) => (
                  <div class="p-3 bg-base-200 rounded-lg hover:bg-base-300 transition-colors">
                    <div class="flex items-start justify-between gap-2 mb-1">
                      <h4 class="font-semibold text-sm truncate flex-1">{note.title}</h4>
                      <Show when={note.is_favorite}>
                        <span class="text-yellow-500">⭐</span>
                      </Show>
                    </div>
                    <p class="text-xs text-gray-400 line-clamp-2 mb-2">{note.content}</p>
                    <div class="flex items-center justify-between text-xs">
                      <span class="badge badge-sm badge-ghost">{note.category}</span>
                      <span class="text-gray-500">{formatDate(note.created_at)}</span>
                    </div>
                  </div>
                )}
              </For>
            </div>
          </Show>
        </Show>
      </div>
    </div>
  );
}
