<rect class="metal" x="2" y="4" width="16" height="78" rx="2" />
<circle class="ok-fill" cx="10" cy="20" r="4" />
<circle class="accent" cx="10" cy="34" r="4" />
<circle class="gold" cx="10" cy="48" r="4" />
<circle class="faint-fill" cx="10" cy="62" r="4" />
<rect class="pcb" x="18" y="8" width="114" height="64" rx="4" />
<rect class="dark" x="52" y="24" width="32" height="26" rx="3" />
<rect class="accent" x="60" y="31" width="16" height="12" rx="2" />
{#each [0, 1, 2] as i}
  <circle class="faint-fill" cx={98 + i * 10} cy="22" r="3.4" />
{/each}
<rect class="faint" x="28" y="56" width="50" height="3" rx="1.5" />
{#each Array(16) as _, i}
  <rect class="gold" x={34 + i * 3.6} y="72" width="2.4" height="10" />
{/each}
