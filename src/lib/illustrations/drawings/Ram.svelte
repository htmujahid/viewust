<!-- ram -->
<rect class="pcb" x="2" y="4" width="156" height="34" rx="3" />
<rect class="accent" x="2" y="4" width="156" height="25" rx="4" />
{#each [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11] as i}
  <rect class="soft-fill" x={10 + i * 12.5} y="8" width="2.4" height="17" rx="1.2" />
{/each}
{#each Array(24) as _, i}
  <rect class="gold" x={5 + i * 6.3} y="31" width="4" height="7" rx="1" />
{/each}
<rect class="screen" x="77" y="30" width="4" height="9" />
