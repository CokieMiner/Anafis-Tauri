import path from 'node:path';
import react from '@vitejs/plugin-react';
import { defineConfig } from 'vite';

const host = process.env.TAURI_DEV_HOST;

// https://vite.dev/config/
export default defineConfig(() => ({
  plugins: [
    react({
      // Optimize JSX runtime
      jsxRuntime: 'automatic',
    }),
  ],

  // Path resolution for cleaner imports
  resolve: {
    alias: {
      // Main src alias
      '@': path.resolve(__dirname, './src'),
    },
    // Deduplicate shared dependencies to prevent multiple instances
    dedupe: [
      '@wendellhu/redi',
      '@univerjs/core',
      '@univerjs/design',
      '@univerjs/engine-render',
      '@univerjs/engine-formula',
      '@univerjs/network',
      'react',
      'react-dom',
    ],
  },

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent Vite from obscuring rust errors
  clearScreen: false,
  // Enhanced build configuration with performance optimizations
  build: {
    // Target modern browsers for better optimization
    target: 'esnext',
    rollupOptions: {
      input: {
        main: './index.html',
        tab: './tab.html',
        settings: './settings.html',
        'uncertainty-calculator': './uncertainty-calculator.html',
        'latex-preview': './latex-preview.html',
        'data-library': './data-library.html',
      },
      output: {
        entryFileNames: 'assets/js/[name]-[hash].js',
        assetFileNames: (assetInfo) => {
          if (assetInfo.name?.endsWith('.css')) {
            return 'assets/css/[name]-[hash][extname]';
          }
          if (assetInfo.name?.match(/\.(png|jpe?g|svg|gif|tiff|bmp|ico)$/i)) {
            return 'assets/images/[name]-[hash][extname]';
          }
          if (assetInfo.name?.match(/\.(woff2?|eot|ttf|otf)$/i)) {
            return 'assets/fonts/[name]-[hash][extname]';
          }
          return 'assets/[name]-[hash][extname]';
        },
      },
      // Tree shaking optimizations
      // Note: moduleSideEffects is set to true to preserve side effects from external packages
      // such as Material-UI CSS imports, Emotion theme initialization, ECharts setup, and other
      // libraries that require their side effects to function properly
      treeshake: {
        moduleSideEffects: true,
        propertyReadSideEffects: false,
        unknownGlobalSideEffects: false,
      },
    },
    outDir: 'dist',
    assetsDir: 'assets',
    // Optimized chunk size limits
    chunkSizeWarningLimit: 800,
    // Source maps for debugging (disabled for production)
    sourcemap: process.env.NODE_ENV === 'development',
    // Enhanced minification
    minify: 'terser' as const,
    terserOptions: {
      compress: {
        drop_console: true,
        drop_debugger: true,
        pure_funcs: ['console.log', 'console.info'],
        passes: 2,
      },
      mangle: {
        safari10: true,
      },
      format: {
        comments: false,
      },
    },
    // CSS optimization
    cssCodeSplit: true,
    cssMinify: true,
    // Rollup optimizations
    reportCompressedSize: false, // Faster builds
    emptyOutDir: true,
  },
  // Base path for production builds
  base: './',
  // Enhanced dependency optimization
  optimizeDeps: {
    include: [
      'react',
      'react-dom',
      'react/jsx-runtime',
      '@mui/material',
      '@mui/icons-material',
      '@emotion/react',
      '@emotion/styled',
      'echarts',
      'katex',
      'react-katex',
      '@tauri-apps/api',
      '@tauri-apps/plugin-dialog',
      'zustand',
      '@dnd-kit/core',
      '@dnd-kit/sortable',
      '@dnd-kit/utilities',
      // Force pre-bundling of critical Univer dependencies
      '@wendellhu/redi',
      '@univerjs/core',
      '@univerjs/design',
      '@univerjs/engine-render',
      '@univerjs/engine-formula',
      '@univerjs/network',
    ],
    exclude: [
      // Exclude large plugin packages that should be chunked separately
      '@univerjs/presets',
    ],
  },

  // Enhanced development server configuration
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: 'ws',
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      ignored: ['**/src-tauri/**', '**/node_modules/**'],
      usePolling: false, // Better performance on most systems
    },
    // Faster cold start
    warmup: {
      clientFiles: [
        './src/main.tsx',
        './src/App.tsx',
        './src/tabs/home/HomeTab.tsx',
        './src/tabs/spreadsheet/SpreadsheetTab.tsx',
      ],
    },
  },

  // Performance optimizations
  esbuild: {
    // Remove console logs in production
    drop: (process.env.NODE_ENV === 'production'
      ? ['console', 'debugger']
      : []) as ('console' | 'debugger')[],
    // Target modern browsers
    target: 'esnext',
  },
}));
