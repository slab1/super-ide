import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'

export default defineConfig({
  plugins: [
    vue(),
    // Monaco editor will be loaded via CDN in this demo
  ],
  server: {
    port: 3000,
    proxy: {
      '/api': {
        target: 'http://localhost:3001',
        changeOrigin: true,
      },
      '/ws': {
        target: 'ws://localhost:3001',
        ws: true,
        changeOrigin: true,
      },
    },
  },
  build: {
    outDir: '../src/ui/web/dist',
    emptyOutDir: true,
    chunkSizeWarningLimit: 1000,
    rollupOptions: {
      output: {
        manualChunks: {
          // Monaco editor chunk
          monaco: ['monaco-editor'],
          // Vue ecosystem
          vue: ['vue', 'pinia'],
          // UI components
          ui: ['lucide-vue-next'],
          // HTTP client
          http: ['axios'],
        },
        chunkFileNames: 'assets/[name]-[hash].js',
      },
    },
  },
})
