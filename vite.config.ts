import { fileURLToPath, URL } from "node:url";
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      // Même alias que `paths` dans tsconfig.app.json : "@/..." → "src/..."
      "@": fileURLToPath(new URL("./src", import.meta.url))
    }
  },
  clearScreen: false,
  server: {
    host: "localhost",
    port: 1420,
    strictPort: true
  },
  envPrefix: ["VITE_", "TAURI_ENV_*"]
});
