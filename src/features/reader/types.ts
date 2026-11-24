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
