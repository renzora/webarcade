import { greetings, setGreetings, setGreetingCount } from './viewport';

export default function LeftPanel() {
    const sendToGreeter = () => {
        // Emit event that greeter listens to
        document.dispatchEvent(new CustomEvent('hello-world:message', {
            detail: {
                from: 'Hello World',
                message: `Greetings at ${new Date().toLocaleTimeString()}!`
            }
        }));
    };

    const clearGreetings = () => {
        setGreetings([]);
        setGreetingCount(0);
    };

    return (
        <div class="p-4">
            <h2 class="text-lg font-bold mb-4">Explorer</h2>
            <div class="space-y-2">
                <button
                    class="btn btn-primary btn-sm w-full"
                    onClick={sendToGreeter}
                >
                    Send to Greeter
                </button>
                <button
                    class="btn btn-outline btn-sm w-full"
                    onClick={clearGreetings}
                >
                    Clear Greetings
                </button>
            </div>
            <div class="mt-4 p-3 bg-base-300 rounded">
                <p class="text-xs text-base-content/60">
                    This plugin communicates with Greeter using custom events.
                </p>
            </div>
        </div>
    );
}
