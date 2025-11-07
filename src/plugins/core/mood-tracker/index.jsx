import { createPlugin } from '@/api/plugin';
import { createSignal, onMount, onCleanup } from 'solid-js';
import { IconMoodSmile } from '@tabler/icons-solidjs';

const WEBARCADE_API = 'http://localhost:3001';

const MoodTrackerViewport = () => {
  const [mood, setMood] = createSignal(5); // 1-10 scale
  const [weight, setWeight] = createSignal('');
  const [sleep, setSleep] = createSignal('');
  const [water, setWater] = createSignal(0);
  const [showBackground, setShowBackground] = createSignal(true);
  const [loading, setLoading] = createSignal(false);
  const [lastUpdated, setLastUpdated] = createSignal(null);

  // Mood emoji helper
  const getMoodEmoji = (moodValue) => {
    if (moodValue >= 9) return '😄';
    if (moodValue >= 7) return '🙂';
    if (moodValue >= 5) return '😐';
    if (moodValue >= 3) return '😔';
    return '😫';
  };

  const getMoodLabel = (moodValue) => {
    if (moodValue >= 9) return 'Great';
    if (moodValue >= 7) return 'Good';
    if (moodValue >= 5) return 'Okay';
    if (moodValue >= 3) return 'Bad';
    return 'Awful';
  };

  // Load current mood data
  const loadMoodData = async () => {
    try {
      const response = await fetch(`${WEBARCADE_API}/mood-tracker/data`);
      const data = await response.json();
      setMood(data.mood || 5);
      setWeight(data.weight ? data.weight.toString() : '');
      setSleep(data.sleep ? data.sleep.toString() : '');
      setWater(data.water || 0);
      setShowBackground(data.show_background !== undefined ? data.show_background : true);
      setLastUpdated(data.updated_at ? new Date(data.updated_at * 1000) : null);
    } catch (error) {
      console.error('Failed to load mood data:', error);
    }
  };

  // Update mood data
  const updateMoodData = async () => {
    setLoading(true);
    try {
      const data = {
        mood: mood(),
        weight: weight() ? parseFloat(weight()) : null,
        sleep: sleep() ? parseFloat(sleep()) : null,
        water: water(),
        show_background: showBackground(),
        updated_at: Math.floor(Date.now() / 1000)
      };

      await fetch(`${WEBARCADE_API}/mood-tracker/data`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(data)
      });

      setLastUpdated(new Date());
      alert('Mood data updated successfully!');
    } catch (error) {
      console.error('Failed to update mood data:', error);
      alert('Failed to update mood data. Please try again.');
    }
    setLoading(false);
  };

  // Reset daily counters (water)
  const resetWater = async () => {
    if (!confirm('Reset water intake to 0?')) return;
    setWater(0);
  };

  // Increment water by 1
  const incrementWater = () => {
    setWater(prev => prev + 1);
  };

  // Decrement water by 1
  const decrementWater = () => {
    setWater(prev => Math.max(0, prev - 1));
  };

  // Fetch Withings weight (placeholder for future integration)
  const fetchWithingsWeight = async () => {
    setLoading(true);
    try {
      const response = await fetch(`${WEBARCADE_API}/mood-tracker/withings/weight`);
      const data = await response.json();
      if (data.weight) {
        setWeight(data.weight.toString());
        alert(`Weight fetched from Withings: ${data.weight} kg`);
      } else {
        alert('No weight data available from Withings.');
      }
    } catch (error) {
      console.error('Failed to fetch Withings weight:', error);
      alert('Withings integration not yet configured.');
    }
    setLoading(false);
  };

  onMount(() => {
    loadMoodData();
  });

  return (
    <div class="p-6 pb-24 space-y-6 overflow-y-auto max-h-screen">
      {/* Header */}
      <div class="bg-base-200 rounded-lg p-4 shadow">
        <h2 class="text-2xl font-bold mb-2">✨ Mood Tracker</h2>
        <p class="text-sm text-gray-400">
          Track your daily mood, weight, sleep, and water intake. Updates will appear live on the mood ticker overlay.
        </p>
        {lastUpdated() && (
          <p class="text-xs text-gray-500 mt-2">
            Last updated: {lastUpdated().toLocaleString()}
          </p>
        )}
      </div>

      {/* Mood Selection */}
      <div class="bg-base-200 rounded-lg p-4 shadow">
        <h3 class="text-xl font-bold mb-4">😊 Current Mood</h3>
        <p class="text-sm text-gray-400 mb-4">
          How are you feeling today? Rate from 1 (awful) to 10 (great)
        </p>

        <div class="space-y-4">
          {/* Current mood display */}
          <div class="flex items-center justify-center gap-4 p-4 bg-base-300 rounded-lg">
            <span class="text-6xl">{getMoodEmoji(mood())}</span>
            <div class="flex flex-col items-center">
              <span class="text-4xl font-bold text-primary">{mood()}</span>
              <span class="text-sm text-gray-400">{getMoodLabel(mood())}</span>
            </div>
          </div>

          {/* Mood slider */}
          <div class="space-y-2">
            <input
              type="range"
              min="1"
              max="10"
              value={mood()}
              onInput={(e) => setMood(parseInt(e.target.value))}
              class="range range-primary"
              disabled={loading()}
            />
            <div class="flex justify-between text-xs text-gray-400 px-2">
              <span>😫 1</span>
              <span>😔 3</span>
              <span>😐 5</span>
              <span>🙂 7</span>
              <span>😄 10</span>
            </div>
          </div>

          {/* Quick select buttons */}
          <div class="flex gap-2">
            {[1, 3, 5, 7, 10].map(value => (
              <button
                onClick={() => setMood(value)}
                class={`btn btn-sm flex-1 ${mood() === value ? 'btn-primary' : 'btn-ghost'}`}
                disabled={loading()}
              >
                {value}
              </button>
            ))}
          </div>
        </div>
      </div>

      {/* Weight */}
      <div class="bg-base-200 rounded-lg p-4 shadow">
        <h3 class="text-xl font-bold mb-4">⚖️ Weight</h3>
        <p class="text-sm text-gray-400 mb-4">
          Enter your current weight or fetch from Withings.
        </p>

        <div class="flex gap-3 items-end">
          <div class="form-control flex-1">
            <label class="label">
              <span class="label-text">Weight (kg)</span>
            </label>
            <input
              type="number"
              step="0.1"
              value={weight()}
              onInput={(e) => setWeight(e.target.value)}
              placeholder="e.g., 75.5"
              class="input input-bordered w-full"
              disabled={loading()}
            />
          </div>
          <button
            onClick={fetchWithingsWeight}
            class="btn btn-secondary"
            disabled={loading()}
          >
            📱 Fetch from Withings
          </button>
        </div>

        <div class="mt-3 p-3 bg-info/20 rounded text-sm">
          <p><strong>💡 Note:</strong> Withings integration requires API setup. For now, enter weight manually.</p>
        </div>
      </div>

      {/* Sleep */}
      <div class="bg-base-200 rounded-lg p-4 shadow">
        <h3 class="text-xl font-bold mb-4">😴 Sleep Duration</h3>
        <p class="text-sm text-gray-400 mb-4">
          How many hours did you sleep last night?
        </p>

        <div class="form-control">
          <label class="label">
            <span class="label-text">Hours of sleep</span>
          </label>
          <input
            type="number"
            step="0.5"
            value={sleep()}
            onInput={(e) => setSleep(e.target.value)}
            placeholder="e.g., 7.5"
            class="input input-bordered w-full"
            disabled={loading()}
          />
        </div>

        <div class="mt-3">
          <p class="text-xs text-gray-400">Quick select:</p>
          <div class="flex gap-2 mt-2">
            {[4, 5, 6, 7, 8, 9, 10].map(hours => (
              <button
                onClick={() => setSleep(hours.toString())}
                class={`btn btn-sm ${sleep() === hours.toString() ? 'btn-primary' : 'btn-ghost'}`}
                disabled={loading()}
              >
                {hours}h
              </button>
            ))}
          </div>
        </div>
      </div>

      {/* Water Intake */}
      <div class="bg-base-200 rounded-lg p-4 shadow">
        <h3 class="text-xl font-bold mb-4">💧 Water Intake</h3>
        <p class="text-sm text-gray-400 mb-4">
          Track how many glasses of water you've had today (approx. 250ml per glass).
        </p>

        <div class="flex items-center justify-center gap-6 py-4">
          <button
            onClick={decrementWater}
            class="btn btn-circle btn-lg btn-error"
            disabled={loading() || water() === 0}
          >
            <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20 12H4" />
            </svg>
          </button>

          <div class="text-center">
            <div class="text-6xl font-bold text-primary">{water()}</div>
            <div class="text-sm text-gray-400 mt-2">
              {water() === 0 ? 'No glasses yet' :
               water() === 1 ? '1 glass' :
               `${water()} glasses`}
            </div>
            <div class="text-xs text-gray-500 mt-1">
              ≈ {(water() * 0.25).toFixed(2)}L
            </div>
          </div>

          <button
            onClick={incrementWater}
            class="btn btn-circle btn-lg btn-success"
            disabled={loading()}
          >
            <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
            </svg>
          </button>
        </div>

        <div class="flex gap-2 mt-4">
          <button
            onClick={resetWater}
            class="btn btn-ghost btn-sm flex-1"
            disabled={loading()}
          >
            🔄 Reset Water
          </button>
        </div>

        <div class="mt-3 p-3 bg-warning/20 rounded text-sm">
          <p><strong>💡 Daily Goal:</strong> Aim for 8 glasses (2L) per day for optimal hydration!</p>
        </div>
      </div>

      {/* Display Options */}
      <div class="bg-base-200 rounded-lg p-4 shadow">
        <h3 class="text-xl font-bold mb-4">🎨 Display Options</h3>
        <p class="text-sm text-gray-400 mb-4">
          Control how the mood ticker appears on stream
        </p>

        <div class="form-control">
          <label class="label cursor-pointer">
            <span class="label-text">Show Background Gradient</span>
            <input
              type="checkbox"
              class="toggle toggle-primary"
              checked={showBackground()}
              onChange={(e) => setShowBackground(e.target.checked)}
              disabled={loading()}
            />
          </label>
          <p class="text-xs text-gray-500 mt-2">
            {showBackground()
              ? '✅ Background gradient is visible'
              : '❌ Transparent background - only stats visible'}
          </p>
        </div>
      </div>

      {/* Save Button */}
      <div class="bg-base-200 rounded-lg p-4 shadow sticky bottom-0">
        <button
          onClick={updateMoodData}
          class="btn btn-primary btn-lg w-full"
          disabled={loading()}
        >
          {loading() ? (
            <>
              <span class="loading loading-spinner"></span>
              Saving...
            </>
          ) : (
            <>💾 Save All Changes</>
          )}
        </button>
        <p class="text-xs text-gray-400 text-center mt-2">
          Updates will appear immediately on the mood ticker overlay
        </p>
      </div>
    </div>
  );
};

export default createPlugin({
  id: 'mood-tracker-plugin',
  name: 'Mood Tracker',
  version: '1.0.0',
  description: 'Track your daily mood, weight, sleep, and water intake',
  author: 'WebArcade Team',

  async onInit() {
    console.log('[Mood Tracker Plugin] Initializing...');
  },

  async onStart(api) {
    console.log('[Mood Tracker Plugin] Starting...');

    // Register Mood Tracker viewport
    api.viewport('mood-tracker', {
      label: 'Mood Tracker',
      component: MoodTrackerViewport,
      icon: IconMoodSmile,
      description: 'Track your daily stats and wellness'
    });
  }
});
