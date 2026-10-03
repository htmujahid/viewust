{#each [0, 1, 2, 3, 4, 5, 6, 7] as i}
  <rect class="gold" x={17 + i * 9.5} y="2" width="5" height="7" rx="1.5" />
  <rect class="gold" x={17 + i * 9.5} y="91" width="5" height="7" rx="1.5" />
  <rect class="gold" x="2" y={17 + i * 9.5} width="7" height="5" rx="1.5" />
  <rect class="gold" x="91" y={17 + i * 9.5} width="7" height="5" rx="1.5" />
{/each}
<rect class="body" x="8" y="8" width="84" height="84" rx="10" />
<rect class="metal" x="21" y="21" width="58" height="58" rx="6" />
<rect class="faint" x="30" y="33" width="40" height="4" rx="2" />
<rect class="faint" x="30" y="43" width="28" height="4" rx="2" />
<rect class="accent" x="30" y="57" width="18" height="10" rx="3" />
<path class="accent" d="M12 78v10h10z" />
