// @ts-check
import { defineConfig } from 'astro/config';

import tailwindcss from '@tailwindcss/vite';
import cloudflare from '@astrojs/cloudflare';

// https://astro.build/config
export default defineConfig({
  vite: {
    plugins: [tailwindcss()],
    server: {
      allowedHosts: ['astro.arc8.dev']
    },
    preview: {
      allowedHosts: ['astro.arc8.dev']
    }
  },

  adapter: cloudflare()
});
