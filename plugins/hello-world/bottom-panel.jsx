import { createSignal, For } from 'solid-js';

export default function BottomPanel() {
    const [logs, setLogs] = createSignal([
        { time: '10:00:00', message: 'Plugin started' },
        { time: '10:00:01', message: 'Viewport registered' },
        { time: '10:00:02', message: 'All panels loaded' },
    ]);

    return (
        <div class="h-full flex flex-col">
            <div class="flex-1 overflow-auto p-2 font-mono text-xs">
                <For each={logs()}>
                    {(log) => (
                        <div class="py-0.5">
                            <span class="text-base-content/40">[{log.time}]</span>
                            <span class="ml-2">{log.message}</span>
                        </div>
                    )}
                </For>
            </div>
        </div>
    );
}
