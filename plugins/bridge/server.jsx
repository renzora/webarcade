import { bridgeFetch, parseJsonResponse } from './config.jsx';

export async function getHealth() {
  const response = await bridgeFetch('/health');
  return parseJsonResponse(response);
}

export async function getStartupTime() {
  const response = await bridgeFetch('/startup-time');
  return parseJsonResponse(response);
}

export async function restartServer() {
  const response = await bridgeFetch('/restart', {
    method: 'POST'
  });
  return parseJsonResponse(response);
}

export async function clearCache() {
  const response = await bridgeFetch('/clear-cache', {
    method: 'POST'
  });
  return parseJsonResponse(response);
}

export async function isServerConnected() {
  try {
    await getHealth();
    return true;
  } catch {
    return false;
  }
}