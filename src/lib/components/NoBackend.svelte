<!-- Shown when the page is opened in a normal browser, where the Rust backend doesn't exist. -->
<main>
  <div class="card">
    <h1>Viewust needs its desktop window</h1>
    <p>
      The hardware readings come from the app's Rust backend, which only exists inside the Tauri
      window. A plain browser tab can't reach it.
    </p>
    <ul>
      <li><code>pnpm tauri dev</code> opens the real app.</li>
      <li><code>pnpm dev:mock</code> serves sample data, to work on the interface in a browser.</li>
    </ul>
  </div>
</main>

<style>
  main {
    display: grid;
    place-items: center;
    min-height: 100vh;
    padding: var(--s-5);
  }
  .card {
    max-width: 460px;
    padding: var(--s-5);
  }
  h1 {
    font-size: var(--fs-title);
    font-weight: 650;
  }
  p,
  ul {
    margin-top: var(--s-3);
    color: var(--text-2);
  }
  ul {
    padding-left: var(--s-5);
  }
  code {
    padding: 1px 6px;
    border-radius: 0;
    background: var(--surface-2);
    color: var(--text);
    font-size: 0.9em;
  }
</style>
