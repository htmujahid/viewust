import { defineConfig } from "vitest/config";
import { sveltekit } from "@sveltejs/kit/vite";
// @ts-expect-error type error without @types/node package
import process from "node:process";
const host = process.env.TAURI_DEV_HOST;

// https://vite.dev/config/
export default defineConfig(() => ({
  plugins: [sveltekit()],

  test: {
    include: ["src/**/*.test.ts"],
  },

  // Pre-bundle every dependency at startup. Left to itself, Vite discovers them
  // the first time a page imports them, re-optimizes mid-session, and the page
  // that triggered it fails with "failed to load virtual css module".
  optimizeDeps: {
    include: [
      "@tauri-apps/api/core",
      "@xyflow/svelte",
      "svelte/animate",
      "svelte/easing",
      "svelte/events",
      "svelte/motion",
      "svelte/reactivity",
      "svelte/store",
      "svelte/transition",
    ],
  },
  ssr: {
    optimizeDeps: {
      include: [
        "svelte/animate",
        "svelte/easing",
        "svelte/events",
        "svelte/motion",
        "svelte/reactivity",
        "svelte/store",
        "svelte/transition",
      ],
    },
  },

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent Vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
    host: host || "127.0.0.1",
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      // 3. tell Vite to ignore watching `src-tauri`
      ignored: ["**/src-tauri/**"],
    },
  },
}));
