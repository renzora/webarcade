import { createSignal, onMount, Show } from 'solid-js';
import twitchStore from './TwitchStore.jsx';
import { IconBrandTwitch, IconCheck, IconTrash, IconAlertCircle, IconUser } from '@tabler/icons-solidjs';

export default function TwitchAccountManager() {
  const [authStatus, setAuthStatus] = createSignal(null);
  const [botAuthStatus, setBotAuthStatus] = createSignal(null);
  const [loading, setLoading] = createSignal(true);
  const [botLoading, setBotLoading] = createSignal(true);
  const [authenticating, setAuthenticating] = createSignal(false);
  const [botAuthenticating, setBotAuthenticating] = createSignal(false);

  onMount(async () => {
    await loadAuthStatus();
    await loadBotAuthStatus();
  });

  const loadAuthStatus = async () => {
    try {
      const status = await twitchStore.fetchStatus();
      setAuthStatus(status);
    } catch (e) {
      console.error('[AccountManager] Failed to load auth status:', e);
      setAuthStatus({ authenticated: false });
    } finally {
      setLoading(false);
    }
  };

  const loadBotAuthStatus = async () => {
    try {
      const response = await fetch('/twitch/bot/auth/status');
      const status = await response.json();
      setBotAuthStatus(status);
    } catch (e) {
      console.error('[AccountManager] Failed to load bot auth status:', e);
      setBotAuthStatus({ authenticated: false });
    } finally {
      setBotLoading(false);
    }
  };

  const handleAuthenticate = async () => {
    try {
      setAuthenticating(true);

      const url = await twitchStore.getAuthUrl();

      const authWindow = window.open(url, 'TwitchOAuth', 'width=600,height=800');

      if (!authWindow) {
        alert('Popup was blocked. Please allow popups for this site and try again.');
        setAuthenticating(false);
        return;
      }

      // Poll to check if authentication is complete
      const checkOAuth = setInterval(async () => {
        try {
          if (authWindow.closed) {
            clearInterval(checkOAuth);
            // Wait a moment for the backend to process
            await new Promise(resolve => setTimeout(resolve, 1000));
            await loadAuthStatus();
            setAuthenticating(false);
          }
        } catch (e) {
          console.error('Error checking OAuth:', e);
        }
      }, 1000);
    } catch (e) {
      console.error('Failed to authenticate:', e);
      alert(`Failed to authenticate: ${e.message}`);
      setAuthenticating(false);
    }
  };

  const handleRevoke = async () => {
    if (!confirm('Are you sure you want to disconnect your Twitch account?')) {
      return;
    }

    try {
      await twitchStore.revokeToken();
      await loadAuthStatus();
    } catch (e) {
      console.error('Failed to revoke auth:', e);
      alert(`Failed to disconnect: ${e.message}`);
    }
  };

  const handleBotAuthenticate = async () => {
    try {
      setBotAuthenticating(true);

      const response = await fetch('/twitch/bot/auth/url');
      const data = await response.json();

      const authWindow = window.open(data.url, 'TwitchBotOAuth', 'width=600,height=800');

      if (!authWindow) {
        alert('Popup was blocked. Please allow popups for this site and try again.');
        setBotAuthenticating(false);
        return;
      }

      // Poll to check if authentication is complete
      const checkOAuth = setInterval(async () => {
        try {
          if (authWindow.closed) {
            clearInterval(checkOAuth);
            // Wait a moment for the backend to process
            await new Promise(resolve => setTimeout(resolve, 1000));
            await loadBotAuthStatus();
            setBotAuthenticating(false);
          }
        } catch (e) {
          console.error('Error checking bot OAuth:', e);
        }
      }, 1000);
    } catch (e) {
      console.error('Failed to authenticate bot:', e);
      alert(`Failed to authenticate bot: ${e.message}`);
      setBotAuthenticating(false);
    }
  };

  const handleBotRevoke = async () => {
    if (!confirm('Are you sure you want to disconnect your bot account?')) {
      return;
    }

    try {
      await fetch('/twitch/bot/auth/revoke', { method: 'POST' });
      await loadBotAuthStatus();
    } catch (e) {
      console.error('Failed to revoke bot auth:', e);
      alert(`Failed to disconnect bot: ${e.message}`);
    }
  };

  return (
    <div class="space-y-4">
      <div class="flex items-center justify-between">
        <h3 class="text-lg font-semibold">Broadcaster Authentication</h3>
      </div>

      <Show when={loading()}>
        <div class="text-center py-8">
          <span class="loading loading-spinner loading-md"></span>
        </div>
      </Show>

      <Show when={!loading()}>
        <Show when={!authStatus()?.authenticated}>
          {/* Not Authenticated */}
          <div class="text-center py-8 space-y-4">
            <div class="text-base-content/50">
              <IconBrandTwitch size={48} class="mx-auto mb-4 opacity-50" />
              <p>Connect your Twitch broadcaster account to get started</p>
            </div>
            <button
              onClick={handleAuthenticate}
              disabled={authenticating()}
              class="btn btn-primary btn-lg gap-2"
            >
              <IconBrandTwitch size={24} />
              {authenticating() ? 'Authenticating...' : 'Authenticate with Twitch'}
            </button>
          </div>
        </Show>

        <Show when={authStatus()?.authenticated}>
          {/* Authenticated */}
          <div class="card bg-primary/10 border-2 border-primary">
            <div class="card-body">
              <div class="flex items-center justify-between">
                <div class="flex items-center gap-3">
                  <div class="avatar">
                    <Show
                      when={authStatus().profile_image_url}
                      fallback={
                        <div class="placeholder">
                          <div class="bg-primary text-primary-content rounded-full w-12">
                            <IconUser size={24} />
                          </div>
                        </div>
                      }
                    >
                      <div class="w-12 rounded-full ring ring-primary ring-offset-base-100 ring-offset-2">
                        <img src={authStatus().profile_image_url} alt={authStatus().username} />
                      </div>
                    </Show>
                  </div>
                  <div>
                    <div class="font-semibold text-lg flex items-center gap-2">
                      {authStatus().username}
                      <span class="badge badge-success badge-sm gap-1">
                        <IconCheck size={12} />
                        Connected
                      </span>
                    </div>
                    <div class="text-sm text-base-content/70">
                      Channel: #{authStatus().username}
                    </div>
                  </div>
                </div>

                <button
                  onClick={handleRevoke}
                  class="btn btn-sm btn-ghost btn-error gap-2"
                >
                  <IconTrash size={16} />
                  Disconnect
                </button>
              </div>

              <Show when={authStatus().is_expired}>
                <div class="alert alert-warning mt-4">
                  <IconAlertCircle size={20} />
                  <span>Your access token has expired. Please re-authenticate.</span>
                </div>
              </Show>
            </div>
          </div>

          <div class="alert alert-info text-sm">
            <IconAlertCircle size={20} />
            <div>
              <strong>Next Steps:</strong>
              <ul class="list-disc list-inside mt-2 space-y-1">
                <li>Authenticate your bot account below to send chat messages</li>
                <li>Make sure the bot account has moderator permissions in your channel</li>
                <li>Your channel is automatically configured as: <strong>#{authStatus().username}</strong></li>
              </ul>
            </div>
          </div>
        </Show>
      </Show>

      {/* Bot Authentication Section */}
      <div class="divider my-6"></div>

      <div class="flex items-center justify-between">
        <h3 class="text-lg font-semibold">Bot Account Authentication (Optional)</h3>
      </div>

      <Show when={botLoading()}>
        <div class="text-center py-8">
          <span class="loading loading-spinner loading-md"></span>
        </div>
      </Show>

      <Show when={!botLoading()}>
        <Show when={!botAuthStatus()?.authenticated}>
          {/* Bot Not Authenticated */}
          <div class="text-center py-8 space-y-4">
            <div class="text-base-content/50">
              <IconBrandTwitch size={48} class="mx-auto mb-4 opacity-50" />
              <p>Connect a separate bot account to send chat messages</p>
              <p class="text-sm mt-2">If not configured, the broadcaster account will be used</p>
            </div>
            <button
              onClick={handleBotAuthenticate}
              disabled={botAuthenticating()}
              class="btn btn-secondary btn-lg gap-2"
            >
              <IconBrandTwitch size={24} />
              {botAuthenticating() ? 'Authenticating...' : 'Authenticate Bot Account'}
            </button>
          </div>
        </Show>

        <Show when={botAuthStatus()?.authenticated}>
          {/* Bot Authenticated */}
          <div class="card bg-secondary/10 border-2 border-secondary">
            <div class="card-body">
              <div class="flex items-center justify-between">
                <div class="flex items-center gap-3">
                  <div class="avatar">
                    <Show
                      when={botAuthStatus().profile_image_url}
                      fallback={
                        <div class="placeholder">
                          <div class="bg-secondary text-secondary-content rounded-full w-12">
                            <IconUser size={24} />
                          </div>
                        </div>
                      }
                    >
                      <div class="w-12 rounded-full ring ring-secondary ring-offset-base-100 ring-offset-2">
                        <img src={botAuthStatus().profile_image_url} alt={botAuthStatus().username} />
                      </div>
                    </Show>
                  </div>
                  <div>
                    <div class="font-semibold text-lg flex items-center gap-2">
                      {botAuthStatus().username}
                      <span class="badge badge-secondary badge-sm gap-1">
                        <IconCheck size={12} />
                        Bot Connected
                      </span>
                    </div>
                    <div class="text-sm text-base-content/70">
                      Bot Account
                    </div>
                  </div>
                </div>

                <button
                  onClick={handleBotRevoke}
                  class="btn btn-sm btn-ghost btn-error gap-2"
                >
                  <IconTrash size={16} />
                  Disconnect
                </button>
              </div>

              <Show when={botAuthStatus().is_expired}>
                <div class="alert alert-warning mt-4">
                  <IconAlertCircle size={20} />
                  <span>Your bot access token has expired. Please re-authenticate.</span>
                </div>
              </Show>
            </div>
          </div>

          <div class="alert alert-success text-sm">
            <IconCheck size={20} />
            <div>
              All chat messages will be sent from <strong>{botAuthStatus().username}</strong>.
              Make sure this account has moderator permissions in your channel!
            </div>
          </div>
        </Show>
      </Show>
    </div>
  );
}
