import { ArrowLeft, ListVideo, Pencil, Star, Trash2 } from "lucide-react";
import { useState } from "react";
import { Link, useNavigate, useParams } from "react-router";

import { EmptyState } from "@/components/common/empty-state";
import { ErrorState } from "@/components/common/error-state";
import { LoadingState } from "@/components/common/loading-state";
import { PageContainer } from "@/components/layout/page-container";
import { PageHeader } from "@/components/layout/page-header";
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
import { Separator } from "@/components/ui/separator";

import { AnimeFormDialog } from "../components/anime-form-dialog";
import { WatchStatusBadge } from "../components/watch-status-badge";
import { seasonLabels } from "../constants";
import { useAnime, useDeleteAnime, useSetAnimeFavorite, useUpdateAnime } from "../queries";
import type { Anime } from "../types";

function formatTimestamp(value: number | null): string {
  if (!value) {
    return "—";
  }
  return new Date(value).toLocaleString("zh-CN", { hour12: false });
}

function progressPercent(anime: Anime): number {
  if (!anime.totalEpisodes || anime.totalEpisodes <= 0) {
    return 0;
  }
  return Math.min(100, Math.round((anime.watchedEpisodes / anime.totalEpisodes) * 100));
}

export function AnimeDetailPage() {
  const params = useParams();
  const navigate = useNavigate();
  const animeId = Number(params.animeId);
  const validId = Number.isInteger(animeId) && animeId > 0;

  const animeQuery = useAnime(validId ? animeId : undefined);
  const updateMutation = useUpdateAnime();
  const deleteMutation = useDeleteAnime();
  const favoriteMutation = useSetAnimeFavorite();

  const [editOpen, setEditOpen] = useState(false);
  const [deleteOpen, setDeleteOpen] = useState(false);

  const anime = animeQuery.data;

  if (!validId) {
    return (
      <PageContainer>
        <ErrorState title="番剧不存在" description="无效的番剧 ID。" />
      </PageContainer>
    );
  }

  if (animeQuery.isPending) {
    return (
      <PageContainer>
        <LoadingState label="正在读取番剧详情…" />
      </PageContainer>
    );
  }

  if (animeQuery.isError || !anime) {
    return (
      <PageContainer>
        <ErrorState
          title="无法读取番剧详情"
          description={animeQuery.error?.message ?? "番剧不存在或已被删除。"}
          onRetry={() => void animeQuery.refetch()}
        />
      </PageContainer>
    );
  }

  const metaParts = [
    anime.originalTitle,
    anime.year ? `${anime.year}${anime.season ? ` · ${seasonLabels[anime.season]}` : ""}` : null,
  ].filter(Boolean);

  const percent = progressPercent(anime);

  return (
    <PageContainer>
      <div>
        <Button asChild variant="ghost" size="sm" className="text-muted-foreground">
          <Link to="/anime">
            <ArrowLeft className="size-4" aria-hidden="true" />
            返回番剧
          </Link>
        </Button>
      </div>

      <PageHeader
        title={anime.title}
        description={metaParts.length > 0 ? metaParts.join(" · ") : undefined}
        actions={
          <>
            <Button
              variant="outline"
              size="sm"
              disabled={favoriteMutation.isPending}
              onClick={() => favoriteMutation.mutate({ id: anime.id, favorite: !anime.isFavorite })}
            >
              <Star
                className={anime.isFavorite ? "fill-amber-300 text-amber-300" : "size-4"}
                aria-hidden="true"
              />
              {anime.isFavorite ? "已收藏" : "收藏"}
            </Button>
            <Button variant="outline" size="sm" onClick={() => setEditOpen(true)}>
              <Pencil className="size-4" aria-hidden="true" />
              编辑
            </Button>
            <Button variant="destructive" size="sm" onClick={() => setDeleteOpen(true)}>
              <Trash2 className="size-4" aria-hidden="true" />
              删除
            </Button>
          </>
        }
      />

      <div className="grid gap-4 lg:grid-cols-3">
        <Card className="bg-card/40 lg:col-span-2">
          <CardHeader>
            <CardTitle className="text-sm">概览</CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="flex flex-wrap items-center gap-2">
              <WatchStatusBadge status={anime.watchStatus} />
              <Badge variant="outline">{anime.rating ? `评分 ${anime.rating}/10` : "未评分"}</Badge>
              <Badge variant="outline">已看 {anime.watchedEpisodes} 集</Badge>
            </div>

            <div className="space-y-1">
              <div className="flex items-center justify-between text-xs text-muted-foreground">
                <span>观看进度</span>
                <span>
                  {anime.totalEpisodes
                    ? `${anime.watchedEpisodes}/${anime.totalEpisodes}（${percent}%）`
                    : "未设置总集数"}
                </span>
              </div>
              {anime.totalEpisodes ? (
                <div className="h-2 overflow-hidden rounded-full bg-muted">
                  <div className="h-full bg-primary" style={{ width: `${percent}%` }} />
                </div>
              ) : null}
            </div>

            {anime.aliases.length > 0 ? (
              <div className="space-y-1">
                <p className="text-xs font-medium text-muted-foreground">别名</p>
                <p className="text-sm">{anime.aliases.join("、")}</p>
              </div>
            ) : null}

            <div className="space-y-1">
              <p className="text-xs font-medium text-muted-foreground">标签</p>
              {anime.tags.length > 0 ? (
                <div className="flex flex-wrap gap-1">
                  {anime.tags.map((tag) => (
                    <span
                      key={tag.id}
                      className="rounded-full bg-muted px-2 py-0.5 text-xs text-muted-foreground"
                    >
                      {tag.name}
                    </span>
                  ))}
                </div>
              ) : (
                <p className="text-sm text-muted-foreground">暂无标签</p>
              )}
            </div>

            <Separator />

            <div className="space-y-1">
              <p className="text-xs font-medium text-muted-foreground">简介</p>
              <p className="text-sm whitespace-pre-wrap text-muted-foreground">
                {anime.summary || "暂无简介"}
              </p>
            </div>

            <div className="space-y-1">
              <p className="text-xs font-medium text-muted-foreground">个人评论</p>
              <p className="text-sm whitespace-pre-wrap text-muted-foreground">
                {anime.review || "暂无评论"}
              </p>
            </div>
          </CardContent>
        </Card>

        <Card className="bg-card/40">
          <CardHeader>
            <CardTitle className="text-sm">详情</CardTitle>
          </CardHeader>
          <CardContent className="space-y-2 text-sm">
            <DetailRow label="年份" value={anime.year?.toString() ?? "—"} />
            <DetailRow label="季度" value={anime.season ? seasonLabels[anime.season] : "—"} />
            <DetailRow label="总集数" value={anime.totalEpisodes?.toString() ?? "—"} />
            <DetailRow label="Bangumi ID" value={anime.bangumiSubjectId?.toString() ?? "—"} />
            <DetailRow label="外部同步" value={formatTimestamp(anime.externalUpdatedAt)} />
            <DetailRow label="创建时间" value={formatTimestamp(anime.createdAt)} />
            <DetailRow label="更新时间" value={formatTimestamp(anime.updatedAt)} />
          </CardContent>
        </Card>
      </div>

      <Card className="bg-card/40">
        <CardHeader>
          <CardTitle className="text-sm">单集与观看记录</CardTitle>
        </CardHeader>
        <CardContent>
          <EmptyState
            icon={ListVideo}
            title="暂无观看记录"
            description="标记单集已看后，观看进度会显示在这里。"
          />
        </CardContent>
      </Card>

      {editOpen ? (
        <AnimeFormDialog
          key={anime.id}
          open
          onOpenChange={setEditOpen}
          anime={anime}
          pending={updateMutation.isPending}
          onSubmit={(input) =>
            updateMutation.mutate({ id: anime.id, input }, { onSuccess: () => setEditOpen(false) })
          }
        />
      ) : null}

      <Dialog open={deleteOpen} onOpenChange={setDeleteOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>删除番剧？</DialogTitle>
            <DialogDescription>
              将删除《{anime.title}
              》及其集数、观看记录、标签与关联关系。笔记不会被删除，但关联会移除。此操作不可撤销。
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant="outline" onClick={() => setDeleteOpen(false)}>
              取消
            </Button>
            <Button
              variant="destructive"
              disabled={deleteMutation.isPending}
              onClick={() =>
                deleteMutation.mutate(anime.id, {
                  onSuccess: () => {
                    setDeleteOpen(false);
                    navigate("/anime");
                  },
                })
              }
            >
              {deleteMutation.isPending ? "删除中…" : "确认删除"}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </PageContainer>
  );
}

function DetailRow({ label, value }: { label: string; value: string }) {
  return (
    <div className="flex items-center justify-between gap-3">
      <span className="text-muted-foreground">{label}</span>
      <span className="text-right">{value}</span>
    </div>
  );
}
