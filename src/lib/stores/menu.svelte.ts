export interface MenuItem {
  label: string;
  onselect?: () => unknown;
  /** Secondary text on the right, such as the signal a command sends. */
  hint?: string;
  danger?: boolean;
  disabled?: boolean;
  separator?: boolean;
}

export const separator: MenuItem = { label: "", separator: true };

class MenuStore {
  items = $state.raw<MenuItem[]>([]);
  x = $state(0);
  y = $state(0);
  title = $state<string | null>(null);
  isOpen = $state(false);

  /** Opens the menu at the pointer. Pass a `title` to say what it is about. */
  open = (event: MouseEvent, items: MenuItem[], title: string | null = null) => {
    event.preventDefault();
    event.stopPropagation();
    this.items = tidy(items);
    this.title = title;
    this.x = event.clientX;
    this.y = event.clientY;
    this.isOpen = this.items.length > 0;
  };

  close = () => {
    this.isOpen = false;
  };
}

/** Drops leading, trailing and doubled separators so callers can build lists freely. */
export function tidy(items: readonly MenuItem[]): MenuItem[] {
  const out: MenuItem[] = [];
  for (const item of items) {
    if (item.separator && (out.length === 0 || out[out.length - 1].separator)) continue;
    out.push(item);
  }
  while (out.length && out[out.length - 1].separator) out.pop();
  return out;
}

export const menu = new MenuStore();
