import { createSignal, onMount } from 'solid-js';

export default function TwitchChatWidget() {
  const [channel, setChannel] = createSignal('');
  const [darkTheme, setDarkTheme] = createSignal(true);

  onMount(() => {
    // Try to get channel from localStorage or config
    const savedChannel = localStorage.getItem('twitch-channel');
    if (savedChannel) {
      setChannel(savedChannel);
    } else {
      // Default channel - you can change this
      setChannel('piratesoftware');
    }
  });

  const getChatUrl = () => {
    const channelName = channel();
    if (!channelName) return '';

    return `https://www.twitch.tv/embed/${channelName}/chat?parent=${window.location.hostname}&darkpopout`;
  };

  return (
    <div class="card bg-base-100 shadow-lg h-full flex flex-col overflow-hidden">
      {/* Header */}
      <div class="card-body p-3 pb-2 flex-shrink-0 bg-gradient-to-br from-purple-500/10 to-purple-900/5">
        <div class="flex items-center justify-between">
          <h3 class="font-bold text-sm flex items-center gap-2">
            <svg class="w-4 h-4 text-purple-500" fill="currentColor" viewBox="0 0 24 24">
              <path d="M11.571 4.714h1.715v5.143H11.57zm4.715 0H18v5.143h-1.714zM6 0L1.714 4.286v15.428h5.143V24l4.286-4.286h3.428L22.286 12V0zm14.571 11.143l-3.428 3.428h-3.429l-3 3v-3H6.857V1.714h13.714Z"/>
            </svg>
            Twitch Chat
          </h3>
          <div class="flex items-center gap-2">
            <input
              type="text"
              placeholder="Channel name"
              class="input input-xs input-bordered w-32"
              value={channel()}
              onInput={(e) => {
                setChannel(e.target.value);
                localStorage.setItem('twitch-channel', e.target.value);
              }}
            />
          </div>
        </div>
      </div>

      {/* Twitch Chat Embed */}
      <div class="flex-1 relative overflow-hidden">
        {channel() ? (
          <iframe
            src={getChatUrl()}
            class="w-full border-0"
            style="height: 400px;"
            scrolling="yes"
            frameborder="0"
            allow="autoplay; fullscreen"
            title="Twitch Chat"
          />
        ) : (
          <div class="h-full flex items-center justify-center">
            <div class="text-center opacity-50">
              <p class="text-sm">Enter a channel name above</p>
              <p class="text-xs mt-1">to display Twitch chat</p>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
