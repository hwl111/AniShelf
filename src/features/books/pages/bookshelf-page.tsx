import { Library, Star, Upload } from "lucide-react";
import { useMemo, useState } from "react";
import { Link, useNavigate, useSearchParams } from "react-router";

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
import { formatBytes } from "@/lib/format";

import { BookCover } from "../components/book-cover";
import { BookImportDialog } from "../components/book-import-dialog";
import { bookFormatLabels, bookFormatOptions, bookSortOptions } from "../constants";
import { useBooks, useBookTags } from "../queries";
import type { BookFilter, BookFormat, BookSort } from "../types";

const selectClassName =
  "h-8 w-full rounded-lg border border-input bg-transparent px-2.5 text-sm outline-none transition-colors focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/50 dark:bg-input/30";

export function BookshelfPage() {
  const navigate = useNavigate();
  const [formatFilter, setFormatFilter] = useState<BookFormat | "">("");
  const [categoryFilter, setCategoryFilter] = useState("");
  const [tagFilter, setTagFilter] = useState("");
  const [favoriteFilter, setFavoriteFilter] = useState<"" | "true" | "false">("");
  const [sort, setSort] = useState<BookSort>("updated_desc");
  const [searchParams, setSearchParams] = useSearchParams();
  const [importOpen, setImportOpen] = useState(() => searchParams.get("action") === "import");

  function closeImport(open: boolean) {
    setImportOpen(open);
    if (!open && searchParams.get("action") === "import") {
      const next = new URLSearchParams(searchParams);
      next.delete("action");
      setSearchParams(next, { replace: true });
    }
  }

  const filter = useMemo<BookFilter>(
    () => ({
      format: formatFilter || undefined,
      category: categoryFilter.trim() || undefined,
      tagId: tagFilter ? Number(tagFilter) : undefined,
      favorite: favoriteFilter === "" ? undefined : favoriteFilter === "true",
      sort,
    }),
    [formatFilter, categoryFilter, tagFilter, favoriteFilter, sort],
  );

  const booksQuery = useBooks(filter);
  const tagsQuery = useBookTags();
  const books = booksQuery.data ?? [];

  return (
    <PageContainer>
      <PageHeader
        title="书架"
        description="管理本地电子书与阅读进度"
        actions={
          <Button onClick={() => setImportOpen(true)}>
            <Upload className="size-4" aria-hidden="true" />
            导入书籍
          </Button>
        }
      />

      <Card className="bg-card/40">
        <CardContent className="grid gap-3 py-4 sm:grid-cols-2 lg:grid-cols-5">
          <div className="space-y-1">
            <Label htmlFor="book-filter-format">格式</Label>
            <select
              id="book-filter-format"
              className={selectClassName}
              value={formatFilter}
              onChange={(event) => setFormatFilter(event.target.value as BookFormat | "")}
            >
              <option value="">全部格式</option>
              {bookFormatOptions.map((option) => (
                <option key={option.value} value={option.value}>
                  {option.label}
                </option>
              ))}
            </select>
          </div>

          <div className="space-y-1">
            <Label htmlFor="book-filter-category">分类</Label>
            <Input
              id="book-filter-category"
              placeholder="全部分类"
              value={categoryFilter}
              onChange={(event) => setCategoryFilter(event.target.value)}
            />
          </div>

          <div className="space-y-1">
            <Label htmlFor="book-filter-tag">标签</Label>
            <select
              id="book-filter-tag"
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
            <Label htmlFor="book-filter-favorite">收藏</Label>
            <select
              id="book-filter-favorite"
              className={selectClassName}
              value={favoriteFilter}
              onChange={(event) => setFavoriteFilter(event.target.value as "" | "true" | "false")}
            >
              <option value="">全部</option>
              <option value="true">已收藏</option>
              <option value="false">未收藏</option>
            </select>
          </div>

          <div className="space-y-1">
            <Label htmlFor="book-filter-sort">排序</Label>
            <select
              id="book-filter-sort"
              className={selectClassName}
              value={sort}
              onChange={(event) => setSort(event.target.value as BookSort)}
            >
              {bookSortOptions.map((option) => (
                <option key={option.value} value={option.value}>
                  {option.label}
                </option>
              ))}
            </select>
          </div>
        </CardContent>
      </Card>

      {booksQuery.isPending ? <LoadingState label="正在读取书架…" /> : null}

      {booksQuery.isError ? (
        <ErrorState
          title="无法读取书架"
          description={booksQuery.error.message}
          onRetry={() => void booksQuery.refetch()}
        />
      ) : null}

      {booksQuery.data && books.length === 0 ? (
        <EmptyState
          icon={Library}
          title="书架还是空的"
          description="导入 EPUB、PDF、TXT 或 Markdown 文件后，会显示在这里。"
          action={
            <Button onClick={() => setImportOpen(true)}>
              <Upload className="size-4" aria-hidden="true" />
              导入书籍
            </Button>
          }
        />
      ) : null}

      {books.length > 0 ? (
        <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
          {books.map((book) => (
            <Link key={book.id} to={`/bookshelf/${book.id}`} className="group block">
              <Card className="h-full gap-0 overflow-hidden bg-card/40 py-0 transition-colors group-hover:border-ring/40 group-hover:bg-card">
                <div className="aspect-[3/4] w-full overflow-hidden bg-muted">
                  <BookCover
                    book={book}
                    coverUrl={book.coverUrl}
                    title={book.title}
                    className="h-full w-full transition-transform group-hover:scale-[1.02]"
                  />
                </div>
                <CardContent className="space-y-2 p-4">
                  <div className="flex items-start justify-between gap-2">
                    <p className="line-clamp-2 font-medium">{book.title}</p>
                    {book.isFavorite ? (
                      <Star
                        className="size-4 shrink-0 fill-amber-300 text-amber-300"
                        aria-label="已收藏"
                      />
                    ) : null}
                  </div>

                  {book.author ? (
                    <p className="truncate text-xs text-muted-foreground">{book.author}</p>
                  ) : null}

                  <div className="flex flex-wrap gap-1.5">
                    <Badge variant="outline">{bookFormatLabels[book.format]}</Badge>
                    <Badge variant="outline">{formatBytes(book.fileSize)}</Badge>
                    {!book.fileExists ? (
                      <Badge
                        variant="outline"
                        className="border-destructive/40 bg-destructive/10 text-destructive"
                      >
                        文件丢失
                      </Badge>
                    ) : null}
                  </div>

                  {book.tags.length > 0 ? (
                    <div className="flex flex-wrap gap-1">
                      {book.tags.map((tag) => (
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
          ))}
        </div>
      ) : null}

      {importOpen ? (
        <BookImportDialog
          open
          onOpenChange={closeImport}
          onImported={(book) => navigate(`/bookshelf/${book.id}`)}
        />
      ) : null}
    </PageContainer>
  );
}
