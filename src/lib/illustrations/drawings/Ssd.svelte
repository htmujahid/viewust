<!-- ssd -->
<rect class="metal" x="4" y="6" width="112" height="78" rx="7" />
<rect class="accent" x="14" y="16" width="72" height="44" rx="4" />
<rect class="soft-fill" x="21" y="24" width="38" height="4" rx="2" />
<rect class="soft-fill" x="21" y="33" width="26" height="4" rx="2" />
<rect class="screen" x="21" y="43" width="14" height="10" rx="2" />
<rect class="dark" x="90" y="68" width="24" height="12" rx="2" />
{#each [0, 1, 2, 3, 4, 5] as i}
  <rect class="gold" x={93 + i * 3.4} y="72" width="2" height="5" />
{/each}
<circle class="faint-fill" cx="12" cy="14" r="2.4" />
<circle class="faint-fill" cx="108" cy="14" r="2.4" />
<circle class="faint-fill" cx="12" cy="76" r="2.4" />
