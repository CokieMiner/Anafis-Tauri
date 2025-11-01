import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import path from 'path';

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
      '@': path.resolve(__dirname, './src'),
      '@/components': path.resolve(__dirname, './src/components'),
      '@/hooks': path.resolve(__dirname, './src/hooks'),
      '@/types': path.resolve(__dirname, './src/types'),
      '@/utils': path.resolve(__dirname, './src/utils'),
      '@/themes': path.resolve(__dirname, './src/themes'),
      '@/pages': path.resolve(__dirname, './src/pages'),
    },
  },

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent Vite from obscuring rust errors
  clearScreen: false,
  // Enhanced build configuration with performance optimizations
  build: {
    // Target modern browsers for better optimization
    target: 'es2022',
    rollupOptions: {
      input: {
        main: "./index.html",
        tab: "./tab.html",
        settings: "./settings.html",
        "uncertainty-calculator": "./uncertainty-calculator.html",
        "latex-preview": "./latex-preview.html",
        "data-library": "./data-library.html",
      },
      output: {
        // Enhanced chunk splitting strategy
        manualChunks: (id) => {
          // React ecosystem
          if (id.includes('react') || id.includes('react-dom')) {
            return 'vendor-react';
          }
          // Material-UI ecosystem
          if (id.includes('@mui') || id.includes('@emotion')) {
            return 'vendor-mui';
          }
          // Univer spreadsheet ecosystem
          if (id.includes('@univerjs')) {
            return 'vendor-univer';
          }
          // Charts and visualization
          if (id.includes('echarts')) {
            return 'vendor-charts';
          }
          // Math and LaTeX
          if (id.includes('katex') || id.includes('react-katex')) {
            return 'vendor-math';
          }
          // Tauri APIs
          if (id.includes('@tauri-apps')) {
            return 'vendor-tauri';
          }
          // Drag and drop
          if (id.includes('@dnd-kit')) {
            return 'vendor-dnd';
          }
          // State management and utilities
          if (id.includes('zustand')) {
            return 'vendor-utils';
          }
          // Node modules that aren't specifically chunked
          if (id.includes('node_modules')) {
            return 'vendor-misc';
          }
        },
        // Optimized file naming
        chunkFileNames: (chunkInfo) => {
          // Helper to create safe filename identifiers
          const createSafeIdentifier = (input: string): string => {
            return input
              .replace(/[^a-zA-Z0-9-_]/g, '-') // Replace special chars with dashes
              .replace(/-+/g, '-') // Collapse multiple dashes
              .replace(/^-|-$/g, '') // Remove leading/trailing dashes
              .substring(0, 50); // Limit length
          };

          // Prefer chunkInfo.name for better identification
          if (chunkInfo.name) {
            return `assets/js/${createSafeIdentifier(chunkInfo.name)}-[hash].js`;
          }

          // Fallback to facadeModuleId with more path context
          if (chunkInfo.facadeModuleId) {
            // Include last 2 path segments for better uniqueness
            const pathParts = chunkInfo.facadeModuleId.split('/').filter(Boolean);
            const relevantParts = pathParts.slice(-2); // Last 2 segments
            const baseName = relevantParts.join('-').replace(/\.(tsx?|jsx?)$/, '');
            return `assets/js/${createSafeIdentifier(baseName)}-[hash].js`;
          }

          return 'assets/js/chunk-[hash].js';
        },
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
    outDir: "dist",
    assetsDir: "assets",
    // Optimized chunk size limits
    chunkSizeWarningLimit: 800,
    // Source maps for debugging (disabled for production)
    sourcemap: process.env.NODE_ENV === 'development',
    // Enhanced minification
    minify: 'terser',
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
  base: "./",
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
    ],
    exclude: [
      // Exclude large libraries that should be chunked separately
      '@univerjs/core',
      '@univerjs/sheets',
    ],
  },

  // Enhanced development server configuration
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      ignored: ["**/src-tauri/**", "**/node_modules/**"],
      usePolling: false, // Better performance on most systems
    },
    // Faster cold start
    warmup: {
      clientFiles: [
        './src/main.tsx',
        './src/App.tsx',
        './src/pages/HomeTab.tsx',
      ],
    },
  },

  // Performance optimizations
  esbuild: {
    // Remove console logs in production
    drop: process.env.NODE_ENV === 'production' ? ['console', 'debugger'] : [],
    // Target modern browsers
    target: 'es2022',
  },
}));
