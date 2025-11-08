import { Show, For, createSignal, onMount } from 'solid-js';
import { IconBrandYoutube, IconReload } from '@tabler/icons-solidjs';
import youtubeStore from './YouTubeStore.jsx';

export default function YouTubeChannelsPanel() {
  onMount(() => {
    if (youtubeStore.authenticated && youtubeStore.channels.length === 0) {
      youtubeStore.fetchChannels();
    }
  });

  const formatNumber = (num) => {
    if (!num) return '0';
    if (num >= 1000000) return (num / 1000000).toFixed(1) + 'M';
    if (num >= 1000) return (num / 1000).toFixed(1) + 'K';
    return num.toString();
  };

  return (
    <div class="flex flex-col h-full">
      <div class="p-4 border-b flex items-center justify-between">
        <div class="flex items-center gap-2">
          <IconBrandYoutube size={20} class="text-red-500" />
          <h2 class="font-semibold">YouTube Channels</h2>
        </div>
        <button
          onClick={() => youtubeStore.fetchChannels()}
          disabled={youtubeStore.loading}
          class="p-1 hover:bg-gray-100 rounded"
        >
          <IconReload size={16} class={youtubeStore.loading ? 'animate-spin' : ''} />
        </button>
      </div>

      <div class="flex-1 overflow-y-auto">
        <Show
          when={youtubeStore.authenticated}
          fallback={
            <div class="p-4 text-center text-gray-500">
              <IconBrandYoutube size={48} class="mx-auto mb-2 opacity-50" />
              <p class="text-sm">Connect your YouTube account in settings</p>
            </div>
          }
        >
          <Show
            when={youtubeStore.channels.length > 0}
            fallback={
              <div class="p-4 text-center text-gray-500">
                <p class="text-sm">
                  {youtubeStore.loading ? 'Loading channels...' : 'No channels found'}
                </p>
              </div>
            }
          >
            <For each={youtubeStore.channels}>
              {(channel) => (
                <div
                  class="p-3 border-b hover:bg-gray-50 cursor-pointer"
                  classList={{ 'bg-blue-50': youtubeStore.selectedChannel === channel.id }}
                  onClick={() => youtubeStore.selectChannel(channel.id)}
                >
                  <div class="flex items-start gap-3">
                    <Show
                      when={channel.thumbnail_url}
                      fallback={
                        <div class="w-12 h-12 bg-gray-200 rounded-full flex items-center justify-center">
                          <IconBrandYoutube size={24} class="text-gray-400" />
                        </div>
                      }
                    >
                      <img
                        src={channel.thumbnail_url}
                        alt={channel.title}
                        class="w-12 h-12 rounded-full"
                      />
                    </Show>
                    <div class="flex-1 min-w-0">
                      <h3 class="font-medium text-sm truncate">{channel.title}</h3>
                      <Show when={channel.custom_url}>
                        <p class="text-xs text-gray-500">{channel.custom_url}</p>
                      </Show>
                      <div class="mt-1 flex gap-3 text-xs text-gray-600">
                        <span>{formatNumber(channel.subscriber_count)} subs</span>
                        <span>{formatNumber(channel.video_count)} videos</span>
                      </div>
                    </div>
                  </div>
                </div>
              )}
            </For>
          </Show>
        </Show>
      </div>
    </div>
  );
}
