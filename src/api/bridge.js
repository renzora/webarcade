/**
 * Bridge API client
 * Uses relative URLs to work with proxy in development (HTTPS)
 * and direct connection in production
 */

// Static constants for overlays (always use direct connection)
export const WEBARCADE_WS = 'ws://localhost:3002';
export const BRIDGE_API = 'http://localhost:3001';

// Dynamic base URL for main app (uses import.meta which may not be available in overlays)
const BRIDGE_BASE_URL = (typeof import.meta !== 'undefined' && import.meta.env?.PROD)
  ? 'http://localhost:3001'
  : '';

export async function bridgeFetch(path, options = {}) {
  const url = `${BRIDGE_BASE_URL}${path}`;
  return fetch(url, options);
}

export default {
  fetch: bridgeFetch,
  baseUrl: BRIDGE_BASE_URL,
};
