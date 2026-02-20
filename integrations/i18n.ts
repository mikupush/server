import type { AstroIntegration } from "astro"

export function reactI18nAutoImport(): AstroIntegration {
  return {
    name: 'i18n-auto-import',
    hooks: {
      'astro:config:setup': ({ updateConfig, injectScript }) => {
        updateConfig({
          vite: {
            plugins: [
              {
                name: 'auto-import-i18n-injector',
                enforce: 'post',
                transform(code, id) {
                  if (/\.(jsx|tsx)$/.test(id) && !id.includes('node_modules')) {
                    return `import '@/i18n/i18n';\n${code}`;
                  }
                }
              }
            ]
          }
        });

        injectScript('page-ssr', 'import "@/i18n/i18n";')
      },
    },
  };
}
