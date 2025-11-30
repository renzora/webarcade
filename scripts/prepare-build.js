#!/usr/bin/env node
/**
 * Pre/post build script that dynamically configures Tauri resources
 * based on which plugin DLLs exist in the plugins/ directory.
 *
 * Usage:
 *   node prepare-build.js          - Add plugins to resources (before build)
 *   node prepare-build.js --reset  - Reset resources to empty (after build)
 */

const fs = require('fs');
const path = require('path');

const ROOT_DIR = path.resolve(__dirname, '..');
const PLUGINS_DIR = path.join(ROOT_DIR, 'plugins');
const TAURI_CONFIG_PATH = path.join(ROOT_DIR, 'src-tauri', 'tauri.conf.json');

function getPluginDlls() {
  if (!fs.existsSync(PLUGINS_DIR)) {
    return [];
  }

  const dlls = [];

  // Scan plugin directories for DLLs
  const entries = fs.readdirSync(PLUGINS_DIR, { withFileTypes: true });
  for (const dirent of entries) {
    if (!dirent.isDirectory()) continue;

    const pluginDir = path.join(PLUGINS_DIR, dirent.name);
    const pluginFiles = fs.readdirSync(pluginDir, { withFileTypes: true });

    for (const file of pluginFiles) {
      if (!file.isFile()) continue;
      const ext = path.extname(file.name).toLowerCase();
      if (ext === '.dll' || ext === '.so' || ext === '.dylib') {
        // Store as { dir: 'plugin-name', file: 'plugin-name.dll' }
        dlls.push({ dir: dirent.name, file: file.name });
      }
    }
  }

  return dlls;
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

  const dlls = getPluginDlls();
  console.log(`[prepare-build] Found ${dlls.length} plugin DLLs:`, dlls.map(d => `${d.dir}/${d.file}`));

  if (dlls.length === 0) {
    // No DLLs found, ensure plugins directory exists and set empty resources
    config.bundle.resources = {};
    fs.writeFileSync(TAURI_CONFIG_PATH, JSON.stringify(config, null, 2));
    console.log('[prepare-build] No plugin DLLs found, resources set to empty');
    return;
  }

  // Build resources object - copy DLL files directly to plugins/ in the bundle
  const resources = {};

  for (const dll of dlls) {
    // Map DLL file directly to plugins/ (no subdirectory in production)
    // Source: ../plugins/plugin-name/plugin-name.dll -> Dest: plugins/plugin-name.dll
    resources[`../plugins/${dll.dir}/${dll.file}`] = `plugins/${dll.file}`;
  }

  // Update config
  config.bundle.resources = resources;

  // Write back
  fs.writeFileSync(TAURI_CONFIG_PATH, JSON.stringify(config, null, 2));
  console.log('[prepare-build] Updated tauri.conf.json with plugin DLL resources');
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
