import { createSignal } from 'solid-js';
import { IconMoodSmile, IconRefresh, IconCopy, IconCheck } from '@tabler/icons-solidjs';

export default function JokeGeneratorWidget() {
  const [joke, setJoke] = createSignal(null);
  const [loading, setLoading] = createSignal(false);
  const [copied, setCopied] = createSignal(false);
  const [category, setCategory] = createSignal('any');

  const categories = [
    { value: 'any', label: 'Any' },
    { value: 'Programming', label: 'Programming' },
    { value: 'Misc', label: 'Misc' },
    { value: 'Dark', label: 'Dark' },
    { value: 'Pun', label: 'Pun' },
    { value: 'Spooky', label: 'Spooky' },
    { value: 'Christmas', label: 'Christmas' }
  ];

  const fetchJoke = async () => {
    setLoading(true);
    try {
      const response = await fetch(`https://v2.jokeapi.dev/joke/${category()}?safe-mode`);
      const data = await response.json();

      if (data.type === 'single') {
        setJoke({ type: 'single', text: data.joke });
      } else {
        setJoke({ type: 'twopart', setup: data.setup, delivery: data.delivery });
      }
    } catch (error) {
      setJoke({ type: 'single', text: 'Failed to fetch joke. Try again!' });
    } finally {
      setLoading(false);
    }
  };

  const copyJoke = async () => {
    try {
      const jokeData = joke();
      const text = jokeData.type === 'single'
        ? jokeData.text
        : `${jokeData.setup}\n\n${jokeData.delivery}`;

      await navigator.clipboard.writeText(text);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch (err) {
      console.error('Failed to copy:', err);
    }
  };

  return (
    <div class="card bg-gradient-to-br from-warning/20 to-warning/5 bg-base-100 shadow-lg h-full flex flex-col p-3">
      {/* Header */}
      <div class="flex items-center gap-1.5 mb-2">
        <IconMoodSmile size={16} class="text-warning opacity-80" />
        <span class="text-xs font-medium opacity-70">Joke Generator</span>
      </div>

      {/* Category Selection */}
      <div class="mb-2">
        <label class="text-xs opacity-50 mb-1 block">Category</label>
        <select
          class="select select-sm select-bordered w-full bg-base-200/50 text-xs"
          value={category()}
          onChange={(e) => setCategory(e.target.value)}
        >
          {categories.map((cat) => (
            <option value={cat.value}>{cat.label}</option>
          ))}
        </select>
      </div>

      {/* Joke Display */}
      <div class="flex-1 bg-base-200/50 rounded-lg p-3 mb-2 overflow-y-auto">
        {!joke() ? (
          <div class="flex items-center justify-center h-full text-xs opacity-50">
            Click "Get Joke" to start
          </div>
        ) : loading() ? (
          <div class="flex items-center justify-center h-full">
            <span class="loading loading-spinner loading-sm text-warning"></span>
          </div>
        ) : joke().type === 'single' ? (
          <p class="text-sm">{joke().text}</p>
        ) : (
          <div class="space-y-2">
            <p class="text-sm font-medium">{joke().setup}</p>
            <p class="text-sm text-warning">{joke().delivery}</p>
          </div>
        )}
      </div>

      {/* Actions */}
      <div class="flex gap-1">
        <button
          class="btn btn-sm btn-warning flex-1 gap-1"
          onClick={fetchJoke}
          disabled={loading()}
        >
          <IconRefresh size={16} class={loading() ? 'animate-spin' : ''} />
          Get Joke
        </button>
        {joke() && (
          <button
            class="btn btn-sm btn-ghost"
            onClick={copyJoke}
            title="Copy joke"
          >
            {copied() ? <IconCheck size={16} class="text-success" /> : <IconCopy size={16} />}
          </button>
        )}
      </div>
    </div>
  );
}
