/**
 * Runs `task` every `everyMs` while `isActive()` is true, starting at once.
 *
 * A run never overlaps the previous one: if a read is still in flight when the
 * timer fires, that tick is skipped rather than queued behind it.
 */
export class Poller {
  #timer: ReturnType<typeof setInterval> | null = null;
  #busy = false;

  constructor(
    private readonly task: () => Promise<void>,
    private readonly everyMs: number,
    private readonly isActive: () => boolean = () => true,
  ) {}

  start(): void {
    this.stop();
    void this.#run();
    this.#timer = setInterval(() => {
      if (this.isActive()) void this.#run();
    }, this.everyMs);
  }

  stop(): void {
    if (this.#timer) clearInterval(this.#timer);
    this.#timer = null;
  }

  async #run(): Promise<void> {
    if (this.#busy) return;
    this.#busy = true;
    try {
      await this.task();
    } finally {
      this.#busy = false;
    }
  }
}
