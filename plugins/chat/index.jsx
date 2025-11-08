export default function ChatPlugin(PluginAPI) {
  return {
    name: 'Chat',
    version: '1.0.0',
    description: 'Chat overlay for streams',

    onInit() {
      console.log('[Chat] Plugin initialized');
    },

    onStart() {
      console.log('[Chat] Plugin started');
    },

    onDispose() {
      console.log('[Chat] Plugin disposed');
    }
  };
}
