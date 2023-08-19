import { defineConfig } from "vite";

export default defineConfig({
  server: {
    port: 8080,
    strictPort: true,
    proxy: {
      "/api/v0": {
        target: "http://127.0.0.1:8081",
        //changeOrigin: true,
        // rewrite: (path) => path.replace(/^\/api\/v0/, ""),
      },
    },
  },
  build: {
    target: 'esnext',
  },
});
