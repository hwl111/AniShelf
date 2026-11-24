import { Activity, BookOpen, Library, Plus, RefreshCw, StickyNote, Tv, Upload } from "lucide-react";
import type { LucideIcon } from "lucide-react";
import { Link } from "react-router";

import { EmptyState } from "@/components/common/empty-state";
import { ErrorState } from "@/components/common/error-state";
import { LoadingState } from "@/components/common/loading-state";
import { PageContainer } from "@/components/layout/page-container";
import { PageHeader } from "@/components/layout/page-header";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { cn } from "@/lib/utils";

import { useDashboard } from "../queries";
import type {
  ActivityItem,
  DashboardAnime,
  DashboardBook,
  DashboardNote,
  DashboardWatchRecord,
} from "../types";

function formatWhen(value: number): string {
  return new Date(value).toLocaleString("zh-CN", { hour12: false });
}

function formatDuration(ms: number): string {
  if (ms <= 0) {
    return "0 分钟";
  }
  const minutes = Math.max(1, Math.round(ms / 60_000));
  if (minutes < 60) {
    return `${minutes} 分钟`;
  }
  const hours = Math.floor(minutes / 60);
  const rest = minutes % 60;
  return rest > 0 ? `${hours} 小时 ${rest} 分钟` : `${hours} 小时`;
}

function EmptySection({ text }: { text: string }) {
  return <p className="py-6 text-center text-sm text-muted-foreground">{text}</p>;
}

function WatchingAnimeList({ items }: { items: DashboardAnime[] }) {
  if (items.length === 0) {
    return <EmptySection text="暂无正在观看的番剧" />;
  }

  return (
    <div className="space-y-3">
      {items.map((anime) => {
        const percent =
          anime.totalEpisodes && anime.totalEpisodes > 0
            ? Math.min(100, Math.round((anime.watchedEpisodes / anime.totalEpisodes) * 100))
            : 0;
        return (
          <Link key={anime.id} to={`/anime/${anime.id}`} className="block space-y-1.5">
            <div className="flex items-center justify-between gap-2 text-sm">
              <span className="truncate font-medium">{anime.title}</span>
              <span className="shrink-0 text-xs text-muted-foreground">
                {anime.watchedEpisodes}/{anime.totalEpisodes ?? "?"} 集
              </span>
            </div>
            {anime.totalEpisodes ? (
              <div className="h-1.5 w-full overflow-hidden rounded-full bg-muted">
                <div className="h-full rounded-full bg-primary" style={{ width: `${percent}%` }} />
              </div>
            ) : null}
          </Link>
        );
      })}
    </div>
  );
}

function RecentWatchList({ items }: { items: DashboardWatchRecord[] }) {
  if (items.length === 0) {
    return <EmptySection text="暂无观看记录" />;
  }

  return (
    <div className="space-y-2">
      {items.map((record) => (
        <Link
          key={record.id}
          to={`/anime/${record.animeId}`}
          className="flex items-center justify-between gap-2 rounded-md px-1 py-1 hover:bg-muted/40"
        >
          <div className="min-w-0">
            <p className="truncate text-sm">{record.animeTitle}</p>
            <p className="truncate text-xs text-muted-foreground">
              第 {record.episodeNumber} 集{record.comment ? ` · ${record.comment}` : ""}
            </p>
          </div>
          <span className="shrink-0 text-xs text-muted-foreground">
            {formatWhen(record.watchedAt)}
          </span>
        </Link>
      ))}
    </div>
  );
}

function RecentBookList({ items }: { items: DashboardBook[] }) {
  if (items.length === 0) {
    return <EmptySection text="暂无阅读记录" />;
  }

  return (
    <div className="space-y-2">
      {items.map((book) => (
        <Link
          key={book.id}
          to={`/bookshelf/${book.id}`}
          className="flex items-center justify-between gap-2 rounded-md px-1 py-1 hover:bg-muted/40"
        >
          <div className="min-w-0">
            <p className="truncate text-sm">{book.title}</p>
            <p className="text-xs text-muted-foreground">
              已阅读 {(book.percent * 100).toFixed(1)}% · {formatWhen(book.lastReadAt)}
            </p>
          </div>
          <div className="h-1.5 w-16 shrink-0 overflow-hidden rounded-full bg-muted">
            <div
              className="h-full rounded-full bg-primary"
              style={{ width: `${Math.min(100, Math.max(0, book.percent * 100))}%` }}
            />
          </div>
        </Link>
      ))}
    </div>
  );
}

function RecentNoteList({ items }: { items: DashboardNote[] }) {
  if (items.length === 0) {
    return <EmptySection text="暂无最近编辑的笔记" />;
  }

  return (
    <div className="space-y-2">
      {items.map((note) => (
        <Link
          key={note.id}
          to={`/notes/${note.id}`}
          className="flex items-center justify-between gap-2 rounded-md px-1 py-1 hover:bg-muted/40"
        >
          <span className="truncate text-sm">{note.title}</span>
          <span className="shrink-0 text-xs text-muted-foreground">
            {formatWhen(note.updatedAt)}
          </span>
        </Link>
      ))}
    </div>
  );
}

const activityIcons: Record<string, LucideIcon> = {
  watch: Tv,
  read: BookOpen,
  note: StickyNote,
  book: Library,
  anime: Plus,
};

function activityHref(item: ActivityItem): string | null {
  switch (item.kind) {
    case "watch":
    case "anime":
      return `/anime/${item.entityId}`;
    case "read":
    case "book":
      return `/bookshelf/${item.entityId}`;
    case "note":
      return `/notes/${item.entityId}`;
    default:
      return null;
  }
}

function ActivityTimeline({ items }: { items: ActivityItem[] }) {
  if (items.length === 0) {
    return (
      <EmptyState
        icon={Activity}
        title="还没有活动记录"
        description="观看番剧、阅读书籍或编辑笔记后，这里会显示最近的活动。"
      />
    );
  }

  return (
    <div className="space-y-1">
      {items.map((item, index) => {
        const Icon = activityIcons[item.kind] ?? Activity;
        const href = activityHref(item);
        const content = (
          <div className="flex items-start gap-3 rounded-md px-2 py-2 hover:bg-muted/40">
            <div className="mt-0.5 flex size-7 shrink-0 items-center justify-center rounded-full bg-muted text-muted-foreground">
              <Icon className="size-3.5" aria-hidden="true" />
            </div>
            <div className="min-w-0 flex-1">
              <p className="truncate text-sm">
                <span className="text-muted-foreground">{item.subtitle}：</span>
                {item.title}
              </p>
              <p className="text-xs text-muted-foreground">{formatWhen(item.occurredAt)}</p>
            </div>
          </div>
        );

        const key = `${item.kind}-${item.entityId}-${item.occurredAt}-${index}`;
        return href ? (
          <Link key={key} to={href}>
            {content}
          </Link>
        ) : (
          <div key={key}>{content}</div>
        );
      })}
    </div>
  );
}

export function DashboardPage() {
  const dashboardQuery = useDashboard();
  const data = dashboardQuery.data;
  const stats = data?.stats;

  const favoriteTotal = stats ? stats.animeFavorite + stats.booksFavorite + stats.notesFavorite : 0;

  return (
    <PageContainer>
      <PageHeader
        title="首页"
        description="追番、阅读与笔记的整体概览"
        actions={
          <Button
            variant="outline"
            size="sm"
            disabled={dashboardQuery.isFetching}
            onClick={() => void dashboardQuery.refetch()}
          >
            <RefreshCw
              className={cn("size-4", dashboardQuery.isFetching && "animate-spin")}
              aria-hidden="true"
            />
            刷新
          </Button>
        }
      />

      <Card className="bg-card/40">
        <CardContent className="flex flex-wrap gap-2 py-3">
          <Button asChild size="sm">
            <Link to="/anime?action=new">
              <Plus className="size-4" aria-hidden="true" />
              添加番剧
            </Link>
          </Button>
          <Button asChild size="sm" variant="outline">
            <Link to="/bookshelf?action=import">
              <Upload className="size-4" aria-hidden="true" />
              导入书籍
            </Link>
          </Button>
          <Button asChild size="sm" variant="outline">
            <Link to="/notes/new">
              <StickyNote className="size-4" aria-hidden="true" />
              新建笔记
            </Link>
          </Button>
        </CardContent>
      </Card>

      {dashboardQuery.isPending ? <LoadingState label="正在读取首页数据…" /> : null}

      {dashboardQuery.isError ? (
        <ErrorState
          title="无法读取首页数据"
          description={dashboardQuery.error.message}
          onRetry={() => void dashboardQuery.refetch()}
        />
      ) : null}

      {data && stats ? (
        <>
          <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-6">
            {[
              {
                label: "在看番剧",
                value: String(stats.animeWatching),
                hint: `共 ${stats.animeTotal} 部`,
              },
              {
                label: "已看集数",
                value: String(stats.watchedEpisodes),
                hint: `${stats.watchRecordsTotal} 条观看记录`,
              },
              {
                label: "阅读中的书籍",
                value: String(stats.booksReading),
                hint: `共 ${stats.booksTotal} 本`,
              },
              {
                label: "笔记",
                value: String(stats.notesTotal),
                hint: `${stats.notesFavorite} 篇收藏`,
              },
              {
                label: "阅读时长",
                value: formatDuration(stats.readingMs),
                hint: `${stats.readingSessions} 次阅读会话`,
              },
              {
                label: "收藏",
                value: String(favoriteTotal),
                hint: "番剧 / 书籍 / 笔记",
              },
            ].map((item) => (
              <Card key={item.label} className="bg-card/40">
                <CardContent className="py-3">
                  <p className="text-xs text-muted-foreground">{item.label}</p>
                  <p className="mt-1 text-xl font-semibold">{item.value}</p>
                  <p className="mt-1 text-xs text-muted-foreground">{item.hint}</p>
                </CardContent>
              </Card>
            ))}
          </div>

          <div className="grid gap-4 lg:grid-cols-2">
            <Card className="bg-card/40">
              <CardHeader>
                <CardTitle className="text-sm">正在追番</CardTitle>
              </CardHeader>
              <CardContent>
                <WatchingAnimeList items={data.watchingAnime} />
              </CardContent>
            </Card>

            <Card className="bg-card/40">
              <CardHeader>
                <CardTitle className="text-sm">最近观看</CardTitle>
              </CardHeader>
              <CardContent>
                <RecentWatchList items={data.recentWatchRecords} />
              </CardContent>
            </Card>

            <Card className="bg-card/40">
              <CardHeader>
                <CardTitle className="text-sm">最近阅读</CardTitle>
              </CardHeader>
              <CardContent>
                <RecentBookList items={data.recentBooks} />
              </CardContent>
            </Card>

            <Card className="bg-card/40">
              <CardHeader>
                <CardTitle className="text-sm">最近笔记</CardTitle>
              </CardHeader>
              <CardContent>
                <RecentNoteList items={data.recentNotes} />
              </CardContent>
            </Card>
          </div>

          <Card className="bg-card/40">
            <CardHeader>
              <CardTitle className="text-sm">活动时间线</CardTitle>
            </CardHeader>
            <CardContent>
              <ActivityTimeline items={data.timeline} />
            </CardContent>
          </Card>
        </>
      ) : null}
    </PageContainer>
  );
}
