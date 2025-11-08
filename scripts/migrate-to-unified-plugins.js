const fs = require('fs');
const path = require('path');

/**
 * Migrates plugins from bridge/src/plugins, src/plugins/core, and src/overlays
 * to a unified plugins/ directory at the root
 */

const ROOT = path.join(__dirname, '..');
const BACKEND_PLUGINS = path.join(ROOT, 'bridge/src/plugins');
const FRONTEND_PLUGINS = path.join(ROOT, 'src/plugins/core');
const OVERLAYS = path.join(ROOT, 'src/overlays');
const TARGET = path.join(ROOT, 'plugins');

// Mapping of overlay files to their plugin names
const OVERLAY_MAPPING = {
  'status.jsx': 'status',
  'ticker.jsx': 'ticker',
  'wheel.jsx': 'wheel',
  'pack-opening.jsx': 'packs',
  'snow.jsx': 'snow',
  'auction.jsx': 'auction',
  'timer.jsx': 'timer',
  'emoji-wall.jsx': 'emoji-wall',
  'viewers.jsx': 'viewer-stats',
  'todos.jsx': 'todos',
  'weight.jsx': 'withings',
  'chat-highlight.jsx': 'chat-highlight',
  'effect.jsx': 'alerts',
  'levelup.jsx': 'levels',
  'watchtime-leaderboard.jsx': 'watchtime',
  'mood-ticker.jsx': 'mood-tracker',
  'layout.jsx': 'layout-manager',
  'goals.jsx': 'goals',
  'alerts.jsx': 'alerts',
  'chat.jsx': 'twitch',
  'roulette.jsx': 'roulette'
};

function copyRecursive(src, dest) {
  const exists = fs.existsSync(src);
  const stats = exists && fs.statSync(src);
  const isDirectory = exists && stats.isDirectory();

  if (isDirectory) {
    if (!fs.existsSync(dest)) {
      fs.mkdirSync(dest, { recursive: true });
    }
    fs.readdirSync(src).forEach(childItemName => {
      copyRecursive(
        path.join(src, childItemName),
        path.join(dest, childItemName)
      );
    });
  } else {
    fs.copyFileSync(src, dest);
  }
}

function migratePlugins() {
  console.log('🚀 Starting plugin migration to unified structure...\n');

  // Ensure target directory exists
  if (!fs.existsSync(TARGET)) {
    fs.mkdirSync(TARGET, { recursive: true });
  }

  // Step 1: Copy all backend plugins
  console.log('📦 Step 1: Copying backend plugins from bridge/src/plugins...');
  if (fs.existsSync(BACKEND_PLUGINS)) {
    const backendItems = fs.readdirSync(BACKEND_PLUGINS, { withFileTypes: true });
    backendItems.forEach(item => {
      if (item.isDirectory() && item.name !== 'node_modules') {
        const src = path.join(BACKEND_PLUGINS, item.name);
        const dest = path.join(TARGET, item.name);
        console.log(`   Copying ${item.name}...`);
        copyRecursive(src, dest);
      }
    });
  }

  // Step 2: Merge frontend plugins
  console.log('\n📦 Step 2: Merging frontend plugins from src/plugins/core...');
  if (fs.existsSync(FRONTEND_PLUGINS)) {
    const frontendItems = fs.readdirSync(FRONTEND_PLUGINS, { withFileTypes: true });
    frontendItems.forEach(item => {
      if (item.isDirectory() && item.name !== 'node_modules') {
        const src = path.join(FRONTEND_PLUGINS, item.name);
        const dest = path.join(TARGET, item.name);

        if (!fs.existsSync(dest)) {
          fs.mkdirSync(dest, { recursive: true });
        }

        console.log(`   Merging ${item.name}...`);

        // Copy all files from frontend plugin
        fs.readdirSync(src).forEach(file => {
          const srcFile = path.join(src, file);
          const destFile = path.join(dest, file);

          if (fs.statSync(srcFile).isFile()) {
            fs.copyFileSync(srcFile, destFile);
          } else if (fs.statSync(srcFile).isDirectory()) {
            copyRecursive(srcFile, destFile);
          }
        });
      }
    });
  }

  // Step 3: Merge overlays
  console.log('\n📦 Step 3: Merging overlays from src/overlays...');
  if (fs.existsSync(OVERLAYS)) {
    const overlayFiles = fs.readdirSync(OVERLAYS).filter(f => f.endsWith('.jsx'));

    overlayFiles.forEach(file => {
      const pluginName = OVERLAY_MAPPING[file];
      if (pluginName) {
        const src = path.join(OVERLAYS, file);
        const dest = path.join(TARGET, pluginName, file);

        // Ensure plugin directory exists
        const pluginDir = path.join(TARGET, pluginName);
        if (!fs.existsSync(pluginDir)) {
          fs.mkdirSync(pluginDir, { recursive: true });
        }

        console.log(`   Moving ${file} to ${pluginName}/`);
        fs.copyFileSync(src, dest);
      } else {
        console.warn(`   ⚠️  No plugin mapping for ${file}`);
      }
    });
  }

  // Step 4: List all plugins in new structure
  console.log('\n✅ Migration complete! Unified plugin structure:');
  const plugins = fs.readdirSync(TARGET, { withFileTypes: true })
    .filter(item => item.isDirectory())
    .map(item => item.name)
    .sort();

  console.log(`\n📂 ${plugins.length} plugins in plugins/ directory:`);
  plugins.forEach(plugin => {
    const pluginPath = path.join(TARGET, plugin);
    const files = fs.readdirSync(pluginPath);
    const hasRust = files.some(f => f.endsWith('.rs'));
    const hasJsx = files.some(f => f.endsWith('.jsx') || f.endsWith('.js'));
    const markers = [];
    if (hasRust) markers.push('Rust');
    if (hasJsx) markers.push('Frontend');
    console.log(`   - ${plugin} ${markers.length > 0 ? `[${markers.join(', ')}]` : ''}`);
  });

  console.log('\n🎉 Migration successful!');
}

// Run the migration
try {
  migratePlugins();
} catch (error) {
  console.error('❌ Error during migration:', error);
  process.exit(1);
}
