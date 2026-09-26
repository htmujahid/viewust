<!-- keyboard -->
<rect class="body" x="10" y="24" width="140" height="54" rx="8" />
{#each [0, 1, 2] as row}
  {#each Array(row === 2 ? 9 : 11) as _, i}
    <rect
      class="key"
      x={(row === 2 ? 25 : 18 + row * 3) + i * 11.4}
      y={31 + row * 11}
      width="9"
      height="8"
      rx="2"
    />
  {/each}
{/each}
<rect class="key" x="36" y="64" width="88" height="8" rx="2" />
<rect class="accent" x="18" y="64" width="14" height="8" rx="2" />
<rect class="accent" x="128" y="64" width="14" height="8" rx="2" />
