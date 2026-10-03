<path class="cable" d="M80 42V24c0-9 8-12 22-12" />
<rect class="body" x="14" y="42" width="132" height="34" rx="10" />
{#each [0, 1, 2, 3] as i}
  <rect class="dark" x={26 + i * 29} y="54" width="20" height="11" rx="2" />
  <rect class="accent soft" x={30 + i * 29} y="58" width="12" height="3" rx="1" />
{/each}
<circle class="led" cx="132" cy="49" r="2.5" />
