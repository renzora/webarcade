import { defineConfig } from '@rspack/cli'
import { rspack } from '@rspack/core'
import { resolve } from 'node:path'
import fs from 'fs'
import refresh from 'solid-refresh/babel'

const isProduction = process.env.NODE_ENV === 'production';

const config = {
  mode: isProduction ? 'production' : 'development',
  entry: {
    main: './src/entry-client.jsx',
  },
  
  experiments: {
    css: true,
  },
  
  infrastructureLogging: {
    level: 'error',
    debug: false,
    colors: false,
  },
  
  devtool: false,
  
  stats: 'normal',
  
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
        exclude: [
          /node_modules/,
        ],
        use: [
          {
            loader: 'babel-loader',
            options: {
              presets: [
                ['solid', {
                  generate: 'dom',
                  hydratable: false,
                  dev: process.env.NODE_ENV !== 'production'
                }]
              ],
              plugins: !isProduction ? [refresh] : [],
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
                  '@tailwindcss/postcss',
                  // Remove all CSS comments including daisyUI banner
                  {
                    postcssPlugin: 'remove-comments',
                    Once(root) {
                      root.walkComments(comment => {
                        comment.remove();
                      });
                    }
                  }
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
    new rspack.HtmlRspackPlugin({
      template: './src/index.html',
    }),
    new rspack.CopyRspackPlugin({
      patterns: [
        {
          from: 'public',
          to: '.',
          noErrorOnMissing: true,
          globOptions: {
            ignore: ['**/.*'],
          },
        },
      ],
    }),
    new rspack.DefinePlugin({
      'process.env.NODE_ENV': JSON.stringify(isProduction ? 'production' : 'development'),
      'import.meta.env.DEV': JSON.stringify(!isProduction),
      'import.meta.env.PROD': JSON.stringify(isProduction),
      'import.meta.env.MODE': JSON.stringify(isProduction ? 'production' : 'development'),
      '__DEV__': JSON.stringify(!isProduction),
    }),
    !isProduction && new rspack.HotModuleReplacementPlugin(),
  ].filter(Boolean),
  
  optimization: {
    splitChunks: {
      chunks: 'all',
      cacheGroups: {
        solidjs: {
          test: /[\\/]node_modules[\\/]solid-js/,
          name: 'solid',
          chunks: 'all',
          priority: 25,
          reuseExistingChunk: true,
        },
        vendor: {
          test: /[\\/]node_modules[\\/]/,
          name: 'vendor',
          chunks: 'all',
          priority: 10,
        },
      },
    },
    minimize: isProduction,
    minimizer: isProduction ? [
      new rspack.LightningCssMinimizerRspackPlugin({
        minimizerOptions: {
          targets: 'defaults',
        },
      }),
    ] : [],
  },
  
  performance: {
    hints: false,
    maxAssetSize: 10000000,
    maxEntrypointSize: 10000000,
  },
  
  output: {
    path: resolve(import.meta.dirname, 'dist'),
    filename: isProduction ? 'assets/[name]-[contenthash].js' : 'assets/[name].js',
    chunkFilename: isProduction ? 'assets/[name]-[contenthash].js' : 'assets/[name].js',
    assetModuleFilename: isProduction ? 'assets/[name]-[contenthash][ext]' : 'assets/[name][ext]',
    clean: true,
    publicPath: '/',
  }
};

// Only add devServer configuration in development
if (!isProduction) {
  config.devServer = {
    port: 3000,
    host: 'localhost',
    hot: true,
    liveReload: false,
    historyApiFallback: {
      disableDotRule: true,
      rewrites: [
        { from: /^\/twitch\/callback$/, to: '/twitch-callback.html' },
        { from: /^\/twitch\/callback\?.*$/, to: '/twitch-callback.html' },
      ],
    },
    static: [
      {
        directory: resolve(import.meta.dirname, 'public'),
        publicPath: '/',
        watch: true,
      },
      {
        directory: resolve(import.meta.dirname, 'dist'),
        publicPath: '/',
      }
    ],
    headers: {
      'Access-Control-Allow-Origin': '*',
      'Access-Control-Allow-Methods': 'GET, POST, PUT, DELETE, PATCH, OPTIONS',
      'Access-Control-Allow-Headers': 'X-Requested-With, Content-Type, Authorization',
    },
    client: {
      logging: 'error',
      progress: false,
      webSocketURL: {
        protocol: 'ws',
        hostname: 'localhost',
        port: 3000,
      },
    },
    setupExitSignals: false,
    compress: false,
    devMiddleware: {
      stats: 'none',
      writeToDisk: false,
    },
    allowedHosts: 'all',
    proxy: [
      {
        // Proxy all API routes to the bridge server
        // This catches any plugin routes automatically
        context: (pathname) => {
          // Proxy all paths starting with plugin-like names (lowercase with dashes/underscores)
          // Skip HMR, webpack, and static assets
          if (pathname.match(/^\/(hot|__webpack|assets\/[^l]|favicon)/)) {
            return false;
          }
          // Proxy if it looks like an API endpoint (not a file extension)
          if (pathname.startsWith('/') && !pathname.match(/\.[a-z0-9]+$/i)) {
            // Check if it's not an HTML page request (has file extension or is root)
            const isStaticFile = pathname.match(/\.(html|css|js|jsx|png|jpg|jpeg|gif|svg|ico|woff|woff2|ttf|eot)$/i);
            const isRoot = pathname === '/' || pathname === '';
            return !isStaticFile && !isRoot;
          }
          return false;
        },
        target: 'http://localhost:3001',
        changeOrigin: true,
        secure: false,
      },
    ],
    onListening: function (devServer) {
      if (!devServer) {
        throw new Error('webpack-dev-server is not defined');
      }
      const { port } = devServer.server.address();
      const protocol = devServer.server.listening && devServer.options.server?.type === 'https' ? 'https' : 'http';
      const url = `${protocol}://localhost:${port}`;
      
      console.log(`\n🎮 WebArcade Development Server\n   Running at: ${url}\n`);
    },
    server: 'http',
  };
}

export default defineConfig(config);