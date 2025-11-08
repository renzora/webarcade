import { createSignal, createRoot } from 'solid-js';

const BRIDGE_URL = 'http://localhost:3001';
const WS_URL = 'ws://localhost:3002';

/**
 * Singleton service for Twitch integration
 */
export class TwitchService {
  static instance = null;

  constructor() {
    if (TwitchService.instance) {
      return TwitchService.instance;
    }

    this.ws = null;
    this.eventHandlers = new Map();

    // Create signals within a root to avoid disposal warnings
    createRoot(() => {
      this.chatMessages = createSignal([]);
      this.botStatus = createSignal({ status: 'disconnected' });
      this.config = createSignal({});
      this.commands = createSignal([]);
    });

    TwitchService.instance = this;
  }

  static getInstance() {
    if (!TwitchService.instance) {
      TwitchService.instance = new TwitchService();
    }
    return TwitchService.instance;
  }

  /**
   * Initialize WebSocket connection
   */
  async initialize() {
    return new Promise((resolve, reject) => {
      try {
        this.ws = new WebSocket(WS_URL);

        this.ws.onopen = () => {
          console.log('[TwitchService] WebSocket connected');
          resolve();
        };

        this.ws.onmessage = (event) => {
          try {
            const data = JSON.parse(event.data);
            this.handleWebSocketMessage(data);
          } catch (e) {
            console.error('[TwitchService] Failed to parse WebSocket message:', e);
          }
        };

        this.ws.onerror = (error) => {
          console.error('[TwitchService] WebSocket error:', error);
          reject(error);
        };

        this.ws.onclose = () => {
          console.log('[TwitchService] WebSocket disconnected');
          // Attempt reconnect after 5 seconds
          setTimeout(() => this.initialize(), 5000);
        };
      } catch (e) {
        reject(e);
      }
    });
  }

  /**
   * Handle incoming WebSocket messages
   */
  handleWebSocketMessage(data) {
    if (data.type === 'twitch_event') {
      this.handleTwitchEvent(data.event);
    } else if (data.type === 'file_change') {
      // Ignore file changes for now
    } else if (data.type === 'connected') {
      console.log('[TwitchService] Connected to bridge');
    }
  }

  /**
   * Handle Twitch events from backend
   */
  handleTwitchEvent(event) {
    const [, setChatMessages] = this.chatMessages;

    switch (event.type) {
      case 'chat_message':
        // Add message to chat
        setChatMessages((prev) => [...prev, event].slice(-100)); // Keep last 100 messages
        this.emit('chat_message', event);
        break;

      case 'connected':
        console.log('[TwitchService] Bot connected to channels:', event.channels);
        this.emit('connected', event);
        this.fetchStatus();
        break;

      case 'disconnected':
        console.log('[TwitchService] Bot disconnected:', event.reason);
        this.emit('disconnected', event);
        this.fetchStatus();
        break;

      case 'user_joined':
      case 'user_left':
      case 'channel_joined':
      case 'channel_parted':
      case 'notice':
      case 'error':
        this.emit(event.type, event);
        break;

      default:
        console.log('[TwitchService] Unknown event type:', event.type);
    }
  }

  /**
   * Event emitter
   */
  on(eventName, handler) {
    if (!this.eventHandlers.has(eventName)) {
      this.eventHandlers.set(eventName, []);
    }
    this.eventHandlers.get(eventName).push(handler);
  }

  off(eventName, handler) {
    if (this.eventHandlers.has(eventName)) {
      const handlers = this.eventHandlers.get(eventName);
      const index = handlers.indexOf(handler);
      if (index !== -1) {
        handlers.splice(index, 1);
      }
    }
  }

  emit(eventName, data) {
    if (this.eventHandlers.has(eventName)) {
      this.eventHandlers.get(eventName).forEach((handler) => {
        try {
          handler(data);
        } catch (e) {
          console.error(`[TwitchService] Error in event handler for ${eventName}:`, e);
        }
      });
    }
  }

  /**
   * API Methods
   */

  async startBot() {
    const response = await fetch(`${BRIDGE_URL}/twitch/start`, {
      method: 'POST',
    });

    if (!response.ok) {
      const error = await response.text();
      throw new Error(error);
    }

    const result = await response.json();
    await this.fetchStatus();
    return result;
  }

  async stopBot() {
    const response = await fetch(`${BRIDGE_URL}/twitch/stop`, {
      method: 'POST',
    });

    if (!response.ok) {
      const error = await response.text();
      throw new Error(error);
    }

    const result = await response.json();
    await this.fetchStatus();
    return result;
  }

  async fetchStatus() {
    try {
      const response = await fetch(`${BRIDGE_URL}/twitch/status`);
      const status = await response.json();
      const [, setBotStatus] = this.botStatus;
      setBotStatus(status);
      return status;
    } catch (e) {
      console.error('[TwitchService] Failed to fetch status:', e);
      return null;
    }
  }

  async getAuthUrl() {
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
  }

  async completeOAuth(code, state) {
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
  }

  async sendMessage(channel, message) {
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
  }

  async fetchConfig() {
    try {
      const response = await fetch(`${BRIDGE_URL}/twitch/config`);
      const config = await response.json();
      const [, setConfig] = this.config;
      setConfig(config);
      return config;
    } catch (e) {
      console.error('[TwitchService] Failed to fetch config:', e);
      return null;
    }
  }

  async saveConfig(config) {
    const response = await fetch(`${BRIDGE_URL}/twitch/config`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(config),
    });

    if (!response.ok) {
      const error = await response.text();
      throw new Error(error);
    }

    await this.fetchConfig();
    return await response.json();
  }

  async fetchCommands() {
    try {
      const response = await fetch(`${BRIDGE_URL}/twitch/commands`);
      const commands = await response.json();
      const [, setCommands] = this.commands;
      setCommands(commands);
      return commands;
    } catch (e) {
      console.error('[TwitchService] Failed to fetch commands:', e);
      return [];
    }
  }

  async registerCommand(command) {
    const response = await fetch(`${BRIDGE_URL}/twitch/register-command`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(command),
    });

    if (!response.ok) {
      const error = await response.text();
      throw new Error(error);
    }

    await this.fetchCommands();
    return await response.json();
  }

  async unregisterCommand(name) {
    const response = await fetch(`${BRIDGE_URL}/twitch/unregister-command`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ name }),
    });

    if (!response.ok) {
      const error = await response.text();
      throw new Error(error);
    }

    await this.fetchCommands();
    return await response.json();
  }

  async joinChannel(channel) {
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
  }

  async partChannel(channel) {
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
  }

  async revokeToken() {
    const response = await fetch(`${BRIDGE_URL}/twitch/revoke`, {
      method: 'POST',
    });

    if (!response.ok) {
      const error = await response.text();
      throw new Error(error);
    }

    await this.fetchConfig();
    return await response.json();
  }

  /**
   * Get reactive signals
   */
  getChatMessages() {
    return this.chatMessages[0];
  }

  getBotStatus() {
    return this.botStatus[0];
  }

  getConfig() {
    return this.config[0];
  }

  getCommands() {
    return this.commands[0];
  }

  /**
   * Disconnect WebSocket
   */
  disconnect() {
    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }
  }
}
