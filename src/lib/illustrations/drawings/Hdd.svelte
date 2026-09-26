<!-- hdd -->
<rect class="metal" x="4" y="4" width="132" height="92" rx="7" />
<circle class="body" cx="62" cy="50" r="38" />
<circle class="screen" cx="62" cy="50" r="30" />
<circle class="line" cx="62" cy="50" r="22" />
<circle class="line" cx="62" cy="50" r="14" />
<circle class="accent" cx="62" cy="50" r="7" />
<circle class="soft-fill" cx="62" cy="50" r="2.5" />
<path class="band thin" d="M118 18 L84 38" />
<circle class="dark" cx="118" cy="18" r="5" />
<rect class="accent soft" x="104" y="60" width="24" height="14" rx="3" />
<rect class="dark" x="104" y="82" width="26" height="9" rx="2" />
{#each [0, 1, 2, 3, 4] as i}
  <rect class="gold" x={107 + i * 4.4} y="85" width="2.4" height="4" />
{/each}
