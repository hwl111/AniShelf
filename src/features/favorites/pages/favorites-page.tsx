import { BookOpen, Library, Star, StickyNote, Tv } from "lucide-react";
import { useState } from "react";
import { Link } from "react-router";

import { EmptyState } from "@/components/common/empty-state";
import { ErrorState } from "@/components/common/error-state";
import { LoadingState } from "@/components/common/loading-state";
import { PageContainer } from "@/components/layout/page-container";
import { PageHeader } from "@/components/layout/page-header";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { useAnimeList, useSetAnimeFavorite } from "@/features/anime/queries";
import { bookFormatLabels } from "@/features/books/constants";
import { useBooks, useSetBookFavorite } from "@/features/books/queries";
import { useNotes, useSetNoteFavorite } from "@/features/notes/queries";
import { formatNoteTime } from "@/features/notes/utils";
import { cn } from "@/lib/utils";

type FavoriteTab = "anime" | "book" | "note";

const tabs: { value: FavoriteTab; label: string; icon: typeof Tv }[] = [
  { value: "anime", label: "番剧", icon: Tv },
  { value: "book", label: "书籍", icon: Library },
  { value: "note", label: "笔记", icon: StickyNote },
];

function FavoriteAnimeList() {
  const query = useAnimeList({ favorite: true, sort: "updated_desc" });
  const favoriteMutation = useSetAnimeFavorite();
  const items = query.data ?? [];

  if (query.isPending) {
    return <LoadingState label="正在读取收藏番剧…" />;
  }

  if (query.isError) {
    return (
      <ErrorState
        title="无法读取收藏番剧"
        description={query.error.message}
        onRetry={() => void query.refetch()}
      />
    );
  }

  if (items.length === 0) {
    return (
      <EmptyState
        icon={Tv}
        title="还没有收藏番剧"
        description="在番剧详情页点击「收藏」后，会显示在这里。"
        action={
          <Button asChild variant="outline">
            <Link to="/anime">去番剧页</Link>
          </Button>
        }
      />
    );
  }

  return (
    <div className="space-y-2">
      {items.map((anime) => (
        <Card key={anime.id} className="bg-card/40">
          <CardContent className="flex items-center justify-between gap-3 py-3">
            <Link to={`/anime/${anime.id}`} className="min-w-0 flex-1">
              <p className="truncate text-sm font-medium">{anime.title}</p>
              <p className="mt-1 truncate text-xs text-muted-foreground">
                {anime.watchedEpisodes}/{anime.totalEpisodes ?? "?"} 集
                {anime.originalTitle ? ` · ${anime.originalTitle}` : ""}
              </p>
            </Link>
            <Button
              variant="ghost"
              size="icon"
              aria-label="取消收藏"
              title="取消收藏"
              disabled={favoriteMutation.isPending}
              onClick={() => favoriteMutation.mutate({ id: anime.id, favorite: false })}
            >
              <Star className="size-4 fill-amber-300 text-amber-300" aria-hidden="true" />
            </Button>
          </CardContent>
        </Card>
      ))}
    </div>
  );
}

function FavoriteBookList() {
  const query = useBooks({ favorite: true, sort: "updated_desc" });
  const favoriteMutation = useSetBookFavorite();
  const items = query.data ?? [];

  if (query.isPending) {
    return <LoadingState label="正在读取收藏书籍…" />;
  }

  if (query.isError) {
    return (
      <ErrorState
        title="无法读取收藏书籍"
        description={query.error.message}
        onRetry={() => void query.refetch()}
      />
    );
  }

  if (items.length === 0) {
    return (
      <EmptyState
        icon={BookOpen}
        title="还没有收藏书籍"
        description="在书籍详情页点击「收藏」后，会显示在这里。"
        action={
          <Button asChild variant="outline">
            <Link to="/bookshelf">去书架</Link>
          </Button>
        }
      />
    );
  }

  return (
    <div className="space-y-2">
      {items.map((book) => (
        <Card key={book.id} className="bg-card/40">
          <CardContent className="flex items-center justify-between gap-3 py-3">
            <Link to={`/bookshelf/${book.id}`} className="min-w-0 flex-1">
              <p className="truncate text-sm font-medium">{book.title}</p>
              <p className="mt-1 truncate text-xs text-muted-foreground">
                {[book.author || null, bookFormatLabels[book.format]].filter(Boolean).join(" · ")}
              </p>
            </Link>
            <Button
              variant="ghost"
              size="icon"
              aria-label="取消收藏"
              title="取消收藏"
              disabled={favoriteMutation.isPending}
              onClick={() => favoriteMutation.mutate({ id: book.id, favorite: false })}
            >
              <Star className="size-4 fill-amber-300 text-amber-300" aria-hidden="true" />
            </Button>
          </CardContent>
        </Card>
      ))}
    </div>
  );
}

function FavoriteNoteList() {
  const query = useNotes({ favorite: true, sort: "updated_desc", limit: 200 });
  const favoriteMutation = useSetNoteFavorite();
  const items = query.data ?? [];

  if (query.isPending) {
    return <LoadingState label="正在读取收藏笔记…" />;
  }

  if (query.isError) {
    return (
      <ErrorState
        title="无法读取收藏笔记"
        description={query.error.message}
        onRetry={() => void query.refetch()}
      />
    );
  }

  if (items.length === 0) {
    return (
      <EmptyState
        icon={StickyNote}
        title="还没有收藏笔记"
        description="在笔记编辑页点击「收藏」后，会显示在这里。"
        action={
          <Button asChild variant="outline">
            <Link to="/notes">去笔记页</Link>
          </Button>
        }
      />
    );
  }

  return (
    <div className="space-y-2">
      {items.map((note) => (
        <Card key={note.id} className="bg-card/40">
          <CardContent className="flex items-center justify-between gap-3 py-3">
            <Link to={`/notes/${note.id}`} className="min-w-0 flex-1">
              <p className="truncate text-sm font-medium">{note.title || "无标题笔记"}</p>
              <p className="mt-1 truncate text-xs text-muted-foreground">
                {note.excerpt || "（无正文）"} · 更新于 {formatNoteTime(note.updatedAt)}
              </p>
            </Link>
            <div className="flex items-center gap-1">
              {note.tags.slice(0, 2).map((tag) => (
                <Badge key={tag.id} variant="outline" className="hidden sm:inline-flex">
                  {tag.name}
                </Badge>
              ))}
              <Button
                variant="ghost"
                size="icon"
                aria-label="取消收藏"
                title="取消收藏"
                disabled={favoriteMutation.isPending}
                onClick={() => favoriteMutation.mutate({ id: note.id, favorite: false })}
              >
                <Star className="size-4 fill-amber-300 text-amber-300" aria-hidden="true" />
              </Button>
            </div>
          </CardContent>
        </Card>
      ))}
    </div>
  );
}

export function FavoritesPage() {
  const [tab, setTab] = useState<FavoriteTab>("anime");

  return (
    <PageContainer>
      <PageHeader title="收藏" description="集中查看收藏的番剧、书籍与笔记" />

      <div className="flex flex-wrap gap-2">
        {tabs.map((item) => (
          <Button
            key={item.value}
            variant={tab === item.value ? "default" : "outline"}
            size="sm"
            className={cn("gap-2")}
            onClick={() => setTab(item.value)}
          >
            <item.icon className="size-4" aria-hidden="true" />
            {item.label}
          </Button>
        ))}
      </div>

      {tab === "anime" ? <FavoriteAnimeList /> : null}
      {tab === "book" ? <FavoriteBookList /> : null}
      {tab === "note" ? <FavoriteNoteList /> : null}
    </PageContainer>
  );
}
