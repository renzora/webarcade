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
    const response = await fetch(`${BRIDGE_URL}/twitch/start`, {
      method: 'POST',
    });

    if (!response.ok) {
      const error = await response.text();
      throw new Error(error);
    }

    const result = await response.json();
    await fetchStatus();
    return result;
  };

  const stopBot = async () => {
    const response = await fetch(`${BRIDGE_URL}/twitch/stop`, {
      method: 'POST',
    });

    if (!response.ok) {
      const error = await response.text();
      throw new Error(error);
    }

    const result = await response.json();
    await fetchStatus();
    return result;
  };

  const fetchStatus = async () => {
    try {
      const response = await fetch(`${BRIDGE_URL}/twitch/status`);
      const status = await response.json();
      setBotStatus(status);
      return status;
    } catch (e) {
      console.error('[TwitchStore] Failed to fetch status:', e);
      return null;
    }
  };

  const getAuthUrl = async () => {
    const response = await fetch(`${BRIDGE_URL}/twitch/auth-url`);

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
    const response = await fetch(`${BRIDGE_URL}/twitch/callback`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ code, state }),
    });

    if (!response.ok) {
      const error = await response.text();
      throw new Error(error);
    }

    return await response.json();
  };

  const sendMessage = async (channel, message) => {
    const response = await fetch(`${BRIDGE_URL}/twitch/send-message`, {
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
      const commandsData = await response.json();
      setCommands(commandsData);
      return commandsData;
    } catch (e) {
      console.error('[TwitchStore] Failed to fetch commands:', e);
      return [];
    }
  };

  const registerCommand = async (command) => {
    const response = await fetch(`${BRIDGE_URL}/twitch/register-command`, {
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
    const response = await fetch(`${BRIDGE_URL}/twitch/unregister-command`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ name }),
    });

    if (!response.ok) {
      const error = await response.text();
      throw new Error(error);
    }

    await fetchCommands();
    return await response.json();
  };

  const joinChannel = async (channel) => {
    const response = await fetch(`${BRIDGE_URL}/twitch/join-channel`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ channel }),
    });

    if (!response.ok) {
      const error = await response.text();
      throw new Error(error);
    }

    return await response.json();
  };

  const partChannel = async (channel) => {
    const response = await fetch(`${BRIDGE_URL}/twitch/part-channel`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ channel }),
    });

    if (!response.ok) {
      const error = await response.text();
      throw new Error(error);
    }

    return await response.json();
  };

  const revokeToken = async () => {
    const response = await fetch(`${BRIDGE_URL}/twitch/revoke`, {
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
    const response = await fetch(`${BRIDGE_URL}/twitch/stream-info`);

    if (!response.ok) {
      const error = await response.text();
      throw new Error(error);
    }

    return await response.json();
  };

  const updateStreamInfo = async (options) => {
    const response = await fetch(`${BRIDGE_URL}/twitch/update-stream-info`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        title: options.title || undefined,
        game_id: options.gameId || undefined,
        broadcaster_language: options.language || undefined,
        tags: options.tags || undefined,
        content_classification_labels: options.contentClassificationLabels || undefined,
        is_branded_content: options.isBrandedContent !== undefined ? options.isBrandedContent : undefined,
      }),
    });

    if (!response.ok) {
      const error = await response.text();
      throw new Error(error);
    }

    return await response.json();
  };

  const searchGames = async (query) => {
    const response = await fetch(`${BRIDGE_URL}/twitch/search-games`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ query }),
    });

    if (!response.ok) {
      const error = await response.text();
      throw new Error(error);
    }

    return await response.json();
  };

  // Account Management
  const getAccounts = async () => {
    console.log('[TwitchStore] Fetching accounts from:', `${BRIDGE_URL}/twitch/accounts`);
    const response = await fetch(`${BRIDGE_URL}/twitch/accounts`);

    console.log('[TwitchStore] Response status:', response.status);
    if (!response.ok) {
      const error = await response.text();
      console.error('[TwitchStore] Error fetching accounts:', error);
      throw new Error(error);
    }

    const data = await response.json();
    console.log('[TwitchStore] Fetched accounts data:', data);
    return data;
  };

  const authenticateAccount = async (code, state, accountType) => {
    const response = await fetch(`${BRIDGE_URL}/twitch/accounts/auth`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ code, state, account_type: accountType }),
    });

    if (!response.ok) {
      const error = await response.text();
      throw new Error(error);
    }

    return await response.json();
  };

  const activateAccount = async (accountId) => {
    const response = await fetch(`${BRIDGE_URL}/twitch/accounts/activate`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ account_id: accountId }),
    });

    if (!response.ok) {
      const error = await response.text();
      throw new Error(error);
    }

    return await response.json();
  };

  const deleteAccount = async (accountId) => {
    const response = await fetch(`${BRIDGE_URL}/twitch/accounts/delete`, {
      method: 'DELETE',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ account_id: accountId }),
    });

    if (!response.ok) {
      const error = await response.text();
      throw new Error(error);
    }

    return await response.json();
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
