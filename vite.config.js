import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'

export default defineConfig({
  plugins: [vue()],
  publicDir: 'demo',
  server: {
    port: 5190,
    strictPort: true,
  },
})
