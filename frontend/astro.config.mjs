// @ts-check
import { defineConfig } from 'astro/config';

export default defineConfig({
  output: 'static',

  vite: {
    resolve: {
      alias: {
        '@': '/src'
      }
    },
    server: {
      proxy: {
        "/api": {
          target: "http://127.0.0.1:8080",
          changeOrigin: true,
          rewrite: (path) => path.replace(/^\/api/, ""),
        },
      },
    },
  },
});
