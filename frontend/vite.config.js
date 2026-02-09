import { fileURLToPath, URL } from 'node:url';
import { defineConfig } from 'vite';
import vue from '@vitejs/plugin-vue';
import tailwindcss from '@tailwindcss/vite';

// https://vite.dev/config/
export default defineConfig({
  plugins: [tailwindcss(), vue()],
  resolve: {
    alias: {
      '@': fileURLToPath(new URL('./src', import.meta.url)),
    },
  },
  server: {
    proxy: {
      '/api': 'http://localhost:8080',
    },
  },
  build: {
    outDir: '../frontend-dist',
    emptyOutDir: true,
    rollupOptions: {
      output: {
        manualChunks: {
          three: ['three', '@tresjs/core', '@tresjs/cientos'],
          markdown: ['marked', 'dompurify'],
        },
      },
    },
    watch: process.env.VITE_WATCH
      ? {
          chokidar: {
            usePolling: true,
            interval: 100,
          },
        }
      : null,
  },
});
