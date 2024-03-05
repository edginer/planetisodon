import { defineConfig } from "vite";
import react from "@vitejs/plugin-react-swc";
import tsconfigPaths from "vite-tsconfig-paths";
import { nodePolyfills } from "vite-plugin-node-polyfills";

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [
    react(),
    tsconfigPaths(),
    nodePolyfills({
      globals: {
        Buffer: true, // can also be 'build', 'dev', or false
      },
    }),
  ],
  server: {
    proxy: {
      "^/planetisodon/(dat/(.\\d{9,10})\\.dat|subject.txt)$": {
        target: "https://planetisodon.eddibb.cc",
        changeOrigin: true,
      },
      "^/test/bbs.cgi$": {
        target: "https://planetisodon.eddibb.cc",
        changeOrigin: true,
      },
    },
  },
});
