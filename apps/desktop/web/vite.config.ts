import babel from "@rolldown/plugin-babel";
import tailwindcss from "@tailwindcss/vite";
import react, { reactCompilerPreset } from "@vitejs/plugin-react";
import { defineConfig } from "vitest/config";

export default defineConfig({
  plugins: [tailwindcss(), react(), babel({ presets: [reactCompilerPreset()] })],
  // 端口固定：src-tauri/tauri.conf.json 的 devUrl 指向它。
  server: { port: 5173, strictPort: true },
  test: { environment: "jsdom" },
});
