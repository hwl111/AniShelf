import { Plus, StickyNote } from "lucide-react";
import { useMemo, useState } from "react";
import { Link, useSearchParams } from "react-router";

import { EmptyState } from "@/components/common/empty-state";
import { ErrorState } from "@/components/common/error-state";
import { LoadingState } from "@/components/common/loading-state";
import { PageContainer } from "@/components/layout/page-container";
import { PageHeader } from "@/components/layout/page-header";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { useAnime } from "@/features/anime/queries";
import { useBook, useBookTags } from "@/features/books/queries";

import { isNoteRelationType, noteRelationTypeLabels, noteSortOptions } from "../constants";
import { useNotes } from "../queries";
import type { NoteFilter, NoteSort } from "../types";
import { formatNoteTime } from "../utils";

const selectClassName =
  "h-8 w-full rounded-lg border border-input bg-transparent px-2.5 text-sm outline-none transition-colors focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/50 dark:bg-input/30";

const MAX_NOTES = 200;

export function NotesPage() {
  const [searchParams, setSearchParams] = useSearchParams();
  const rawRelationType = searchParams.get("relationType");
  const rawRelationId = Number(searchParams.get("relationId"));
  const relationType =
    isNoteRelationType(rawRelationType) && Number.isInteger(rawRelationId) && rawRelationId > 0
      ? rawRelationType
      : undefined;
  const relationId = relationType ? rawRelationId : undefined;

  const animeQuery = useAnime(relationType === "anime" ? relationId : undefined);
  const bookQuery = useBook(relationType === "book" ? relationId : undefined);
  const tagsQuery = useBookTags();

  const [query, setQuery] = useState("");
  const [tagId, setTagId] = useState("");
  const [sort, setSort] = useState<NoteSort>("updated_desc");
  const [visibleCount, setVisibleCount] = useState(50);

  const filter = useMemo<NoteFilter>(
    () => ({
      query: query.trim() || undefined,
      tagId: tagId ? Number(tagId) : undefined,
      relationType,
      relationId,
      sort,
      limit: visibleCount,
      offset: 0,
    }),
    [query, tagId, relationType, relationId, sort, visibleCount],
  );

  const notesQuery = useNotes(filter);
  const notes = notesQuery.data ?? [];

  const relationLabel =
    relationType === "anime"
      ? animeQuery.data?.title
      : relationType === "book"
        ? bookQuery.data?.title
        : relationType
          ? `${noteRelationTypeLabels[relationType]} #${relationId}`
          : null;

  const newNoteHref =
    relationType && relationId
      ? `/notes/new?relationType=${relationType}&relationId=${relationId}`
      : "/notes/new";

  const hasFilter = Boolean(query.trim() || tagId || relationType);

  function clearRelationFilter() {
    const next = new URLSearchParams(searchParams);
    next.delete("relationType");
    next.delete("relationId");
    setSearchParams(next);
    setVisibleCount(50);
  }

  function clearAllFilters() {
    setQuery("");
    setTagId("");
    setSort("updated_desc");
    setVisibleCount(50);
    if (relationType) {
      clearRelationFilter();
    }
  }

  return (
    <PageContainer>
      <PageHeader
        title="笔记"
        description="记录想法，关联番剧、书籍与集数，并在笔记之间建立链接"
        actions={
          <Button asChild>
            <Link to={newNoteHref}>
              <Plus className="size-4" aria-hidden="true" />
              新建笔记
            </Link>
          </Button>
        }
      />

      {relationType && relationId ? (
        <Card className="bg-card/40">
          <CardContent className="flex flex-wrap items-center justify-between gap-3 py-3 text-sm">
            <span>
              正在查看关联到{" "}
              <span className="font-medium">
                {relationType === "episode" ? "集数" : "《"}
                {relationLabel ?? "…"}
                {relationType === "episode" ? "" : "》"}
              </span>{" "}
              的笔记
            </span>
            <Button variant="ghost" size="sm" onClick={clearRelationFilter}>
              清除筛选
            </Button>
          </CardContent>
        </Card>
      ) : null}

      <Card className="bg-card/40">
        <CardContent className="grid gap-3 py-4 sm:grid-cols-3">
          <div className="space-y-1">
            <Label htmlFor="note-filter-query">搜索</Label>
            <Input
              id="note-filter-query"
              placeholder="按标题或正文搜索"
              value={query}
              onChange={(event) => {
                setQuery(event.target.value);
                setVisibleCount(50);
              }}
            />
          </div>

          <div className="space-y-1">
            <Label htmlFor="note-filter-tag">标签</Label>
            <select
              id="note-filter-tag"
              className={selectClassName}
              value={tagId}
              onChange={(event) => {
                setTagId(event.target.value);
                setVisibleCount(50);
              }}
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
            <Label htmlFor="note-filter-sort">排序</Label>
            <select
              id="note-filter-sort"
              className={selectClassName}
              value={sort}
              onChange={(event) => {
                setSort(event.target.value as NoteSort);
                setVisibleCount(50);
              }}
            >
              {noteSortOptions.map((option) => (
                <option key={option.value} value={option.value}>
                  {option.label}
                </option>
              ))}
            </select>
          </div>
        </CardContent>
      </Card>

      {notesQuery.isPending ? <LoadingState label="正在读取笔记…" /> : null}

      {notesQuery.isError ? (
        <ErrorState
          title="无法读取笔记"
          description={notesQuery.error.message}
          onRetry={() => void notesQuery.refetch()}
        />
      ) : null}

      {notesQuery.data && notes.length === 0 ? (
        <EmptyState
          icon={StickyNote}
          title={hasFilter ? "没有匹配的笔记" : "还没有笔记"}
          description={
            hasFilter
              ? "换一个关键词或清除筛选后再试。"
              : "创建第一篇笔记，记录阅读与观看中的想法。"
          }
          action={
            hasFilter ? (
              <Button variant="outline" onClick={clearAllFilters}>
                清除筛选
              </Button>
            ) : (
              <Button asChild>
                <Link to={newNoteHref}>
                  <Plus className="size-4" aria-hidden="true" />
                  新建笔记
                </Link>
              </Button>
            )
          }
        />
      ) : null}

      <div className="flex flex-col gap-3">
        {notes.map((note) => (
          <Link key={note.id} to={`/notes/${note.id}`} className="group block">
            <Card className="bg-card/40 transition-colors group-hover:border-ring/40">
              <CardContent className="space-y-2 py-4">
                <div className="flex flex-wrap items-start justify-between gap-2">
                  <h2 className="text-base font-medium group-hover:text-primary">
                    {note.title || "无标题笔记"}
                  </h2>
                  <span className="text-xs text-muted-foreground">
                    更新于 {formatNoteTime(note.updatedAt)}
                  </span>
                </div>
                <p className="line-clamp-2 text-sm text-muted-foreground">
                  {note.excerpt || "（无正文）"}
                </p>
                {note.tags.length > 0 ? (
                  <div className="flex flex-wrap gap-1">
                    {note.tags.map((tag) => (
                      <Badge key={tag.id} variant="outline">
                        {tag.name}
                      </Badge>
                    ))}
                  </div>
                ) : null}
              </CardContent>
            </Card>
          </Link>
        ))}
      </div>

      {notes.length >= visibleCount && visibleCount < MAX_NOTES ? (
        <div className="flex justify-center">
          <Button
            variant="outline"
            onClick={() => setVisibleCount((value) => Math.min(MAX_NOTES, value + 50))}
          >
            加载更多
          </Button>
        </div>
      ) : null}
    </PageContainer>
  );
}
