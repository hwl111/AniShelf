import type { ReaderPreferences, ReaderTheme } from "./types";

export const DEFAULT_READER_PREFERENCES: ReaderPreferences = {
  fontSize: 18,
  lineHeight: 1.7,
  theme: "dark",
};

export const readerThemeLabels: Record<ReaderTheme, string> = {
  dark: "深色",
  light: "浅色",
  sepia: "护眼",
};

export const readerThemeColors: Record<
  ReaderTheme,
  { background: string; text: string; accent: string }
> = {
  dark: { background: "#101318", text: "#e6e8ee", accent: "#7c9cff" },
  light: { background: "#faf9f6", text: "#1f2937", accent: "#2563eb" },
  sepia: { background: "#f4ecd8", text: "#4b3a2a", accent: "#9a6b3f" },
};

const READER_THEMES: ReaderTheme[] = ["dark", "light", "sepia"];

export function parseReaderPreferences(json: string | null | undefined): ReaderPreferences {
  if (!json) {
    return DEFAULT_READER_PREFERENCES;
  }

  try {
    const value = JSON.parse(json) as Partial<ReaderPreferences>;
    const fontSize =
      typeof value.fontSize === "number" && Number.isFinite(value.fontSize)
        ? Math.min(36, Math.max(12, value.fontSize))
        : DEFAULT_READER_PREFERENCES.fontSize;
    const lineHeight =
      typeof value.lineHeight === "number" && Number.isFinite(value.lineHeight)
        ? Math.min(2.4, Math.max(1.2, value.lineHeight))
        : DEFAULT_READER_PREFERENCES.lineHeight;
    const theme = READER_THEMES.includes(value.theme as ReaderTheme)
      ? (value.theme as ReaderTheme)
      : DEFAULT_READER_PREFERENCES.theme;

    return { fontSize, lineHeight, theme };
  } catch {
    return DEFAULT_READER_PREFERENCES;
  }
}

export function parseCfi(locationJson: string | null | undefined): string | null {
  if (!locationJson) {
    return null;
  }

  try {
    const value = JSON.parse(locationJson) as { cfi?: unknown };
    return typeof value.cfi === "string" && value.cfi ? value.cfi : null;
  } catch {
    return null;
  }
}
