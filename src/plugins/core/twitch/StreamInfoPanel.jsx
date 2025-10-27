import { createSignal, onMount, Show, createEffect } from 'solid-js';
import twitchStore from './TwitchStore.jsx';
import {
  IconInfoCircle,
  IconUsers,
  IconEye,
  IconClock,
  IconDeviceTv,
  IconAlertCircle,
  IconCheck,
  IconSearch,
} from '@tabler/icons-solidjs';

export default function StreamInfoPanel() {
  const [streamInfo, setStreamInfo] = createSignal(null);
  const [loading, setLoading] = createSignal(true);
  const [saving, setSaving] = createSignal(false);
  const [message, setMessage] = createSignal('');
  const [isLive, setIsLive] = createSignal(false);

  // Form fields
  const [title, setTitle] = createSignal('');
  const [gameSearch, setGameSearch] = createSignal('');
  const [selectedGame, setSelectedGame] = createSignal(null);
  const [gameResults, setGameResults] = createSignal([]);
  const [searchingGames, setSearchingGames] = createSignal(false);
  const [tags, setTags] = createSignal([]);
  const [tagInput, setTagInput] = createSignal('');
  const [isBrandedContent, setIsBrandedContent] = createSignal(false);
  const [contentLabels, setContentLabels] = createSignal([]);

  let searchTimeout = null;

  onMount(async () => {
    await loadStreamInfo();
    setLoading(false);
  });

  const loadStreamInfo = async () => {
    try {
      const data = await twitchStore.getStreamInfo();
      console.log('[StreamInfoPanel] Received data:', data);
      setStreamInfo(data);

      // Set form values from current stream/channel info (or stream info if live)
      const sourceData = data.stream || data.channel;

      if (sourceData) {
        console.log('[StreamInfoPanel] Source data:', sourceData);
        setTitle(sourceData.title || '');

        if (sourceData.game_id && sourceData.game_name) {
          setSelectedGame({
            id: sourceData.game_id,
            name: sourceData.game_name,
          });
          setGameSearch(sourceData.game_name);
        }

        // Set tags from stream data (tags are only on stream object when live)
        if (data.stream?.tags) {
          setTags(data.stream.tags);
        }
      }

      // Check if stream is live
      const live = !!data.stream;
      console.log('[StreamInfoPanel] Is live:', live, 'Stream data:', data.stream);
      setIsLive(live);
    } catch (e) {
      console.error('Failed to load stream info:', e);
      setMessage(`error:${e.message}`);
      setTimeout(() => setMessage(''), 5000);
    }
  };

  // Search for games with debouncing
  createEffect(() => {
    const query = gameSearch();

    if (!query || query.length < 2) {
      setGameResults([]);
      return;
    }

    // Clear previous timeout
    if (searchTimeout) {
      clearTimeout(searchTimeout);
    }

    // Set new timeout
    searchTimeout = setTimeout(async () => {
      setSearchingGames(true);
      try {
        const results = await twitchStore.searchGames(query);
        setGameResults(results);
      } catch (e) {
        console.error('Failed to search games:', e);
      } finally {
        setSearchingGames(false);
      }
    }, 300);
  });

  const selectGame = (game) => {
    setSelectedGame(game);
    setGameSearch(game.name);
    setGameResults([]);
  };

  const clearGameSelection = () => {
    setSelectedGame(null);
    setGameSearch('');
    setGameResults([]);
  };

  const addTag = () => {
    const tag = tagInput().trim();
    if (!tag) return;

    if (tags().length >= 10) {
      setMessage('error:Maximum 10 tags allowed');
      setTimeout(() => setMessage(''), 3000);
      return;
    }

    if (tags().includes(tag)) {
      setMessage('error:Tag already added');
      setTimeout(() => setMessage(''), 3000);
      return;
    }

    setTags([...tags(), tag]);
    setTagInput('');
  };

  const removeTag = (tagToRemove) => {
    setTags(tags().filter((t) => t !== tagToRemove));
  };

  const toggleContentLabel = (label) => {
    if (contentLabels().includes(label)) {
      setContentLabels(contentLabels().filter((l) => l !== label));
    } else {
      setContentLabels([...contentLabels(), label]);
    }
  };

  const handleUpdate = async () => {
    if (!title().trim() && !selectedGame()) {
      setMessage('error:Please enter a title or select a game');
      setTimeout(() => setMessage(''), 3000);
      return;
    }

    setSaving(true);
    setMessage('');

    try {
      await twitchStore.updateStreamInfo({
        title: title().trim() || null,
        gameId: selectedGame()?.id || null,
        tags: tags().length > 0 ? tags() : null,
        contentClassificationLabels: contentLabels().length > 0 ? contentLabels() : null,
        isBrandedContent: isBrandedContent(),
      });

      setMessage('success');
      setTimeout(() => setMessage(''), 3000);

      // Reload stream info after a short delay
      setTimeout(() => loadStreamInfo(), 500);
    } catch (e) {
      console.error('Failed to update stream info:', e);
      setMessage(`error:${e.message}`);
      setTimeout(() => setMessage(''), 5000);
    } finally {
      setSaving(false);
    }
  };

  const formatUptime = (startedAt) => {
    if (!startedAt) return '0h 0m';

    const start = new Date(startedAt);
    const now = new Date();
    const diff = Math.floor((now - start) / 1000); // seconds

    const hours = Math.floor(diff / 3600);
    const minutes = Math.floor((diff % 3600) / 60);

    return `${hours}h ${minutes}m`;
  };

  return (
    <div class="h-full flex flex-col bg-base-200">
      {/* Header */}
      <div class="flex items-center justify-between bg-base-100 border-b border-base-300 px-3 py-2">
        <div class="flex items-center gap-2">
          <IconInfoCircle size={16} class="text-primary" />
          <span class="text-sm font-semibold">Stream Info</span>
        </div>
        <button
          class="btn btn-ghost btn-xs"
          onClick={loadStreamInfo}
          disabled={loading()}
        >
          Refresh
        </button>
      </div>

      <div class="flex-1 overflow-y-auto p-3">
        <Show
          when={!loading()}
          fallback={
            <div class="flex items-center justify-center h-full">
              <span class="loading loading-spinner loading-sm"></span>
            </div>
          }
        >
          <Show
            when={streamInfo()}
            fallback={
              <div class="text-center py-6">
                <IconAlertCircle size={32} class="mx-auto mb-3 opacity-30" />
                <p class="text-xs font-semibold mb-2">Not Authenticated</p>
                <p class="text-xs text-base-content/60 mb-3">
                  Please log in with Twitch first
                </p>
              </div>
            }
          >
            <div class="space-y-3">
              {/* Authenticated Account Warning */}
              <Show when={streamInfo()?.user}>
                <div class="alert alert-warning py-2 px-3">
                  <IconAlertCircle size={16} />
                  <div class="flex-1">
                    <div class="text-xs font-semibold">
                      Authenticated as: {streamInfo()?.user?.display_name} (@{streamInfo()?.user?.login})
                    </div>
                    <div class="text-[10px] mt-1">
                      If this is not your broadcaster account, go to Twitch Settings → Revoke Token → Login with your main account
                    </div>
                  </div>
                </div>
              </Show>

              {/* Stream Status */}
              <div class="card bg-base-100 shadow-sm">
                <div class="card-body p-3">
                  <h4 class="text-xs font-semibold mb-2 flex items-center gap-2">
                    <IconEye size={14} />
                    Stream Status
                  </h4>
                  <div class="space-y-2">
                    <div class="flex items-center justify-between">
                      <span class="text-xs text-base-content/70">Status</span>
                      <span class={`badge badge-sm ${isLive() ? 'badge-error' : 'badge-ghost'}`}>
                        {isLive() ? 'LIVE' : 'Offline'}
                      </span>
                    </div>
                    <Show when={isLive()}>
                      <div class="flex items-center justify-between">
                        <span class="text-xs text-base-content/70">Viewers</span>
                        <span class="text-sm font-semibold flex items-center gap-1">
                          <IconUsers size={14} class="text-info" />
                          {streamInfo()?.stream?.viewer_count || 0}
                        </span>
                      </div>
                      <div class="flex items-center justify-between">
                        <span class="text-xs text-base-content/70">Uptime</span>
                        <span class="text-sm font-semibold flex items-center gap-1">
                          <IconClock size={14} class="text-warning" />
                          {formatUptime(streamInfo()?.stream?.started_at)}
                        </span>
                      </div>
                    </Show>
                  </div>
                </div>
              </div>

              {/* Edit Stream Info */}
              <div class="card bg-base-100 shadow-sm">
                <div class="card-body p-3">
                  <h4 class="text-xs font-semibold mb-3 flex items-center gap-2">
                    <IconDeviceTv size={14} />
                    Edit Stream Info
                  </h4>

                  {/* Success/Error Message */}
                  <Show when={message()}>
                    <div
                      class={`alert ${
                        message() === 'success' ? 'alert-success' : 'alert-error'
                      } py-2 px-3 text-xs mb-3`}
                    >
                      <Show when={message() === 'success'}>
                        <IconCheck size={14} />
                        <span>Stream info updated successfully!</span>
                      </Show>
                      <Show when={message().startsWith('error:')}>
                        <IconAlertCircle size={14} />
                        <span>{message().replace('error:', '')}</span>
                      </Show>
                    </div>
                  </Show>

                  <div class="space-y-3">
                    {/* Title Input */}
                    <div class="form-control">
                      <label class="label py-1">
                        <span class="label-text text-xs">Stream Title</span>
                      </label>
                      <input
                        type="text"
                        class="input input-sm input-bordered w-full text-xs"
                        placeholder="Enter stream title..."
                        value={title()}
                        onInput={(e) => setTitle(e.target.value)}
                        maxLength={140}
                      />
                      <label class="label py-0.5">
                        <span class="label-text-alt text-[10px] text-base-content/50">
                          {title().length}/140 characters
                        </span>
                      </label>
                    </div>

                    {/* Game/Category Search */}
                    <div class="form-control">
                      <label class="label py-1">
                        <span class="label-text text-xs">Category</span>
                        <Show when={selectedGame()}>
                          <button
                            class="label-text-alt text-[10px] link link-hover"
                            onClick={clearGameSelection}
                          >
                            Clear
                          </button>
                        </Show>
                      </label>
                      <div class="relative">
                        <input
                          type="text"
                          class="input input-sm input-bordered w-full text-xs pr-8"
                          placeholder="Search for a game/category..."
                          value={gameSearch()}
                          onInput={(e) => setGameSearch(e.target.value)}
                          disabled={!!selectedGame()}
                        />
                        <div class="absolute right-2 top-1/2 -translate-y-1/2">
                          <Show
                            when={searchingGames()}
                            fallback={<IconSearch size={14} class="opacity-30" />}
                          >
                            <span class="loading loading-spinner loading-xs"></span>
                          </Show>
                        </div>
                      </div>

                      {/* Game Search Results */}
                      <Show when={gameResults().length > 0 && !selectedGame()}>
                        <div class="mt-1 max-h-48 overflow-y-auto border border-base-300 rounded-lg bg-base-200">
                          {gameResults().map((game) => (
                            <button
                              class="w-full text-left px-3 py-2 hover:bg-base-300 text-xs flex items-center gap-2 border-b border-base-300 last:border-b-0"
                              onClick={() => selectGame(game)}
                            >
                              <Show when={game.box_art_url}>
                                <img
                                  src={game.box_art_url.replace('{width}', '52').replace('{height}', '72')}
                                  alt={game.name}
                                  class="w-6 h-8 object-cover rounded"
                                />
                              </Show>
                              <span class="flex-1">{game.name}</span>
                            </button>
                          ))}
                        </div>
                      </Show>

                      {/* Selected Game Display */}
                      <Show when={selectedGame()}>
                        <div class="mt-1 flex items-center gap-2 p-2 bg-base-200 rounded-lg border border-base-300">
                          <Show when={selectedGame().box_art_url}>
                            <img
                              src={selectedGame().box_art_url.replace('{width}', '52').replace('{height}', '72')}
                              alt={selectedGame().name}
                              class="w-6 h-8 object-cover rounded"
                            />
                          </Show>
                          <span class="text-xs flex-1">{selectedGame().name}</span>
                        </div>
                      </Show>
                    </div>

                    {/* Tags Editor */}
                    <div class="form-control">
                      <label class="label py-1">
                        <span class="label-text text-xs">Tags (max 10)</span>
                        <span class="label-text-alt text-[10px] text-base-content/50">
                          {tags().length}/10
                        </span>
                      </label>
                      <div class="flex gap-1">
                        <input
                          type="text"
                          class="input input-sm input-bordered flex-1 text-xs"
                          placeholder="Add a tag..."
                          value={tagInput()}
                          onInput={(e) => setTagInput(e.target.value)}
                          onKeyPress={(e) => {
                            if (e.key === 'Enter') {
                              e.preventDefault();
                              addTag();
                            }
                          }}
                        />
                        <button
                          class="btn btn-sm btn-square"
                          onClick={addTag}
                          disabled={!tagInput().trim() || tags().length >= 10}
                        >
                          +
                        </button>
                      </div>
                      <Show when={tags().length > 0}>
                        <div class="flex flex-wrap gap-1 mt-2">
                          {tags().map((tag) => (
                            <div class="badge badge-sm gap-1">
                              <span>{tag}</span>
                              <button
                                class="text-xs hover:text-error"
                                onClick={() => removeTag(tag)}
                              >
                                ×
                              </button>
                            </div>
                          ))}
                        </div>
                      </Show>
                    </div>

                    {/* Branded Content Toggle */}
                    <div class="form-control">
                      <label class="label cursor-pointer py-1">
                        <span class="label-text text-xs">Branded Content (Sponsored Stream)</span>
                        <input
                          type="checkbox"
                          class="toggle toggle-sm"
                          checked={isBrandedContent()}
                          onChange={(e) => setIsBrandedContent(e.target.checked)}
                        />
                      </label>
                      <label class="label py-0">
                        <span class="label-text-alt text-[10px] text-base-content/50">
                          Check if this stream has paid promotion or sponsorship
                        </span>
                      </label>
                    </div>

                    {/* Content Classification Labels */}
                    <div class="form-control">
                      <label class="label py-1">
                        <span class="label-text text-xs">Content Warnings</span>
                      </label>
                      <div class="space-y-2">
                        <div class="flex items-center justify-between">
                          <label class="label-text text-xs cursor-pointer flex-1">
                            Drugs, Intoxication, or Excessive Tobacco Use
                          </label>
                          <input
                            type="checkbox"
                            class="checkbox checkbox-xs"
                            checked={contentLabels().includes('DrugsIntoxication')}
                            onChange={() => toggleContentLabel('DrugsIntoxication')}
                          />
                        </div>
                        <div class="flex items-center justify-between">
                          <label class="label-text text-xs cursor-pointer flex-1">
                            Sexual Themes
                          </label>
                          <input
                            type="checkbox"
                            class="checkbox checkbox-xs"
                            checked={contentLabels().includes('SexualThemes')}
                            onChange={() => toggleContentLabel('SexualThemes')}
                          />
                        </div>
                        <div class="flex items-center justify-between">
                          <label class="label-text text-xs cursor-pointer flex-1">
                            Violent and Graphic Depictions
                          </label>
                          <input
                            type="checkbox"
                            class="checkbox checkbox-xs"
                            checked={contentLabels().includes('ViolentGraphic')}
                            onChange={() => toggleContentLabel('ViolentGraphic')}
                          />
                        </div>
                        <div class="flex items-center justify-between">
                          <label class="label-text text-xs cursor-pointer flex-1">
                            Gambling
                          </label>
                          <input
                            type="checkbox"
                            class="checkbox checkbox-xs"
                            checked={contentLabels().includes('Gambling')}
                            onChange={() => toggleContentLabel('Gambling')}
                          />
                        </div>
                        <div class="flex items-center justify-between">
                          <label class="label-text text-xs cursor-pointer flex-1">
                            Mature-Rated Game
                          </label>
                          <input
                            type="checkbox"
                            class="checkbox checkbox-xs"
                            checked={contentLabels().includes('MatureGame')}
                            onChange={() => toggleContentLabel('MatureGame')}
                          />
                        </div>
                      </div>
                    </div>

                    {/* Update Button */}
                    <button
                      class="btn btn-primary btn-sm btn-block"
                      onClick={handleUpdate}
                      disabled={saving()}
                    >
                      <Show when={saving()} fallback="Update Stream Info">
                        <span class="loading loading-spinner loading-xs"></span>
                        Updating...
                      </Show>
                    </button>
                  </div>
                </div>
              </div>

              {/* Current Channel Info */}
              <Show when={streamInfo()?.channel}>
                <div class="card bg-base-100 shadow-sm">
                  <div class="card-body p-3">
                    <h4 class="text-xs font-semibold mb-2">Current Channel Info (Live from Twitch)</h4>
                    <div class="space-y-1.5 text-xs">
                      <div>
                        <span class="text-base-content/60">Title: </span>
                        <span class="font-medium break-words">{streamInfo()?.channel?.title || 'Not set'}</span>
                      </div>
                      <div>
                        <span class="text-base-content/60">Category: </span>
                        <span class="font-medium">{streamInfo()?.channel?.game_name || 'Not set'}</span>
                      </div>
                      <div>
                        <span class="text-base-content/60">Category ID: </span>
                        <span class="font-medium font-mono text-[10px]">{streamInfo()?.channel?.game_id || 'None'}</span>
                      </div>
                      <div>
                        <span class="text-base-content/60">Language: </span>
                        <span class="font-medium">{streamInfo()?.channel?.broadcaster_language || 'en'}</span>
                      </div>
                      <div>
                        <span class="text-base-content/60">Delay: </span>
                        <span class="font-medium">{streamInfo()?.channel?.delay || 0}s</span>
                      </div>
                    </div>
                  </div>
                </div>
              </Show>

              {/* Stream Info (when live) */}
              <Show when={streamInfo()?.stream}>
                <div class="card bg-base-100 shadow-sm">
                  <div class="card-body p-3">
                    <h4 class="text-xs font-semibold mb-2">Live Stream Info</h4>
                    <div class="space-y-1.5 text-xs">
                      <div>
                        <span class="text-base-content/60">Stream Title: </span>
                        <span class="font-medium break-words">{streamInfo()?.stream?.title || 'Not set'}</span>
                      </div>
                      <div>
                        <span class="text-base-content/60">Game: </span>
                        <span class="font-medium">{streamInfo()?.stream?.game_name || 'Not set'}</span>
                      </div>
                      <div>
                        <span class="text-base-content/60">Language: </span>
                        <span class="font-medium">{streamInfo()?.stream?.language || 'en'}</span>
                      </div>
                      <div>
                        <span class="text-base-content/60">Mature: </span>
                        <span class="font-medium">{streamInfo()?.stream?.is_mature ? 'Yes' : 'No'}</span>
                      </div>
                      <Show when={streamInfo()?.stream?.tags?.length > 0}>
                        <div>
                          <span class="text-base-content/60">Tags: </span>
                          <span class="font-medium">{streamInfo()?.stream?.tags?.join(', ')}</span>
                        </div>
                      </Show>
                    </div>
                  </div>
                </div>
              </Show>

              {/* Debug: Raw Data */}
              <Show when={streamInfo()}>
                <details class="collapse collapse-arrow bg-base-100 shadow-sm">
                  <summary class="collapse-title text-xs font-semibold min-h-0 py-2 px-3">
                    Debug: Raw API Response
                  </summary>
                  <div class="collapse-content">
                    <pre class="text-[9px] overflow-x-auto bg-base-200 p-2 rounded">
                      {JSON.stringify(streamInfo(), null, 2)}
                    </pre>
                  </div>
                </details>
              </Show>
            </div>
          </Show>
        </Show>
      </div>
    </div>
  );
}
