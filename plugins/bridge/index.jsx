import * as fileAPI from './files.jsx';
import * as serverAPI from './server.jsx';

export class BridgeService {
  constructor() {
    // Delegate to API functions
  }

  // File operations - delegate to API
  async readFile(path) {
    return fileAPI.readFile(path);
  }

  async readBinaryFile(path) {
    return fileAPI.readBinaryFile(path);
  }

  async writeFile(path, content) {
    return fileAPI.writeFile(path, content);
  }

  async writeBinaryFile(path, base64Content) {
    return fileAPI.writeBinaryFile(path, base64Content);
  }

  async deleteFile(path) {
    return fileAPI.deleteFile(path);
  }

  async listDirectory(path = '') {
    return fileAPI.listDirectory(path);
  }

  getFileUrl(path) {
    return fileAPI.getFileUrl(path);
  }

  // Server operations - delegate to API
  async getHealth() {
    return serverAPI.getHealth();
  }

  async isServerConnected() {
    return serverAPI.isServerConnected();
  }

  async restartServer() {
    return serverAPI.restartServer();
  }

  async clearCache() {
    return serverAPI.clearCache();
  }
}

export const bridgeService = new BridgeService();
export { default as BridgeStatus } from './BridgeStatus.jsx';
export { default as BridgeViewport } from './BridgeViewport.jsx';
export { default as BridgePlugin } from './BridgePlugin.jsx';
export { default } from './BridgePluginClass.jsx';