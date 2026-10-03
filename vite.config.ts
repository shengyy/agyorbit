import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";

// Tauri serves the dev build from a fixed port and reads `dist/` for releases.
export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: { port: 1420, strictPort: true },
  build: { target: "safari18", outDir: "dist", emptyOutDir: true },
});
