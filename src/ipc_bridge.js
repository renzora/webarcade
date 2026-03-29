// WebArcade IPC Bridge
// Injected into the webview — provides window.__WEBARCADE__ API
(function() {
    'use strict';

    const pendingCalls = new Map();
    let callId = 0;

    const hasNativeIpc = typeof window.ipc !== 'undefined' && typeof window.ipc.postMessage === 'function';

    window.__WEBARCADE_IPC_CALLBACK__ = function(response) {
        const pending = pendingCalls.get(response.id);
        if (pending) {
            pendingCalls.delete(response.id);
            if (response.success) {
                pending.resolve(response.data);
            } else {
                pending.reject(new Error(response.error || 'IPC call failed'));
            }
        }
    };

    function ipcCall(command, args = {}) {
        return new Promise((resolve, reject) => {
            if (!hasNativeIpc) {
                if (command === 'isMaximized') return resolve(false);
                if (command === 'getSize') return resolve({ width: window.innerWidth, height: window.innerHeight });
                if (command === 'getPosition') return resolve({ x: 0, y: 0 });
                if (command === 'ping') return resolve('pong');
                return resolve(null);
            }

            const id = ++callId;
            pendingCalls.set(id, { resolve, reject });

            setTimeout(() => {
                if (pendingCalls.has(id)) {
                    pendingCalls.delete(id);
                    reject(new Error(`IPC call '${command}' timed out`));
                }
            }, 5000);

            try {
                window.ipc.postMessage(JSON.stringify({ id, command, args }));
            } catch (e) {
                pendingCalls.delete(id);
                reject(e);
            }
        });
    }

    function ipcCallSync(command, args = {}) {
        if (!hasNativeIpc) return null;
        try {
            window.ipc.postMessage(JSON.stringify({ id: ++callId, command, args }));
        } catch (e) {}
        return null;
    }

    document.addEventListener('mousedown', (e) => {
        if (!hasNativeIpc) return;
        const drag = e.target.closest('[data-drag-region]');
        if (drag && e.button === 0 && !e.target.closest('button, a, input, select, textarea, [role="button"]')) {
            e.preventDefault();
            ipcCallSync('startDrag');
        }
    });

    document.addEventListener('dblclick', (e) => {
        if (!hasNativeIpc) return;
        const drag = e.target.closest('[data-drag-region]');
        if (drag && !e.target.closest('button, a, input, select, textarea, [role="button"]')) {
            ipcCall('toggleMaximize');
        }
    });

    window.__WEBARCADE__ = {
        window: {
            close: () => ipcCall('close'),
            minimize: () => ipcCall('minimize'),
            maximize: () => ipcCall('maximize'),
            unmaximize: () => ipcCall('unmaximize'),
            toggleMaximize: () => ipcCall('toggleMaximize'),
            isMaximized: () => ipcCall('isMaximized'),
            setFullscreen: (enabled = true) => ipcCall('fullscreen', { enabled }),
            setSize: (width, height) => ipcCall('setSize', { width, height }),
            getSize: () => ipcCall('getSize'),
            setPosition: (x, y) => ipcCall('setPosition', { x, y }),
            getPosition: () => ipcCall('getPosition'),
            setMinSize: (width, height) => ipcCall('setMinSize', { width, height }),
            setMaxSize: (width, height) => ipcCall('setMaxSize', { width, height }),
            center: () => ipcCall('center'),
            setTitle: (title) => ipcCall('setTitle', { title }),
            startDrag: () => ipcCallSync('startDrag'),
        },
        isNative: hasNativeIpc,
    };
})();
