import { defineConfig } from 'vite';
import { resolve } from 'path';

export default defineConfig({
  root: 'ui',
  base: './',
  build: {
    outDir: '../dist',
    emptyOutDir: true,
    rollupOptions: {
      input: {
        main: resolve(__dirname, 'ui/index.html'),
      },
    },
  },
  server: {
    port: 1420,
    strictPort: true,
  },
});
