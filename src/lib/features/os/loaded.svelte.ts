import { errorMessage } from "$lib/api/client";

/** One list that is read the first time its view appears, and again on request. */
export class Loaded<T> {
  data = $state.raw<T | null>(null);
  error = $state<string | null>(null);
  loading = $state(false);

  constructor(private readonly read: () => Promise<T>) {}

  load = async () => {
    this.loading = true;
    try {
      this.data = await this.read();
      this.error = null;
    } catch (e) {
      this.error = errorMessage(e);
    } finally {
      this.loading = false;
    }
  };

  ensure = () => {
    if (this.data === null && !this.loading) void this.load();
  };
}
