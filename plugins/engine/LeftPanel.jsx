import { createSignal, For } from 'solid-js';
import { IconCube, IconSphere, IconCylinder, IconCone, IconPlus, IconTrash } from '@tabler/icons-solidjs';

export default function LeftPanel() {
    const [objects, setObjects] = createSignal([
        { id: 1, name: 'Cube', type: 'box', icon: IconCube },
        { id: 2, name: 'Ground', type: 'ground', icon: IconCube }
    ]);

    const [selectedId, setSelectedId] = createSignal(1);

    const primitives = [
        { type: 'box', name: 'Cube', icon: IconCube },
        { type: 'sphere', name: 'Sphere', icon: IconSphere },
        { type: 'cylinder', name: 'Cylinder', icon: IconCylinder },
        { type: 'cone', name: 'Cone', icon: IconCone }
    ];

    const addObject = (primitive) => {
        const newId = Date.now();
        setObjects(prev => [...prev, {
            id: newId,
            name: `${primitive.name} ${objects().length + 1}`,
            type: primitive.type,
            icon: primitive.icon
        }]);
        setSelectedId(newId);
    };

    const deleteObject = (id) => {
        setObjects(prev => prev.filter(obj => obj.id !== id));
        if (selectedId() === id) {
            setSelectedId(objects()[0]?.id || null);
        }
    };

    return (
        <div class="h-full flex flex-col text-sm">
            {/* Header */}
            <div class="p-3 border-b border-base-300">
                <h2 class="font-semibold text-base-content">Scene Hierarchy</h2>
            </div>

            {/* Add Primitives */}
            <div class="p-2 border-b border-base-300">
                <div class="text-xs text-base-content/60 mb-2 px-1">Add Primitive</div>
                <div class="flex gap-1 flex-wrap">
                    <For each={primitives}>
                        {(prim) => (
                            <button
                                onClick={() => addObject(prim)}
                                class="p-2 rounded hover:bg-base-300 text-base-content/70 hover:text-base-content transition-colors"
                                title={`Add ${prim.name}`}
                            >
                                <prim.icon size={18} />
                            </button>
                        )}
                    </For>
                </div>
            </div>

            {/* Object List */}
            <div class="flex-1 overflow-y-auto p-2">
                <For each={objects()}>
                    {(obj) => (
                        <div
                            class={`flex items-center gap-2 px-2 py-1.5 rounded cursor-pointer group ${
                                selectedId() === obj.id
                                    ? 'bg-primary/20 text-primary'
                                    : 'hover:bg-base-300 text-base-content'
                            }`}
                            onClick={() => setSelectedId(obj.id)}
                        >
                            <obj.icon size={16} class="flex-shrink-0" />
                            <span class="flex-1 truncate">{obj.name}</span>
                            <button
                                onClick={(e) => {
                                    e.stopPropagation();
                                    deleteObject(obj.id);
                                }}
                                class="p-1 rounded opacity-0 group-hover:opacity-100 hover:bg-error/20 hover:text-error transition-all"
                                title="Delete"
                            >
                                <IconTrash size={14} />
                            </button>
                        </div>
                    )}
                </For>
            </div>

            {/* Footer */}
            <div class="p-2 border-t border-base-300 text-xs text-base-content/50">
                {objects().length} objects in scene
            </div>
        </div>
    );
}
