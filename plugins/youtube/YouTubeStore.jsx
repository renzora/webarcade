import { createStore } from 'solid-js/store';
import { createSignal } from 'solid-js';

const [youtubeStore, setYoutubeStore] = createStore({
  authenticated: false,
  userId: null,
  channels: [],
  selectedChannel: null,
  analytics: null,
  loading: false,
  error: null,
});

const [authChecking, setAuthChecking] = createSignal(false);

const youtubeApi = {
  // Auth methods
  async checkAuthStatus() {
    try {
      setAuthChecking(true);
      const response = await fetch('http://localhost:3001/youtube/auth/status');
      const data = await response.json();

      setYoutubeStore({
        authenticated: data.authenticated || false,
        userId: data.user_id || null,
      });

      // If authenticated, fetch channels
      if (data.authenticated) {
        await youtubeApi.fetchChannels();
      }
    } catch (error) {
      console.error('[YouTube] Failed to check auth status:', error);
      setYoutubeStore({ error: error.message });
    } finally {
      setAuthChecking(false);
    }
  },

  async getAuthUrl() {
    try {
      const response = await fetch('http://localhost:3001/youtube/auth/url');
      const data = await response.json();
      return data.url;
    } catch (error) {
      console.error('[YouTube] Failed to get auth URL:', error);
      throw error;
    }
  },

  async authenticate() {
    try {
      const authUrl = await youtubeApi.getAuthUrl();
      window.open(authUrl, '_blank', 'width=600,height=700');

      // Poll for auth status
      const pollInterval = setInterval(async () => {
        await youtubeApi.checkAuthStatus();
        if (youtubeStore.authenticated) {
          clearInterval(pollInterval);
        }
      }, 2000);

      // Stop polling after 5 minutes
      setTimeout(() => clearInterval(pollInterval), 300000);
    } catch (error) {
      console.error('[YouTube] Authentication failed:', error);
      setYoutubeStore({ error: error.message });
    }
  },

  async revokeAuth() {
    try {
      setYoutubeStore({ loading: true, error: null });
      await fetch('http://localhost:3001/youtube/auth/revoke', { method: 'POST' });
      setYoutubeStore({
        authenticated: false,
        userId: null,
        channels: [],
        selectedChannel: null,
        analytics: null,
      });
    } catch (error) {
      console.error('[YouTube] Failed to revoke auth:', error);
      setYoutubeStore({ error: error.message });
    } finally {
      setYoutubeStore({ loading: false });
    }
  },

  // Channel methods
  async fetchChannels() {
    try {
      setYoutubeStore({ loading: true, error: null });
      const response = await fetch('http://localhost:3001/youtube/channels');

      if (!response.ok) {
        throw new Error('Failed to fetch channels');
      }

      const data = await response.json();
      setYoutubeStore({ channels: data.channels || [] });

      // Auto-select first channel if available
      if (data.channels && data.channels.length > 0 && !youtubeStore.selectedChannel) {
        setYoutubeStore({ selectedChannel: data.channels[0].id });
      }
    } catch (error) {
      console.error('[YouTube] Failed to fetch channels:', error);
      setYoutubeStore({ error: error.message });
    } finally {
      setYoutubeStore({ loading: false });
    }
  },

  async fetchChannel(channelId) {
    try {
      setYoutubeStore({ loading: true, error: null });
      const response = await fetch(`http://localhost:3001/youtube/channels/${channelId}`);

      if (!response.ok) {
        throw new Error('Failed to fetch channel');
      }

      const data = await response.json();

      // Update the channel in the channels array
      setYoutubeStore('channels', channels =>
        channels.map(ch => ch.id === channelId ? data.channel : ch)
      );

      return data.channel;
    } catch (error) {
      console.error('[YouTube] Failed to fetch channel:', error);
      setYoutubeStore({ error: error.message });
    } finally {
      setYoutubeStore({ loading: false });
    }
  },

  selectChannel(channelId) {
    setYoutubeStore({ selectedChannel: channelId, analytics: null });
  },

  // Analytics methods
  async fetchAnalytics(channelId, startDate, endDate) {
    try {
      setYoutubeStore({ loading: true, error: null });

      const params = new URLSearchParams();
      if (startDate) params.append('start_date', startDate);
      if (endDate) params.append('end_date', endDate);

      const response = await fetch(
        `http://localhost:3001/youtube/analytics/${channelId}?${params.toString()}`
      );

      if (!response.ok) {
        throw new Error('Failed to fetch analytics');
      }

      const data = await response.json();
      setYoutubeStore({ analytics: data.analytics });

      return data.analytics;
    } catch (error) {
      console.error('[YouTube] Failed to fetch analytics:', error);
      setYoutubeStore({ error: error.message });
    } finally {
      setYoutubeStore({ loading: false });
    }
  },

  async fetchAnalyticsReport(channelId, startDate, endDate, metrics, dimensions) {
    try {
      setYoutubeStore({ loading: true, error: null });

      const params = new URLSearchParams({
        start_date: startDate,
        end_date: endDate,
        metrics: metrics.join(','),
      });

      if (dimensions && dimensions.length > 0) {
        params.append('dimensions', dimensions.join(','));
      }

      const response = await fetch(
        `http://localhost:3001/youtube/analytics/${channelId}/report?${params.toString()}`
      );

      if (!response.ok) {
        throw new Error('Failed to fetch analytics report');
      }

      const data = await response.json();
      return data;
    } catch (error) {
      console.error('[YouTube] Failed to fetch analytics report:', error);
      setYoutubeStore({ error: error.message });
      throw error;
    } finally {
      setYoutubeStore({ loading: false });
    }
  },

  // Utility methods
  clearError() {
    setYoutubeStore({ error: null });
  },
};

export default {
  ...youtubeStore,
  ...youtubeApi,
  authChecking,
};
