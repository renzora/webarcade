import { createSignal, createResource, Show } from "solid-js";
import Titlebar from "./components/Titlebar";

const fetchHello = async (name) => {
  const resp = await fetch(`/api/hello?name=${name}`);
  return resp.json();
};

export default function App() {
  const [name, setName] = createSignal("World");
  const [data] = createResource(name, fetchHello);

  return (
    <div class="flex flex-col h-screen bg-base-300 text-base-content">
      <Titlebar title="WebArcade Test" />

      <div class="flex-1 flex flex-col items-center justify-center gap-6 p-8">
        <h1 class="text-4xl font-bold">It works!</h1>

        <div class="form-control">
          <input
            type="text"
            class="input input-bordered"
            placeholder="Enter a name"
            value={name()}
            onInput={(e) => setName(e.target.value)}
          />
        </div>

        <Show when={!data.loading} fallback={<span class="loading loading-spinner" />}>
          <div class="mockup-code w-80">
            <pre><code>{JSON.stringify(data(), null, 2)}</code></pre>
          </div>
        </Show>

        <div class="flex gap-2">
          <button class="btn btn-primary" onClick={() => setName("WebArcade")}>
            Reset
          </button>
          <button class="btn btn-secondary" onClick={async () => {
            const resp = await fetch("/api/time");
            const d = await resp.json();
            alert(`Server time: ${d.timestamp}`);
          }}>
            Get Time
          </button>
        </div>
      </div>
    </div>
  );
}
