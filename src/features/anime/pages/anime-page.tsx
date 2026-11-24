import { Plus, Star, Tv } from "lucide-react";
import { useMemo, useState } from "react";
import { Link, useNavigate } from "react-router";

import { EmptyState } from "@/components/common/empty-state";
import { ErrorState } from "@/components/common/error-state";
import { LoadingState } from "@/components/common/loading-state";
import { PageContainer } from "@/components/layout/page-container";
import { PageHeader } from "@/components/layout/page-header";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";

import { AnimeFormDialog } from "../components/anime-form-dialog";
import { selectClassName } from "../components/field-styles";
import { WatchStatusBadge } from "../components/watch-status-badge";
import { animeSortOptions, seasonLabels, watchStatusOptions } from "../constants";
import { useAnimeList, useAnimeTags, useCreateAnime } from "../queries";
import type { Anime, AnimeFilter, AnimeSort, WatchStatus } from "../types";

function progressPercent(anime: Anime): number {
  if (!anime.totalEpisodes || anime.totalEpisodes <= 0) {
    return 0;
  }
  return Math.min(100, Math.round((anime.watchedEpisodes / anime.totalEpisodes) * 100));
}

export function AnimePage() {
  const navigate = useNavigate();
  const [statusFilter, setStatusFilter] = useState<WatchStatus | "">("");
  const [yearFilter, setYearFilter] = useState("");
  const [tagFilter, setTagFilter] = useState("");
  const [sort, setSort] = useState<AnimeSort>("updated_desc");
  const [formOpen, setFormOpen] = useState(false);

  const filter = useMemo<AnimeFilter>(() => {
    const parsedYear = yearFilter.trim() ? Number(yearFilter) : undefined;
    return {
      status: statusFilter || undefined,
      year: parsedYear !== undefined && Number.isFinite(parsedYear) ? parsedYear : undefined,
      tagId: tagFilter ? Number(tagFilter) : undefined,
      sort,
    };
  }, [statusFilter, yearFilter, tagFilter, sort]);

  const listQuery = useAnimeList(filter);
  const tagsQuery = useAnimeTags();
  const createMutation = useCreateAnime();

  return (
    <PageContainer>
      <PageHeader
        title="番剧"
        description="管理追番进度、评分与观看记录"
        actions={
          <Button onClick={() => setFormOpen(true)}>
            <Plus className="size-4" aria-hidden="true" />
            添加番剧
          </Button>
        }
      />

      <Card className="bg-card/40">
        <CardContent className="grid gap-3 py-4 sm:grid-cols-2 lg:grid-cols-4">
          <div className="space-y-1">
            <Label htmlFor="filter-status">状态</Label>
            <select
              id="filter-status"
              className={selectClassName}
              value={statusFilter}
              onChange={(event) => setStatusFilter(event.target.value as WatchStatus | "")}
            >
              <option value="">全部状态</option>
              {watchStatusOptions.map((option) => (
                <option key={option.value} value={option.value}>
                  {option.label}
                </option>
              ))}
            </select>
          </div>

          <div className="space-y-1">
            <Label htmlFor="filter-year">年份</Label>
            <Input
              id="filter-year"
              type="number"
              min={1900}
              max={2200}
              placeholder="全部年份"
              value={yearFilter}
              onChange={(event) => setYearFilter(event.target.value)}
            />
          </div>

          <div className="space-y-1">
            <Label htmlFor="filter-tag">标签</Label>
            <select
              id="filter-tag"
              className={selectClassName}
              value={tagFilter}
              onChange={(event) => setTagFilter(event.target.value)}
            >
              <option value="">全部标签</option>
              {(tagsQuery.data ?? []).map((tag) => (
                <option key={tag.id} value={tag.id}>
                  {tag.name}
                </option>
              ))}
            </select>
          </div>

          <div className="space-y-1">
            <Label htmlFor="filter-sort">排序</Label>
            <select
              id="filter-sort"
              className={selectClassName}
              value={sort}
              onChange={(event) => setSort(event.target.value as AnimeSort)}
            >
              {animeSortOptions.map((option) => (
                <option key={option.value} value={option.value}>
                  {option.label}
                </option>
              ))}
            </select>
          </div>
        </CardContent>
      </Card>

      {listQuery.isPending ? <LoadingState label="正在读取番剧列表…" /> : null}

      {listQuery.isError ? (
        <ErrorState
          title="无法读取番剧列表"
          description={listQuery.error.message}
          onRetry={() => void listQuery.refetch()}
        />
      ) : null}

      {listQuery.data && listQuery.data.length === 0 ? (
        <EmptyState
          icon={Tv}
          title="还没有番剧"
          description="点击“添加番剧”开始记录；之后也可以从 Bangumi 搜索导入。"
          action={
            <Button onClick={() => setFormOpen(true)}>
              <Plus className="size-4" aria-hidden="true" />
              添加番剧
            </Button>
          }
        />
      ) : null}

      {listQuery.data && listQuery.data.length > 0 ? (
        <div className="grid gap-4 sm:grid-cols-2 xl:grid-cols-3">
          {listQuery.data.map((anime) => {
            const percent = progressPercent(anime);
            return (
              <Link key={anime.id} to={`/anime/${anime.id}`} className="group block h-full">
                <Card className="h-full bg-card/40 transition-colors group-hover:border-ring/40 group-hover:bg-card">
                  <CardHeader className="space-y-1">
                    <div className="flex items-start justify-between gap-2">
                      <CardTitle className="line-clamp-2 text-base">{anime.title}</CardTitle>
                      {anime.isFavorite ? (
                        <Star
                          className="size-4 shrink-0 fill-amber-300 text-amber-300"
                          aria-label="已收藏"
                        />
                      ) : null}
                    </div>
                    {anime.originalTitle ? (
                      <p className="truncate text-xs text-muted-foreground">
                        {anime.originalTitle}
                      </p>
                    ) : null}
                  </CardHeader>
                  <CardContent className="space-y-3">
                    <div className="flex flex-wrap items-center gap-1.5">
                      <WatchStatusBadge status={anime.watchStatus} />
                      {anime.year ? (
                        <Badge variant="outline">
                          {anime.year}
                          {anime.season ? ` · ${seasonLabels[anime.season]}` : ""}
                        </Badge>
                      ) : null}
                      {anime.rating ? <Badge variant="outline">★ {anime.rating}</Badge> : null}
                    </div>

                    <div className="space-y-1">
                      <div className="flex items-center justify-between text-xs text-muted-foreground">
                        <span>观看进度</span>
                        <span>
                          {anime.totalEpisodes
                            ? `${anime.watchedEpisodes}/${anime.totalEpisodes}`
                            : `已看 ${anime.watchedEpisodes} 集`}
                        </span>
                      </div>
                      {anime.totalEpisodes ? (
                        <div className="h-1.5 overflow-hidden rounded-full bg-muted">
                          <div
                            className="h-full rounded-full bg-primary"
                            style={{ width: `${percent}%` }}
                          />
                        </div>
                      ) : null}
                    </div>

                    {anime.tags.length > 0 ? (
                      <div className="flex flex-wrap gap-1">
                        {anime.tags.map((tag) => (
                          <span
                            key={tag.id}
                            className="rounded-full bg-muted px-2 py-0.5 text-[11px] text-muted-foreground"
                          >
                            {tag.name}
                          </span>
                        ))}
                      </div>
                    ) : null}
                  </CardContent>
                </Card>
              </Link>
            );
          })}
        </div>
      ) : null}

      {formOpen ? (
        <AnimeFormDialog
          open
          onOpenChange={setFormOpen}
          pending={createMutation.isPending}
          onSubmit={(input) =>
            createMutation.mutate(input, {
              onSuccess: (anime) => {
                setFormOpen(false);
                navigate(`/anime/${anime.id}`);
              },
            })
          }
        />
      ) : null}
    </PageContainer>
  );
}
