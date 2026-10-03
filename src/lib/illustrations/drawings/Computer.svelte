<rect class="body" x="50" y="4" width="60" height="90" rx="7" />
<rect class="dark" x="57" y="11" width="46" height="9" rx="2" />
<rect class="faint" x="62" y="14.5" width="26" height="2" rx="1" />
<rect class="dark" x="57" y="24" width="46" height="9" rx="2" />
<rect class="faint" x="62" y="27.5" width="18" height="2" rx="1" />
<circle class="accent" cx="80" cy="47" r="6.5" />
<circle class="glint" cx="80" cy="47" r="2.4" />
<circle class="led" cx="63" cy="47" r="2" />
<circle class="faint-fill" cx="97" cy="47" r="2" />
<rect class="dark" x="63" y="58" width="10" height="3.5" rx="1.5" />
<rect class="dark" x="87" y="58" width="10" height="3.5" rx="1.5" />
{#each [0, 1, 2, 3, 4] as i}
  <rect class="faint" x="58" y={68 + i * 4.6} width="44" height="2" rx="1" />
{/each}
<rect class="faint-fill" x="56" y="94" width="10" height="3" rx="1.5" />
<rect class="faint-fill" x="94" y="94" width="10" height="3" rx="1.5" />
