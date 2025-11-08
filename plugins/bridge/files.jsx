import { bridgeFetch, parseJsonResponse } from './config.jsx';

export async function readFile(path) {
  const encodedPath = path.split('/').map(segment => encodeURIComponent(segment)).join('/');
  const response = await bridgeFetch(`/read/${encodedPath}`);
  const data = await parseJsonResponse(response);
  return data.content;
}

export async function readBinaryFile(path) {
  const encodedPath = path.split('/').map(segment => encodeURIComponent(segment)).join('/');
  const response = await bridgeFetch(`/file/${encodedPath}`);
  const blob = await response.blob();
  
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => {
      const result = reader.result;
      if (typeof result === 'string' && result.includes(',')) {
        resolve(result.split(',')[1]);
      } else {
        reject(new Error('Failed to convert blob to base64'));
      }
    };
    reader.onerror = () => reject(reader.error);
    reader.readAsDataURL(blob);
  });
}

export async function writeFile(path, content) {
  const encodedPath = path.split('/').map(segment => encodeURIComponent(segment)).join('/');
  const response = await bridgeFetch(`/write/${encodedPath}`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ content })
  });
  return parseJsonResponse(response);
}

export async function writeBinaryFile(path, base64Content) {
  const encodedPath = path.split('/').map(segment => encodeURIComponent(segment)).join('/');
  const response = await bridgeFetch(`/write-binary/${encodedPath}`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ base64_content: base64Content })
  });
  return parseJsonResponse(response);
}

export async function deleteFile(path) {
  const encodedPath = path.split('/').map(segment => encodeURIComponent(segment)).join('/');
  const response = await bridgeFetch(`/delete/${encodedPath}`, {
    method: 'DELETE'
  });
  return parseJsonResponse(response);
}

export async function listDirectory(path = '') {
  // Don't encode the full path since the bridge server expects forward slashes
  // Just encode individual path components to handle special characters
  const encodedPath = path.split('/').map(segment => encodeURIComponent(segment)).join('/');
  const response = await bridgeFetch(`/list/${encodedPath}`);
  return parseJsonResponse(response);
}

export function getFileUrl(path) {
  const encodedPath = path.split('/').map(segment => encodeURIComponent(segment)).join('/');
  return `http://localhost:3001/file/${encodedPath}`;
}