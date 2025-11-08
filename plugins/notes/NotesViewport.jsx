import { createSignal, onMount, createEffect, For, Show } from 'solid-js';
import {
  IconPlus,
  IconX,
  IconStar,
  IconStarFilled,
  IconEdit,
  IconTrash,
  IconSearch,
  IconMoodHappy,
  IconMoodSad,
  IconMoodCry,
  IconMoodSmile,
  IconMoodNeutral,
  IconMoodAngry,
  IconFilter,
  IconTag
} from '@tabler/icons-solidjs';

const WEBARCADE_API = 'http://localhost:3001';

const MOODS = [
  { value: 'happy', label: 'Happy', icon: IconMoodHappy, color: 'text-green-500' },
  { value: 'excited', label: 'Excited', icon: IconMoodSmile, color: 'text-yellow-500' },
  { value: 'neutral', label: 'Neutral', icon: IconMoodNeutral, color: 'text-gray-500' },
  { value: 'sad', label: 'Sad', icon: IconMoodSad, color: 'text-blue-500' },
  { value: 'frustrated', label: 'Frustrated', icon: IconMoodAngry, color: 'text-orange-500' },
  { value: 'stressed', label: 'Stressed', icon: IconMoodCry, color: 'text-red-500' },
];

const DEFAULT_CATEGORIES = ['General', 'Journal', 'Ideas', 'Todo', 'Work', 'Personal'];

export default function NotesViewport() {
  const [notes, setNotes] = createSignal([]);
  const [categories, setCategories] = createSignal(DEFAULT_CATEGORIES);
  const [loading, setLoading] = createSignal(false);
  const [searchQuery, setSearchQuery] = createSignal('');
  const [selectedCategory, setSelectedCategory] = createSignal('All');
  const [showFavoritesOnly, setShowFavoritesOnly] = createSignal(false);

  // Note editor state
  const [isEditing, setIsEditing] = createSignal(false);
  const [editingNote, setEditingNote] = createSignal(null);
  const [noteTitle, setNoteTitle] = createSignal('');
  const [noteContent, setNoteContent] = createSignal('');
  const [noteCategory, setNoteCategory] = createSignal('General');
  const [noteMood, setNoteMood] = createSignal(null);
  const [noteTags, setNoteTags] = createSignal([]);
  const [tagInput, setTagInput] = createSignal('');

  onMount(async () => {
    await loadNotes();
    await loadCategories();
  });

  const loadNotes = async () => {
    setLoading(true);
    try {
      const response = await fetch(`${WEBARCADE_API}/notes`);
      const data = await response.json();
      setNotes(data);
    } catch (e) {
      console.error('Failed to load notes:', e);
    }
    setLoading(false);
  };

  const loadCategories = async () => {
    try {
      const response = await fetch(`${WEBARCADE_API}/notes/categories`);
      const data = await response.json();
      // Merge with default categories
      const allCategories = [...new Set([...DEFAULT_CATEGORIES, ...data])];
      setCategories(allCategories);
    } catch (e) {
      console.error('Failed to load categories:', e);
    }
  };

  const filteredNotes = () => {
    let filtered = notes();

    // Filter by search query
    if (searchQuery()) {
      const query = searchQuery().toLowerCase();
      filtered = filtered.filter(note =>
        note.title.toLowerCase().includes(query) ||
        note.content.toLowerCase().includes(query) ||
        note.tags.some(tag => tag.toLowerCase().includes(query))
      );
    }

    // Filter by category
    if (selectedCategory() !== 'All') {
      filtered = filtered.filter(note => note.category === selectedCategory());
    }

    // Filter by favorites
    if (showFavoritesOnly()) {
      filtered = filtered.filter(note => note.is_favorite);
    }

    return filtered;
  };

  const openNoteEditor = (note = null) => {
    if (note) {
      setEditingNote(note);
      setNoteTitle(note.title);
      setNoteContent(note.content);
      setNoteCategory(note.category);
      setNoteMood(note.mood);
      setNoteTags(note.tags || []);
    } else {
      setEditingNote(null);
      setNoteTitle('');
      setNoteContent('');
      setNoteCategory('General');
      setNoteMood(null);
      setNoteTags([]);
    }
    setIsEditing(true);
  };

  const closeNoteEditor = () => {
    setIsEditing(false);
    setEditingNote(null);
    setNoteTitle('');
    setNoteContent('');
    setNoteCategory('General');
    setNoteMood(null);
    setNoteTags([]);
    setTagInput('');
  };

  const saveNote = async () => {
    const title = noteTitle().trim() || 'Untitled Note';
    const content = noteContent().trim();

    if (!content) {
      alert('Note content cannot be empty');
      return;
    }

    setLoading(true);
    try {
      const payload = {
        title,
        content,
        category: noteCategory(),
        mood: noteMood(),
        tags: noteTags()
      };

      if (editingNote()) {
        // Update existing note
        await fetch(`${WEBARCADE_API}/notes`, {
          method: 'PUT',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ id: editingNote().id, ...payload })
        });
      } else {
        // Create new note
        await fetch(`${WEBARCADE_API}/notes`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify(payload)
        });
      }

      await loadNotes();
      await loadCategories();
      closeNoteEditor();
    } catch (e) {
      console.error('Failed to save note:', e);
      alert('Failed to save note');
    }
    setLoading(false);
  };

  const deleteNote = async (noteId) => {
    if (!confirm('Are you sure you want to delete this note?')) return;

    setLoading(true);
    try {
      await fetch(`${WEBARCADE_API}/notes`, {
        method: 'DELETE',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ id: noteId })
      });
      await loadNotes();
    } catch (e) {
      console.error('Failed to delete note:', e);
      alert('Failed to delete note');
    }
    setLoading(false);
  };

  const toggleFavorite = async (noteId) => {
    try {
      await fetch(`${WEBARCADE_API}/notes/toggle-favorite`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ id: noteId })
      });
      await loadNotes();
    } catch (e) {
      console.error('Failed to toggle favorite:', e);
    }
  };

  const addTag = () => {
    const tag = tagInput().trim();
    if (tag && !noteTags().includes(tag)) {
      setNoteTags([...noteTags(), tag]);
      setTagInput('');
    }
  };

  const removeTag = (tag) => {
    setNoteTags(noteTags().filter(t => t !== tag));
  };

  const formatDate = (timestamp) => {
    const date = new Date(timestamp * 1000);
    return date.toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit'
    });
  };

  const getMoodIcon = (moodValue) => {
    const mood = MOODS.find(m => m.value === moodValue);
    return mood ? { Icon: mood.icon, color: mood.color } : null;
  };

  return (
    <div class="flex flex-col h-screen bg-base-100">
      {/* Header */}
      <div class="p-6 border-b border-base-300">
        <div class="flex items-center justify-between mb-4">
          <h1 class="text-3xl font-bold">Notes & Journal</h1>
          <button
            onClick={() => openNoteEditor()}
            class="btn btn-primary gap-2"
            disabled={loading()}
          >
            <IconPlus size={20} />
            New Note
          </button>
        </div>

        {/* Filters */}
        <div class="flex flex-wrap gap-3">
          {/* Search */}
          <div class="relative flex-1 min-w-64">
            <IconSearch class="absolute left-3 top-1/2 -translate-y-1/2 text-gray-400" size={20} />
            <input
              type="text"
              value={searchQuery()}
              onInput={(e) => setSearchQuery(e.target.value)}
              placeholder="Search notes, tags..."
              class="input input-bordered w-full pl-10"
            />
          </div>

          {/* Category Filter */}
          <select
            value={selectedCategory()}
            onChange={(e) => setSelectedCategory(e.target.value)}
            class="select select-bordered"
          >
            <option value="All">All Categories</option>
            <For each={categories()}>
              {(cat) => <option value={cat}>{cat}</option>}
            </For>
          </select>

          {/* Favorites Toggle */}
          <button
            onClick={() => setShowFavoritesOnly(!showFavoritesOnly())}
            class={`btn ${showFavoritesOnly() ? 'btn-warning' : 'btn-ghost'}`}
          >
            <IconStar size={20} />
            Favorites
          </button>
        </div>
      </div>

      {/* Notes Grid */}
      <div class="flex-1 overflow-y-auto p-6">
        <Show when={!loading()} fallback={
          <div class="flex items-center justify-center h-full">
            <span class="loading loading-spinner loading-lg"></span>
          </div>
        }>
          <Show when={filteredNotes().length > 0} fallback={
            <div class="flex flex-col items-center justify-center h-full text-gray-500">
              <IconFilter size={64} class="mb-4 opacity-50" />
              <p class="text-xl">No notes found</p>
              <p class="text-sm">Try adjusting your filters or create a new note</p>
            </div>
          }>
            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
              <For each={filteredNotes()}>
                {(note) => {
                  const moodData = note.mood ? getMoodIcon(note.mood) : null;
                  return (
                    <div class="card bg-base-200 shadow-lg hover:shadow-xl transition-all border border-base-300">
                      <div class="card-body p-4">
                        {/* Header */}
                        <div class="flex items-start justify-between gap-2 mb-2">
                          <h3 class="card-title text-lg flex-1 break-words">{note.title}</h3>
                          <button
                            onClick={() => toggleFavorite(note.id)}
                            class="btn btn-ghost btn-xs text-yellow-500 hover:text-yellow-600"
                          >
                            {note.is_favorite ? <IconStarFilled size={20} /> : <IconStar size={20} />}
                          </button>
                        </div>

                        {/* Content Preview */}
                        <p class="text-sm text-gray-400 line-clamp-4 mb-3">{note.content}</p>

                        {/* Metadata */}
                        <div class="flex flex-wrap gap-2 mb-3">
                          <span class="badge badge-primary badge-sm">{note.category}</span>
                          <Show when={moodData}>
                            <span class={`badge badge-ghost badge-sm ${moodData.color}`}>
                              <moodData.Icon size={14} class="mr-1" />
                              {note.mood}
                            </span>
                          </Show>
                          <For each={note.tags || []}>
                            {(tag) => (
                              <span class="badge badge-outline badge-sm">
                                <IconTag size={12} class="mr-1" />
                                {tag}
                              </span>
                            )}
                          </For>
                        </div>

                        {/* Footer */}
                        <div class="flex items-center justify-between text-xs text-gray-500">
                          <span>{formatDate(note.created_at)}</span>
                          <div class="flex gap-1">
                            <button
                              onClick={() => openNoteEditor(note)}
                              class="btn btn-ghost btn-xs"
                              title="Edit"
                            >
                              <IconEdit size={16} />
                            </button>
                            <button
                              onClick={() => deleteNote(note.id)}
                              class="btn btn-ghost btn-xs text-error"
                              title="Delete"
                            >
                              <IconTrash size={16} />
                            </button>
                          </div>
                        </div>
                      </div>
                    </div>
                  );
                }}
              </For>
            </div>
          </Show>
        </Show>
      </div>

      {/* Note Editor Modal */}
      <Show when={isEditing()}>
        <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4">
          <div class="bg-base-200 rounded-lg shadow-2xl w-full max-w-3xl max-h-[90vh] overflow-hidden flex flex-col">
            {/* Modal Header */}
            <div class="p-6 border-b border-base-300 flex items-center justify-between">
              <h2 class="text-2xl font-bold">
                {editingNote() ? 'Edit Note' : 'New Note'}
              </h2>
              <button onClick={closeNoteEditor} class="btn btn-ghost btn-sm btn-circle">
                <IconX size={20} />
              </button>
            </div>

            {/* Modal Body */}
            <div class="flex-1 overflow-y-auto p-6 space-y-4">
              {/* Title */}
              <div class="form-control">
                <label class="label">
                  <span class="label-text font-semibold">Title</span>
                </label>
                <input
                  type="text"
                  value={noteTitle()}
                  onInput={(e) => setNoteTitle(e.target.value)}
                  placeholder="Note title..."
                  class="input input-bordered w-full"
                  autofocus
                />
              </div>

              {/* Content */}
              <div class="form-control">
                <label class="label">
                  <span class="label-text font-semibold">Content</span>
                </label>
                <textarea
                  value={noteContent()}
                  onInput={(e) => setNoteContent(e.target.value)}
                  placeholder="Write your note here..."
                  class="textarea textarea-bordered w-full h-64 resize-none"
                />
              </div>

              {/* Category & Mood Row */}
              <div class="grid grid-cols-2 gap-4">
                {/* Category */}
                <div class="form-control">
                  <label class="label">
                    <span class="label-text font-semibold">Category</span>
                  </label>
                  <select
                    value={noteCategory()}
                    onChange={(e) => setNoteCategory(e.target.value)}
                    class="select select-bordered w-full"
                  >
                    <For each={categories()}>
                      {(cat) => <option value={cat}>{cat}</option>}
                    </For>
                  </select>
                </div>

                {/* Mood */}
                <div class="form-control">
                  <label class="label">
                    <span class="label-text font-semibold">Mood (Optional)</span>
                  </label>
                  <select
                    value={noteMood() || ''}
                    onChange={(e) => setNoteMood(e.target.value || null)}
                    class="select select-bordered w-full"
                  >
                    <option value="">None</option>
                    <For each={MOODS}>
                      {(mood) => <option value={mood.value}>{mood.label}</option>}
                    </For>
                  </select>
                </div>
              </div>

              {/* Tags */}
              <div class="form-control">
                <label class="label">
                  <span class="label-text font-semibold">Tags</span>
                </label>
                <div class="flex gap-2 mb-2">
                  <input
                    type="text"
                    value={tagInput()}
                    onInput={(e) => setTagInput(e.target.value)}
                    onKeyPress={(e) => e.key === 'Enter' && (e.preventDefault(), addTag())}
                    placeholder="Add a tag..."
                    class="input input-bordered flex-1"
                  />
                  <button onClick={addTag} class="btn btn-primary">
                    <IconPlus size={16} />
                  </button>
                </div>
                <div class="flex flex-wrap gap-2">
                  <For each={noteTags()}>
                    {(tag) => (
                      <div class="badge badge-lg gap-2">
                        {tag}
                        <button onClick={() => removeTag(tag)} class="btn btn-ghost btn-xs btn-circle">
                          <IconX size={14} />
                        </button>
                      </div>
                    )}
                  </For>
                </div>
              </div>
            </div>

            {/* Modal Footer */}
            <div class="p-6 border-t border-base-300 flex justify-end gap-3">
              <button onClick={closeNoteEditor} class="btn btn-ghost">
                Cancel
              </button>
              <button
                onClick={saveNote}
                class="btn btn-primary"
                disabled={loading() || !noteContent().trim()}
              >
                {loading() ? <span class="loading loading-spinner"></span> : 'Save Note'}
              </button>
            </div>
          </div>
        </div>
      </Show>
    </div>
  );
}
