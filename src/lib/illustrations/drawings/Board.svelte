<rect class="pcb" x="4" y="4" width="152" height="152" rx="9" />
<rect class="dark" x="4" y="32" width="11" height="62" rx="2" />
<rect class="faint" x="7" y="38" width="5" height="4" rx="1" />
<rect class="faint" x="7" y="46" width="5" height="4" rx="1" />
<rect class="faint" x="7" y="54" width="5" height="4" rx="1" />
<rect class="faint" x="7" y="62" width="5" height="4" rx="1" />
<rect class="faint" x="7" y="70" width="5" height="4" rx="1" />
<rect class="faint-fill" x="48" y="12" width="48" height="15" rx="2" />
<rect class="faint-fill" x="22" y="38" width="17" height="48" rx="2" />
{#each [0, 1, 2, 3, 4, 5] as i}
  <rect class="body" x={52 + i * 7} y="15" width="3" height="9" rx="1" />
{/each}
<rect class="dark" x="46" y="36" width="52" height="52" rx="4" />
<rect class="screen" x="53" y="43" width="38" height="38" rx="3" />
<rect class="accent" x="58" y="48" width="12" height="3" rx="1.5" />
{#each [0, 1, 2, 3] as i}
  <rect class="dark" x={108 + i * 9} y="20" width="6" height="72" rx="2" />
  <rect class="accent soft" x={109 + i * 9} y="22" width="4" height="12" rx="1" />
{/each}
<rect class="metal" x="143" y="22" width="10" height="38" rx="2" />
<rect class="dark" x="95" y="100" width="26" height="26" rx="3" />
<rect class="accent" x="102" y="107" width="12" height="12" rx="2" />
<rect class="dark" x="22" y="102" width="64" height="7" rx="2" />
<rect class="dark" x="22" y="118" width="64" height="7" rx="2" />
<rect class="dark" x="22" y="134" width="64" height="7" rx="2" />
<rect class="gold" x="124" y="132" width="26" height="9" rx="2" />
<circle class="faint-fill" cx="14" cy="14" r="2.6" />
<circle class="faint-fill" cx="146" cy="14" r="2.6" />
<circle class="faint-fill" cx="14" cy="146" r="2.6" />
<circle class="faint-fill" cx="146" cy="146" r="2.6" />
