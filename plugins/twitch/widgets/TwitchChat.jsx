import { createSignal, createEffect, Show, For } from 'solid-js';
import { bridgeFetch } from '@/api/bridge';
import { IconSend, IconRefresh } from '@tabler/icons-solidjs';

export default function TwitchChat() {
  const [messages, setMessages] = createSignal([]);
  const [newMessage, setNewMessage] = createSignal('');
  const [channel, setChannel] = createSignal('');
  const [loading, setLoading] = createSignal(false);
  const [sending, setSending] = createSignal(false);

  const loadMessages = async () => {
    setLoading(true);

    try {
      const response = await bridgeFetch('/twitch/irc/messages');
      const data = await response.json();
      setMessages(data);

      // Get channel from accounts
      const accountsResponse = await bridgeFetch('/twitch/accounts');
      const accounts = await accountsResponse.json();
      const broadcaster = accounts.find(acc => acc.account_type === 'broadcaster');
      if (broadcaster) {
        setChannel(broadcaster.username);
      }
    } catch (e) {
      console.error('Failed to load messages:', e);
    } finally {
      setLoading(false);
    }
  };

  const sendMessage = async (e) => {
    e.preventDefault();

    const msg = newMessage().trim();
    if (!msg || !channel()) return;

    setSending(true);

    try {
      await bridgeFetch('/twitch/irc/send', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          channel: channel(),
          message: msg
        })
      });

      setNewMessage('');
      setTimeout(loadMessages, 1000); // Reload messages after sending
    } catch (e) {
      console.error('Failed to send message:', e);
    } finally {
      setSending(false);
    }
  };

  createEffect(() => {
    loadMessages();

    // Auto-refresh every 5 seconds
    const interval = setInterval(loadMessages, 5000);
    return () => clearInterval(interval);
  });

  const formatTimestamp = (timestamp) => {
    return new Date(timestamp * 1000).toLocaleTimeString();
  };

  return (
    <div class="p-4 space-y-4 h-full flex flex-col">
      <div class="flex items-center justify-between">
        <h2 class="text-lg font-bold">Twitch Chat</h2>
        <button
          class="btn btn-sm btn-ghost"
          onClick={loadMessages}
          disabled={loading()}
        >
          <IconRefresh class="w-4 h-4" />
        </button>
      </div>

      <Show when={channel()}>
        <div class="text-sm text-base-content/60">
          Channel: #{channel()}
        </div>
      </Show>

      {/* Messages */}
      <div class="flex-1 overflow-y-auto bg-base-100 rounded-lg p-3 space-y-2 min-h-0">
        <Show
          when={!loading()}
          fallback={
            <div class="flex justify-center p-8">
              <span class="loading loading-spinner loading-lg"></span>
            </div>
          }
        >
          <Show
            when={messages().length > 0}
            fallback={
              <p class="text-sm text-base-content/60 text-center p-4">
                No messages yet
              </p>
            }
          >
            <For each={messages()}>
              {(message) => (
                <div class="text-sm">
                  <span class="text-xs text-base-content/40">
                    {formatTimestamp(message.timestamp)}
                  </span>
                  {' '}
                  <span
                    class="font-bold"
                    style={{ color: message.color || '#ffffff' }}
                  >
                    {message.display_name || message.username}:
                  </span>
                  {' '}
                  <span>{message.message}</span>
                </div>
              )}
            </For>
          </Show>
        </Show>
      </div>

      {/* Send Message Form */}
      <form onSubmit={sendMessage} class="flex gap-2">
        <input
          type="text"
          class="input input-bordered flex-1"
          placeholder="Type a message..."
          value={newMessage()}
          onInput={(e) => setNewMessage(e.target.value)}
          disabled={sending() || !channel()}
        />
        <button
          type="submit"
          class="btn btn-primary"
          disabled={sending() || !newMessage().trim() || !channel()}
        >
          <IconSend class="w-4 h-4" />
        </button>
      </form>
    </div>
  );
}
