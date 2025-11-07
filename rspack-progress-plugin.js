/**
 * Rspack Progress Plugin
 * Reports build progress to the backend server
 */
export class RspackProgressPlugin {
  constructor(options = {}) {
    this.backendUrl = options.backendUrl || 'http://localhost:3001';
    this.pluginName = 'RspackProgressPlugin';
  }

  apply(compiler) {
    let startTime;
    let lastProgress = 0;

    // Report build start
    compiler.hooks.compile.tap(this.pluginName, () => {
      startTime = Date.now();
      lastProgress = 0;
      this.reportProgress({
        state: 'compiling',
        progress: 0,
        message: 'Starting compilation...',
      });
    });

    // Report build progress
    compiler.hooks.compilation.tap(this.pluginName, (compilation) => {
      // Hook into build module to track progress
      compilation.hooks.buildModule.tap(this.pluginName, (module) => {
        // Throttle updates to avoid spamming the backend
        const currentTime = Date.now();
        if (currentTime - (this.lastUpdate || 0) > 500) {
          this.lastUpdate = currentTime;

          // Estimate progress based on time
          const elapsed = currentTime - startTime;
          const estimatedProgress = Math.min(Math.floor((elapsed / 5000) * 80), 80);

          if (estimatedProgress > lastProgress) {
            lastProgress = estimatedProgress;
            this.reportProgress({
              state: 'compiling',
              progress: estimatedProgress,
              message: 'Building modules...',
            });
          }
        }
      });
    });

    // Report build completion
    compiler.hooks.done.tap(this.pluginName, (stats) => {
      const hasErrors = stats.hasErrors();
      const hasWarnings = stats.hasWarnings();

      const duration = Date.now() - startTime;

      if (hasErrors) {
        const errors = stats.compilation.errors;
        this.reportProgress({
          state: 'error',
          progress: 100,
          message: `Build failed with ${errors.length} error(s)`,
          errors: errors.slice(0, 3).map(err => err.message),
        });
      } else if (hasWarnings) {
        this.reportProgress({
          state: 'warning',
          progress: 100,
          message: `Build completed with warnings (${duration}ms)`,
        });
      } else {
        this.reportProgress({
          state: 'success',
          progress: 100,
          message: `Build completed successfully (${duration}ms)`,
        });
      }
    });

    // Report build failure
    compiler.hooks.failed.tap(this.pluginName, (error) => {
      this.reportProgress({
        state: 'error',
        progress: 100,
        message: 'Build failed',
        errors: [error.message],
      });
    });
  }

  reportProgress(data) {
    // Send progress to backend
    fetch(`${this.backendUrl}/system/build-progress`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        ...data,
        timestamp: Date.now(),
      }),
    }).catch(() => {
      // Silently fail if backend is not available
    });
  }
}
