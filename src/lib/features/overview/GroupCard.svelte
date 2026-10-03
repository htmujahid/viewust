<script lang="ts">
  import { deviceUrl } from "$lib/map/model";

  import type { Group } from "./summary";

  let { group }: { group: Group } = $props();
</script>

<section class="card block">
  <h2>{group.title}</h2>
  <ul>
    {#each group.items as c (c.id)}
      <li>
        <a href={deviceUrl(c.id)}>
          <span class="name">{c.name}</span>
          {#if c.subtitle}<small class="sub">{c.subtitle}</small>{/if}
        </a>
      </li>
    {/each}
  </ul>
</section>

<style>
  .block {
    padding: var(--s-5);
  }
  h2 {
    margin-bottom: var(--s-2);
    color: var(--text-2);
    font-size: var(--fs-label);
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.08em;
  }
  h2::before {
    content: "// ";
    color: var(--accent);
  }
  ul {
    margin: 0;
    padding: 0;
    list-style: none;
  }
  li + li {
    border-top: 1px solid var(--border);
  }
  a {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: var(--s-2) var(--s-2);
    margin: 0 calc(var(--s-2) * -1);
    color: inherit;
    text-decoration: none;
  }
  a:hover {
    background: var(--surface-2);
  }
  a:hover .name {
    color: var(--accent);
  }
  .name {
    font-family: var(--font-mono);
    font-size: var(--fs-small);
    font-weight: 600;
    overflow-wrap: anywhere;
  }
  .sub {
    color: var(--text-2);
    font-size: var(--fs-small);
  }
</style>
