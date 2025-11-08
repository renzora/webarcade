export const BRIDGE_CONFIG = {
  baseUrl: 'http://localhost:3001',
  timeout: 30000,
  retries: 3
};

export async function bridgeFetch(endpoint, options = {}) {
  const url = `${BRIDGE_CONFIG.baseUrl}${endpoint}`;
  
  const response = await fetch(url, {
    timeout: BRIDGE_CONFIG.timeout,
    ...options
  });

  if (!response.ok) {
    const error = new Error(`Bridge API Error: ${response.status} ${response.statusText}`);
    error.status = response.status;
    error.response = response;
    throw error;
  }

  return response;
}

export async function parseJsonResponse(response) {
  try {
    return await response.json();
  } catch {
    throw new Error('Failed to parse JSON response from bridge server');
  }
}