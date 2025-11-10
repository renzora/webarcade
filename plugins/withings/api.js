const API_BASE = 'http://localhost:3001/withings';

export const withingsAPI = {
  // Auth methods
  async getAuthConfig() {
    const response = await fetch(`${API_BASE}/auth/config`);
    if (!response.ok) throw new Error('Failed to fetch auth config');
    return response.json();
  },

  async setAuthConfig(config) {
    const response = await fetch(`${API_BASE}/auth/config`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(config),
    });
    if (!response.ok) throw new Error('Failed to set auth config');
    return response.json();
  },

  async handleAuthCallback(code, state) {
    const response = await fetch(`${API_BASE}/auth/callback`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ code, state }),
    });
    if (!response.ok) throw new Error('Failed to handle auth callback');
    return response.json();
  },

  async refreshToken() {
    const response = await fetch(`${API_BASE}/auth/refresh`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
    });
    if (!response.ok) throw new Error('Failed to refresh token');
    return response.json();
  },

  async getAuthStatus() {
    const response = await fetch(`${API_BASE}/auth/status`);
    if (!response.ok) throw new Error('Failed to fetch auth status');
    return response.json();
  },

  // Measurement methods
  async getMeasurements() {
    const response = await fetch(`${API_BASE}/measurements`);
    if (!response.ok) throw new Error('Failed to fetch measurements');
    return response.json();
  },

  async getLatestMeasurements() {
    const response = await fetch(`${API_BASE}/measurements/latest`);
    if (!response.ok) throw new Error('Failed to fetch latest measurements');
    return response.json();
  },

  async syncMeasurements() {
    const response = await fetch(`${API_BASE}/measurements/sync`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
    });
    if (!response.ok) throw new Error('Failed to sync measurements');
    return response.json();
  },

  async getMeasurementStats() {
    const response = await fetch(`${API_BASE}/measurements/stats`);
    if (!response.ok) throw new Error('Failed to fetch measurement stats');
    return response.json();
  },
};
