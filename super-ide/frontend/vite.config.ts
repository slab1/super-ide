import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import monaco  from 'vite-plugin-monaco-editor'

export default defineConfig({
  plugins: [
    vue(),
    monaco({
      languageWorkers: ['editorWorkerService', 'css', 'html', 'json', 'typescript'],
    }),
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
  },
})
