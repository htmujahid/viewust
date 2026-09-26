import type { Detail } from "$lib/api/types";

export interface Section {
  title: string;
  rows: Detail[];
}

/** Groups flat rows by their `section`, in the order each section first appears. */
export function groupBySection(rows: readonly Detail[]): Section[] {
  const sections: Section[] = [];
  for (const row of rows) {
    let section = sections.find((s) => s.title === row.section);
    if (!section) sections.push((section = { title: row.section, rows: [] }));
    section.rows.push(row);
  }
  return sections;
}

/**
 * Puts the sections named in `priority` first, in that order, and leaves the
 * rest in their original order after them.
 */
export function sortSections(sections: readonly Section[], priority: readonly string[]): Section[] {
  const rank = (title: string) => {
    const i = priority.indexOf(title);
    return i === -1 ? priority.length : i;
  };
  return sections
    .map((section, index) => ({ section, index }))
    .sort((a, b) => rank(a.section.title) - rank(b.section.title) || a.index - b.index)
    .map((x) => x.section);
}

/** Plain text for the clipboard. */
export function sectionsToText(heading: string, sections: readonly Section[]): string {
  return [
    heading,
    ...sections.flatMap((s) => [`\n[${s.title}]`, ...s.rows.map((r) => `${r.label}: ${r.value}`)]),
  ].join("\n");
}

/** An id safe to use in a URL fragment, for jumping to a section. */
export const sectionSlug = (title: string) =>
  "s-" + title.toLowerCase().replace(/[^a-z0-9]+/g, "-");
