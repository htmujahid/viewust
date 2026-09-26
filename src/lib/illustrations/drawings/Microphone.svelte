<!-- microphone -->
<path class="band thin" d="M54 36v8a26 26 0 0 0 52 0v-8" />
<path class="line" d="M80 70v14" />
<rect class="body" x="56" y="83" width="48" height="7" rx="3.5" />
<rect class="body" x="62" y="6" width="36" height="52" rx="18" />
{#each [16, 23, 30, 44, 51] as y}
  <rect class="faint" x="69" {y} width="22" height="2" rx="1" />
{/each}
<rect class="accent" x="62" y="34" width="36" height="6" />
<circle class="led" cx="80" cy="37" r="1.6" />
