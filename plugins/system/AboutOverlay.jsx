import { Show } from 'solid-js';
import { IconX, IconBrandGithub, IconBrandDiscord, IconBrandTwitch, IconBrandYoutube, IconBrandTiktok, IconWorld, IconCode } from '@tabler/icons-solidjs';

export default function AboutOverlay({ isOpen, onClose }) {
  const handleOverlayClick = (e) => {
    if (e.target === e.currentTarget) {
      onClose();
    }
  };

  return (
    <Show when={isOpen()}>
      <div
        class="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-[200] animate-in fade-in duration-300"
        onClick={handleOverlayClick}
      >
        <div class="bg-base-200 rounded-2xl border border-base-300 shadow-2xl max-w-2xl w-full mx-4 animate-in zoom-in-95 duration-300">
          {/* Header */}
          <div class="flex items-center justify-between p-6 border-b border-base-300">
            <div class="flex items-center gap-4">
              <div class="w-12 h-12 bg-gradient-to-br from-primary to-secondary rounded-xl flex items-center justify-center">
                <IconWorld class="w-6 h-6 text-primary-content" />
              </div>
              <div>
                <h2 class="text-2xl font-bold text-base-content">WebArcade</h2>
                <p class="text-sm text-base-content/60">v1.0.0</p>
              </div>
            </div>
            <button
              onClick={onClose}
              class="w-8 h-8 flex items-center justify-center text-base-content/60 hover:text-base-content hover:bg-base-300 rounded-lg transition-colors"
            >
              <IconX class="w-4 h-4" />
            </button>
          </div>

          {/* Content */}
          <div class="p-6">
            <div class="mb-6">
              <h3 class="font-semibold text-base-content mb-3">About</h3>
              <p class="text-sm text-base-content/70 leading-relaxed">
                WebArcade is a plugin-based platform for building custom tools and integrations. Create your own
                plugins to connect to any API, control hardware, automate tasks, or build interfaces for anything
                you can imagine. Powered by a fast Rust backend and a modern web UI.
              </p>
            </div>

            {/* Technology Stack */}
            <div class="mb-6">
              <h3 class="font-semibold text-base-content mb-3">Technology Stack</h3>
              <div class="grid grid-cols-2 md:grid-cols-4 gap-3">
                {[
                  { name: 'SolidJS', desc: 'UI Framework' },
                  { name: 'Tauri', desc: 'Desktop App' },
                  { name: 'Rust', desc: 'Backend Bridge' },
                  { name: 'TailwindCSS', desc: 'Styling' },
                  { name: 'DaisyUI', desc: 'Components' },
                  { name: 'Rspack', desc: 'Bundler' },
                  { name: 'Bun', desc: 'Runtime' },
                  { name: 'SQLite', desc: 'Database' }
                ].map(tech => (
                  <div class="p-2 bg-gradient-to-br from-primary/10 to-secondary/10 rounded-lg border border-base-content/10">
                    <div class="font-medium text-xs text-base-content">{tech.name}</div>
                    <div class="text-xs text-base-content/50">{tech.desc}</div>
                  </div>
                ))}
              </div>
            </div>

            {/* Links */}
            <div class="mb-6">
              <h3 class="font-semibold text-base-content mb-3">Follow Developer</h3>
              <div class="flex flex-wrap gap-3">
                <a
                  href="https://github.com/pianoplayerjames/webarcade"
                  target="_blank"
                  rel="noopener noreferrer"
                  class="flex items-center gap-2 px-4 py-2 bg-base-300/50 hover:bg-base-300 rounded-lg transition-colors"
                >
                  <IconBrandGithub class="w-5 h-5" />
                  <span class="text-sm">GitHub</span>
                </a>
                <a
                  href="https://discord.gg/G9WBkSu6Ta"
                  target="_blank"
                  rel="noopener noreferrer"
                  class="flex items-center gap-2 px-4 py-2 bg-base-300/50 hover:bg-base-300 rounded-lg transition-colors"
                >
                  <IconBrandDiscord class="w-5 h-5" />
                  <span class="text-sm">Discord</span>
                </a>
                <a
                  href="https://twitch.tv/pianojames"
                  target="_blank"
                  rel="noopener noreferrer"
                  class="flex items-center gap-2 px-4 py-2 bg-base-300/50 hover:bg-base-300 rounded-lg transition-colors"
                >
                  <IconBrandTwitch class="w-5 h-5" />
                  <span class="text-sm">Twitch</span>
                </a>
                <a
                  href="https://youtube.com/@chessjames"
                  target="_blank"
                  rel="noopener noreferrer"
                  class="flex items-center gap-2 px-4 py-2 bg-base-300/50 hover:bg-base-300 rounded-lg transition-colors"
                >
                  <IconBrandYoutube class="w-5 h-5" />
                  <span class="text-sm">YouTube</span>
                </a>
                <a
                  href="https://tiktok.com/@pianoplayerjames"
                  target="_blank"
                  rel="noopener noreferrer"
                  class="flex items-center gap-2 px-4 py-2 bg-base-300/50 hover:bg-base-300 rounded-lg transition-colors"
                >
                  <IconBrandTiktok class="w-5 h-5" />
                  <span class="text-sm">TikTok</span>
                </a>
              </div>
            </div>

            {/* Footer */}
            <div class="pt-6 border-t border-base-300 text-center">
              <p class="text-xs text-base-content/50 mb-2">
                Built with passion for developers worldwide
              </p>
              <p class="text-xs text-base-content/40">
                Copyright © {new Date().getFullYear()} WebArcade. All rights reserved.
              </p>
            </div>
          </div>
        </div>
      </div>
    </Show>
  );
}
