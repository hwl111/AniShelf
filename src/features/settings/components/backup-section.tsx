import { open, save } from "@tauri-apps/plugin-dialog";
import { Download, FolderOpen, RefreshCw, RotateCcw, Upload } from "lucide-react";
import { useState } from "react";
import { toast } from "sonner";

import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { formatBytes } from "@/lib/format";

import { restartApp } from "../api";
import {
  useAppInfo,
  useBackups,
  useCancelPendingRestore,
  useCreateBackup,
  useStageRestore,
} from "../queries";
import type { BackupInfo, RestoreResult } from "../types";

const kindLabels: Record<string, string> = {
  manual: "手动备份",
  pre_restore: "恢复前自动备份",
  pre_migration: "迁移前自动备份",
  other: "其他",
};

interface RestoreTarget {
  path: string;
  fileName: string;
  info?: BackupInfo;
}

function formatTime(value: number): string {
  if (!value) {
    return "—";
  }
  return new Date(value).toLocaleString("zh-CN", { hour12: false });
}

export function BackupSection() {
  const backupsQuery = useBackups();
  const appInfoQuery = useAppInfo();
  const createMutation = useCreateBackup();
  const restoreMutation = useStageRestore();
  const cancelMutation = useCancelPendingRestore();

  const [restoreTarget, setRestoreTarget] = useState<RestoreTarget | null>(null);
  const [restoreResult, setRestoreResult] = useState<RestoreResult | null>(null);

  const backups = backupsQuery.data ?? [];

  async function handleSaveAs() {
    try {
      const destination = await save({
        title: "另存数据库备份",
        defaultPath: `AniShelf-backup-${new Date().toISOString().slice(0, 19).replace(/[:T]/g, "-")}.sqlite`,
        filters: [{ name: "SQLite 数据库", extensions: ["sqlite"] }],
      });
      if (typeof destination === "string") {
        createMutation.mutate(destination);
      }
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "无法打开保存对话框");
    }
  }

  async function handleChooseRestoreFile() {
    try {
      const selected = await open({
        multiple: false,
        directory: false,
        title: "选择数据库备份",
        filters: [{ name: "SQLite 数据库", extensions: ["sqlite", "sqlite3", "db"] }],
      });
      if (typeof selected === "string") {
        const fileName = selected.split(/[\\/]/).pop() ?? selected;
        setRestoreTarget({ path: selected, fileName });
      }
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "无法打开文件选择器");
    }
  }

  function handleConfirmRestore() {
    if (!restoreTarget) {
      return;
    }
    restoreMutation.mutate(restoreTarget.path, {
      onSuccess: (result) => {
        setRestoreTarget(null);
        setRestoreResult(result);
      },
    });
  }

  async function handleRestart() {
    try {
      await restartApp();
    } catch {
      toast.error("自动重启失败，请手动关闭并重新打开应用以应用恢复");
    }
  }

  return (
    <Card className="bg-card/40 md:col-span-2">
      <CardHeader className="flex flex-row flex-wrap items-center justify-between gap-2">
        <CardTitle className="text-sm">备份与恢复</CardTitle>
        <div className="flex flex-wrap gap-2">
          <Button
            size="sm"
            variant="outline"
            onClick={() => void handleSaveAs()}
            disabled={createMutation.isPending}
          >
            <Download className="size-4" aria-hidden="true" />
            另存为
          </Button>
          <Button
            size="sm"
            onClick={() => createMutation.mutate(null)}
            disabled={createMutation.isPending}
          >
            <FolderOpen className="size-4" aria-hidden="true" />
            {createMutation.isPending ? "备份中…" : "立即备份"}
          </Button>
          <Button
            size="sm"
            variant="outline"
            onClick={() => void backupsQuery.refetch()}
            disabled={backupsQuery.isFetching}
          >
            <RefreshCw
              className={backupsQuery.isFetching ? "size-4 animate-spin" : "size-4"}
              aria-hidden="true"
            />
            刷新
          </Button>
        </div>
      </CardHeader>

      <CardContent className="space-y-3">
        <p className="text-sm text-muted-foreground">
          数据库备份会包含番剧、书籍元数据、笔记、阅读进度与设置；原始书籍文件、封面与附件位于应用数据目录中，需单独备份。
        </p>
        {appInfoQuery.data ? (
          <p className="font-mono text-xs break-all text-muted-foreground">
            备份目录：{appInfoQuery.data.backupsDir}
          </p>
        ) : null}

        {backupsQuery.isPending ? (
          <p className="text-sm text-muted-foreground">正在读取备份列表…</p>
        ) : null}

        {backupsQuery.isError ? (
          <p className="text-sm text-destructive">{backupsQuery.error.message}</p>
        ) : null}

        {backupsQuery.data && backups.length === 0 ? (
          <p className="rounded-lg border border-dashed border-border px-3 py-6 text-center text-sm text-muted-foreground">
            还没有数据库备份。
          </p>
        ) : null}

        {backups.map((backup) => (
          <div
            key={backup.path}
            className="flex flex-col gap-2 rounded-lg border border-border/70 px-3 py-2.5 sm:flex-row sm:items-center sm:justify-between"
          >
            <div className="min-w-0">
              <div className="flex flex-wrap items-center gap-2 text-sm">
                <span className="truncate font-medium">{backup.fileName}</span>
                <span className="rounded-full bg-muted px-2 py-0.5 text-[11px] text-muted-foreground">
                  {kindLabels[backup.kind] ?? "其他"}
                </span>
              </div>
              <p className="mt-1 text-xs text-muted-foreground">
                {formatTime(backup.createdAt)} · {formatBytes(backup.sizeBytes)} · 版本{" "}
                {backup.appVersion ?? "未知"} · schema {backup.schemaVersion ?? "未知"}
              </p>
            </div>
            <Button
              size="sm"
              variant="outline"
              onClick={() =>
                setRestoreTarget({
                  path: backup.path,
                  fileName: backup.fileName,
                  info: backup,
                })
              }
            >
              <RotateCcw className="size-4" aria-hidden="true" />
              恢复
            </Button>
          </div>
        ))}

        <div>
          <Button size="sm" variant="outline" onClick={() => void handleChooseRestoreFile()}>
            <Upload className="size-4" aria-hidden="true" />
            从文件恢复
          </Button>
        </div>
      </CardContent>

      {restoreTarget ? (
        <Dialog
          open
          onOpenChange={(openValue) => {
            if (!openValue) {
              setRestoreTarget(null);
            }
          }}
        >
          <DialogContent>
            <DialogHeader>
              <DialogTitle>恢复数据库备份？</DialogTitle>
              <DialogDescription>
                将先自动备份当前数据库，再把所选备份暂存为待恢复文件。重启应用后才会替换当前数据库；恢复失败时保留当前数据。
                书籍原文件不包含在数据库备份中，需要单独备份。
              </DialogDescription>
            </DialogHeader>

            <div className="space-y-2 text-sm">
              <p className="font-medium break-all">{restoreTarget.fileName}</p>
              <p className="font-mono text-xs break-all text-muted-foreground">
                {restoreTarget.path}
              </p>
              {restoreTarget.info ? (
                <p className="text-xs text-muted-foreground">
                  创建于 {formatTime(restoreTarget.info.createdAt)} · 应用版本{" "}
                  {restoreTarget.info.appVersion ?? "未知"} · schema{" "}
                  {restoreTarget.info.schemaVersion ?? "未知"}
                </p>
              ) : null}
            </div>

            <DialogFooter>
              <Button variant="outline" onClick={() => setRestoreTarget(null)}>
                取消
              </Button>
              <Button onClick={handleConfirmRestore} disabled={restoreMutation.isPending}>
                {restoreMutation.isPending ? "准备中…" : "确认恢复"}
              </Button>
            </DialogFooter>
          </DialogContent>
        </Dialog>
      ) : null}

      {restoreResult ? (
        <Dialog
          open
          onOpenChange={(openValue) => {
            if (!openValue) {
              setRestoreResult(null);
            }
          }}
        >
          <DialogContent>
            <DialogHeader>
              <DialogTitle>恢复已准备</DialogTitle>
              <DialogDescription>
                重启应用后才会应用所选备份。当前数据库已在恢复前自动备份。
              </DialogDescription>
            </DialogHeader>

            <div className="space-y-2 text-sm">
              <p className="text-muted-foreground">恢复来源</p>
              <p className="font-mono text-xs break-all">{restoreResult.sourcePath}</p>
              <p className="text-muted-foreground">恢复前自动备份</p>
              <p className="font-mono text-xs break-all">{restoreResult.preRestoreBackup}</p>
              <p className="text-xs text-muted-foreground">
                在重启前仍可继续使用当前数据；如改变主意，可取消本次待恢复文件。
              </p>
            </div>

            <DialogFooter className="flex-wrap gap-2">
              <Button
                variant="outline"
                disabled={cancelMutation.isPending}
                onClick={() =>
                  cancelMutation.mutate(undefined, {
                    onSuccess: () => setRestoreResult(null),
                  })
                }
              >
                取消本次恢复
              </Button>
              <Button variant="outline" onClick={() => setRestoreResult(null)}>
                稍后重启
              </Button>
              <Button onClick={() => void handleRestart()}>立即重启</Button>
            </DialogFooter>
          </DialogContent>
        </Dialog>
      ) : null}
    </Card>
  );
}
