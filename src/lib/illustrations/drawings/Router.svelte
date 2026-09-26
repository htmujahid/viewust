<!-- router -->
<path class="cable" d="M34 40 L24 8" />
<path class="cable" d="M116 40 L126 8" />
<circle class="accent" cx="24" cy="8" r="4" />
<circle class="accent" cx="126" cy="8" r="4" />
<path class="wave" d="M60 30c8-9 22-9 30 0M50 20c14-16 36-16 50 0" />
<rect class="body" x="6" y="40" width="138" height="48" rx="9" />
{#each [0, 1, 2, 3] as i}
  <rect class="dark" x={20 + i * 22} y="66" width="16" height="12" rx="2" />
  <rect class="gold" x={24 + i * 22} y="70" width="8" height="3" rx="1" />
{/each}
<circle class="led" cx="24" cy="52" r="3" />
<circle class="led" cx="36" cy="52" r="3" />
<circle class="faint-fill" cx="48" cy="52" r="3" />
<rect class="faint" x="100" y="49" width="30" height="4" rx="2" />
