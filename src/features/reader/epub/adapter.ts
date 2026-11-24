import { Book, type NavItem, type Rendition } from "epubjs";

import { readerThemeColors } from "../preferences";
import type { ReaderPreferences } from "../types";

export interface TocEntry {
  id: string;
  href: string;
  label: string;
  depth: number;
}

export interface OpenedEpub {
  book: Book;
  rendition: Rendition;
}

export async function openEpub(container: HTMLElement, data: ArrayBuffer): Promise<OpenedEpub> {
  const book = new Book({ openAs: "binary" });
  await book.open(data, "binary");
  await book.ready;

  const rendition = book.renderTo(container, {
    width: "100%",
    height: "100%",
    flow: "paginated",
    allowScriptedContent: false,
  });

  return { book, rendition };
}

export function flattenToc(items: NavItem[], depth = 0): TocEntry[] {
  const entries: TocEntry[] = [];

  for (const item of items) {
    entries.push({
      id: item.id,
      href: item.href,
      label: item.label || "未命名章节",
      depth,
    });

    if (item.subitems && item.subitems.length > 0) {
      entries.push(...flattenToc(item.subitems, depth + 1));
    }
  }

  return entries;
}

export function findTocLabel(entries: TocEntry[], href: string): string | null {
  const cleanHref = href.split("#")[0];

  const exact = entries.find((entry) => entry.href.split("#")[0] === cleanHref);
  if (exact) {
    return exact.label;
  }

  const prefix = entries.find((entry) => cleanHref.startsWith(entry.href.split("#")[0]));
  return prefix?.label ?? null;
}

export function applyReaderPreferences(rendition: Rendition, preferences: ReaderPreferences): void {
  const colors = readerThemeColors[preferences.theme];

  rendition.themes.override("color", colors.text, true);
  rendition.themes.override("background", colors.background, true);
  rendition.themes.override("line-height", String(preferences.lineHeight), true);
  rendition.themes.fontSize(`${preferences.fontSize}px`);
}
