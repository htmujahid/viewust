// Tauri has no Node.js server to render pages, so the app is a client-side SPA.
// See: https://svelte.dev/docs/kit/single-page-apps
// See: https://v2.tauri.app/start/frontend/sveltekit/
export const ssr = false;

/** `pnpm dev:mock` swaps the Rust backend for fixtures, so the UI runs in a browser. */
export async function load() {
  if (import.meta.env.VITE_MOCK) {
    const { installMockBackend } = await import("$lib/api/mock");
    installMockBackend();
  }
}
