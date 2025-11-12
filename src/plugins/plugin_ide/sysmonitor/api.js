import { bridge } from '@/api/bridge';

export async function fetchStats() {
  const response = await bridge('/sysmonitor/stats');
  return response.json();
}

export async function fetchHistory() {
  const response = await bridge('/sysmonitor/history');
  return response.json();
}

export async function saveStats() {
  const response = await bridge('/sysmonitor/save-stats', {
    method: 'POST'
  });
  return response.json();
}
