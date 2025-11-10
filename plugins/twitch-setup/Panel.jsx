import { createSignal, createEffect, Show } from 'solid-js';
import { bridgeFetch } from '@/api/bridge';
import { IconCheck, IconX, IconAlertCircle, IconBrandTwitch } from '@tabler/icons-solidjs';

export default function TwitchSetupPanel() {
  const [isConfigured, setIsConfigured] = createSignal(false);
  const [accounts, setAccounts] = createSignal([]);
  const [ircStatus, setIrcStatus] = createSignal(null);
  const [loading, setLoading] = createSignal(false);

  const loadStatus = async () => {
    setLoading(true);

    try {
      const [setupResponse, accountsResponse, ircResponse] = await Promise.all([
        bridgeFetch('/twitch/setup/status'),
        bridgeFetch('/twitch/accounts'),
        bridgeFetch('/twitch/irc/status')
      ]);

      const setupData = await setupResponse.json();
      const accountsData = await accountsResponse.json();
      const ircData = await ircResponse.json();

      setIsConfigured(setupData.is_configured);
      setAccounts(accountsData);
      setIrcStatus(ircData);
    } catch (e) {
      console.error('Failed to load status:', e);
    } finally {
      setLoading(false);
    }
  };

  createEffect(() => {
    loadStatus();

    // Auto-refresh every 10 seconds
    const interval = setInterval(loadStatus, 10000);
    return () => clearInterval(interval);
  });

  const getAccount = (type) => {
    return accounts().find(acc => acc.account_type === type);
  };

  const getSetupProgress = () => {
    let completed = 0;
    let total = 3;

    if (isConfigured()) completed++;
    if (getAccount('broadcaster')) completed++;
    if (ircStatus()?.connected) completed++;

    return { completed, total, percent: Math.round((completed / total) * 100) };
  };

  return (
    <div class="p-4 space-y-4">
      <div class="flex items-center gap-2">
        <IconBrandTwitch class="w-5 h-5 text-primary" />
        <h2 class="text-lg font-bold">Twitch Setup</h2>
      </div>

      <Show
        when={!loading()}
        fallback={
          <div class="flex justify-center p-4">
            <span class="loading loading-spinner loading-md"></span>
          </div>
        }
      >
        {/* Setup Progress */}
        <div class="space-y-2">
          <div class="flex justify-between text-sm">
            <span class="font-medium">Setup Progress</span>
            <span>{getSetupProgress().percent}%</span>
          </div>
          <progress
            class="progress progress-primary w-full"
            value={getSetupProgress().completed}
            max={getSetupProgress().total}
          ></progress>
        </div>

        {/* Status Checklist */}
        <div class="space-y-3">
          <div class="flex items-start gap-2">
            <Show
              when={isConfigured()}
              fallback={<IconX class="w-5 h-5 text-error flex-shrink-0 mt-0.5" />}
            >
              <IconCheck class="w-5 h-5 text-success flex-shrink-0 mt-0.5" />
            </Show>
            <div class="flex-1">
              <div class="text-sm font-medium">App Credentials</div>
              <div class="text-xs text-base-content/60">
                {isConfigured() ? 'Client ID & Secret configured' : 'Not configured'}
              </div>
            </div>
          </div>

          <div class="flex items-start gap-2">
            <Show
              when={getAccount('broadcaster')}
              fallback={<IconX class="w-5 h-5 text-error flex-shrink-0 mt-0.5" />}
            >
              <IconCheck class="w-5 h-5 text-success flex-shrink-0 mt-0.5" />
            </Show>
            <div class="flex-1">
              <div class="text-sm font-medium">Broadcaster Account</div>
              <div class="text-xs text-base-content/60">
                {getAccount('broadcaster')?.username || 'Not connected'}
              </div>
            </div>
          </div>

          <div class="flex items-start gap-2">
            <Show
              when={getAccount('bot')}
              fallback={<IconAlertCircle class="w-5 h-5 text-warning flex-shrink-0 mt-0.5" />}
            >
              <IconCheck class="w-5 h-5 text-success flex-shrink-0 mt-0.5" />
            </Show>
            <div class="flex-1">
              <div class="text-sm font-medium">Bot Account</div>
              <div class="text-xs text-base-content/60">
                {getAccount('bot')?.username || 'Optional'}
              </div>
            </div>
          </div>

          <div class="flex items-start gap-2">
            <Show
              when={ircStatus()?.connected}
              fallback={<IconX class="w-5 h-5 text-error flex-shrink-0 mt-0.5" />}
            >
              <IconCheck class="w-5 h-5 text-success flex-shrink-0 mt-0.5" />
            </Show>
            <div class="flex-1">
              <div class="text-sm font-medium">IRC Connection</div>
              <div class="text-xs text-base-content/60">
                {ircStatus()?.connected
                  ? `Connected to #${ircStatus()?.channel}`
                  : 'Disconnected'}
              </div>
            </div>
          </div>
        </div>

        {/* Overall Status */}
        <div class="divider my-2"></div>

        <Show
          when={getSetupProgress().completed === getSetupProgress().total}
          fallback={
            <div class="alert alert-warning">
              <IconAlertCircle class="w-5 h-5" />
              <div class="text-xs">
                <div class="font-medium">Setup Incomplete</div>
                <div>Complete all steps to enable Twitch integration</div>
              </div>
            </div>
          }
        >
          <div class="alert alert-success">
            <IconCheck class="w-5 h-5" />
            <div class="text-xs">
              <div class="font-medium">Ready!</div>
              <div>Twitch integration is fully configured</div>
            </div>
          </div>
        </Show>

        {/* Quick Actions */}
        <div class="text-xs text-base-content/60 space-y-1">
          <div>• Check Twitch Chat widget for messages</div>
          <div>• Monitor events in Twitch Events</div>
          <div>• Build plugins using Twitch API</div>
        </div>
      </Show>
    </div>
  );
}
