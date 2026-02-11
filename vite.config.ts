import { defineConfig } from 'vite'
import { globSync } from 'glob'
import react from '@vitejs/plugin-react'
import path from 'path'

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './web'),
    },
  },
  root: 'web',
  build: {
    outDir: '../dist',
    emptyOutDir: true,
    rollupOptions: {
      input: Object.fromEntries(globSync('web/**/*.html').map(
        (file) => {
          const entryName = path.basename(file.replace(path.extname(file), ''))
          return [entryName, file]
        }
      ))
    },
  },
})
