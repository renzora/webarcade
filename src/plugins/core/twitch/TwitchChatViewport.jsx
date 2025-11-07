import { createSignal, onMount, onCleanup, For, Show, createEffect } from 'solid-js';
import twitchStore from './TwitchStore.jsx';
import { IconSend, IconHash, IconCircleFilled, IconUser, IconAlertCircle } from '@tabler/icons-solidjs';

export default function TwitchChatViewport() {
  const [messages, setMessages] = createSignal([]);
  const [messageInput, setMessageInput] = createSignal('');
  const [selectedChannel, setSelectedChannel] = createSignal('');
  const [status, setStatus] = createSignal({ status: 'disconnected', connected_channels: [] });

  let chatContainerRef;
  let shouldAutoScroll = true;

  onMount(async () => {
    const currentStatus = await twitchStore.fetchStatus();
    if (currentStatus) {
      setStatus({ ...currentStatus, connected_channels: currentStatus.connected_channels || [] });
      if (currentStatus.connected_channels && currentStatus.connected_channels.length > 0) {
        setSelectedChannel(currentStatus.connected_channels[0]);
      }
    }

    const handleChatMessage = (event) => {
      setMessages((prev) => [...prev, event].slice(-200));
    };

    const handleConnected = (event) => {
      setStatus((prev) => ({
        ...prev,
        status: 'connected',
        connected_channels: event.channels
      }));
      if (event.channels.length > 0 && !selectedChannel()) {
        setSelectedChannel(event.channels[0]);
      }
    };

    const handleDisconnected = () => {
      setStatus((prev) => ({
        ...prev,
        status: 'disconnected'
      }));
    };

    twitchStore.on('chat_message', handleChatMessage);
    twitchStore.on('connected', handleConnected);
    twitchStore.on('disconnected', handleDisconnected);

    onCleanup(() => {
      twitchStore.off('chat_message', handleChatMessage);
      twitchStore.off('connected', handleConnected);
      twitchStore.off('disconnected', handleDisconnected);
    });
  });

  createEffect(() => {
    if (messages().length > 0 && shouldAutoScroll && chatContainerRef) {
      chatContainerRef.scrollTop = chatContainerRef.scrollHeight;
    }
  });

  const handleScroll = () => {
    if (!chatContainerRef) return;
    const { scrollTop, scrollHeight, clientHeight } = chatContainerRef;
    shouldAutoScroll = scrollTop + clientHeight >= scrollHeight - 50;
  };

  const handleSendMessage = async () => {
    const message = messageInput().trim();
    if (!message || !selectedChannel()) return;

    try {
      await twitchStore.sendMessage(selectedChannel(), message);
      setMessageInput('');
    } catch (e) {
      // Show error in UI
    }
  };

  const handleKeyPress = (e) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSendMessage();
    }
  };

  const getBadgeColor = (badge) => {
    if (badge.startsWith('broadcaster')) return 'badge-error';
    if (badge.startsWith('moderator')) return 'badge-success';
    if (badge.startsWith('subscriber')) return 'badge-primary';
    if (badge.startsWith('vip')) return 'badge-secondary';
    return 'badge-ghost';
  };

  const formatTimestamp = (timestamp) => {
    const date = new Date(timestamp * 1000);
    return date.toLocaleTimeString('en-US', { hour: '2-digit', minute: '2-digit' });
  };

  const filteredMessages = () => {
    return messages().filter(m => !selectedChannel() || m.channel === selectedChannel());
  };

  return (
    <div class="h-full flex flex-col bg-base-200">
      {/* Header */}
      <div class="flex items-center justify-between bg-base-100 border-b border-base-300 px-3 py-2 gap-2">
        <div class="flex items-center gap-2 min-w-0 flex-1">
          <div class={`badge badge-xs ${status().status === 'connected' ? 'badge-success' : 'badge-error'} gap-1`}>
            <IconCircleFilled size={6} />
            <span class="text-[10px]">{status().status === 'connected' ? 'On' : 'Off'}</span>
          </div>

          <Show when={status().connected_channels?.length > 0}>
            <select
              class="select select-bordered select-xs min-w-0 flex-1 text-xs h-6"
              value={selectedChannel()}
              onChange={(e) => setSelectedChannel(e.target.value)}
            >
              {status().connected_channels?.map((channel) => (
                <option value={channel}>#{channel}</option>
              ))}
            </select>
          </Show>
        </div>

        <div class="badge badge-neutral badge-sm gap-1">
          <span class="text-[10px]">{filteredMessages().length}</span>
        </div>
      </div>

      {/* Chat Messages */}
      <div
        ref={chatContainerRef}
        onScroll={handleScroll}
        class="flex-1 overflow-y-auto p-2 space-y-1"
      >
        <Show
          when={filteredMessages().length > 0}
          fallback={
            <div class="flex items-center justify-center h-full">
              <div class="text-center p-4">
                <IconHash size={32} class="mx-auto mb-2 opacity-30" />
                <p class="text-sm font-semibold mb-1">No messages yet</p>
                <Show when={status().status === 'connected'} fallback={
                  <p class="text-xs text-base-content/60">Bot is not connected</p>
                }>
                  <p class="text-xs text-base-content/60">Waiting for chat...</p>
                </Show>
              </div>
            </div>
          }
        >
          <For each={filteredMessages()}>
            {(msg) => (
              <div class="flex gap-2 px-2 py-1 hover:bg-base-300/50 rounded">
                <div class="flex-shrink-0">
                  <div class="w-6 h-6 rounded-full bg-primary/20 flex items-center justify-center">
                    <span class="text-xs font-bold" style={{ color: msg.color || '#9146FF' }}>
                      {msg.username[0].toUpperCase()}
                    </span>
                  </div>
                </div>

                <div class="flex-1 min-w-0">
                  <div class="flex items-center gap-1.5 mb-0.5">
                    <span
                      class="font-semibold text-xs"
                      style={{ color: msg.color || '#9146FF' }}
                    >
                      {msg.username}
                    </span>

                    <Show when={msg.badges && msg.badges.length > 0}>
                      <For each={msg.badges.slice(0, 2)}>
                        {(badge) => (
                          <div class={`badge badge-xs h-3 px-1 text-[9px] ${getBadgeColor(badge)}`}>
                            {badge.split('/')[0]}
                          </div>
                        )}
                      </For>
                    </Show>

                    <time class="text-[10px] opacity-50 ml-auto">
                      {formatTimestamp(msg.timestamp)}
                    </time>
                  </div>

                  <div class="text-xs text-base-content break-words">
                    {msg.message}
                  </div>
                </div>
              </div>
            )}
          </For>
        </Show>
      </div>

      {/* Message Input */}
      <div class="p-2 bg-base-100 border-t border-base-300">
        <Show
          when={status().status === 'connected' && selectedChannel()}
          fallback={
            <div class="alert alert-warning alert-sm py-2">
              <IconAlertCircle size={16} />
              <span class="text-xs">Bot not connected</span>
            </div>
          }
        >
          <div class="flex gap-2">
            <input
              type="text"
              placeholder={`Message #${selectedChannel()}`}
              class="input input-bordered input-sm flex-1 text-xs h-8"
              value={messageInput()}
              onInput={(e) => setMessageInput(e.target.value)}
              onKeyPress={handleKeyPress}
            />
            <button
              class="btn btn-primary btn-sm gap-1 h-8 min-h-0"
              onClick={handleSendMessage}
              disabled={!messageInput().trim()}
            >
              <IconSend size={14} />
            </button>
          </div>
        </Show>
      </div>
    </div>
  );
}
