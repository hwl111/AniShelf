import { useMutation, useQuery, useQueryClient, type QueryClient } from "@tanstack/react-query";
import { toast } from "sonner";

import * as api from "./api";
import type { AnimeFilter, AnimeInput, EpisodeInput, WatchRecordInput } from "./types";

export const animeKeys = {
  all: ["anime"] as const,
  list: (filter: AnimeFilter) => [...animeKeys.all, "list", filter] as const,
  detail: (id: number) => [...animeKeys.all, "detail", id] as const,
  tags: ["tags"] as const,
  episodes: (animeId: number) => [...animeKeys.all, "episodes", animeId] as const,
  watchRecords: (animeId: number) => [...animeKeys.all, "watch-records", animeId] as const,
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

export function useEpisodes(animeId: number) {
  return useQuery({
    queryKey: animeKeys.episodes(animeId),
    queryFn: () => api.listEpisodes(animeId),
  });
}

export function useWatchRecords(animeId: number) {
  return useQuery({
    queryKey: animeKeys.watchRecords(animeId),
    queryFn: () => api.listWatchRecords(animeId),
  });
}

function invalidateAnimeProgress(queryClient: QueryClient, animeId: number) {
  void queryClient.invalidateQueries({ queryKey: animeKeys.episodes(animeId) });
  void queryClient.invalidateQueries({ queryKey: animeKeys.watchRecords(animeId) });
  void queryClient.invalidateQueries({ queryKey: animeKeys.all });
}

export function useCreateEpisode(animeId: number) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (input: EpisodeInput) => api.createEpisode(animeId, input),
    onSuccess: (episode) => {
      invalidateAnimeProgress(queryClient, animeId);
      toast.success(`已添加第 ${episode.episodeNumber} 集`);
    },
    onError: (error) => toast.error(errorMessage(error)),
  });
}

export function useUpdateEpisode(animeId: number) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ id, input }: { id: number; input: EpisodeInput }) =>
      api.updateEpisode(id, input),
    onSuccess: (episode) => {
      invalidateAnimeProgress(queryClient, animeId);
      toast.success(`已更新第 ${episode.episodeNumber} 集`);
    },
    onError: (error) => toast.error(errorMessage(error)),
  });
}

export function useDeleteEpisode(animeId: number) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (id: number) => api.deleteEpisode(id),
    onSuccess: () => {
      invalidateAnimeProgress(queryClient, animeId);
      toast.success("单集已删除");
    },
    onError: (error) => toast.error(errorMessage(error)),
  });
}

export function useSetEpisodeWatched(animeId: number) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({
      id,
      watched,
      watchedAt,
      comment,
    }: {
      id: number;
      watched: boolean;
      watchedAt: number | null;
      comment: string | null;
    }) => api.setEpisodeWatched(id, watched, watchedAt, comment),
    onSuccess: (episode) => {
      invalidateAnimeProgress(queryClient, animeId);
      toast.success(
        episode.isWatched
          ? `已标记第 ${episode.episodeNumber} 集为已看`
          : `已标记第 ${episode.episodeNumber} 集为未看`,
      );
    },
    onError: (error) => toast.error(errorMessage(error)),
  });
}

export function useAddWatchRecord(animeId: number) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (input: WatchRecordInput) => api.addWatchRecord(animeId, input),
    onSuccess: (record) => {
      invalidateAnimeProgress(queryClient, animeId);
      toast.success(`已记录第 ${record.episodeNumber} 集`);
    },
    onError: (error) => toast.error(errorMessage(error)),
  });
}

export function useUpdateWatchRecord(animeId: number) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({
      id,
      watchedAt,
      comment,
    }: {
      id: number;
      watchedAt: number | null;
      comment: string;
    }) => api.updateWatchRecord(id, watchedAt, comment),
    onSuccess: () => {
      invalidateAnimeProgress(queryClient, animeId);
      toast.success("观看记录已更新");
    },
    onError: (error) => toast.error(errorMessage(error)),
  });
}

export function useDeleteWatchRecord(animeId: number) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (id: number) => api.deleteWatchRecord(id),
    onSuccess: () => {
      invalidateAnimeProgress(queryClient, animeId);
      toast.success("观看记录已删除");
    },
    onError: (error) => toast.error(errorMessage(error)),
  });
}
