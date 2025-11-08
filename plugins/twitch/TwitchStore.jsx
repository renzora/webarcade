import { createRoot, createSignal } from 'solid-js';

const BRIDGE_URL = ''; // Use relative URLs for proxy compatibility
const WS_URL = 'ws://localhost:3002';

/**
 * SolidJS store for Twitch integration with single WebSocket connection
 */
function createTwitchStore() {
  // Reactive signals
  const [chatMessages, setChatMessages] = createSignal([]);
  const [botStatus, setBotStatus] = createSignal({ status: 'disconnected' });
  const [config, setConfig] = createSignal({});
  const [commands, setCommands] = createSignal([]);
  const [wsConnected, setWsConnected] = createSignal(false);

  // WebSocket instance
  let ws = null;
  let shouldReconnect = true;
  let reconnectTimeout = null;

  // Event handlers map
  const eventHandlers = new Map();

  /**
   * Initialize WebSocket connection
   */
  const connect = () => {
    // Prevent duplicate connections
    if (ws?.readyState === WebSocket.OPEN || ws?.readyState === WebSocket.CONNECTING) {
      console.log('[TwitchStore] Already connected or connecting');
      return Promise.resolve();
    }

    // Clear any pending reconnect
    if (reconnectTimeout) {
      clearTimeout(reconnectTimeout);
      reconnectTimeout = null;
    }

    return new Promise((resolve, reject) => {
      try {
        // Close existing connection if any
        if (ws) {
          ws.close();
          ws = null;
        }

        ws = new WebSocket(WS_URL);

        ws.onopen = () => {
          console.log('[TwitchStore] WebSocket connected');
          setWsConnected(true);
          resolve();
        };

        ws.onmessage = (event) => {
          try {
            const data = JSON.parse(event.data);
            handleWebSocketMessage(data);
          } catch (e) {
            console.error('[TwitchStore] Failed to parse WebSocket message:', e);
          }
        };

        ws.onerror = (error) => {
          console.error('[TwitchStore] WebSocket error:', error);
          reject(error);
        };

        ws.onclose = () => {
          console.log('[TwitchStore] WebSocket disconnected');
          setWsConnected(false);
          ws = null;

          // Auto-reconnect if not explicitly disconnected
          if (shouldReconnect) {
            console.log('[TwitchStore] Reconnecting in 5 seconds...');
            reconnectTimeout = setTimeout(() => {
              connect();
            }, 5000);
          }
        };
      } catch (e) {
        reject(e);
      }
    });
  };

  /**
   * Disconnect WebSocket and prevent auto-reconnect
   */
  const disconnect = () => {
    shouldReconnect = false;

    if (reconnectTimeout) {
      clearTimeout(reconnectTimeout);
      reconnectTimeout = null;
    }

    if (ws) {
      ws.close();
      ws = null;
    }

    setWsConnected(false);
  };

  /**
   * Handle incoming WebSocket messages
   */
  const handleWebSocketMessage = (data) => {
    if (data.type === 'twitch_event') {
      handleTwitchEvent(data.event);
    } else if (data.type === 'file_change') {
      // Ignore file changes for now
    } else if (data.type === 'connected') {
      console.log('[TwitchStore] Connected to bridge');
    }
  };

  /**
   * Handle Twitch events from backend
   */
  const handleTwitchEvent = (event) => {
    switch (event.type) {
      case 'chat_message':
        // Add message to chat
        setChatMessages((prev) => [...prev, event].slice(-100)); // Keep last 100 messages
        emit('chat_message', event);
        break;

      case 'connected':
        console.log('[TwitchStore] Bot connected to channels:', event.channels);
        emit('connected', event);
        fetchStatus();
        break;

      case 'disconnected':
        console.log('[TwitchStore] Bot disconnected:', event.reason);
        emit('disconnected', event);
        fetchStatus();
        break;

      case 'user_joined':
      case 'user_left':
      case 'channel_joined':
      case 'channel_parted':
      case 'notice':
      case 'error':
        emit(event.type, event);
        break;

      default:
        console.log('[TwitchStore] Unknown event type:', event.type);
    }
  };

  /**
   * Event emitter
   */
  const on = (eventName, handler) => {
    if (!eventHandlers.has(eventName)) {
      eventHandlers.set(eventName, []);
    }
    eventHandlers.get(eventName).push(handler);
  };

  const off = (eventName, handler) => {
    if (eventHandlers.has(eventName)) {
      const handlers = eventHandlers.get(eventName);
      const index = handlers.indexOf(handler);
      if (index !== -1) {
        handlers.splice(index, 1);
      }
    }
  };

  const emit = (eventName, data) => {
    if (eventHandlers.has(eventName)) {
      eventHandlers.get(eventName).forEach((handler) => {
        try {
          handler(data);
        } catch (e) {
          console.error(`[TwitchStore] Error in event handler for ${eventName}:`, e);
        }
      });
    }
  };

  /**
   * API Methods
   */

  const startBot = async () => {
    // Bot start functionality would be implemented when IRC client is ready
    console.log('[TwitchStore] Bot start not yet implemented');
    return { success: false, message: 'Bot functionality not yet implemented' };
  };

  const stopBot = async () => {
    // Bot stop functionality would be implemented when IRC client is ready
    console.log('[TwitchStore] Bot stop not yet implemented');
    return { success: false, message: 'Bot functionality not yet implemented' };
  };

  const fetchStatus = async () => {
    try {
      const response = await fetch(`${BRIDGE_URL}/twitch/auth/status`);
      if (!response.ok) {
        return { authenticated: false };
      }
      const status = await response.json();
      setBotStatus(status);
      return status;
    } catch (e) {
      console.error('[TwitchStore] Failed to fetch status:', e);
      return { authenticated: false };
    }
  };

  const getAuthUrl = async () => {
    const response = await fetch(`${BRIDGE_URL}/twitch/auth/url`);

    if (!response.ok) {
      const error = await response.text();
      throw new Error(error || 'Failed to get auth URL');
    }

    const data = await response.json();

    if (!data.url) {
      throw new Error('No auth URL returned from server');
    }

    return data.url;
  };

  const completeOAuth = async (code, state) => {
    const response = await fetch(`${BRIDGE_URL}/twitch/auth/callback?code=${code}&state=${state}`, {
      method: 'GET',
    });

    if (!response.ok) {
      const error = await response.text();
      throw new Error(error);
    }

    return await response.json();
  };

  const sendMessage = async (channel, message) => {
    const response = await fetch(`${BRIDGE_URL}/twitch/messages/send`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ channel, message }),
    });

    if (!response.ok) {
      const error = await response.text();
      throw new Error(error);
    }

    return await response.json();
  };

  const fetchConfig = async () => {
    try {
      const response = await fetch(`${BRIDGE_URL}/twitch/config`);
      const configData = await response.json();
      setConfig(configData);
      return configData;
    } catch (e) {
      console.error('[TwitchStore] Failed to fetch config:', e);
      return null;
    }
  };

  const saveConfig = async (configData) => {
    const response = await fetch(`${BRIDGE_URL}/twitch/config`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(configData),
    });

    if (!response.ok) {
      const error = await response.text();
      throw new Error(error);
    }

    await fetchConfig();
    return await response.json();
  };

  const fetchCommands = async () => {
    try {
      const response = await fetch(`${BRIDGE_URL}/twitch/commands`);
      if (!response.ok) {
        return { commands: [] };
      }
      const commandsData = await response.json();
      setCommands(commandsData.commands || []);
      return commandsData.commands || [];
    } catch (e) {
      console.error('[TwitchStore] Failed to fetch commands:', e);
      return [];
    }
  };

  const registerCommand = async (command) => {
    const response = await fetch(`${BRIDGE_URL}/twitch/commands/register`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(command),
    });

    if (!response.ok) {
      const error = await response.text();
      throw new Error(error);
    }

    await fetchCommands();
    return await response.json();
  };

  const unregisterCommand = async (name) => {
    // This endpoint doesn't exist yet in the backend
    console.log('[TwitchStore] Unregister command not yet implemented');
    return { success: false, message: 'Not implemented' };
  };

  const joinChannel = async (channel) => {
    // For now, just update the channels in config
    const currentConfig = await fetchConfig();
    const channels = currentConfig.channels || [];
    if (!channels.includes(channel)) {
      channels.push(channel);
      return await saveConfig({ ...currentConfig, channels });
    }
    return { success: true };
  };

  const partChannel = async (channel) => {
    // For now, just update the channels in config
    const currentConfig = await fetchConfig();
    const channels = (currentConfig.channels || []).filter(ch => ch !== channel);
    return await saveConfig({ ...currentConfig, channels });
  };

  const revokeToken = async () => {
    const response = await fetch(`${BRIDGE_URL}/twitch/auth/revoke`, {
      method: 'POST',
    });

    if (!response.ok) {
      const error = await response.text();
      throw new Error(error);
    }

    await fetchConfig();
    return await response.json();
  };

  const getStreamInfo = async () => {
    // Stream info endpoints not yet implemented in backend
    console.log('[TwitchStore] Stream info not yet implemented');
    return { stream: null };
  };

  const updateStreamInfo = async (options) => {
    // Stream info endpoints not yet implemented in backend
    console.log('[TwitchStore] Update stream info not yet implemented');
    return { success: false, message: 'Not implemented' };
  };

  const searchGames = async (query) => {
    // Game search not yet implemented in backend
    console.log('[TwitchStore] Search games not yet implemented');
    return { games: [] };
  };

  // Account Management
  const getAccounts = async () => {
    // For now, return auth status as a single account
    // This can be expanded when multi-account support is added to backend
    try {
      const authStatus = await fetchStatus();
      if (authStatus && authStatus.authenticated) {
        return {
          accounts: [{
            user_id: authStatus.user_id,
            username: authStatus.username,
            account_type: 'bot',
            is_active: true,
            expires_at: authStatus.expires_at,
            is_expired: authStatus.is_expired
          }]
        };
      }
      return { accounts: [] };
    } catch (e) {
      console.error('[TwitchStore] Error fetching accounts:', e);
      return { accounts: [] };
    }
  };

  const authenticateAccount = async (code, state, accountType) => {
    // Use the existing OAuth callback
    return await completeOAuth(code, state);
  };

  const activateAccount = async (accountId) => {
    // Not needed for single account setup
    console.log('[TwitchStore] Account activation not needed for single account');
    return { success: true };
  };

  const deleteAccount = async (accountId) => {
    // Use revoke token for now
    return await revokeToken();
  };

  // Return the store API
  return {
    // Signals (read-only)
    chatMessages,
    botStatus,
    config,
    commands,
    wsConnected,

    // WebSocket controls
    connect,
    disconnect,

    // Event handlers
    on,
    off,

    // API methods
    startBot,
    stopBot,
    fetchStatus,
    getAuthUrl,
    completeOAuth,
    sendMessage,
    fetchConfig,
    saveConfig,
    fetchCommands,
    registerCommand,
    unregisterCommand,
    joinChannel,
    partChannel,
    revokeToken,
    getStreamInfo,
    updateStreamInfo,
    searchGames,

    // Account Management
    getAccounts,
    authenticateAccount,
    activateAccount,
    deleteAccount,
  };
}

// Create the store in a persistent root (survives hot reloads)
export default createRoot(createTwitchStore);
