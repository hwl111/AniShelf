import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { toast } from "sonner";

import * as api from "./api";
import type { BookFilter, BookUpdateInput } from "./types";

export const bookKeys = {
  all: ["books"] as const,
  list: (filter: BookFilter) => [...bookKeys.all, "list", filter] as const,
  detail: (id: number) => [...bookKeys.all, "detail", id] as const,
  tags: ["tags"] as const,
};

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : "操作失败，请重试";
}

export function useBooks(filter: BookFilter) {
  return useQuery({
    queryKey: bookKeys.list(filter),
    queryFn: () => api.listBooks(filter),
  });
}

export function useBook(id: number | undefined) {
  return useQuery({
    queryKey: bookKeys.detail(id ?? 0),
    queryFn: () => api.getBook(id as number),
    enabled: id !== undefined,
  });
}

export function useBookTags() {
  return useQuery({
    queryKey: bookKeys.tags,
    queryFn: api.listBookTags,
  });
}

export function useImportBook() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ path, copyToLibrary }: { path: string; copyToLibrary: boolean }) =>
      api.importBook(path, copyToLibrary),
    onSuccess: (result) => {
      void queryClient.invalidateQueries({ queryKey: bookKeys.all });
      void queryClient.invalidateQueries({ queryKey: bookKeys.tags });
      toast.success(
        result.duplicate ? `已存在《${result.book.title}》` : `已导入《${result.book.title}》`,
      );
    },
    onError: (error) => toast.error(errorMessage(error)),
  });
}

export function useUpdateBook() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ id, input }: { id: number; input: BookUpdateInput }) =>
      api.updateBook(id, input),
    onSuccess: (book) => {
      void queryClient.invalidateQueries({ queryKey: bookKeys.all });
      toast.success(`已更新《${book.title}》`);
    },
    onError: (error) => toast.error(errorMessage(error)),
  });
}

export function useDeleteBook() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (id: number) => api.deleteBook(id),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: bookKeys.all });
      toast.success("书籍已删除");
    },
    onError: (error) => toast.error(errorMessage(error)),
  });
}

export function useSetBookFavorite() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ id, favorite }: { id: number; favorite: boolean }) =>
      api.setBookFavorite(id, favorite),
    onSuccess: (book) => {
      void queryClient.invalidateQueries({ queryKey: bookKeys.all });
      toast.success(book.isFavorite ? "已加入收藏" : "已取消收藏");
    },
    onError: (error) => toast.error(errorMessage(error)),
  });
}

export function useRelocateBook() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ id, newPath }: { id: number; newPath: string }) => api.relocateBook(id, newPath),
    onSuccess: (book) => {
      void queryClient.invalidateQueries({ queryKey: bookKeys.all });
      toast.success(`已重新定位《${book.title}》`);
    },
    onError: (error) => toast.error(errorMessage(error)),
  });
}
