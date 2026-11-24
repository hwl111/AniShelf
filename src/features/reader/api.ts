import { invokeCommand } from "@/lib/ipc";

import type {
  Bookmark,
  BookmarkInput,
  ProgressInput,
  ReaderSettings,
  ReadingProgress,
  TextBookInfo,
  TextChunk,
} from "./types";

export function readBookBytes(bookId: number): Promise<ArrayBuffer> {
  return invokeCommand<ArrayBuffer>("read_book_bytes", { bookId });
}

export function getTextBookInfo(bookId: number): Promise<TextBookInfo> {
  return invokeCommand<TextBookInfo>("get_text_book_info", { bookId });
}

export function readTextChunk(
  bookId: number,
  offset: number,
  maxBytes: number,
  encoding: string,
): Promise<TextChunk> {
  return invokeCommand<TextChunk>("read_text_chunk", {
    bookId,
    offset,
    maxBytes,
    encoding,
  });
}

export function getReadingProgress(bookId: number): Promise<ReadingProgress | null> {
  return invokeCommand<ReadingProgress | null>("get_reading_progress", { bookId });
}

export function saveReadingProgress(
  bookId: number,
  input: ProgressInput,
): Promise<ReadingProgress> {
  return invokeCommand<ReadingProgress>("save_reading_progress", { bookId, input });
}

export function listBookmarks(bookId: number): Promise<Bookmark[]> {
  return invokeCommand<Bookmark[]>("list_bookmarks", { bookId });
}

export function createBookmark(bookId: number, input: BookmarkInput): Promise<Bookmark> {
  return invokeCommand<Bookmark>("create_bookmark", { bookId, input });
}

export function deleteBookmark(bookmarkId: number): Promise<void> {
  return invokeCommand<void>("delete_bookmark", { bookmarkId });
}

export function getReaderSettings(scope: string): Promise<ReaderSettings | null> {
  return invokeCommand<ReaderSettings | null>("get_reader_settings", { scope });
}

export function saveReaderSettings(
  scope: string,
  preferencesJson: string,
): Promise<ReaderSettings> {
  return invokeCommand<ReaderSettings>("save_reader_settings", { scope, preferencesJson });
}

export function startReadingSession(bookId: number): Promise<number> {
  return invokeCommand<number>("start_reading_session", { bookId });
}

export function endReadingSession(
  sessionId: number,
  endLocationJson: string | null,
): Promise<void> {
  return invokeCommand<void>("end_reading_session", { sessionId, endLocationJson });
}

export function openExternalUrl(url: string): Promise<void> {
  return invokeCommand<void>("open_external_url", { url });
}
