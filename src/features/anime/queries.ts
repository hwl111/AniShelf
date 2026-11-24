import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { toast } from "sonner";

import * as api from "./api";
import type { AnimeFilter, AnimeInput } from "./types";

export const animeKeys = {
  all: ["anime"] as const,
  list: (filter: AnimeFilter) => [...animeKeys.all, "list", filter] as const,
  detail: (id: number) => [...animeKeys.all, "detail", id] as const,
  tags: ["tags"] as const,
};

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : "操作失败，请重试";
}

export function useAnimeList(filter: AnimeFilter) {
  return useQuery({
    queryKey: animeKeys.list(filter),
    queryFn: () => api.listAnime(filter),
  });
}

export function useAnime(id: number | undefined) {
  return useQuery({
    queryKey: animeKeys.detail(id ?? 0),
    queryFn: () => api.getAnime(id as number),
    enabled: id !== undefined,
  });
}

export function useAnimeTags() {
  return useQuery({
    queryKey: animeKeys.tags,
    queryFn: api.listTags,
  });
}

export function useCreateAnime() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (input: AnimeInput) => api.createAnime(input),
    onSuccess: (anime) => {
      void queryClient.invalidateQueries({ queryKey: animeKeys.all });
      void queryClient.invalidateQueries({ queryKey: animeKeys.tags });
      toast.success(`已添加《${anime.title}》`);
    },
    onError: (error) => toast.error(errorMessage(error)),
  });
}

export function useUpdateAnime() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ id, input }: { id: number; input: AnimeInput }) => api.updateAnime(id, input),
    onSuccess: (anime) => {
      void queryClient.invalidateQueries({ queryKey: animeKeys.all });
      void queryClient.invalidateQueries({ queryKey: animeKeys.tags });
      toast.success(`已更新《${anime.title}》`);
    },
    onError: (error) => toast.error(errorMessage(error)),
  });
}

export function useDeleteAnime() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (id: number) => api.deleteAnime(id),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: animeKeys.all });
      toast.success("番剧已删除");
    },
    onError: (error) => toast.error(errorMessage(error)),
  });
}

export function useSetAnimeFavorite() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ id, favorite }: { id: number; favorite: boolean }) =>
      api.setAnimeFavorite(id, favorite),
    onSuccess: (anime) => {
      void queryClient.invalidateQueries({ queryKey: animeKeys.all });
      toast.success(anime.isFavorite ? "已加入收藏" : "已取消收藏");
    },
    onError: (error) => toast.error(errorMessage(error)),
  });
}
