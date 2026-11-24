import type { Tag } from "@/features/anime/types";

export type BookFormat = "epub" | "pdf" | "txt" | "markdown";
export type BookSort = "updated_desc" | "imported_desc" | "title_asc" | "size_desc";

export interface Book {
  id: number;
  title: string;
  author: string;
  format: BookFormat;
  filePath: string;
  sourcePath: string | null;
  fileSize: number;
  fileMtime: number | null;
  coverPath: string | null;
  coverUrl: string | null;
  description: string;
  category: string;
  importedAt: number;
  lastOpenedAt: number | null;
  createdAt: number;
  updatedAt: number;
  tags: Tag[];
  isFavorite: boolean;
  fileExists: boolean;
}

export interface BookUpdateInput {
  title: string;
  author: string;
  description: string;
  category: string;
  tags: string[];
}

export interface BookFilter {
  format?: BookFormat;
  category?: string;
  tagId?: number;
  favorite?: boolean;
  sort?: BookSort;
}

export interface BookImportResult {
  book: Book;
  duplicate: boolean;
}

export interface BookFileStatus {
  exists: boolean;
  path: string;
  message: string;
}
