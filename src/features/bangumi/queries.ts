import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { toast } from "sonner";

import { animeKeys } from "@/features/anime/queries";

import * as api from "./api";

export const bangumiKeys = {
  all: ["bangumi"] as const,
  tokenStatus: ["bangumi", "token-status"] as const,
  preview: (animeId: number) => ["bangumi", "preview", animeId] as const,
};

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : "操作失败，请重试";
}

export function useBangumiSearch() {
  return useMutation({
    mutationFn: (keyword: string) => api.searchBangumi(keyword),
  });
}

export function useImportBangumi() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (subjectId: number) => api.importBangumiSubject(subjectId),
    onSuccess: (anime) => {
      void queryClient.invalidateQueries({ queryKey: animeKeys.all });
      void queryClient.invalidateQueries({ queryKey: animeKeys.tags });
      toast.success(`已导入《${anime.title}》`);
    },
    onError: (error) => toast.error(errorMessage(error)),
  });
}

export function useBangumiRefreshPreview(animeId: number, enabled: boolean) {
  return useQuery({
    queryKey: bangumiKeys.preview(animeId),
    queryFn: () => api.previewBangumiRefresh(animeId),
    enabled,
    staleTime: 0,
    retry: false,
  });
}

export function useApplyBangumiRefresh(animeId: number) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: () => api.applyBangumiRefresh(animeId),
    onSuccess: (anime) => {
      void queryClient.invalidateQueries({ queryKey: animeKeys.all });
      void queryClient.invalidateQueries({ queryKey: bangumiKeys.preview(animeId) });
      toast.success(`已刷新《${anime.title}》的外部元数据`);
    },
    onError: (error) => toast.error(errorMessage(error)),
  });
}

export function useBangumiTokenStatus() {
  return useQuery({
    queryKey: bangumiKeys.tokenStatus,
    queryFn: api.getBangumiTokenStatus,
  });
}

export function useSetBangumiToken() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (token: string | null) => api.setBangumiToken(token),
    onSuccess: (status) => {
      void queryClient.invalidateQueries({ queryKey: bangumiKeys.tokenStatus });
      toast.success(status.hasToken ? "Bangumi Token 已保存" : "Bangumi Token 已清除");
    },
    onError: (error) => toast.error(errorMessage(error)),
  });
}

export function useCheckBangumiConnection() {
  return useMutation({
    mutationFn: api.checkBangumiConnection,
    onError: (error) => toast.error(errorMessage(error)),
  });
}
