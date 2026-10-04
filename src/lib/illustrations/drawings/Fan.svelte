<rect class="body" x="6" y="6" width="98" height="98" rx="10" />
{#each [14, 96] as x}
  {#each [14, 96] as y}
    <circle class="faint" cx={x} cy={y} r="3.5" />
  {/each}
{/each}
<circle class="dark" cx="55" cy="55" r="41" />
<circle class="line" cx="55" cy="55" r="41" />
{#each [0, 1, 2, 3, 4, 5, 6] as i}
  <path
    class="faint-fill"
    d="M55 55 C 55 34, 66 28, 72 22 C 60 24, 50 32, 50 44 Z"
    transform="rotate({i * 51.4} 55 55)"
  />
{/each}
<circle class="accent" cx="55" cy="55" r="10" />
<circle class="dark" cx="55" cy="55" r="4" />
