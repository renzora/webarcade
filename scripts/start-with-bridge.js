#!/usr/bin/env node

/**
 * Start the bridge server and wait for it to be ready before starting the next command
 */

const { spawn } = require('child_process');
const http = require('http');

const BRIDGE_URL = 'http://localhost:3001';
const MAX_RETRIES = 180; // 3 minutes
const RETRY_INTERVAL = 1000;

let retries = 0;
let bridgeProcess = null;

// Get the command to run after bridge is ready
const nextCommand = process.argv.slice(2).join(' ');

if (!nextCommand) {
  console.error('Usage: node start-with-bridge.js <command>');
  process.exit(1);
}

console.log('🚀 Starting bridge server...');

// Check if release binary exists
const fs = require('fs');
const path = require('path');
const isWindows = process.platform === 'win32';
const releaseBinary = path.join('bridge', 'target', 'release', isWindows ? 'webarcade-bridge.exe' : 'webarcade-bridge');

// Start the bridge server
if (fs.existsSync(releaseBinary)) {
  console.log('✅ Using pre-built release binary (faster startup)');
  bridgeProcess = spawn(
    releaseBinary,
    [],
    {
      stdio: 'inherit',
      shell: isWindows,
      env: { ...process.env, RUST_LOG: 'info' },
    }
  );
} else {
  console.log('⚠️  No release binary found, compiling from source (slower)...');
  console.log('💡 Tip: Run "bun run build:bridge" once to speed up future startups');
  bridgeProcess = spawn(
    isWindows ? 'cargo.exe' : 'cargo',
    ['run', '--manifest-path', 'bridge/Cargo.toml', '--bin', 'webarcade-bridge', '--features', 'nvidia'],
    {
      stdio: 'inherit',
      shell: isWindows,
      env: { ...process.env, RUST_LOG: 'info' },
    }
  );
}

bridgeProcess.on('error', (error) => {
  console.error('❌ Failed to start bridge:', error.message);
  process.exit(1);
});

bridgeProcess.on('exit', (code) => {
  if (code !== 0 && code !== null) {
    console.error(`❌ Bridge exited with code ${code}`);
    process.exit(code);
  }
});

function checkBridge() {
  return new Promise((resolve) => {
    const req = http.get(`${BRIDGE_URL}/api/health`, (res) => {
      resolve(res.statusCode === 200);
    });

    req.on('error', () => resolve(false));
    req.setTimeout(1000, () => {
      req.destroy();
      resolve(false);
    });
  });
}

async function waitForBridge() {
  console.log('🔍 Waiting for bridge server to be ready...');

  while (retries < MAX_RETRIES) {
    const isReady = await checkBridge();

    if (isReady) {
      console.log('✅ Bridge server is ready!');
      console.log(`🎮 Starting: ${nextCommand}`);

      // Start the next command
      const nextProcess = spawn(nextCommand, {
        stdio: 'inherit',
        shell: true,
      });

      nextProcess.on('exit', (code) => {
        // Kill bridge when app exits
        if (bridgeProcess) {
          bridgeProcess.kill();
        }
        process.exit(code || 0);
      });

      return;
    }

    retries++;

    if (retries % 10 === 0) {
      console.log(`⏳ Still waiting for bridge... (${retries}s elapsed)`);
    }

    await new Promise((resolve) => setTimeout(resolve, RETRY_INTERVAL));
  }

  console.error('❌ Bridge server failed to start within timeout');
  if (bridgeProcess) {
    bridgeProcess.kill();
  }
  process.exit(1);
}

// Handle cleanup on exit
process.on('SIGINT', () => {
  console.log('\n⚠️  Shutting down...');
  if (bridgeProcess) {
    bridgeProcess.kill();
  }
  process.exit(0);
});

process.on('SIGTERM', () => {
  if (bridgeProcess) {
    bridgeProcess.kill();
  }
  process.exit(0);
});

// Start waiting for bridge
waitForBridge();
