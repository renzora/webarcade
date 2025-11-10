import { createSignal } from 'solid-js';
import { IconTestPipe, IconBrandTwitch, IconSparkles } from '@tabler/icons-solidjs';

export default function TwitchTest() {
    const [loading, setLoading] = createSignal(false);
    const [message, setMessage] = createSignal('');
    const [messageType, setMessageType] = createSignal('success');

    const testIRC = async (type) => {
        setLoading(true);
        setMessage('');

        try {
            const response = await bridgeFetch('/twitch/test/irc', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ type })
            });

            const result = await response.json();

            if (result.success) {
                setMessage(`✓ IRC test "${type}" completed successfully`);
                setMessageType('success');
            } else {
                setMessage(`✗ IRC test failed: ${result.error || 'Unknown error'}`);
                setMessageType('error');
            }
        } catch (err) {
            setMessage(`✗ Error: ${err.message}`);
            setMessageType('error');
        } finally {
            setLoading(false);
        }
    };

    const testEventSub = async (type) => {
        setLoading(true);
        setMessage('');

        try {
            const response = await bridgeFetch('/twitch/test/eventsub', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ type })
            });

            const result = await response.json();

            if (result.success) {
                setMessage(`✓ EventSub test "${type}" completed successfully`);
                setMessageType('success');
            } else {
                setMessage(`✗ EventSub test failed: ${result.error || 'Unknown error'}`);
                setMessageType('error');
            }
        } catch (err) {
            setMessage(`✗ Error: ${err.message}`);
            setMessageType('error');
        } finally {
            setLoading(false);
        }
    };

    return (
        <div class="bg-base-200 rounded-lg shadow-lg p-4 h-full">
            <div class="flex items-center gap-2 mb-4">
                <IconTestPipe class="w-5 h-5" />
                <h3 class="text-lg font-semibold">Twitch Test Suite</h3>
            </div>

            {/* IRC Tests */}
            <div class="mb-4">
                <div class="flex items-center gap-2 mb-2">
                    <IconBrandTwitch class="w-4 h-4" />
                    <h4 class="font-medium">IRC Tests</h4>
                </div>
                <div class="flex flex-wrap gap-2">
                    <button
                        class="btn btn-sm btn-outline"
                        onClick={() => testIRC('message')}
                        disabled={loading()}
                    >
                        Test Message
                    </button>
                    <button
                        class="btn btn-sm btn-outline"
                        onClick={() => testIRC('bulk_messages')}
                        disabled={loading()}
                    >
                        Bulk Messages (5)
                    </button>
                    <button
                        class="btn btn-sm btn-outline"
                        onClick={() => testIRC('connection')}
                        disabled={loading()}
                    >
                        Check Connection
                    </button>
                </div>
            </div>

            {/* EventSub Tests */}
            <div class="mb-4">
                <div class="flex items-center gap-2 mb-2">
                    <IconSparkles class="w-4 h-4" />
                    <h4 class="font-medium">EventSub Tests</h4>
                </div>
                <div class="flex flex-wrap gap-2">
                    <button
                        class="btn btn-sm btn-outline"
                        onClick={() => testEventSub('follow')}
                        disabled={loading()}
                    >
                        Follow Event
                    </button>
                    <button
                        class="btn btn-sm btn-outline"
                        onClick={() => testEventSub('subscribe')}
                        disabled={loading()}
                    >
                        Subscribe Event
                    </button>
                    <button
                        class="btn btn-sm btn-outline"
                        onClick={() => testEventSub('cheer')}
                        disabled={loading()}
                    >
                        Cheer Event
                    </button>
                    <button
                        class="btn btn-sm btn-outline"
                        onClick={() => testEventSub('raid')}
                        disabled={loading()}
                    >
                        Raid Event
                    </button>
                    <button
                        class="btn btn-sm btn-outline"
                        onClick={() => testEventSub('bulk_events')}
                        disabled={loading()}
                    >
                        Bulk Events (5)
                    </button>
                </div>
            </div>

            {/* Status Message */}
            {message() && (
                <div
                    class={`alert ${messageType() === 'success' ? 'alert-success' : 'alert-error'}`}
                >
                    <span>{message()}</span>
                </div>
            )}

            {loading() && (
                <div class="mt-4 text-sm opacity-70">
                    <span class="loading loading-spinner loading-sm mr-2"></span>
                    Running test...
                </div>
            )}
        </div>
    );
}
