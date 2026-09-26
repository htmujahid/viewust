<!-- psu -->
<path class="cable" d="M138 70c8 0 8-8 8-16M138 80c10 0 10-14 10-24" />
<path class="line" d="M138 90c12 0 12-22 12-40" />
<rect class="body" x="6" y="12" width="132" height="90" rx="9" />
<circle class="dark" cx="54" cy="57" r="36" />
{#each [10, 20, 30] as r}
  <circle class="line" cx="54" cy="57" {r} />
{/each}
{#each [0, 1, 2, 3, 4, 5] as i}
  <rect
    class="faint-fill"
    x="52.5"
    y="25"
    width="3"
    height="64"
    rx="1.5"
    transform="rotate({i * 30} 54 57)"
  />
{/each}
<circle class="accent" cx="54" cy="57" r="8" />
<rect class="dark" x="104" y="24" width="26" height="18" rx="3" />
<rect class="accent" x="112" y="28" width="10" height="10" rx="2" />
<rect class="accent soft" x="102" y="52" width="30" height="5" rx="2" />
<rect class="faint" x="102" y="63" width="30" height="3" rx="1.5" />
<rect class="faint" x="102" y="72" width="22" height="3" rx="1.5" />
