#!/usr/bin/env node
/**
 * Pre/post build script that dynamically configures Tauri resources
 * based on which plugin files exist in dist/plugins/.
 *
 * Usage:
 *   node prepare-build.js          - Add plugins to resources (before build)
 *   node prepare-build.js --reset  - Reset resources to empty (after build)
 */

const fs = require('fs');
const path = require('path');

const ROOT_DIR = path.resolve(__dirname, '..');
const BUILD_PLUGINS_DIR = path.join(ROOT_DIR, 'build', 'plugins');
const TAURI_CONFIG_PATH = path.join(ROOT_DIR, 'src-tauri', 'tauri.conf.json');

function getPluginFiles() {
  if (!fs.existsSync(BUILD_PLUGINS_DIR)) {
    return [];
  }

  const plugins = [];

  // Scan build/plugins for .js and .dll files
  const entries = fs.readdirSync(BUILD_PLUGINS_DIR, { withFileTypes: true });
  for (const entry of entries) {
    if (!entry.isFile()) continue;

    const ext = path.extname(entry.name).toLowerCase();
    if (ext === '.dll' || ext === '.so' || ext === '.dylib' || ext === '.js') {
      plugins.push(entry.name);
    }
  }

  return plugins;
}

function updateTauriConfig(reset = false) {
  // Read current config
  const config = JSON.parse(fs.readFileSync(TAURI_CONFIG_PATH, 'utf-8'));

  if (reset) {
    // Reset to empty resources for dev mode
    config.bundle.resources = {};
    fs.writeFileSync(TAURI_CONFIG_PATH, JSON.stringify(config, null, 2));
    console.log('[prepare-build] Reset tauri.conf.json resources to empty');
    return;
  }

  const plugins = getPluginFiles();
  console.log(`[prepare-build] Found ${plugins.length} plugin files:`, plugins);

  if (plugins.length === 0) {
    config.bundle.resources = {};
    fs.writeFileSync(TAURI_CONFIG_PATH, JSON.stringify(config, null, 2));
    console.log('[prepare-build] No plugin files found, resources set to empty');
    return;
  }

  // Build resources object - copy files from build/plugins/ to plugins/ in the bundle
  const resources = {};

  for (const plugin of plugins) {
    // Source: ../build/plugins/plugin.js -> Dest: plugins/plugin.js
    resources[`../build/plugins/${plugin}`] = `plugins/${plugin}`;
  }

  // Update config
  config.bundle.resources = resources;

  // Write back
  fs.writeFileSync(TAURI_CONFIG_PATH, JSON.stringify(config, null, 2));
  console.log('[prepare-build] Updated tauri.conf.json with plugin resources');
}

function main() {
  const args = process.argv.slice(2);
  const reset = args.includes('--reset');

  if (reset) {
    console.log('[prepare-build] Resetting config...');
  } else {
    console.log('[prepare-build] Preparing build...');
  }

  try {
    updateTauriConfig(reset);
    console.log('[prepare-build] Done!');
  } catch (error) {
    console.error('[prepare-build] Error:', error.message);
    process.exit(1);
  }
}

main();
