import { CheckCircle2, Circle, History, ListVideo, Pencil, Plus, Trash2 } from "lucide-react";
import { useState } from "react";

import { EmptyState } from "@/components/common/empty-state";
import { ErrorState } from "@/components/common/error-state";
import { LoadingState } from "@/components/common/loading-state";
import { Badge } from "@/components/ui/badge";
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
import { useNoteRelationImpact } from "@/features/notes/queries";
import { cn } from "@/lib/utils";

import {
  useAddWatchRecord,
  useCreateEpisode,
  useDeleteEpisode,
  useDeleteWatchRecord,
  useEpisodes,
  useSetEpisodeWatched,
  useUpdateEpisode,
  useUpdateWatchRecord,
  useWatchRecords,
} from "../queries";
import type { Anime, Episode, WatchRecord } from "../types";
import { formatDateTime } from "./date-time";
import { EpisodeFormDialog } from "./episode-form-dialog";
import { WatchRecordFormDialog, type WatchRecordFormValues } from "./watch-record-form-dialog";

type EpisodeDialogState = { mode: "create" } | { mode: "edit"; episode: Episode } | null;
type RecordDialogState =
  | { mode: "add"; episodeNumber: number; watchedAt: number }
  | { mode: "edit"; record: WatchRecord }
  | null;
type DeleteTarget =
  { type: "episode"; episode: Episode } | { type: "record"; record: WatchRecord } | null;

export function EpisodeSection({ anime }: { anime: Anime }) {
  const episodesQuery = useEpisodes(anime.id);
  const recordsQuery = useWatchRecords(anime.id);
  const createEpisode = useCreateEpisode(anime.id);
  const updateEpisode = useUpdateEpisode(anime.id);
  const deleteEpisode = useDeleteEpisode(anime.id);
  const setWatched = useSetEpisodeWatched(anime.id);
  const addRecord = useAddWatchRecord(anime.id);
  const updateRecord = useUpdateWatchRecord(anime.id);
  const deleteRecord = useDeleteWatchRecord(anime.id);

  const [episodeDialog, setEpisodeDialog] = useState<EpisodeDialogState>(null);
  const [recordDialog, setRecordDialog] = useState<RecordDialogState>(null);
  const [unwatchTarget, setUnwatchTarget] = useState<Episode | null>(null);
  const [deleteTarget, setDeleteTarget] = useState<DeleteTarget>(null);
  const deleteEpisodeId = deleteTarget?.type === "episode" ? deleteTarget.episode.id : 0;
  const episodeNoteImpact = useNoteRelationImpact(
    "episode",
    deleteEpisodeId,
    deleteTarget?.type === "episode",
  );

  const episodes = episodesQuery.data ?? [];
  const records = recordsQuery.data ?? [];
  const lastEpisodeNumber = episodes[episodes.length - 1]?.episodeNumber ?? 0;
  const nextEpisodeNumber = lastEpisodeNumber + 1;
  const nextUnwatchedNumber =
    episodes.find((episode) => !episode.isWatched)?.episodeNumber ?? nextEpisodeNumber;
  const watchedCount = episodes.filter((episode) => episode.isWatched).length;

  function openAddRecord() {
    setRecordDialog({
      mode: "add",
      episodeNumber: nextUnwatchedNumber,
      watchedAt: Date.now(),
    });
  }

  function toggleWatched(episode: Episode) {
    if (episode.isWatched) {
      setUnwatchTarget(episode);
    } else {
      setWatched.mutate({ id: episode.id, watched: true, watchedAt: null, comment: null });
    }
  }

  return (
    <>
      <Card className="bg-card/40">
        <CardHeader className="flex flex-row items-start justify-between gap-3">
          <div>
            <CardTitle className="text-sm">单集</CardTitle>
            <p className="mt-1 text-xs text-muted-foreground">
              共 {episodes.length} 集，已看 {watchedCount} 集
            </p>
          </div>
          <div className="flex flex-wrap justify-end gap-2">
            <Button variant="outline" size="sm" onClick={openAddRecord}>
              <History className="size-4" aria-hidden="true" />
              记录观看
            </Button>
            <Button
              variant="outline"
              size="sm"
              onClick={() => setEpisodeDialog({ mode: "create" })}
            >
              <Plus className="size-4" aria-hidden="true" />
              添加单集
            </Button>
          </div>
        </CardHeader>
        <CardContent className="space-y-3">
          {episodesQuery.isPending ? <LoadingState label="正在读取单集…" /> : null}

          {episodesQuery.isError ? (
            <ErrorState
              title="无法读取单集"
              description={episodesQuery.error.message}
              onRetry={() => void episodesQuery.refetch()}
            />
          ) : null}

          {episodesQuery.data && episodes.length === 0 ? (
            <EmptyState
              icon={ListVideo}
              title="还没有单集"
              description="设置总集数会自动生成单集，也可以手动添加。"
              action={
                <Button variant="outline" onClick={() => setEpisodeDialog({ mode: "create" })}>
                  <Plus className="size-4" aria-hidden="true" />
                  添加单集
                </Button>
              }
            />
          ) : null}

          {episodes.length > 0 ? (
            <div className="grid gap-2 lg:grid-cols-2">
              {episodes.map((episode) => (
                <div
                  key={episode.id}
                  className="flex items-start gap-3 rounded-lg border border-border/60 bg-background/30 px-3 py-2"
                >
                  <span
                    className={cn(
                      "mt-0.5 flex size-8 shrink-0 items-center justify-center rounded-full text-xs font-medium",
                      episode.isWatched
                        ? "bg-emerald-500/15 text-emerald-300"
                        : "bg-muted text-muted-foreground",
                    )}
                  >
                    {episode.episodeNumber}
                  </span>

                  <div className="min-w-0 flex-1">
                    <div className="flex flex-wrap items-center gap-2">
                      <p className="truncate text-sm">
                        {episode.title || `第 ${episode.episodeNumber} 集`}
                      </p>
                      {episode.watchCount > 1 ? (
                        <Badge variant="outline">{episode.watchCount} 次观看</Badge>
                      ) : null}
                    </div>
                    <p className="mt-0.5 text-xs text-muted-foreground">
                      {episode.isWatched
                        ? `最近观看：${formatDateTime(episode.lastWatchedAt)}`
                        : "未看"}
                      {episode.airedAt ? ` · 播出：${formatDateTime(episode.airedAt)}` : ""}
                    </p>
                  </div>

                  <div className="flex shrink-0 items-center gap-0.5">
                    <Button
                      variant="ghost"
                      size="icon-sm"
                      aria-label={episode.isWatched ? "标记未看" : "标记已看"}
                      title={episode.isWatched ? "标记未看" : "标记已看"}
                      onClick={() => toggleWatched(episode)}
                    >
                      {episode.isWatched ? (
                        <CheckCircle2 className="size-4 text-emerald-400" aria-hidden="true" />
                      ) : (
                        <Circle className="size-4" aria-hidden="true" />
                      )}
                    </Button>
                    <Button
                      variant="ghost"
                      size="icon-sm"
                      aria-label="编辑单集"
                      title="编辑单集"
                      onClick={() => setEpisodeDialog({ mode: "edit", episode })}
                    >
                      <Pencil className="size-4" aria-hidden="true" />
                    </Button>
                    <Button
                      variant="ghost"
                      size="icon-sm"
                      aria-label="删除单集"
                      title="删除单集"
                      onClick={() => setDeleteTarget({ type: "episode", episode })}
                    >
                      <Trash2 className="size-4" aria-hidden="true" />
                    </Button>
                  </div>
                </div>
              ))}
            </div>
          ) : null}
        </CardContent>
      </Card>

      <Card className="bg-card/40">
        <CardHeader>
          <CardTitle className="text-sm">观看历史</CardTitle>
        </CardHeader>
        <CardContent className="space-y-3">
          {recordsQuery.isPending ? <LoadingState label="正在读取观看记录…" /> : null}

          {recordsQuery.isError ? (
            <ErrorState
              title="无法读取观看记录"
              description={recordsQuery.error.message}
              onRetry={() => void recordsQuery.refetch()}
            />
          ) : null}

          {recordsQuery.data && records.length === 0 ? (
            <EmptyState
              icon={ListVideo}
              title="暂无观看记录"
              description="标记已看或补录历史后会显示在这里。"
              action={
                <Button variant="outline" onClick={openAddRecord}>
                  <History className="size-4" aria-hidden="true" />
                  记录观看
                </Button>
              }
            />
          ) : null}

          {records.length > 0 ? (
            <div className="space-y-2">
              {records.map((record) => (
                <div
                  key={record.id}
                  className="flex items-start gap-3 rounded-lg border border-border/60 bg-background/30 px-3 py-2"
                >
                  <span className="mt-0.5 flex size-8 shrink-0 items-center justify-center rounded-full bg-muted text-xs font-medium text-muted-foreground">
                    {record.episodeNumber}
                  </span>
                  <div className="min-w-0 flex-1">
                    <p className="text-sm">
                      第 {record.episodeNumber} 集 · {formatDateTime(record.watchedAt)}
                    </p>
                    {record.comment ? (
                      <p className="mt-0.5 text-xs whitespace-pre-wrap text-muted-foreground">
                        {record.comment}
                      </p>
                    ) : null}
                  </div>
                  <div className="flex shrink-0 items-center gap-0.5">
                    <Button
                      variant="ghost"
                      size="icon-sm"
                      aria-label="编辑观看记录"
                      title="编辑观看记录"
                      onClick={() => setRecordDialog({ mode: "edit", record })}
                    >
                      <Pencil className="size-4" aria-hidden="true" />
                    </Button>
                    <Button
                      variant="ghost"
                      size="icon-sm"
                      aria-label="删除观看记录"
                      title="删除观看记录"
                      onClick={() => setDeleteTarget({ type: "record", record })}
                    >
                      <Trash2 className="size-4" aria-hidden="true" />
                    </Button>
                  </div>
                </div>
              ))}
            </div>
          ) : null}
        </CardContent>
      </Card>

      {episodeDialog ? (
        <EpisodeFormDialog
          key={
            episodeDialog.mode === "edit" ? `episode-${episodeDialog.episode.id}` : "episode-create"
          }
          open
          onOpenChange={(open) => {
            if (!open) {
              setEpisodeDialog(null);
            }
          }}
          episode={episodeDialog.mode === "edit" ? episodeDialog.episode : null}
          defaultEpisodeNumber={nextEpisodeNumber}
          pending={
            episodeDialog.mode === "edit" ? updateEpisode.isPending : createEpisode.isPending
          }
          onSubmit={(input) => {
            if (episodeDialog.mode === "edit") {
              updateEpisode.mutate(
                { id: episodeDialog.episode.id, input },
                { onSuccess: () => setEpisodeDialog(null) },
              );
            } else {
              createEpisode.mutate(input, { onSuccess: () => setEpisodeDialog(null) });
            }
          }}
        />
      ) : null}

      {recordDialog ? (
        <WatchRecordFormDialog
          key={recordDialog.mode === "edit" ? `record-${recordDialog.record.id}` : "record-add"}
          open
          onOpenChange={(open) => {
            if (!open) {
              setRecordDialog(null);
            }
          }}
          episodeNumber={
            recordDialog.mode === "add"
              ? recordDialog.episodeNumber
              : recordDialog.record.episodeNumber
          }
          defaultWatchedAt={
            recordDialog.mode === "add" ? recordDialog.watchedAt : recordDialog.record.watchedAt
          }
          record={recordDialog.mode === "edit" ? recordDialog.record : null}
          pending={recordDialog.mode === "edit" ? updateRecord.isPending : addRecord.isPending}
          onSubmit={(values: WatchRecordFormValues) => {
            if (recordDialog.mode === "edit") {
              updateRecord.mutate(
                {
                  id: recordDialog.record.id,
                  watchedAt: values.watchedAt,
                  comment: values.comment,
                },
                { onSuccess: () => setRecordDialog(null) },
              );
            } else {
              addRecord.mutate(
                {
                  episodeNumber: values.episodeNumber,
                  watchedAt: values.watchedAt,
                  comment: values.comment,
                },
                { onSuccess: () => setRecordDialog(null) },
              );
            }
          }}
        />
      ) : null}

      {unwatchTarget ? (
        <Dialog
          open
          onOpenChange={(open) => {
            if (!open) {
              setUnwatchTarget(null);
            }
          }}
        >
          <DialogContent>
            <DialogHeader>
              <DialogTitle>标记为未看？</DialogTitle>
              <DialogDescription>
                第 {unwatchTarget.episodeNumber} 集的 {unwatchTarget.watchCount}{" "}
                条观看记录将被删除。此操作不可撤销。
              </DialogDescription>
            </DialogHeader>
            <DialogFooter>
              <Button variant="outline" onClick={() => setUnwatchTarget(null)}>
                取消
              </Button>
              <Button
                variant="destructive"
                disabled={setWatched.isPending}
                onClick={() =>
                  setWatched.mutate(
                    { id: unwatchTarget.id, watched: false, watchedAt: null, comment: null },
                    { onSuccess: () => setUnwatchTarget(null) },
                  )
                }
              >
                {setWatched.isPending ? "处理中…" : "标记未看"}
              </Button>
            </DialogFooter>
          </DialogContent>
        </Dialog>
      ) : null}

      {deleteTarget ? (
        <Dialog
          open
          onOpenChange={(open) => {
            if (!open) {
              setDeleteTarget(null);
            }
          }}
        >
          <DialogContent>
            <DialogHeader>
              <DialogTitle>
                {deleteTarget.type === "episode" ? "删除单集？" : "删除观看记录？"}
              </DialogTitle>
              <DialogDescription>
                {deleteTarget.type === "episode"
                  ? `将删除第 ${deleteTarget.episode.episodeNumber} 集；已有观看记录会保留但不再关联该集，关联笔记会被移除。`
                  : `将删除第 ${deleteTarget.record.episodeNumber} 集于 ${formatDateTime(
                      deleteTarget.record.watchedAt,
                    )} 的观看记录。此操作不可撤销。`}
              </DialogDescription>
              {deleteTarget.type === "episode" &&
              episodeNoteImpact.data &&
              episodeNoteImpact.data.count > 0 ? (
                <p className="text-sm text-muted-foreground">
                  有 {episodeNoteImpact.data.count}{" "}
                  篇笔记关联到本集，删除后这些关联会被移除，笔记本身保留。
                </p>
              ) : null}
            </DialogHeader>
            <DialogFooter>
              <Button variant="outline" onClick={() => setDeleteTarget(null)}>
                取消
              </Button>
              <Button
                variant="destructive"
                disabled={deleteEpisode.isPending || deleteRecord.isPending}
                onClick={() => {
                  if (deleteTarget.type === "episode") {
                    deleteEpisode.mutate(deleteTarget.episode.id, {
                      onSuccess: () => setDeleteTarget(null),
                    });
                  } else {
                    deleteRecord.mutate(deleteTarget.record.id, {
                      onSuccess: () => setDeleteTarget(null),
                    });
                  }
                }}
              >
                确认删除
              </Button>
            </DialogFooter>
          </DialogContent>
        </Dialog>
      ) : null}
    </>
  );
}
