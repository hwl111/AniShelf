import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { toast } from "sonner";

import * as api from "./api";

export const backupKeys = {
  all: ["backups"] as const,
  list: () => [...backupKeys.all, "list"] as const,
  appInfo: ["app-info"] as const,
};

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : "操作失败，请重试";
}

export function useBackups() {
  return useQuery({
    queryKey: backupKeys.list(),
    queryFn: api.listBackups,
    staleTime: 0,
    refetchOnMount: "always",
  });
}

export function useAppInfo() {
  return useQuery({
    queryKey: backupKeys.appInfo,
    queryFn: api.getAppInfo,
  });
}

export function useCreateBackup() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (destination: string | null) => api.createBackup(destination),
    onSuccess: (backup) => {
      void queryClient.invalidateQueries({ queryKey: backupKeys.all });
      toast.success(`备份已创建：${backup.fileName}`);
    },
    onError: (error) => toast.error(errorMessage(error)),
  });
}

export function useStageRestore() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (sourcePath: string) => api.stageRestore(sourcePath),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: backupKeys.all });
    },
    onError: (error) => toast.error(errorMessage(error)),
  });
}

export function useCancelPendingRestore() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: api.cancelPendingRestore,
    onSuccess: (cancelled) => {
      void queryClient.invalidateQueries({ queryKey: backupKeys.all });
      toast.success(cancelled ? "已取消待恢复的数据" : "没有待恢复的数据");
    },
    onError: (error) => toast.error(errorMessage(error)),
  });
}
