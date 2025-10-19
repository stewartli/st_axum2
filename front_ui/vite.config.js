import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'

// https://vite.dev/config/
export default defineConfig({
  plugins: [vue()],
  build: {
    outDir: "./../static/",
    rollupOptions: {
      output: {
        entryFileNames: `asset/[name].js`,
        chunkFileNames: `asset/[name].js`,
        assetFileNames: `asset/[name].[ext]`,
      },
    },
  },
})
