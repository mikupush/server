import { defineConfig } from 'astro/config'
import react from '@astrojs/react'
import tailwindcss from '@tailwindcss/vite'
import { reactI18nAutoImport } from './integrations/i18n.ts'
import path from 'path'

// https://astro.build/config
export default defineConfig({
  srcDir: './web',
  i18n: {
    defaultLocale: 'en',
    locales: ['en', 'es'],
    routing: {
      prefixDefaultLocale: false
    }
  },
  build: {
    assets: 'assets',
    format: 'file'
  },
  server: {
    cors: true,
  },
  vite: {
    server: {
      proxy: {
        '/u': 'http://localhost:8080',
        '/api': 'http://localhost:8080'
      }
    },
    resolve: {
      alias: {
        '@': path.resolve('./web')
      }
    },
    build: {
      assetsDir: 'assets',
    },
    plugins: [tailwindcss()]
  },
  integrations: [
    react(),
    reactI18nAutoImport()
  ]
});
