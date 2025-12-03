import { createSignal } from 'solid-js';
import { IconCopy, IconCheck } from '@tabler/icons-solidjs';

function Code(props) {
  const [copied, setCopied] = createSignal(false);

  const copy = () => {
    navigator.clipboard.writeText(props.children);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div class="relative group my-4">
      <pre class="bg-base-300 rounded-lg p-4 overflow-x-auto text-sm font-mono text-base-content">
        <code>{props.children.trim()}</code>
      </pre>
      <button
        class="absolute top-2 right-2 btn btn-xs btn-ghost opacity-0 group-hover:opacity-100"
        onClick={copy}
      >
        {copied() ? <IconCheck class="w-3 h-3" /> : <IconCopy class="w-3 h-3" />}
      </button>
    </div>
  );
}

export default function GuideViewport() {
  return (
    <div class="h-full overflow-y-auto bg-base-200">
      <div class="max-w-3xl mx-auto p-8">
        <h1 class="text-3xl font-bold mb-6">Quick Start</h1>
        <p class="text-base-content/70 mb-6">Create a plugin in under a minute.</p>

        <div class="space-y-6">
          <div>
            <h2 class="text-lg font-semibold mb-2">1. Create Plugin</h2>
            <Code lang="bash">{`bun run plugin:new my-plugin`}</Code>
          </div>

          <div>
            <h2 class="text-lg font-semibold mb-2">2. Edit index.jsx</h2>
            <Code lang="jsx">{`import { createPlugin } from '@/api/plugin';

export default createPlugin({
  id: 'my-plugin',
  name: 'My Plugin',
  version: '1.0.0',

  async onStart(api) {
    api.viewport('main', {
      label: 'My Plugin',
      component: () => <div class="p-4">Hello World</div>
    });
    api.open('main');
  }
});`}</Code>
          </div>

          <div>
            <h2 class="text-lg font-semibold mb-2">3. Build & Run</h2>
            <Code lang="bash">{`bun run plugin:build my-plugin`}</Code>
          </div>
        </div>
      </div>
    </div>
  );
}