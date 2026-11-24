import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { toast } from "sonner";

import * as api from "./api";
import type { BookmarkInput } from "./types";

export const readerKeys = {
  progress: (bookId: number) => ["reader", "progress", bookId] as const,
  bookmarks: (bookId: number) => ["reader", "bookmarks", bookId] as const,
  settings: (scope: string) => ["reader", "settings", scope] as const,
  textInfo: (bookId: number) => ["reader", "text-info", bookId] as const,
};

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : "操作失败，请重试";
}

export function useReadingProgress(bookId: number) {
  return useQuery({
    queryKey: readerKeys.progress(bookId),
    queryFn: () => api.getReadingProgress(bookId),
    enabled: Number.isInteger(bookId) && bookId > 0,
    staleTime: 0,
    refetchOnMount: "always",
  });
}

export function useBookmarks(bookId: number) {
  return useQuery({
    queryKey: readerKeys.bookmarks(bookId),
    queryFn: () => api.listBookmarks(bookId),
    enabled: Number.isInteger(bookId) && bookId > 0,
  });
}

export function useReaderSettings(scope: string) {
  return useQuery({
    queryKey: readerKeys.settings(scope),
    queryFn: () => api.getReaderSettings(scope),
  });
}

export function useTextBookInfo(bookId: number) {
  return useQuery({
    queryKey: readerKeys.textInfo(bookId),
    queryFn: () => api.getTextBookInfo(bookId),
    enabled: Number.isInteger(bookId) && bookId > 0,
    staleTime: 5 * 60_000,
  });
}

export function useCreateBookmark(bookId: number) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (input: BookmarkInput) => api.createBookmark(bookId, input),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: readerKeys.bookmarks(bookId) });
      toast.success("书签已添加");
    },
    onError: (error) => toast.error(errorMessage(error)),
  });
}

export function useDeleteBookmark(bookId: number) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (bookmarkId: number) => api.deleteBookmark(bookmarkId),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: readerKeys.bookmarks(bookId) });
      toast.success("书签已删除");
    },
    onError: (error) => toast.error(errorMessage(error)),
  });
}
