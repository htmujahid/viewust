<script lang="ts">
  /** Pauses and resumes a live page's updates. */
  let { live = $bindable() }: { live: boolean } = $props();
</script>

<button class="btn" class:on={live} onclick={() => (live = !live)} aria-pressed={live}>
  <span class="dot" class:pulse={live}></span>
  {live ? "Live" : "Paused"}
</button>

<style>
  .dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--text-3);
  }
  .on .dot {
    background: var(--ok);
  }
  .pulse {
    animation: pulse 1.6s ease-in-out infinite;
  }
  @keyframes pulse {
    50% {
      opacity: 0.35;
    }
  }
</style>
