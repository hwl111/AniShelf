import type { Tag } from "@/features/anime/types";
import { invokeCommand } from "@/lib/ipc";

import type { Book, BookFileStatus, BookFilter, BookImportResult, BookUpdateInput } from "./types";

export function listBooks(filter: BookFilter = {}): Promise<Book[]> {
  return invokeCommand<Book[]>("list_books", { filter });
}

export function getBook(id: number): Promise<Book> {
  return invokeCommand<Book>("get_book", { id });
}

export function importBook(path: string, copyToLibrary: boolean): Promise<BookImportResult> {
  return invokeCommand<BookImportResult>("import_book", { path, copyToLibrary });
}

export function updateBook(id: number, input: BookUpdateInput): Promise<Book> {
  return invokeCommand<Book>("update_book", { id, input });
}

export function deleteBook(id: number): Promise<void> {
  return invokeCommand<void>("delete_book", { id });
}

export function saveGeneratedCover(id: number, dataUrl: string): Promise<Book> {
  return invokeCommand<Book>("save_generated_cover", { id, dataUrl });
}

export function setBookFavorite(id: number, favorite: boolean): Promise<Book> {
  return invokeCommand<Book>("set_book_favorite", { id, favorite });
}

export function checkBookFile(id: number): Promise<BookFileStatus> {
  return invokeCommand<BookFileStatus>("check_book_file", { id });
}

export function relocateBook(id: number, newPath: string): Promise<Book> {
  return invokeCommand<Book>("relocate_book", { id, newPath });
}

export function listBookTags(): Promise<Tag[]> {
  return invokeCommand<Tag[]>("list_tags");
}
