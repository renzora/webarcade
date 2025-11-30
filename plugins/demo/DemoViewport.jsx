import { createSignal, onMount, onCleanup } from 'solid-js';
import { IconLayoutDashboard, IconBox, IconChevronRight, IconFileText, IconPalette, IconCheck } from '@tabler/icons-solidjs';
import { api } from '@/api/bridge';
import confetti from 'canvas-confetti';

// BabylonJS - import only what we need
import { Engine } from '@babylonjs/core/Engines/engine';
import { Scene } from '@babylonjs/core/scene';
import { ArcRotateCamera } from '@babylonjs/core/Cameras/arcRotateCamera';
import { HemisphericLight } from '@babylonjs/core/Lights/hemisphericLight';
import { DirectionalLight } from '@babylonjs/core/Lights/directionalLight';
import { MeshBuilder } from '@babylonjs/core/Meshes/meshBuilder';
import { StandardMaterial } from '@babylonjs/core/Materials/standardMaterial';
import { Vector3, Color3, Color4 } from '@babylonjs/core/Maths/math';

// ============================================
// SECTION: Confetti Demo
// ============================================
function ConfettiSection() {
  const fireConfetti = (type) => {
    switch (type) {
      case 'basic':
        confetti({
          particleCount: 100,
          spread: 70,
          origin: { y: 0.6 }
        });
        break;
      case 'fireworks':
        const duration = 3000;
        const end = Date.now() + duration;
        const colors = ['#ff0000', '#00ff00', '#0000ff', '#ffff00', '#ff00ff'];

        (function frame() {
          confetti({
            particleCount: 5,
            angle: 60,
            spread: 55,
            origin: { x: 0 },
            colors: colors
          });
          confetti({
            particleCount: 5,
            angle: 120,
            spread: 55,
            origin: { x: 1 },
            colors: colors
          });
          if (Date.now() < end) requestAnimationFrame(frame);
        }());
        break;
      case 'stars':
        confetti({
          particleCount: 50,
          spread: 360,
          ticks: 100,
          origin: { x: 0.5, y: 0.5 },
          shapes: ['star'],
          colors: ['#FFD700', '#FFA500', '#FF6347']
        });
        break;
      case 'snow':
        const snowDuration = 3000;
        const snowEnd = Date.now() + snowDuration;
        (function snowFrame() {
          confetti({
            particleCount: 3,
            startVelocity: 0,
            ticks: 200,
            origin: { x: Math.random(), y: -0.1 },
            colors: ['#ffffff', '#e0e0e0'],
            shapes: ['circle'],
            gravity: 0.5,
            scalar: 1.5,
            drift: Math.random() - 0.5
          });
          if (Date.now() < snowEnd) requestAnimationFrame(snowFrame);
        }());
        break;
    }
  };

  return (
    <div class="card bg-gradient-to-r from-pink-500/10 to-purple-500/10 border border-pink-500/20 shadow-lg">
      <div class="card-body">
        <h2 class="card-title text-pink-500">
          <IconPalette class="w-6 h-6" />
          Confetti Party
        </h2>
        <p class="text-base-content/70 mb-4">
          Click the buttons to trigger different confetti effects!
        </p>
        <div class="flex flex-wrap gap-2">
          <button class="btn btn-primary" onClick={() => fireConfetti('basic')}>
            Basic Burst
          </button>
          <button class="btn btn-secondary" onClick={() => fireConfetti('fireworks')}>
            Fireworks
          </button>
          <button class="btn btn-warning" onClick={() => fireConfetti('stars')}>
            Stars
          </button>
          <button class="btn btn-info" onClick={() => fireConfetti('snow')}>
            Snow
          </button>
        </div>
      </div>
    </div>
  );
}

// ============================================
// SECTION: BabylonJS 3D Scene
// ============================================
function BabylonSection() {
  let canvasRef;
  let engineRef;

  onMount(() => {
    if (!canvasRef) return;

    const engine = new Engine(canvasRef, true);
    engineRef = engine;
    const scene = new Scene(engine);
    scene.clearColor = new Color4(0, 0, 0, 0);

    // Camera
    const camera = new ArcRotateCamera(
      'camera',
      Math.PI / 4,
      Math.PI / 3,
      5,
      Vector3.Zero(),
      scene
    );
    camera.attachControl(canvasRef, true);
    camera.wheelPrecision = 50;

    // Lighting
    const light = new HemisphericLight('light', new Vector3(0, 1, 0), scene);
    light.intensity = 0.7;

    const dirLight = new DirectionalLight('dirLight', new Vector3(-1, -2, -1), scene);
    dirLight.intensity = 0.5;

    // Create cube with gradient material
    const cube = MeshBuilder.CreateBox('cube', { size: 1.5 }, scene);
    const cubeMaterial = new StandardMaterial('cubeMat', scene);
    cubeMaterial.diffuseColor = new Color3(0.4, 0.6, 1);
    cubeMaterial.specularColor = new Color3(0.5, 0.5, 0.5);
    cubeMaterial.emissiveColor = new Color3(0.1, 0.1, 0.2);
    cube.material = cubeMaterial;
    cube.position.y = 0.75;

    // Create grid
    const gridSize = 10;
    for (let i = -gridSize / 2; i <= gridSize / 2; i++) {
      const lineX = MeshBuilder.CreateLines('lineX' + i, {
        points: [new Vector3(i, 0, -gridSize / 2), new Vector3(i, 0, gridSize / 2)]
      }, scene);
      lineX.color = new Color3(0.3, 0.5, 0.7);

      const lineZ = MeshBuilder.CreateLines('lineZ' + i, {
        points: [new Vector3(-gridSize / 2, 0, i), new Vector3(gridSize / 2, 0, i)]
      }, scene);
      lineZ.color = new Color3(0.3, 0.5, 0.7);
    }

    // Animate cube rotation
    scene.registerBeforeRender(() => {
      cube.rotation.y += 0.01;
      cube.rotation.x += 0.005;
    });

    engine.runRenderLoop(() => {
      scene.render();
    });

    // Handle resize
    const resizeHandler = () => engine.resize();
    window.addEventListener('resize', resizeHandler);

    onCleanup(() => {
      window.removeEventListener('resize', resizeHandler);
      engine.dispose();
    });
  });

  // Prevent scroll from propagating to parent viewport
  const handleWheel = (e) => {
    e.stopPropagation();
  };

  return (
    <div class="card bg-gradient-to-r from-blue-500/10 to-cyan-500/10 border border-blue-500/20 shadow-lg">
      <div class="card-body">
        <h2 class="card-title text-blue-500">
          <IconBox class="w-6 h-6" />
          3D Scene (BabylonJS)
        </h2>
        <p class="text-base-content/70 mb-4">
          Interactive 3D rendering with BabylonJS. Drag to rotate, scroll to zoom.
        </p>
        <div
          class="rounded-lg overflow-hidden bg-base-300 border border-base-content/10"
          onWheel={handleWheel}
        >
          <canvas ref={canvasRef} class="w-full h-64" />
        </div>
      </div>
    </div>
  );
}

// ============================================
// SECTION: Brightness Control
// ============================================
function MonitorSlider(props) {
  const [localValue, setLocalValue] = createSignal(props.brightness);
  let debounceTimer;

  const sendUpdate = async (value) => {
    try {
      const response = await api('demo/brightness', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ brightness: value, monitor: props.name })
      });
      const data = await response.json();
      if (data.success) {
        props.onStatus(`${props.name}: ${value}%`);
      } else {
        props.onStatus('Error: ' + data.error);
      }
    } catch (error) {
      props.onStatus('Error: ' + error.message);
    }
  };

  const handleInput = (e) => {
    const value = parseInt(e.target.value);
    setLocalValue(value);
    clearTimeout(debounceTimer);
    debounceTimer = setTimeout(() => sendUpdate(value), 200);
  };

  onCleanup(() => clearTimeout(debounceTimer));

  return (
    <div class="space-y-2">
      <div class="flex justify-between text-sm">
        <span class="font-medium">{props.name}</span>
        <span class="text-primary font-semibold">{localValue()}%</span>
      </div>
      <input
        type="range"
        min="0"
        max="100"
        value={localValue()}
        onInput={handleInput}
        class="range range-warning"
      />
    </div>
  );
}

function BrightnessSection() {
  const [monitors, setMonitors] = createSignal([]);
  const [loading, setLoading] = createSignal(false);
  const [status, setStatus] = createSignal('');

  const fetchBrightness = async () => {
    setLoading(true);
    try {
      const response = await api('demo/brightness');
      const data = await response.json();
      setMonitors(data.monitors || []);
    } catch (error) {
      setStatus('Error: ' + error.message);
    }
    setLoading(false);
  };

  onMount(fetchBrightness);

  return (
    <div class="card bg-gradient-to-r from-yellow-500/10 to-orange-500/10 border border-yellow-500/20 shadow-lg">
      <div class="card-body">
        <h2 class="card-title text-yellow-500">
          <IconPalette class="w-6 h-6" />
          Monitor Brightness
        </h2>
        <p class="text-base-content/70 mb-4">
          Control your monitor brightness directly from the browser using native APIs.
        </p>
        {loading() ? (
          <span class="loading loading-spinner loading-md"></span>
        ) : monitors().length === 0 ? (
          <p class="text-warning text-sm">No controllable monitors detected (may require DDC/CI support)</p>
        ) : (
          <div class="space-y-4">
            {monitors().map((monitor, i) => (
              <MonitorSlider
                key={i}
                name={monitor.name}
                brightness={monitor.brightness}
                onStatus={setStatus}
              />
            ))}
          </div>
        )}
        {status() && (
          <p class={`text-sm mt-2 ${status().includes('Error') ? 'text-error' : 'text-success'}`}>
            {status()}
          </p>
        )}
      </div>
    </div>
  );
}

// ============================================
// SECTION: System Info Cards
// ============================================
function SystemInfoSection() {
  const [cpuInfo, setCpuInfo] = createSignal(null);
  const [gpuInfo, setGpuInfo] = createSignal(null);
  const [ramInfo, setRamInfo] = createSignal(null);

  let refreshInterval;

  const fetchAll = async () => {
    const [cpuRes, gpuRes, ramRes] = await Promise.all([
      api('demo/cpu').then(r => r.json()).catch(() => null),
      api('demo/gpu').then(r => r.json()).catch(() => null),
      api('demo/ram').then(r => r.json()).catch(() => null)
    ]);
    setCpuInfo(cpuRes);
    setGpuInfo(gpuRes);
    setRamInfo(ramRes);
  };

  onMount(() => {
    fetchAll();
    refreshInterval = setInterval(fetchAll, 2000);
  });

  onCleanup(() => clearInterval(refreshInterval));

  return (
    <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
      {/* CPU */}
      <div class="card bg-base-100 shadow-lg">
        <div class="card-body p-4">
          <h3 class="font-semibold text-blue-500">CPU</h3>
          {cpuInfo() ? (
            <div class="text-sm space-y-1">
              <p class="truncate font-medium">{cpuInfo().brand}</p>
              <div class="flex justify-between">
                <span class="text-base-content/60">Usage:</span>
                <span class="text-primary font-bold">{cpuInfo().usage_percent?.toFixed(0)}%</span>
              </div>
              <progress class="progress progress-primary w-full h-2" value={cpuInfo().usage_percent || 0} max="100"></progress>
            </div>
          ) : <span class="loading loading-spinner loading-sm"></span>}
        </div>
      </div>

      {/* GPU */}
      <div class="card bg-base-100 shadow-lg">
        <div class="card-body p-4">
          <h3 class="font-semibold text-green-500">GPU</h3>
          {gpuInfo() ? (
            gpuInfo().available ? (
              <div class="text-sm space-y-1">
                <p class="truncate font-medium">{gpuInfo().name}</p>
                <div class="flex justify-between">
                  <span class="text-base-content/60">Temp:</span>
                  <span class={gpuInfo().temperature_c > 70 ? 'text-error' : 'text-success'}>
                    {gpuInfo().temperature_c}°C
                  </span>
                </div>
                {gpuInfo().utilization && (
                  <progress class="progress progress-success w-full h-2" value={gpuInfo().utilization.gpu_percent || 0} max="100"></progress>
                )}
              </div>
            ) : <p class="text-warning text-xs">No NVIDIA GPU</p>
          ) : <span class="loading loading-spinner loading-sm"></span>}
        </div>
      </div>

      {/* RAM */}
      <div class="card bg-base-100 shadow-lg">
        <div class="card-body p-4">
          <h3 class="font-semibold text-purple-500">RAM</h3>
          {ramInfo() ? (
            <div class="text-sm space-y-1">
              <div class="flex justify-between">
                <span class="text-base-content/60">Used:</span>
                <span>{ramInfo().used_gb?.toFixed(1)} / {ramInfo().total_gb?.toFixed(0)} GB</span>
              </div>
              <div class="flex justify-between">
                <span class="text-base-content/60">Usage:</span>
                <span class="text-secondary font-bold">{ramInfo().usage_percent?.toFixed(0)}%</span>
              </div>
              <progress class="progress progress-secondary w-full h-2" value={ramInfo().usage_percent || 0} max="100"></progress>
            </div>
          ) : <span class="loading loading-spinner loading-sm"></span>}
        </div>
      </div>
    </div>
  );
}

// ============================================
// SECTION: Native Backend Demo
// ============================================
function NativeBackendSection() {
  const [backendMessage, setBackendMessage] = createSignal('');
  const [notificationStatus, setNotificationStatus] = createSignal('');

  const callBackend = async () => {
    try {
      const response = await api('demo/hello');
      const data = await response.json();
      setBackendMessage(data.message);
    } catch (error) {
      setBackendMessage('Error: ' + error.message);
    }
  };

  const sendNotification = async () => {
    try {
      setNotificationStatus('Sending...');
      const response = await api('demo/notify', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          title: 'WebArcade Demo',
          message: 'Hello from the Demo Plugin!'
        })
      });
      const data = await response.json();
      setNotificationStatus(data.success ? 'Notification sent!' : 'Error: ' + data.error);
    } catch (error) {
      setNotificationStatus('Error: ' + error.message);
    }
  };

  return (
    <div class="card bg-gradient-to-r from-orange-500/10 to-red-500/10 border border-orange-500/20 shadow-lg">
      <div class="card-body">
        <h2 class="card-title text-orange-500">
          <IconBox class="w-6 h-6" />
          Native Rust Backend
        </h2>
        <p class="text-base-content/70 mb-4">
          Interact with the native Rust backend - call functions and trigger OS notifications.
        </p>
        <div class="flex flex-wrap gap-4">
          <div class="flex flex-col gap-2">
            <button class="btn btn-outline btn-warning" onClick={callBackend}>
              Call Backend
            </button>
            {backendMessage() && (
              <div class="text-sm text-success bg-success/10 p-2 rounded max-w-xs">
                {backendMessage()}
              </div>
            )}
          </div>
          <div class="flex flex-col gap-2">
            <button class="btn btn-warning" onClick={sendNotification}>
              Send Notification
            </button>
            {notificationStatus() && (
              <div class={`text-sm p-2 rounded ${notificationStatus().includes('Error') ? 'text-error bg-error/10' : 'text-success bg-success/10'}`}>
                {notificationStatus()}
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}

// ============================================
// SECTION: UI Components Showcase
// ============================================
function UIComponentsSection() {
  return (
    <div class="card bg-base-100 shadow-lg">
      <div class="card-body">
        <h2 class="card-title">
          <IconLayoutDashboard class="w-6 h-6" />
          Plugin UI Components
        </h2>
        <p class="text-base-content/70 mb-4">
          WebArcade plugins can register these UI extension points:
        </p>
        <div class="grid grid-cols-2 md:grid-cols-4 gap-3">
          <div class="p-3 bg-primary/10 rounded-lg text-center">
            <IconLayoutDashboard class="w-6 h-6 mx-auto text-primary mb-1" />
            <p class="text-xs font-medium">Viewport</p>
          </div>
          <div class="p-3 bg-secondary/10 rounded-lg text-center">
            <IconBox class="w-6 h-6 mx-auto text-secondary mb-1" />
            <p class="text-xs font-medium">Left Panel</p>
          </div>
          <div class="p-3 bg-accent/10 rounded-lg text-center">
            <IconPalette class="w-6 h-6 mx-auto text-accent mb-1" />
            <p class="text-xs font-medium">Right Panel</p>
          </div>
          <div class="p-3 bg-info/10 rounded-lg text-center">
            <IconChevronRight class="w-6 h-6 mx-auto text-info mb-1" />
            <p class="text-xs font-medium">Bottom Panel</p>
          </div>
          <div class="p-3 bg-warning/10 rounded-lg text-center">
            <IconCheck class="w-6 h-6 mx-auto text-warning mb-1" />
            <p class="text-xs font-medium">Toolbar</p>
          </div>
          <div class="p-3 bg-success/10 rounded-lg text-center">
            <IconFileText class="w-6 h-6 mx-auto text-success mb-1" />
            <p class="text-xs font-medium">Menu</p>
          </div>
          <div class="p-3 bg-error/10 rounded-lg text-center">
            <IconFileText class="w-6 h-6 mx-auto text-error mb-1" />
            <p class="text-xs font-medium">Footer</p>
          </div>
          <div class="p-3 bg-base-300 rounded-lg text-center">
            <IconBox class="w-6 h-6 mx-auto text-base-content/50 mb-1" />
            <p class="text-xs font-medium">Backend</p>
          </div>
        </div>
      </div>
    </div>
  );
}

// ============================================
// MAIN VIEWPORT
// ============================================
export default function DemoViewport() {
  return (
    <div class="w-full h-full flex flex-col bg-base-200 p-6 overflow-auto">
      {/* Header */}
      <div class="mb-8">
        <h1 class="text-3xl font-bold text-primary flex items-center gap-3">
          <IconLayoutDashboard class="w-8 h-8" />
          Demo Plugin Showcase
        </h1>
        <p class="text-base-content/70 mt-2">
          Explore the capabilities of WebArcade plugins - from 3D graphics to native system APIs.
        </p>
      </div>

      {/* Live System Stats */}
      <div class="mb-6">
        <h2 class="text-lg font-semibold mb-3 flex items-center gap-2">
          <IconBox class="w-5 h-5 text-primary" />
          Live System Stats
        </h2>
        <SystemInfoSection />
      </div>

      {/* Interactive Features Grid */}
      <div class="grid grid-cols-1 lg:grid-cols-2 gap-6 mb-6">
        <ConfettiSection />
        <BabylonSection />
      </div>

      {/* Hardware Control */}
      <div class="grid grid-cols-1 lg:grid-cols-2 gap-6 mb-6">
        <BrightnessSection />
        <NativeBackendSection />
      </div>

      {/* UI Components */}
      <UIComponentsSection />
    </div>
  );
}
