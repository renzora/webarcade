import { createSignal, createEffect, onCleanup, Show } from 'solid-js';
import { Dumbbell, TrendingUp, TrendingDown, Minus } from 'lucide-solid';
import { withingsAPI } from '../api';

export default function MuscleMassWidget() {
  const [measurements, setMeasurements] = createSignal([]);
  const [loading, setLoading] = createSignal(true);
  const [latestMuscle, setLatestMuscle] = createSignal(null);
  const [musclePercent, setMusclePercent] = createSignal(null);
  const [trend, setTrend] = createSignal(null);

  const fetchData = async () => {
    try {
      const data = await withingsAPI.getMeasurements();
      const filtered = data.filter(m => m.muscle_mass !== null).slice(0, 30);
      setMeasurements(filtered);

      if (filtered.length > 0) {
        const latest = filtered[0];
        setLatestMuscle(latest.muscle_mass);

        // Calculate muscle percentage (muscle_mass / weight * 100)
        if (latest.weight && latest.muscle_mass) {
          const percent = (latest.muscle_mass / latest.weight) * 100;
          setMusclePercent(percent);
        }

        // Calculate trend
        if (filtered.length >= 2) {
          const current = filtered[0].muscle_mass;
          const previous = filtered[1].muscle_mass;
          const change = current - previous;

          if (Math.abs(change) < 0.1) {
            setTrend({ type: 'stable', value: 0 });
          } else if (change > 0) {
            setTrend({ type: 'up', value: change });
          } else {
            setTrend({ type: 'down', value: Math.abs(change) });
          }
        }
      }
    } catch (error) {
      console.error('Failed to fetch muscle mass data:', error);
    } finally {
      setLoading(false);
    }
  };

  createEffect(() => {
    fetchData();

    const interval = setInterval(fetchData, 60000);
    onCleanup(() => clearInterval(interval));
  });

  const getMinMax = () => {
    const muscles = measurements().map(m => m.muscle_mass);
    if (muscles.length === 0) return { min: 0, max: 100 };

    const min = Math.min(...muscles);
    const max = Math.max(...muscles);
    const padding = (max - min) * 0.1 || 5;

    return {
      min: Math.floor(min - padding),
      max: Math.ceil(max + padding),
    };
  };

  const generateSparkline = () => {
    const data = measurements();
    if (data.length === 0) return '';

    const { min, max } = getMinMax();
    const range = max - min;
    const width = 100;
    const height = 40;

    const points = data
      .slice()
      .reverse()
      .map((m, i) => {
        const x = (i / (data.length - 1)) * width;
        const y = height - ((m.muscle_mass - min) / range) * height;
        return `${x},${y}`;
      })
      .join(' ');

    return `M ${points.split(' ').map((p, i) => (i === 0 ? 'M ' : 'L ') + p).join(' ')}`;
  };

  const getTrendIcon = () => {
    const t = trend();
    if (!t) return null;

    switch (t.type) {
      case 'up':
        return <TrendingUp class="w-4 h-4 text-success" />;
      case 'down':
        return <TrendingDown class="w-4 h-4 text-error" />;
      case 'stable':
        return <Minus class="w-4 h-4 text-info" />;
      default:
        return null;
    }
  };

  const getTrendColor = () => {
    const t = trend();
    if (!t) return 'text-base-content';

    switch (t.type) {
      case 'up':
        return 'text-success';
      case 'down':
        return 'text-error';
      case 'stable':
        return 'text-info';
      default:
        return 'text-base-content';
    }
  };

  const getMuscleStatus = () => {
    const percent = musclePercent();
    if (!percent) return { status: 'Unknown', color: 'text-base-content' };

    // General guidelines:
    // Men: 42-52% is good, Women: 36-42% is good
    // Using general middle range: 38-48%
    if (percent >= 38 && percent <= 52) {
      return { status: 'Excellent', color: 'text-success' };
    } else if (percent >= 32 && percent < 38) {
      return { status: 'Good', color: 'text-info' };
    } else if (percent < 32) {
      return { status: 'Low', color: 'text-warning' };
    } else {
      return { status: 'Very High', color: 'text-success' };
    }
  };

  return (
    <div class="card bg-base-100 shadow-lg h-full">
      <div class="card-body p-4">
        <Show when={loading()}>
          <div class="flex justify-center items-center h-full">
            <span class="loading loading-spinner loading-sm"></span>
          </div>
        </Show>

        <Show when={!loading()}>
          <div class="flex items-center justify-between mb-2">
            <div class="flex items-center gap-2">
              <Dumbbell class="w-4 h-4 text-secondary" />
              <h3 class="text-sm font-semibold text-base-content/70">Muscle Mass</h3>
            </div>
            <Show when={trend()}>
              <div class="flex items-center gap-1">
                {getTrendIcon()}
                <span class={`text-xs font-medium ${getTrendColor()}`}>
                  {trend()?.value?.toFixed(1)} kg
                </span>
              </div>
            </Show>
          </div>

          <Show when={latestMuscle() !== null} fallback={
            <div class="text-center text-base-content/50 py-8">
              <p class="text-sm">No muscle mass data</p>
            </div>
          }>
            <div class="flex items-baseline gap-2 mb-1">
              <span class="text-3xl font-bold text-secondary">
                {latestMuscle()?.toFixed(1)}
              </span>
              <span class="text-sm text-base-content/60">kg</span>
            </div>

            <Show when={musclePercent()}>
              <div class="flex items-center gap-2 mb-4">
                <span class={`text-sm font-medium ${getMuscleStatus().color}`}>
                  {musclePercent()?.toFixed(1)}% of body weight
                </span>
                <div class={`badge badge-sm ${getMuscleStatus().color}`}>
                  {getMuscleStatus().status}
                </div>
              </div>
            </Show>

            {/* Sparkline Chart */}
            <Show when={measurements().length > 1}>
              <div class="w-full h-16 mb-2">
                <svg viewBox="0 0 100 40" class="w-full h-full" preserveAspectRatio="none">
                  {/* Grid lines */}
                  <line x1="0" y1="0" x2="100" y2="0" stroke="currentColor" stroke-opacity="0.1" stroke-width="0.5" />
                  <line x1="0" y1="20" x2="100" y2="20" stroke="currentColor" stroke-opacity="0.1" stroke-width="0.5" />
                  <line x1="0" y1="40" x2="100" y2="40" stroke="currentColor" stroke-opacity="0.1" stroke-width="0.5" />

                  {/* Line chart */}
                  <path
                    d={generateSparkline()}
                    fill="none"
                    stroke="hsl(var(--s))"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                  />

                  {/* Area fill */}
                  <path
                    d={`${generateSparkline()} L 100,40 L 0,40 Z`}
                    fill="hsl(var(--s))"
                    fill-opacity="0.1"
                  />
                </svg>
              </div>

              <div class="flex justify-between text-xs text-base-content/50">
                <span>{measurements()[measurements().length - 1]?.muscle_mass?.toFixed(1)} kg</span>
                <span class="text-base-content/70">Last 30 measurements</span>
                <span>{measurements()[0]?.muscle_mass?.toFixed(1)} kg</span>
              </div>
            </Show>

            {/* Stats */}
            <div class="mt-4 pt-4 border-t border-base-300">
              <div class="grid grid-cols-2 gap-2 text-xs">
                <div>
                  <div class="text-base-content/50">Average</div>
                  <div class="font-semibold">
                    {measurements().length > 0
                      ? (measurements().reduce((sum, m) => sum + m.muscle_mass, 0) / measurements().length).toFixed(1)
                      : 'N/A'} kg
                  </div>
                </div>
                <div>
                  <div class="text-base-content/50">Measurements</div>
                  <div class="font-semibold">{measurements().length}</div>
                </div>
              </div>
            </div>

            {/* Muscle Mass Guide */}
            <div class="mt-2 p-2 bg-base-200 rounded text-xs">
              <div class="font-medium mb-1">Healthy Range:</div>
              <div class="text-base-content/60">
                Men: 42-52% • Women: 36-42%
              </div>
            </div>
          </Show>
        </Show>
      </div>
    </div>
  );
}
