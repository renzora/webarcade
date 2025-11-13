const fs = require('fs');
const path = require('path');

/**
 * Automatically discover plugins from src/plugins directory
 * and generate both frontend (plugins.json) and backend (generated.rs) configurations
 *
 * Note: Core components (dashboard, database, developer, menu, settings, system)
 * have been moved to src/components and are no longer plugins.
 * They are initialized directly in the app.
 */

const PLUGINS_DIR = path.join(__dirname, '../src/plugins');
const FRONTEND_OUTPUT = path.join(__dirname, '../src/api/plugin/plugins.json');
const BACKEND_OUTPUT = path.join(__dirname, '../src-tauri/src/bridge/plugins/generated.rs');

// Core plugins have higher priority (negative numbers load first)
const CORE_PLUGINS = [];

function toPascalCase(str) {
  return str.split('_').map(word =>
    word.charAt(0).toUpperCase() + word.slice(1)
  ).join('');
}

function scanWidgetsInPlugin(pluginPath) {
  const widgetsDir = path.join(pluginPath, 'widgets');
  const widgets = [];

  if (!fs.existsSync(widgetsDir)) {
    return widgets;
  }

  try {
    const files = fs.readdirSync(widgetsDir, { withFileTypes: true });

    for (const file of files) {
      if (file.isFile() && file.name.endsWith('.jsx')) {
        widgets.push(file.name);
      }
    }
  } catch (error) {
    // Ignore errors reading widgets directory
  }

  return widgets;
}

function scanPluginsDirectory(baseDir, relativePath = '') {
  const frontendPlugins = [];
  const backendPlugins = [];
  const fullPath = path.join(baseDir, relativePath);

  if (!fs.existsSync(fullPath)) {
    return { frontendPlugins, backendPlugins };
  }

  const items = fs.readdirSync(fullPath, { withFileTypes: true });

  for (const item of items) {
    if (!item.isDirectory()) continue;

    const itemPath = path.join(relativePath, item.name);
    const itemFullPath = path.join(baseDir, itemPath);

    // Skip the developer/projects directory
    const normalizedPath = itemPath.replace(/\\/g, '/');
    if (normalizedPath === 'developer/projects' || normalizedPath.startsWith('developer/projects/')) {
      console.log(`⏭️  Skipping development projects directory: ${itemPath}`);
      continue;
    }

    // Check for frontend plugin files
    const indexPath = path.join(itemFullPath, 'index.jsx');
    const indexJsPath = path.join(itemFullPath, 'index.js');
    const widgetPath = path.join(itemFullPath, 'Widget.jsx');
    const widgetsDir = path.join(itemFullPath, 'widgets');

    // Check for backend plugin files
    const modPath = path.join(itemFullPath, 'mod.rs');

    const hasIndex = fs.existsSync(indexPath) || fs.existsSync(indexJsPath);
    const hasWidgetsDir = fs.existsSync(widgetsDir) && fs.statSync(widgetsDir).isDirectory();
    const hasModRs = fs.existsSync(modPath);

    // Frontend plugin detection
    if (hasIndex || hasWidgetsDir) {
      const mainFile = hasIndex ? (fs.existsSync(indexPath) ? 'index.jsx' : 'index.js') : null;
      const id = itemPath.replace(/\\/g, '-').replace(/\//g, '-');
      const isCore = CORE_PLUGINS.some(core => itemPath.replace(/\\/g, '/') === core);

      // Determine priority
      let priority = 1;
      if (id === 'bridge') priority = -2;
      else if (id === 'default') priority = -1;
      else if (isCore) priority = 0;

      const hasWidget = fs.existsSync(widgetPath);
      const widgetFiles = scanWidgetsInPlugin(itemFullPath);

      frontendPlugins.push({
        id,
        path: `/src/plugins/${itemPath.replace(/\\/g, '/')}`,
        main: mainFile,
        widget: hasWidget ? 'Widget.jsx' : null,
        widgets: widgetFiles.length > 0 ? widgetFiles : null,
        enabled: true,
        priority
      });
    }

    // Backend plugin detection
    if (hasModRs) {
      const pluginName = item.name;
      // Validate Rust identifier
      if (/^[a-zA-Z_][a-zA-Z0-9_]*$/.test(pluginName)) {
        backendPlugins.push(pluginName);
      } else {
        console.warn(`⚠️  Skipping backend plugin '${pluginName}' - invalid Rust module name`);
      }
    }

    // If no plugin files found, recursively scan subdirectories
    if (!hasIndex && !hasWidgetsDir && !hasModRs) {
      const nested = scanPluginsDirectory(baseDir, itemPath);
      frontendPlugins.push(...nested.frontendPlugins);
      backendPlugins.push(...nested.backendPlugins);
    }
  }

  return { frontendPlugins, backendPlugins };
}

function generateFrontendConfig(plugins) {
  console.log('📦 Generating frontend plugins configuration...');

  // Sort by priority
  plugins.sort((a, b) => a.priority - b.priority);

  const config = {
    plugins,
    generatedAt: new Date().toISOString(),
    generator: 'discover-plugins.js'
  };

  // Ensure output directory exists
  const outputDir = path.dirname(FRONTEND_OUTPUT);
  if (!fs.existsSync(outputDir)) {
    fs.mkdirSync(outputDir, { recursive: true });
  }

  // Write the configuration
  fs.writeFileSync(FRONTEND_OUTPUT, JSON.stringify(config, null, 2), 'utf-8');

  console.log(`✅ Generated ${FRONTEND_OUTPUT} with ${plugins.length} frontend plugins`);
  plugins.forEach(plugin => {
    console.log(`   - ${plugin.id} (priority: ${plugin.priority})`);
  });
}

function generateBackendCode(plugins) {
  console.log('\n🦀 Generating backend plugins registration...');

  // Sort plugins
  plugins.sort();

  // Generate module declarations with path attributes
  let code = '// Auto-generated by scripts/discover-plugins.js\n';
  code += '// DO NOT EDIT MANUALLY\n\n';

  for (const plugin of plugins) {
    code += `#[path = "../../../../src/plugins/${plugin}/mod.rs"]\n`;
    code += `pub mod ${plugin};\n`;
  }

  code += '\n';

  // Generate registration function
  code += 'use crate::bridge::core::plugin_manager::PluginManager;\n\n';
  code += '/// Auto-generated plugin registration function\n';
  code += 'pub fn register_all_plugins(manager: &mut PluginManager) {\n';
  code += `    log::info!("📦 Registering ${plugins.length} auto-discovered plugins...");\n\n`;

  // Group plugins
  const corePlugins = plugins.filter(p => p === 'database');
  const foundationalPlugins = plugins.filter(p => ['currency', 'notes', 'goals', 'todos', 'counters'].includes(p));
  const gamePlugins = plugins.filter(p => ['auction', 'roulette', 'levels', 'wheel', 'packs'].includes(p));
  const otherPlugins = plugins.filter(p =>
    !corePlugins.includes(p) &&
    !foundationalPlugins.includes(p) &&
    !gamePlugins.includes(p)
  );

  if (corePlugins.length > 0) {
    code += '    // Tier 0: Core plugins\n';
    for (const plugin of corePlugins) {
      const structName = toPascalCase(plugin);
      code += `    manager.register(${plugin}::${structName}Plugin);\n`;
    }
    code += '\n';
  }

  if (foundationalPlugins.length > 0) {
    code += '    // Tier 1: Foundational plugins\n';
    for (const plugin of foundationalPlugins) {
      const structName = toPascalCase(plugin);
      code += `    manager.register(${plugin}::${structName}Plugin);\n`;
    }
    code += '\n';
  }

  if (gamePlugins.length > 0) {
    code += '    // Tier 2: Game plugins\n';
    for (const plugin of gamePlugins) {
      const structName = toPascalCase(plugin);
      code += `    manager.register(${plugin}::${structName}Plugin);\n`;
    }
    code += '\n';
  }

  if (otherPlugins.length > 0) {
    code += '    // Tier 3+: Other plugins\n';
    for (const plugin of otherPlugins) {
      const structName = toPascalCase(plugin);
      code += `    manager.register(${plugin}::${structName}Plugin);\n`;
    }
    code += '\n';
  }

  code += `    log::info!("✅ Plugin registration complete (${plugins.length} plugins)");\n`;
  code += '}\n';

  // Write the generated code
  fs.writeFileSync(BACKEND_OUTPUT, code, 'utf-8');

  console.log(`✅ Generated ${BACKEND_OUTPUT} with ${plugins.length} backend plugins`);
  plugins.forEach(plugin => {
    console.log(`   - ${plugin}`);
  });
}

function generateAllPlugins() {
  console.log('🔍 Scanning for plugins in src/plugins...\n');

  const { frontendPlugins, backendPlugins } = scanPluginsDirectory(PLUGINS_DIR);

  if (frontendPlugins.length > 0) {
    generateFrontendConfig(frontendPlugins);
  } else {
    console.log('⚠️  No frontend plugins found');
  }

  if (backendPlugins.length > 0) {
    generateBackendCode(backendPlugins);
  } else {
    console.log('⚠️  No backend plugins found');
  }

  console.log('\n✨ Plugin discovery complete!');
}

// Run the generator
try {
  generateAllPlugins();
} catch (error) {
  console.error('❌ Error generating plugins:', error);
  process.exit(1);
}
