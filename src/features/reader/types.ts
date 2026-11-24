export type ReaderTheme = "dark" | "light" | "sepia";

export interface ReaderPreferences {
  fontSize: number;
  lineHeight: number;
  theme: ReaderTheme;
}

export interface ReadingProgress {
  bookId: number;
  locationType: string;
  locationJson: string;
  percent: number;
  updatedAt: number;
}

export interface Bookmark {
  id: number;
  bookId: number;
  locationType: string;
  locationJson: string;
  label: string;
  createdAt: number;
}

export interface ReaderSettings {
  scope: string;
  preferencesJson: string;
  updatedAt: number;
}

export interface ProgressInput {
  locationType: string;
  locationJson: string;
  percent: number;
}

export interface BookmarkInput {
  locationType: string;
  locationJson: string;
  label: string;
}

export interface TextBookInfo {
  bookId: number;
  totalBytes: number;
  encoding: string;
  hasBom: boolean;
  warning: string | null;
}

export interface TextChunk {
  text: string;
  startOffset: number;
  nextOffset: number;
  totalBytes: number;
  eof: boolean;
  hadErrors: boolean;
}
