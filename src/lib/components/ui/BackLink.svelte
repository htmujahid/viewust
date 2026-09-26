<script lang="ts">
  let { href, children }: { href: string; children: import("svelte").Snippet } = $props();
</script>

<a class="btn" {href}>
  <svg
    width="16"
    height="16"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    stroke-width="2"
    stroke-linecap="round"
    stroke-linejoin="round"
    aria-hidden="true"
  >
    <path d="M19 12H5M11 6l-6 6 6 6" />
  </svg>
  {@render children()}
</a>
