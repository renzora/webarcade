import { createSignal, onMount, For, Show } from 'solid-js';
import twitchStore from './TwitchStore.jsx';
import { IconUserPlus, IconCheck, IconTrash, IconRobot, IconBroadcast } from '@tabler/icons-solidjs';

export default function TwitchAccountManager() {
  const [accounts, setAccounts] = createSignal([]);
  const [loading, setLoading] = createSignal(true);
  const [addingAccount, setAddingAccount] = createSignal(false);
  const [accountType, setAccountType] = createSignal('broadcaster'); // 'bot' or 'broadcaster'

  onMount(async () => {
    await loadAccounts();
  });

  const loadAccounts = async () => {
    try {
      console.log('[AccountManager] Loading accounts...');
      const accountList = await twitchStore.getAccounts();
      console.log('[AccountManager] Received accounts:', accountList);
      setAccounts(accountList);
    } catch (e) {
      console.error('[AccountManager] Failed to load accounts:', e);
    } finally {
      setLoading(false);
    }
  };

  const handleAddAccount = async () => {
    try {
      setAddingAccount(true);

      // Store account type in localStorage for the callback to use
      localStorage.setItem('twitch_account_type', accountType());
      localStorage.setItem('twitch_multi_account_mode', 'true');

      const url = await twitchStore.getAuthUrl();

      const authWindow = window.open(url, 'TwitchOAuth', 'width=600,height=800');

      if (!authWindow) {
        alert('Popup was blocked. Please allow popups for this site and try again.');
        localStorage.removeItem('twitch_account_type');
        localStorage.removeItem('twitch_multi_account_mode');
        setAddingAccount(false);
        return;
      }

      // Poll to check if authentication is complete
      const checkOAuth = setInterval(async () => {
        try {
          if (authWindow.closed) {
            clearInterval(checkOAuth);
            // Clean up localStorage
            localStorage.removeItem('twitch_account_type');
            localStorage.removeItem('twitch_multi_account_mode');
            // Wait a moment for the backend to process
            await new Promise(resolve => setTimeout(resolve, 1000));
            await loadAccounts();
            setAddingAccount(false);
          }
        } catch (e) {
          console.error('Error checking OAuth:', e);
        }
      }, 1000);
    } catch (e) {
      console.error('Failed to add account:', e);
      alert(`Failed to add account: ${e.message}`);
      localStorage.removeItem('twitch_account_type');
      localStorage.removeItem('twitch_multi_account_mode');
      setAddingAccount(false);
    }
  };

  const handleActivateAccount = async (accountId) => {
    try {
      await twitchStore.activateAccount(accountId);
      await loadAccounts();
    } catch (e) {
      console.error('Failed to activate account:', e);
      alert(`Failed to activate account: ${e.message}`);
    }
  };

  const handleDeleteAccount = async (accountId, username) => {
    if (!confirm(`Are you sure you want to remove account "${username}"?`)) {
      return;
    }

    try {
      await twitchStore.deleteAccount(accountId);
      await loadAccounts();
    } catch (e) {
      console.error('Failed to delete account:', e);
      alert(`Failed to delete account: ${e.message}`);
    }
  };

  return (
    <div class="space-y-4">
      <div class="flex items-center justify-between">
        <h3 class="text-lg font-semibold text-white">Authenticated Accounts</h3>
        <button
          onClick={handleAddAccount}
          disabled={addingAccount()}
          class="btn btn-sm btn-primary gap-2"
        >
          <IconUserPlus size={16} />
          {addingAccount() ? 'Authenticating...' : 'Add Account'}
        </button>
      </div>

      {/* Account Type Selector (for next auth) */}
      <div class="flex items-center gap-4 p-3 bg-base-200 rounded-lg">
        <span class="text-sm text-base-content/70">Next account type:</span>
        <div class="flex gap-2">
          <button
            onClick={() => setAccountType('bot')}
            class={`btn btn-sm gap-2 ${accountType() === 'bot' ? 'btn-primary' : 'btn-ghost'}`}
          >
            <IconRobot size={16} />
            Bot
          </button>
          <button
            onClick={() => setAccountType('broadcaster')}
            class={`btn btn-sm gap-2 ${accountType() === 'broadcaster' ? 'btn-primary' : 'btn-ghost'}`}
          >
            <IconBroadcast size={16} />
            Broadcaster
          </button>
        </div>
      </div>

      <Show when={loading()}>
        <div class="text-center py-8">
          <span class="loading loading-spinner loading-md"></span>
        </div>
      </Show>

      <Show when={!loading() && accounts().length === 0}>
        <div class="text-center py-8 text-base-content/50">
          No accounts connected. Add an account to get started.
        </div>
      </Show>

      <Show when={!loading() && accounts().length > 0}>
        <div class="space-y-2">
          <For each={accounts()}>
            {(account) => (
              <div class={`flex items-center justify-between p-4 rounded-lg border-2 ${
                account.is_active
                  ? 'bg-primary/10 border-primary'
                  : 'bg-base-200 border-base-300'
              }`}>
                <div class="flex items-center gap-3">
                  {account.account_type === 'bot' ? (
                    <IconRobot size={24} class="text-primary" />
                  ) : (
                    <IconBroadcast size={24} class="text-secondary" />
                  )}
                  <div>
                    <div class="font-semibold text-white">
                      {account.display_name || account.username}
                      {account.is_active && (
                        <span class="ml-2 badge badge-success badge-sm gap-1">
                          <IconCheck size={12} />
                          Active
                        </span>
                      )}
                    </div>
                    <div class="text-sm text-base-content/50">
                      @{account.username} • {account.account_type}
                    </div>
                  </div>
                </div>

                <div class="flex gap-2">
                  <Show when={!account.is_active}>
                    <button
                      onClick={() => handleActivateAccount(account.id)}
                      class="btn btn-sm btn-ghost"
                    >
                      Activate
                    </button>
                  </Show>
                  <button
                    onClick={() => handleDeleteAccount(account.id, account.username)}
                    class="btn btn-sm btn-ghost btn-error"
                  >
                    <IconTrash size={16} />
                  </button>
                </div>
              </div>
            )}
          </For>
        </div>
      </Show>

      <div class="alert alert-info text-sm">
        <div>
          <strong>How it works:</strong>
          <ul class="list-disc list-inside mt-2 space-y-1">
            <li><strong>Bot Account:</strong> Used for chat commands and basic bot functions</li>
            <li><strong>Broadcaster Account:</strong> Required for stream title, category, and channel settings</li>
            <li>Both accounts use the same Twitch application (Client ID/Secret)</li>
            <li>Only one account is active at a time - switch as needed</li>
          </ul>
        </div>
      </div>
    </div>
  );
}
