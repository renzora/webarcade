const fs = require('fs');
const path = require('path');

/**
 * Automatically discover plugins from src/plugins directory
 * and generate plugins.json configuration
 */

const PLUGINS_DIR = path.join(__dirname, '../plugins');
const OUTPUT_FILE = path.join(__dirname, '../src/api/plugin/plugins.json');

// Core plugins have higher priority (negative numbers load first)
const CORE_PLUGINS = [
  'bridge',
  'default',
  'plugins' // The plugin manager itself
];

function scanPluginsDirectory(baseDir, relativePath = '') {
  const plugins = [];
  const fullPath = path.join(baseDir, relativePath);

  if (!fs.existsSync(fullPath)) {
    return plugins;
  }

  const items = fs.readdirSync(fullPath, { withFileTypes: true });

  for (const item of items) {
    if (!item.isDirectory()) continue;

    const itemPath = path.join(relativePath, item.name);
    const itemFullPath = path.join(baseDir, itemPath);

    // Check if this directory contains a plugin entry point
    const indexPath = path.join(itemFullPath, 'index.jsx');
    const indexJsPath = path.join(itemFullPath, 'index.js');

    if (fs.existsSync(indexPath) || fs.existsSync(indexJsPath)) {
      // Found a plugin!
      const mainFile = fs.existsSync(indexPath) ? 'index.jsx' : 'index.js';
      const id = itemPath.replace(/\\/g, '-').replace(/\//g, '-');
      const isCore = CORE_PLUGINS.some(core => itemPath.replace(/\\/g, '/') === core);

      // Determine priority
      let priority = 1;
      if (id === 'bridge') priority = -2;
      else if (id === 'default') priority = -1;
      else if (isCore) priority = 0;

      plugins.push({
        id,
        path: `/plugins/${itemPath.replace(/\\/g, '/')}`,
        main: mainFile,
        enabled: true,
        priority
      });
    } else {
      // Recursively scan subdirectories
      plugins.push(...scanPluginsDirectory(baseDir, itemPath));
    }
  }

  return plugins;
}

function generatePluginsConfig() {
  console.log('🔍 Scanning for plugins...');

  const plugins = scanPluginsDirectory(PLUGINS_DIR);

  // Sort by priority
  plugins.sort((a, b) => a.priority - b.priority);

  const config = {
    plugins,
    generatedAt: new Date().toISOString(),
    generator: 'discover-plugins.js'
  };

  // Ensure output directory exists
  const outputDir = path.dirname(OUTPUT_FILE);
  if (!fs.existsSync(outputDir)) {
    fs.mkdirSync(outputDir, { recursive: true });
  }

  // Write the configuration
  fs.writeFileSync(OUTPUT_FILE, JSON.stringify(config, null, 2), 'utf-8');

  console.log(`✅ Generated plugins.json with ${plugins.length} plugins`);
  plugins.forEach(plugin => {
    console.log(`   - ${plugin.id} (priority: ${plugin.priority})`);
  });

  return plugins;
}

// Run the generator
try {
  generatePluginsConfig();
} catch (error) {
  console.error('❌ Error generating plugins config:', error);
  process.exit(1);
}
