import { createSignal } from 'solid-js';

export default function RightPanel() {
    const [position, setPosition] = createSignal({ x: 0, y: 0.5, z: 0 });
    const [rotation, setRotation] = createSignal({ x: 0, y: 0, z: 0 });
    const [scale, setScale] = createSignal({ x: 1, y: 1, z: 1 });
    const [color, setColor] = createSignal('#4a90d9');

    const InputField = (props) => (
        <div class="flex items-center gap-2">
            <span class="w-4 text-xs text-base-content/60">{props.label}</span>
            <input
                type="number"
                step={props.step || 0.1}
                value={props.value}
                onInput={(e) => props.onChange(parseFloat(e.target.value) || 0)}
                class="flex-1 bg-base-300 border border-base-300 rounded px-2 py-1 text-xs focus:border-primary focus:outline-none"
            />
        </div>
    );

    const Section = (props) => (
        <div class="border-b border-base-300">
            <div class="px-3 py-2 bg-base-300/50 text-xs font-medium text-base-content/80">
                {props.title}
            </div>
            <div class="p-3 space-y-2">
                {props.children}
            </div>
        </div>
    );

    return (
        <div class="h-full flex flex-col text-sm overflow-y-auto">
            {/* Header */}
            <div class="p-3 border-b border-base-300">
                <h2 class="font-semibold text-base-content">Properties</h2>
                <p class="text-xs text-base-content/60 mt-1">Cube</p>
            </div>

            {/* Transform Section */}
            <Section title="Transform">
                <div class="space-y-3">
                    <div>
                        <div class="text-xs text-base-content/60 mb-1.5">Position</div>
                        <div class="grid grid-cols-3 gap-2">
                            <InputField label="X" value={position().x} onChange={(v) => setPosition(p => ({...p, x: v}))} />
                            <InputField label="Y" value={position().y} onChange={(v) => setPosition(p => ({...p, y: v}))} />
                            <InputField label="Z" value={position().z} onChange={(v) => setPosition(p => ({...p, z: v}))} />
                        </div>
                    </div>

                    <div>
                        <div class="text-xs text-base-content/60 mb-1.5">Rotation</div>
                        <div class="grid grid-cols-3 gap-2">
                            <InputField label="X" value={rotation().x} onChange={(v) => setRotation(p => ({...p, x: v}))} step={1} />
                            <InputField label="Y" value={rotation().y} onChange={(v) => setRotation(p => ({...p, y: v}))} step={1} />
                            <InputField label="Z" value={rotation().z} onChange={(v) => setRotation(p => ({...p, z: v}))} step={1} />
                        </div>
                    </div>

                    <div>
                        <div class="text-xs text-base-content/60 mb-1.5">Scale</div>
                        <div class="grid grid-cols-3 gap-2">
                            <InputField label="X" value={scale().x} onChange={(v) => setScale(p => ({...p, x: v}))} />
                            <InputField label="Y" value={scale().y} onChange={(v) => setScale(p => ({...p, y: v}))} />
                            <InputField label="Z" value={scale().z} onChange={(v) => setScale(p => ({...p, z: v}))} />
                        </div>
                    </div>
                </div>
            </Section>

            {/* Material Section */}
            <Section title="Material">
                <div>
                    <div class="text-xs text-base-content/60 mb-1.5">Color</div>
                    <div class="flex items-center gap-2">
                        <input
                            type="color"
                            value={color()}
                            onInput={(e) => setColor(e.target.value)}
                            class="w-8 h-8 rounded cursor-pointer border border-base-300"
                        />
                        <input
                            type="text"
                            value={color()}
                            onInput={(e) => setColor(e.target.value)}
                            class="flex-1 bg-base-300 border border-base-300 rounded px-2 py-1 text-xs focus:border-primary focus:outline-none uppercase"
                        />
                    </div>
                </div>
            </Section>

            {/* Rendering Section */}
            <Section title="Rendering">
                <div class="space-y-2">
                    <label class="flex items-center gap-2 cursor-pointer">
                        <input type="checkbox" checked class="checkbox checkbox-sm checkbox-primary" />
                        <span class="text-xs">Visible</span>
                    </label>
                    <label class="flex items-center gap-2 cursor-pointer">
                        <input type="checkbox" checked class="checkbox checkbox-sm checkbox-primary" />
                        <span class="text-xs">Cast Shadows</span>
                    </label>
                    <label class="flex items-center gap-2 cursor-pointer">
                        <input type="checkbox" checked class="checkbox checkbox-sm checkbox-primary" />
                        <span class="text-xs">Receive Shadows</span>
                    </label>
                </div>
            </Section>
        </div>
    );
}
