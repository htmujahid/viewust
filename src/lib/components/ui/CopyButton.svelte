<script lang="ts">
  let {
    text,
    label = "Copy",
    disabled = false,
  }: { text: () => string; label?: string; disabled?: boolean } = $props();

  let copied = $state(false);

  async function copy() {
    try {
      await navigator.clipboard.writeText(text());
      copied = true;
      setTimeout(() => (copied = false), 1500);
    } catch {
      // clipboard unavailable (e.g. blocked by the webview): nothing to confirm
    }
  }
</script>

<button class="btn" onclick={copy} {disabled}>{copied ? "Copied" : label}</button>
