import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

const host = process.env.TAURI_DEV_HOST;

// https://vite.dev/config/
export default defineConfig(() => ({
  plugins: [react()],

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent Vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
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
      // 3. tell Vite to ignore watching `src-tauri`
      ignored: ["**/src-tauri/**"],
    },
  },
  // Build configuration for multiple entry points with code splitting
  build: {
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
        manualChunks: {
          // Vendor chunks for large libraries
          'vendor-react': ['react', 'react-dom'],
          'vendor-mui': ['@mui/material', '@mui/icons-material', '@emotion/react', '@emotion/styled'],
          'vendor-univer': [
            '@univerjs/core',
            '@univerjs/design',
            '@univerjs/docs',
            '@univerjs/docs-ui',
            '@univerjs/engine-formula',
            '@univerjs/engine-render',
            '@univerjs/presets',
            '@univerjs/sheets',
            '@univerjs/sheets-formula',
            '@univerjs/sheets-ui',
            '@univerjs/ui'
          ],
          'vendor-charts': ['echarts'],
          'vendor-math': ['katex', 'react-katex'],
          'vendor-tauri': ['@tauri-apps/api', '@tauri-apps/plugin-dialog'],
          'vendor-dnd': ['@dnd-kit/core', '@dnd-kit/sortable', '@dnd-kit/utilities'],
          'vendor-utils': ['zustand'],
        },
        // Optimize chunk file names
        chunkFileNames: (chunkInfo) => {
          const facadeModuleId = chunkInfo.facadeModuleId
            ? chunkInfo.facadeModuleId.split('/').pop()?.replace('.tsx', '').replace('.ts', '')
            : 'chunk';
          return `assets/${facadeModuleId}-[hash].js`;
        },
        assetFileNames: (assetInfo) => {
          if (assetInfo.name?.endsWith('.css')) {
            return 'assets/[hash][extname]';
          }
          return 'assets/[hash][extname]';
        },
      },
    },
    outDir: "dist",
    assetsDir: "assets",
    // Increase chunk size warning limit since we're optimizing
    chunkSizeWarningLimit: 1000,
    // Enable source maps for debugging
    sourcemap: false,
    // Minimize for production
    minify: 'terser',
  },
  // Base path for production builds
  base: "./",
  // Optimize dependencies
  optimizeDeps: {
    include: [
      'react',
      'react-dom',
      '@mui/material',
      '@mui/icons-material',
      'echarts',
      'katex',
      '@tauri-apps/api',
      'zustand'
    ],
  },
}));
