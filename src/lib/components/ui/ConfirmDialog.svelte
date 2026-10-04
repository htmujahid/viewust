<script lang="ts">
  import { confirmation } from "$lib/stores/confirm.svelte";

  const o = $derived(confirmation.options);

  function onkeydown(e: KeyboardEvent) {
    if (o && e.key === "Escape") {
      e.stopPropagation();
      confirmation.answer(false);
    }
  }
</script>

<svelte:window onkeydowncapture={onkeydown} />

{#if o}
  <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
  <div class="scrim" onclick={() => confirmation.answer(false)}>
    <div
      class="card dialog"
      role="alertdialog"
      aria-modal="true"
      aria-labelledby="confirm-title"
      tabindex="-1"
      onclick={(e) => e.stopPropagation()}
    >
      <h2 id="confirm-title">{o.title}</h2>
      {#if o.body}<p>{o.body}</p>{/if}
      <div class="buttons">
        <button class="btn" onclick={() => confirmation.answer(false)}>Cancel</button>
        <!-- svelte-ignore a11y_autofocus -->
        <button
          class="btn confirm"
          class:danger={o.danger}
          autofocus
          onclick={() => confirmation.answer(true)}
        >
          {o.confirmLabel ?? "Confirm"}
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .scrim {
    position: fixed;
    inset: 0;
    z-index: 110;
    display: grid;
    place-items: center;
    background: rgb(0 0 0 / 0.55);
  }
  .dialog {
    width: min(440px, calc(100vw - 32px));
    margin: 0;
    padding: var(--s-4) var(--s-5);
    box-shadow: 0 16px 48px rgb(0 0 0 / 0.6);
  }
  h2 {
    margin-bottom: var(--s-2);
    font-size: 17px;
    font-weight: 650;
  }
  p {
    color: var(--text-2);
  }
  .buttons {
    display: flex;
    justify-content: flex-end;
    gap: var(--s-2);
    margin-top: var(--s-5);
  }
  .confirm.danger {
    border-color: var(--danger);
    background: var(--danger-soft);
    color: var(--danger);
  }
</style>
