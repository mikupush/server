import { defineConfig } from "astro/config";
import react from '@astrojs/react';
import tailwindcss from "@tailwindcss/vite";
import path from "path";

// https://astro.build/config
export default defineConfig({
  srcDir: './web',
  build: {
    assets: 'assets',
    format: 'file'
  },
  vite: {
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
  integrations: [react()]
});
