import { For } from 'solid-js';
import { footerButtons } from '@/api/plugin';

const Footer = () => {
  return (
    <div class="fixed bottom-0 left-0 right-0 h-6 bg-base-200 backdrop-blur-md border-t border-base-content/10 text-xs flex items-center justify-end px-3 pointer-events-auto z-50 rounded-t-none">
      {/* Plugin footer buttons */}
      <div class="flex items-center gap-4">
        <For each={Array.from(footerButtons().entries())}>
          {([_id, button]) => {
            const Component = button.component;
            return Component ? <Component /> : null;
          }}
        </For>
      </div>
    </div>
  );
};

export default Footer;
