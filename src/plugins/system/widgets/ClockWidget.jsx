import { createSignal, onMount, onCleanup, For } from 'solid-js';

export default function ClockWidget() {
  const [time, setTime] = createSignal(new Date());

  onMount(() => {
    const interval = setInterval(() => {
      setTime(new Date());
    }, 1000);

    onCleanup(() => clearInterval(interval));
  });

  const getRotation = () => {
    const now = time();
    const seconds = now.getSeconds();
    const minutes = now.getMinutes();
    const hours = now.getHours() % 12;

    return {
      second: seconds * 6, // 360 / 60
      minute: minutes * 6 + seconds * 0.1, // 360 / 60 + smooth movement
      hour: hours * 30 + minutes * 0.5 // 360 / 12 + smooth movement
    };
  };

  const rotation = () => getRotation();

  const formatTime = () => {
    const now = time();
    return now.toLocaleTimeString('en-US', {
      hour: '2-digit',
      minute: '2-digit',
      hour12: true
    });
  };

  const formatDate = () => {
    const now = time();
    return now.toLocaleDateString('en-US', {
      weekday: 'short',
      month: 'short',
      day: 'numeric'
    });
  };

  return (
    <div class="card bg-gradient-to-br from-info/20 to-info/5 bg-base-100 shadow-lg h-full flex flex-col justify-between p-4">
      {/* Clock Face */}
      <div class="flex-1 flex items-center justify-center">
        <div class="relative w-32 h-32">
          {/* Clock Circle */}
          <div class="absolute inset-0 rounded-full border-4 border-base-content/20 bg-base-200/50">
            {/* Hour Markers */}
            <For each={Array.from({ length: 12 })}>
              {(_, i) => {
                const angle = i() * 30 - 90;
                const x = 50 + 38 * Math.cos((angle * Math.PI) / 180);
                const y = 50 + 38 * Math.sin((angle * Math.PI) / 180);
                return (
                  <div
                    class="absolute w-1 h-1 bg-base-content/40 rounded-full"
                    style={{
                      left: `${x}%`,
                      top: `${y}%`,
                      transform: 'translate(-50%, -50%)'
                    }}
                  />
                );
              }}
            </For>

            {/* Center Dot */}
            <div class="absolute top-1/2 left-1/2 w-2 h-2 bg-info rounded-full transform -translate-x-1/2 -translate-y-1/2 z-10" />

            {/* Hour Hand */}
            <div
              class="absolute top-1/2 left-1/2 origin-bottom transition-transform duration-300"
              style={{
                width: '3px',
                height: '35px',
                'background': 'currentColor',
                'border-radius': '2px',
                transform: `translate(-50%, -100%) rotate(${rotation().hour}deg)`,
                color: 'var(--fallback-bc,oklch(var(--bc)/0.8))'
              }}
            />

            {/* Minute Hand */}
            <div
              class="absolute top-1/2 left-1/2 origin-bottom transition-transform duration-300"
              style={{
                width: '2.5px',
                height: '45px',
                'background': 'currentColor',
                'border-radius': '2px',
                transform: `translate(-50%, -100%) rotate(${rotation().minute}deg)`,
                color: 'var(--fallback-bc,oklch(var(--bc)/0.9))'
              }}
            />

            {/* Second Hand */}
            <div
              class="absolute top-1/2 left-1/2 origin-bottom transition-transform duration-1000"
              style={{
                width: '1.5px',
                height: '50px',
                'background': '#ef4444',
                'border-radius': '1px',
                transform: `translate(-50%, -100%) rotate(${rotation().second}deg)`
              }}
            />
          </div>
        </div>
      </div>

      {/* Digital Time Display */}
      <div class="text-center space-y-1">
        <div class="text-2xl font-bold text-info">
          {formatTime()}
        </div>
        <div class="text-xs opacity-60">
          {formatDate()}
        </div>
      </div>
    </div>
  );
}
