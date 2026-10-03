export const ssr = false;

export async function load() {
  if (import.meta.env.VITE_MOCK) {
    const { installMockBackend } = await import("$lib/api/mock");
    installMockBackend();
  }
}
