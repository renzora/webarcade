export default function Titlebar(props) {
  const win = window.__WEBARCADE__?.window;

  return (
    <div
      class="h-9 bg-base-200 flex items-center justify-between px-3 select-none"
      data-drag-region
    >
      <span class="text-sm opacity-70">{props.title}</span>

      <div class="flex gap-1">
        <button class="btn btn-ghost btn-xs" onClick={() => win?.minimize()}>
          &#x2014;
        </button>
        <button class="btn btn-ghost btn-xs" onClick={() => win?.toggleMaximize()}>
          &#x25A1;
        </button>
        <button class="btn btn-ghost btn-xs hover:btn-error" onClick={() => win?.close()}>
          &#x2715;
        </button>
      </div>
    </div>
  );
}
