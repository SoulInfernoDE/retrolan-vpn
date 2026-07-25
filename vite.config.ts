import { defineConfig } from 'vite';

export default defineConfig({
  root: 'ui',
  build: {
    outDir: '../dist',
    emptyOutDir: true,
  },
  server: {
    port: 1420,
    strictPort: true,
  },
  clearScreen: false,
});