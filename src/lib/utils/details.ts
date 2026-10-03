import type { Detail } from "$lib/api/types";

export interface Section {
  title: string;
  rows: Detail[];
}

export function groupBySection(rows: readonly Detail[]): Section[] {
  const sections: Section[] = [];
  for (const row of rows) {
    let section = sections.find((s) => s.title === row.section);
    if (!section) sections.push((section = { title: row.section, rows: [] }));
    section.rows.push(row);
  }
  return sections;
}

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

export function sectionsToText(heading: string, sections: readonly Section[]): string {
  return [
    heading,
    ...sections.flatMap((s) => [`\n[${s.title}]`, ...s.rows.map((r) => `${r.label}: ${r.value}`)]),
  ].join("\n");
}

export const sectionSlug = (title: string) =>
  "s-" + title.toLowerCase().replace(/[^a-z0-9]+/g, "-");
