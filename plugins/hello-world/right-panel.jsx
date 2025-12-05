export default function RightPanel() {
    return (
        <div class="p-4">
            <h2 class="text-lg font-bold mb-4">Properties</h2>
            <div class="space-y-3">
                <div>
                    <label class="text-xs text-base-content/60">Name</label>
                    <input
                        type="text"
                        class="input input-sm input-bordered w-full"
                        value="Hello World"
                        readonly
                    />
                </div>
                <div>
                    <label class="text-xs text-base-content/60">Version</label>
                    <input
                        type="text"
                        class="input input-sm input-bordered w-full"
                        value="1.0.0"
                        readonly
                    />
                </div>
                <div>
                    <label class="text-xs text-base-content/60">Status</label>
                    <div class="badge badge-success">Active</div>
                </div>
            </div>
        </div>
    );
}
