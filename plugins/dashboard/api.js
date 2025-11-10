// Dashboard API Client
const API_BASE = 'http://localhost:3001/dashboard';

export const dashboardAPI = {
  // Dashboard CRUD
  async getDashboards() {
    const response = await fetch(`${API_BASE}/dashboards`);
    if (!response.ok) throw new Error('Failed to fetch dashboards');
    return response.json();
  },

  async createDashboard(name) {
    const response = await fetch(`${API_BASE}/dashboards`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ name }),
    });
    if (!response.ok) throw new Error('Failed to create dashboard');
    return response.json();
  },

  async updateDashboard(id, name) {
    const response = await fetch(`${API_BASE}/dashboards/${id}`, {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ name }),
    });
    if (!response.ok) throw new Error('Failed to update dashboard');
    return response.json();
  },

  async deleteDashboard(id) {
    const response = await fetch(`${API_BASE}/dashboards/${id}`, {
      method: 'DELETE',
    });
    if (!response.ok) throw new Error('Failed to delete dashboard');
    return response.json();
  },

  // Widget Instance CRUD
  async getWidgets(dashboardId) {
    const response = await fetch(`${API_BASE}/dashboards/${dashboardId}/widgets`);
    if (!response.ok) throw new Error('Failed to fetch widgets');
    return response.json();
  },

  async createWidget(dashboardId, widgetId, orderIndex = 0, columns = 1, config = null) {
    const response = await fetch(`${API_BASE}/dashboards/${dashboardId}/widgets`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        widget_id: widgetId,
        order_index: orderIndex,
        columns,
        config,
      }),
    });
    if (!response.ok) throw new Error('Failed to create widget');
    return response.json();
  },

  async updateWidget(id, updates) {
    const response = await fetch(`${API_BASE}/widgets/${id}`, {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(updates),
    });
    if (!response.ok) throw new Error('Failed to update widget');
    return response.json();
  },

  async deleteWidget(id) {
    const response = await fetch(`${API_BASE}/widgets/${id}`, {
      method: 'DELETE',
    });
    if (!response.ok) throw new Error('Failed to delete widget');
    return response.json();
  },

  async reorderWidgets(dashboardId, widgetIds) {
    const response = await fetch(`${API_BASE}/dashboards/${dashboardId}/widgets/reorder`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ widget_ids: widgetIds }),
    });
    if (!response.ok) throw new Error('Failed to reorder widgets');
    return response.json();
  },
};
