<!-- nic -->
<rect class="metal" x="2" y="4" width="16" height="78" rx="2" />
<rect class="dark" x="5" y="28" width="11" height="14" rx="1.5" />
<circle class="led" cx="8" cy="34" r="1.2" />
<rect class="pcb" x="18" y="8" width="114" height="64" rx="4" />
<rect class="dark" x="54" y="22" width="30" height="30" rx="3" />
<rect class="accent" x="62" y="30" width="14" height="14" rx="2" />
{#each [0, 1, 2, 3, 4] as i}
  <circle class="faint-fill" cx={30 + i * 6} cy="20" r="2.2" />
{/each}
<rect class="faint" x="96" y="20" width="26" height="3" rx="1.5" />
<rect class="faint" x="96" y="28" width="20" height="3" rx="1.5" />
{#each Array(16) as _, i}
  <rect class="gold" x={34 + i * 3.6} y="72" width="2.4" height="10" />
{/each}
