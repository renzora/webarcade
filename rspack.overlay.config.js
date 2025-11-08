import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';
import { resolve } from 'node:path';
import fs from 'fs';

const isProduction = process.env.NODE_ENV === 'production';

// Dynamically find all overlay .jsx files from plugins directory
const pluginsDir = resolve(import.meta.dirname, 'plugins');
const overlayEntries = {};

if (fs.existsSync(pluginsDir)) {
  const plugins = fs.readdirSync(pluginsDir, { withFileTypes: true });

  plugins.forEach(plugin => {
    if (plugin.isDirectory()) {
      const pluginPath = resolve(pluginsDir, plugin.name);
      const files = fs.readdirSync(pluginPath);

      files.forEach(file => {
        // Look for overlay.jsx or overlay.tsx files in each plugin
        if (file === 'overlay.jsx' || file === 'overlay.tsx') {
          // Use plugin name as overlay name
          overlayEntries[plugin.name] = `./plugins/${plugin.name}/${file}`;
        }
      });
    }
  });
}

console.log('Building overlays:', Object.keys(overlayEntries));

const config = {
  mode: isProduction ? 'production' : 'development',
  entry: overlayEntries,

  experiments: {
    css: true,
  },

  devtool: false,

  resolve: {
    alias: {
      '@': resolve(import.meta.dirname, 'src')
    },
    extensions: ['.js', '.jsx', '.ts', '.tsx', '.json'],
    fullySpecified: false
  },

  module: {
    rules: [
      {
        test: /\.(jsx|tsx)$/,
        use: [
          {
            loader: 'babel-loader',
            options: {
              presets: [
                ['solid', {
                  generate: 'dom',
                  hydratable: false,
                  dev: false // Always build overlays in prod mode for performance
                }]
              ],
            },
          },
        ],
      },
      {
        test: /\.css$/,
        use: [
          {
            loader: 'postcss-loader',
            options: {
              postcssOptions: {
                plugins: [
                  '@tailwindcss/postcss'
                ]
              }
            }
          }
        ],
        type: 'css'
      }
    ],
  },

  plugins: [
    // Create an HTML file for each overlay entry
    ...Object.keys(overlayEntries).map(name =>
      new rspack.HtmlRspackPlugin({
        filename: `${name}.html`,
        chunks: [name],
        inject: 'body',
        minify: isProduction,
        templateContent: `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>WebArcade Overlay - ${name}</title>
  <style>
    /* Force light color scheme to prevent browser theme from affecting background */
    :root {
      color-scheme: only light;
    }
    * {
      box-sizing: border-box;
    }
    html, body, #root {
      margin: 0 !important;
      padding: 0 !important;
      background: transparent !important;
      background-color: transparent !important;
      overflow: hidden !important;
    }
    /* Override any DaisyUI theme backgrounds */
    body, html, #root, .bg-base-100, .bg-base-200, .bg-base-300 {
      background: transparent !important;
      background-color: transparent !important;
    }
    @keyframes slideUp {
      from { opacity: 0; transform: translateY(30px) scale(0.95); }
      to { opacity: 1; transform: translateY(0) scale(1); }
    }
    @keyframes fadeIn {
      from { opacity: 0; }
      to { opacity: 1; }
    }
  </style>
</head>
<body>
  <div id="root"></div>
</body>
</html>`,
      })
    ),
  ],

  optimization: {
    minimize: true,
    splitChunks: false, // Don't split chunks for overlays - keep them standalone
  },

  output: {
    path: resolve(import.meta.dirname, 'dist/overlays'),
    filename: '[name].js',
    clean: true,
    publicPath: '/overlay/',
  }
};

export default defineConfig(config);