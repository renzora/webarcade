export const BRIDGE_API = 'http://localhost:3001';
export const WEBARCADE_WS = 'ws://localhost:3002';

export async function bridge(path, options = {}) {
  const url = `${BRIDGE_API}${path}`;
  return fetch(url, options);
}

export default {
  fetch: bridge,
  baseUrl: BRIDGE_API,
};
