import { createSignal, For } from 'solid-js';
import { IconAlertCircle, IconHeart, IconStar, IconGift, IconSwords, IconDiamond, IconSparkles, IconDeviceTv, IconCopy, IconCheck } from '@tabler/icons-solidjs';

const BRIDGE_URL = 'http://localhost:3001';
const OVERLAY_URL = `${BRIDGE_URL}/overlay/alerts`;

export default function AlertsViewport() {
  const [triggering, setTriggering] = createSignal(null);
  const [copied, setCopied] = createSignal(false);

  const alertTypes = [
    {
      id: 'follow',
      name: 'Follow',
      icon: IconHeart,
      color: 'text-pink-500',
      bgColor: 'bg-pink-500/10',
      description: 'New follower alert with heart explosion'
    },
    {
      id: 'sub',
      name: 'Subscription',
      icon: IconStar,
      color: 'text-blue-500',
      bgColor: 'bg-blue-500/10',
      description: 'New subscriber with star shower'
    },
    {
      id: 'resub',
      name: 'Resubscription',
      icon: IconStar,
      color: 'text-purple-500',
      bgColor: 'bg-purple-500/10',
      description: 'Resubscription with enhanced stars'
    },
    {
      id: 'gift_sub',
      name: 'Gift Sub',
      icon: IconGift,
      color: 'text-yellow-500',
      bgColor: 'bg-yellow-500/10',
      description: 'Gift subscription with presents'
    },
    {
      id: 'raid',
      name: 'Raid',
      icon: IconSwords,
      color: 'text-green-500',
      bgColor: 'bg-green-500/10',
      description: 'Incoming raid with marching cubes'
    },
    {
      id: 'bits',
      name: 'Bits / Cheer',
      icon: IconDiamond,
      color: 'text-purple-400',
      bgColor: 'bg-purple-400/10',
      description: 'Bits donation with spinning gems'
    },
    {
      id: 'channel_points',
      name: 'Channel Points',
      icon: IconSparkles,
      color: 'text-cyan-500',
      bgColor: 'bg-cyan-500/10',
      description: 'Channel points redemption with glowing orbs'
    }
  ];

  const triggerAlert = async (alertType) => {
    setTriggering(alertType);
    try {
      const response = await fetch(`${BRIDGE_URL}/twitch/alert/test`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ alert_type: alertType })
      });

      const result = await response.json();
      if (!result.success) {
        throw new Error(result.error || 'Failed to trigger alert');
      }

      console.log(`✅ ${alertType} alert triggered`);
    } catch (error) {
      console.error(`❌ Failed to trigger ${alertType} alert:`, error);
      alert(`Failed to trigger alert: ${error.message}`);
    } finally {
      setTimeout(() => setTriggering(null), 500);
    }
  };

  const copyOverlayUrl = () => {
    navigator.clipboard.writeText(OVERLAY_URL);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const triggerAllAlerts = async () => {
    for (const alert of alertTypes) {
      await triggerAlert(alert.id);
      // Wait 7 seconds between alerts to avoid overlap
      await new Promise(resolve => setTimeout(resolve, 7000));
    }
  };

  return (
    <div class="h-full flex flex-col bg-base-200">
      {/* Header */}
      <div class="p-6 bg-base-100 border-b border-base-300">
        <div class="flex items-center justify-between">
          <div>
            <h1 class="text-3xl font-bold flex items-center gap-3">
              <IconAlertCircle size={32} class="text-primary" />
              Alerts Overlay
            </h1>
            <p class="text-base-content/70 mt-2">
              Test and preview your stream alerts with stunning 3D Babylon.js animations
            </p>
          </div>
          <button
            class="btn btn-primary btn-lg gap-2"
            onClick={triggerAllAlerts}
            disabled={triggering() !== null}
          >
            <IconDeviceTv size={24} />
            Test All Alerts
          </button>
        </div>
      </div>

      {/* Content */}
      <div class="flex-1 overflow-y-auto p-6">
        <div class="max-w-6xl mx-auto space-y-6">
          {/* Overlay URL Card */}
          <div class="card bg-base-100 shadow-xl">
            <div class="card-body">
              <h2 class="card-title">
                <IconDeviceTv size={24} />
                Overlay URL for OBS
              </h2>
              <div class="flex items-center gap-3">
                <input
                  type="text"
                  class="input input-bordered flex-1 font-mono"
                  value={OVERLAY_URL}
                  readonly
                />
                <button
                  class={`btn ${copied() ? 'btn-success' : 'btn-secondary'} gap-2`}
                  onClick={copyOverlayUrl}
                >
                  {copied() ? <IconCheck size={20} /> : <IconCopy size={20} />}
                  {copied() ? 'Copied!' : 'Copy URL'}
                </button>
              </div>

              <div class="alert alert-info mt-4">
                <IconDeviceTv size={24} />
                <div>
                  <h3 class="font-bold">How to add to OBS</h3>
                  <ol class="text-sm list-decimal list-inside mt-2 space-y-1">
                    <li>Add a <strong>Browser Source</strong> in OBS</li>
                    <li>Paste the URL above</li>
                    <li>Set Width: <strong>1920</strong>, Height: <strong>1080</strong></li>
                    <li>Check ✅ "Shutdown source when not visible"</li>
                    <li>Check ✅ "Refresh browser when scene becomes active"</li>
                  </ol>
                </div>
              </div>
            </div>
          </div>

          {/* Alert Types */}
          <div class="card bg-base-100 shadow-xl">
            <div class="card-body">
              <h2 class="card-title mb-4">Test Individual Alerts</h2>
              <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                <For each={alertTypes}>
                  {(alert) => (
                    <div class={`card ${alert.bgColor} border-2 border-base-300 transition-all hover:border-primary`}>
                      <div class="card-body p-4">
                        <div class="flex items-start justify-between gap-4">
                          <div class="flex items-center gap-3 flex-1">
                            <div class={`p-3 rounded-lg ${alert.bgColor}`}>
                              <alert.icon size={28} class={alert.color} />
                            </div>
                            <div class="flex-1">
                              <h3 class="font-bold text-lg">{alert.name}</h3>
                              <p class="text-sm text-base-content/70">{alert.description}</p>
                            </div>
                          </div>
                          <button
                            class={`btn btn-primary ${triggering() === alert.id ? 'loading' : ''}`}
                            onClick={() => triggerAlert(alert.id)}
                            disabled={triggering() !== null}
                          >
                            {triggering() === alert.id ? 'Triggering...' : 'Test'}
                          </button>
                        </div>
                      </div>
                    </div>
                  )}
                </For>
              </div>
            </div>
          </div>

          {/* Animation Features */}
          <div class="card bg-base-100 shadow-xl">
            <div class="card-body">
              <h2 class="card-title">
                <IconSparkles size={24} class="text-primary" />
                Animation Features
              </h2>
              <div class="grid grid-cols-1 md:grid-cols-2 gap-4 mt-4">
                <div class="flex items-start gap-3">
                  <div class="badge badge-primary badge-lg">3D</div>
                  <div>
                    <h3 class="font-semibold">Babylon.js 3D Graphics</h3>
                    <p class="text-sm text-base-content/70">GPU-accelerated 3D meshes and animations</p>
                  </div>
                </div>
                <div class="flex items-start gap-3">
                  <div class="badge badge-secondary badge-lg">FX</div>
                  <div>
                    <h3 class="font-semibold">Particle Systems</h3>
                    <p class="text-sm text-base-content/70">Thousands of particles for each effect</p>
                  </div>
                </div>
                <div class="flex items-start gap-3">
                  <div class="badge badge-accent badge-lg">Q</div>
                  <div>
                    <h3 class="font-semibold">Queue System</h3>
                    <p class="text-sm text-base-content/70">Multiple alerts queued automatically</p>
                  </div>
                </div>
                <div class="flex items-start gap-3">
                  <div class="badge badge-success badge-lg">⚡</div>
                  <div>
                    <h3 class="font-semibold">Auto-Cleanup</h3>
                    <p class="text-sm text-base-content/70">Alerts auto-hide after 6 seconds</p>
                  </div>
                </div>
              </div>
            </div>
          </div>

          {/* Alert Details */}
          <div class="card bg-base-100 shadow-xl">
            <div class="card-body">
              <h2 class="card-title">Alert Animations</h2>
              <div class="overflow-x-auto">
                <table class="table">
                  <thead>
                    <tr>
                      <th>Alert Type</th>
                      <th>3D Objects</th>
                      <th>Particles</th>
                      <th>Special Effects</th>
                    </tr>
                  </thead>
                  <tbody>
                    <tr>
                      <td class="font-semibold flex items-center gap-2">
                        <IconHeart size={18} class="text-pink-500" />
                        Follow
                      </td>
                      <td>12 Hearts</td>
                      <td>500 particles</td>
                      <td>Pink explosion, pulsing hearts</td>
                    </tr>
                    <tr>
                      <td class="font-semibold flex items-center gap-2">
                        <IconStar size={18} class="text-blue-500" />
                        Subscription
                      </td>
                      <td>20 Stars</td>
                      <td>1000 particles</td>
                      <td>Star shower, tier-based colors</td>
                    </tr>
                    <tr>
                      <td class="font-semibold flex items-center gap-2">
                        <IconStar size={18} class="text-purple-500" />
                        Resubscription
                      </td>
                      <td>20 Stars</td>
                      <td>1000 particles</td>
                      <td>Enhanced stars, month display</td>
                    </tr>
                    <tr>
                      <td class="font-semibold flex items-center gap-2">
                        <IconGift size={18} class="text-yellow-500" />
                        Gift Sub
                      </td>
                      <td>8 Gift Boxes + Ribbons</td>
                      <td>800 confetti</td>
                      <td>Bouncing gifts, golden ribbons</td>
                    </tr>
                    <tr>
                      <td class="font-semibold flex items-center gap-2">
                        <IconSwords size={18} class="text-green-500" />
                        Raid
                      </td>
                      <td>Up to 50 Cubes</td>
                      <td>1500 particles</td>
                      <td>Marching army, rainbow colors</td>
                    </tr>
                    <tr>
                      <td class="font-semibold flex items-center gap-2">
                        <IconDiamond size={18} class="text-purple-400" />
                        Bits
                      </td>
                      <td>30 Gems</td>
                      <td>1200 sparkles</td>
                      <td>Spiraling gems, purple-gold gradient</td>
                    </tr>
                    <tr>
                      <td class="font-semibold flex items-center gap-2">
                        <IconSparkles size={18} class="text-cyan-500" />
                        Channel Points
                      </td>
                      <td>15 Glowing Orbs</td>
                      <td>800 particles</td>
                      <td>Pulsating cyan orbs</td>
                    </tr>
                  </tbody>
                </table>
              </div>
            </div>
          </div>

          {/* Documentation Link */}
          <div class="card bg-gradient-to-br from-primary/20 to-secondary/20 border-2 border-primary/30">
            <div class="card-body">
              <h2 class="card-title">
                📚 Documentation
              </h2>
              <p class="text-base-content/80">
                For detailed documentation on customizing animations, adding EventSub support for real-time events, and more,
                check out <code class="bg-base-300 px-2 py-1 rounded">ALERTS_README.md</code> in the project root.
              </p>
              <div class="card-actions justify-end mt-4">
                <div class="badge badge-lg badge-primary">Built with Babylon.js</div>
                <div class="badge badge-lg badge-secondary">Powered by SolidJS</div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
