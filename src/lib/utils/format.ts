// Number and duration formatting shared by every page.

export function formatDuration(total: number): string {
  const d = Math.floor(total / 86400);
  const h = Math.floor((total % 86400) / 3600);
  const m = Math.floor((total % 3600) / 60);
  if (d) return `${d}d ${h}h`;
  if (h) return `${h}h ${m}m`;
  return `${m}m ${Math.floor(total % 60)}s`;
}

export function formatBytes(bytes: number): string {
  if (bytes <= 0) return "0 B";
  const units = ["B", "KiB", "MiB", "GiB", "TiB"];
  const i = Math.min(Math.floor(Math.log(bytes) / Math.log(1024)), units.length - 1);
  return `${(bytes / 1024 ** i).toFixed(i === 0 ? 0 : 1)} ${units[i]}`;
}

export function formatRate(bytesPerSecond: number): string {
  if (bytesPerSecond < 1) return "0 B/s";
  const units = ["B/s", "kB/s", "MB/s", "GB/s"];
  const i = Math.min(Math.floor(Math.log(bytesPerSecond) / Math.log(1000)), units.length - 1);
  const v = bytesPerSecond / 1000 ** i;
  return `${v >= 100 || i === 0 ? v.toFixed(0) : v.toFixed(1)} ${units[i]}`;
}
