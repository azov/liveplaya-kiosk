import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import wasm from "vite-plugin-wasm";

export default defineConfig({
  plugins: [react(), wasm()], //wasmPack('../libjs')],
  server: {
    port: 8080,
    strictPort: true,
    proxy: {
      "/api/v0": {
        target: "http://127.0.0.1:8081",
        //changeOrigin: true,
        //rewrite: (path) => path.replace(/^\/api\/v0/, ""),
      },
    },
  },
  build: {
    target: 'esnext',
    watch: {
      include: [
        "../../back/**",
        "../../libjs/**",
        "../../librs/**",
      ],
    },
  },
});
