const fs = require('fs');
const path = require('path');

/**
 * Rename all overlay files to overlay.jsx in each plugin
 */

const PLUGINS_DIR = path.join(__dirname, '../plugins');

// Mapping of old overlay filenames to their plugin directories
const OVERLAY_FILES = {
  'alerts': ['alerts.jsx', 'effect.jsx'],
  'auction': ['auction.jsx'],
  'chat-highlight': ['chat-highlight.jsx'],
  'twitch': ['chat.jsx'],
  'emoji-wall': ['emoji-wall.jsx'],
  'goals': ['goals.jsx'],
  'layout-manager': ['layout.jsx'],
  'levels': ['levelup.jsx'],
  'mood-tracker': ['mood-ticker.jsx'],
  'packs': ['pack-opening.jsx'],
  'roulette': ['roulette.jsx'],
  'snow': ['snow.jsx'],
  'status': ['status.jsx'],
  'ticker': ['ticker.jsx'],
  'timer': ['timer.jsx'],
  'todos': ['todos.jsx'],
  'viewer-stats': ['viewers.jsx'],
  'watchtime': ['watchtime-leaderboard.jsx'],
  'withings': ['weight.jsx'],
  'wheel': ['wheel.jsx']
};

function renameOverlays() {
  console.log('🎨 Renaming overlay files to overlay.jsx...\n');

  let renamedCount = 0;
  let errors = [];

  for (const [plugin, overlayFiles] of Object.entries(OVERLAY_FILES)) {
    const pluginPath = path.join(PLUGINS_DIR, plugin);

    if (!fs.existsSync(pluginPath)) {
      console.warn(`⚠️  Plugin directory not found: ${plugin}`);
      continue;
    }

    // If multiple overlay files exist for one plugin, merge/prioritize
    let foundFile = null;
    for (const overlayFile of overlayFiles) {
      const oldPath = path.join(pluginPath, overlayFile);
      if (fs.existsSync(oldPath)) {
        foundFile = { name: overlayFile, path: oldPath };
        break; // Use the first one found
      }
    }

    if (foundFile) {
      const newPath = path.join(pluginPath, 'overlay.jsx');

      // Check if overlay.jsx already exists
      if (fs.existsSync(newPath) && foundFile.path !== newPath) {
        console.log(`   ⚠️  ${plugin}: overlay.jsx already exists, skipping ${foundFile.name}`);
        continue;
      }

      try {
        if (foundFile.path !== newPath) {
          fs.renameSync(foundFile.path, newPath);
          console.log(`   ✓ ${plugin}: ${foundFile.name} → overlay.jsx`);
          renamedCount++;
        } else {
          console.log(`   ✓ ${plugin}: already named overlay.jsx`);
        }
      } catch (error) {
        errors.push(`${plugin}: ${error.message}`);
        console.error(`   ✗ ${plugin}: Error renaming ${foundFile.name} - ${error.message}`);
      }
    }
  }

  console.log(`\n✅ Renamed ${renamedCount} overlay files`);

  if (errors.length > 0) {
    console.error(`\n❌ ${errors.length} errors:`);
    errors.forEach(err => console.error(`   - ${err}`));
    process.exit(1);
  }
}

// Run the renaming
try {
  renameOverlays();
} catch (error) {
  console.error('❌ Error renaming overlays:', error);
  process.exit(1);
}
