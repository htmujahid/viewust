<rect class="metal" x="0" y="6" width="7" height="82" rx="2" />
<rect class="dark" x="6" y="6" width="188" height="64" rx="9" />
<rect class="accent" x="16" y="9" width="170" height="2.4" rx="1.2" />
{#each [58, 142] as cx}
  <circle class="body" {cx} cy="40" r="26" />
  <circle class="dark" {cx} cy="40" r="22" />
  {#each [0, 1, 2, 3, 4, 5, 6] as i}
    <ellipse
      class="faint-fill"
      cx="0"
      cy="-11"
      rx="5.5"
      ry="11"
      transform="translate({cx} 40) rotate({i * 51.4})"
    />
  {/each}
  <circle class="accent" {cx} cy="40" r="6" />
{/each}
<rect class="body" x="150" y="0" width="34" height="8" rx="2" />
<rect class="faint" x="84" y="30" width="30" height="2" rx="1" />
<rect class="faint" x="84" y="36" width="30" height="2" rx="1" />
<rect class="faint" x="84" y="42" width="30" height="2" rx="1" />
<rect class="pcb" x="14" y="70" width="170" height="10" />
{#each Array(18) as _, i}
  <rect class="gold" x={22 + i * 3.6} y="80" width="2.4" height="10" />
{/each}
{#each Array(10) as _, i}
  <rect class="gold" x={92 + i * 3.6} y="80" width="2.4" height="10" />
{/each}
