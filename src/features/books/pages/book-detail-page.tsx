import { open } from "@tauri-apps/plugin-dialog";
import { ArrowLeft, BookOpen, FolderSearch, Pencil, Star, Trash2 } from "lucide-react";
import { useState } from "react";
import { Link, useNavigate, useParams } from "react-router";
import { toast } from "sonner";

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
import { useReadingProgress } from "@/features/reader/queries";
import { formatBytes } from "@/lib/format";
import { cn } from "@/lib/utils";

import { BookCover } from "../components/book-cover";
import { BookFormDialog } from "../components/book-form-dialog";
import { bookFormatLabels } from "../constants";
import {
  useBook,
  useDeleteBook,
  useRelocateBook,
  useSetBookFavorite,
  useUpdateBook,
} from "../queries";
import type { BookFormat } from "../types";

function formatTimestamp(value: number | null): string {
  if (!value) {
    return "—";
  }
  return new Date(value).toLocaleString("zh-CN", { hour12: false });
}

function formatFilter(format: BookFormat): { name: string; extensions: string[] } {
  switch (format) {
    case "epub":
      return { name: "EPUB", extensions: ["epub"] };
    case "pdf":
      return { name: "PDF", extensions: ["pdf"] };
    case "txt":
      return { name: "TXT", extensions: ["txt"] };
    case "markdown":
      return { name: "Markdown", extensions: ["md", "markdown"] };
  }
}

export function BookDetailPage() {
  const params = useParams();
  const navigate = useNavigate();
  const bookId = Number(params.bookId);
  const validId = Number.isInteger(bookId) && bookId > 0;

  const bookQuery = useBook(validId ? bookId : undefined);
  const readingProgress = useReadingProgress(validId ? bookId : 0);
  const updateMutation = useUpdateBook();
  const deleteMutation = useDeleteBook();
  const favoriteMutation = useSetBookFavorite();
  const relocateMutation = useRelocateBook();

  const [editOpen, setEditOpen] = useState(false);
  const [deleteOpen, setDeleteOpen] = useState(false);

  const book = bookQuery.data;

  if (!validId) {
    return (
      <PageContainer>
        <ErrorState title="书籍不存在" description="无效的书籍 ID。" />
      </PageContainer>
    );
  }

  if (bookQuery.isPending) {
    return (
      <PageContainer>
        <LoadingState label="正在读取书籍详情…" />
      </PageContainer>
    );
  }

  if (bookQuery.isError || !book) {
    return (
      <PageContainer>
        <ErrorState
          title="无法读取书籍详情"
          description={bookQuery.error?.message ?? "书籍不存在或已被删除。"}
          onRetry={() => void bookQuery.refetch()}
        />
      </PageContainer>
    );
  }

  async function handleRelocate() {
    if (!book) {
      return;
    }

    try {
      const selected = await open({
        multiple: false,
        directory: false,
        title: "重新定位书籍文件",
        filters: [formatFilter(book.format)],
      });

      if (typeof selected === "string") {
        relocateMutation.mutate({ id: book.id, newPath: selected });
      }
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "无法打开文件选择器");
    }
  }

  return (
    <PageContainer>
      <div>
        <Button asChild variant="ghost" size="sm" className="text-muted-foreground">
          <Link to="/bookshelf">
            <ArrowLeft className="size-4" aria-hidden="true" />
            返回书架
          </Link>
        </Button>
      </div>

      <PageHeader
        title={book.title}
        description={
          [book.author || null, bookFormatLabels[book.format]].filter(Boolean).join(" · ") ||
          undefined
        }
        actions={
          <>
            {book.format === "epub" && book.fileExists ? (
              <Button asChild size="sm">
                <Link to={`/reader/${book.id}`}>
                  <BookOpen className="size-4" aria-hidden="true" />
                  阅读
                </Link>
              </Button>
            ) : null}
            <Button
              variant="outline"
              size="sm"
              disabled={favoriteMutation.isPending}
              onClick={() => favoriteMutation.mutate({ id: book.id, favorite: !book.isFavorite })}
            >
              <Star
                className={book.isFavorite ? "fill-amber-300 text-amber-300" : "size-4"}
                aria-hidden="true"
              />
              {book.isFavorite ? "已收藏" : "收藏"}
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
        <Card className="gap-0 overflow-hidden bg-card/40 py-0">
          <div className="aspect-[3/4] w-full bg-muted">
            <BookCover coverUrl={book.coverUrl} title={book.title} className="h-full w-full" />
          </div>
        </Card>

        <Card className="bg-card/40 lg:col-span-2">
          <CardHeader>
            <CardTitle className="text-sm">书籍信息</CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="flex flex-wrap items-center gap-2">
              <Badge variant="outline">{bookFormatLabels[book.format]}</Badge>
              <Badge variant="outline">{formatBytes(book.fileSize)}</Badge>
              {book.isFavorite ? <Badge variant="outline">已收藏</Badge> : null}
              {!book.fileExists ? (
                <Badge
                  variant="outline"
                  className="border-destructive/40 bg-destructive/10 text-destructive"
                >
                  文件丢失
                </Badge>
              ) : null}
            </div>

            <div className="space-y-1">
              <p className="text-xs font-medium text-muted-foreground">分类</p>
              <p className="text-sm">{book.category || "未分类"}</p>
            </div>

            <div className="space-y-1">
              <p className="text-xs font-medium text-muted-foreground">标签</p>
              {book.tags.length > 0 ? (
                <div className="flex flex-wrap gap-1">
                  {book.tags.map((tag) => (
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
                {book.description || "暂无简介"}
              </p>
            </div>
          </CardContent>
        </Card>
      </div>

      <Card className={cn("bg-card/40", !book.fileExists && "border-destructive/40")}>
        <CardHeader className="flex flex-row items-center justify-between gap-3">
          <CardTitle className="text-sm">文件</CardTitle>
          <Button
            variant="outline"
            size="sm"
            disabled={relocateMutation.isPending}
            onClick={() => void handleRelocate()}
          >
            <FolderSearch className="size-4" aria-hidden="true" />
            {relocateMutation.isPending ? "定位中…" : "重新定位"}
          </Button>
        </CardHeader>
        <CardContent className="space-y-2 text-sm">
          <div className="flex items-center justify-between gap-3">
            <span className="text-muted-foreground">状态</span>
            <span className={book.fileExists ? "text-emerald-400" : "text-destructive"}>
              {book.fileExists ? "文件可用" : "文件不存在或不可访问"}
            </span>
          </div>
          <div className="flex flex-col gap-1 sm:flex-row sm:items-start sm:justify-between">
            <span className="text-muted-foreground">文件路径</span>
            <span className="text-right font-mono text-xs break-all">{book.filePath}</span>
          </div>
          {book.sourcePath ? (
            <div className="flex flex-col gap-1 sm:flex-row sm:items-start sm:justify-between">
              <span className="text-muted-foreground">原始位置</span>
              <span className="text-right font-mono text-xs break-all">{book.sourcePath}</span>
            </div>
          ) : null}
          <div className="flex items-center justify-between gap-3">
            <span className="text-muted-foreground">文件大小</span>
            <span>{formatBytes(book.fileSize)}</span>
          </div>
          <div className="flex items-center justify-between gap-3">
            <span className="text-muted-foreground">文件修改时间</span>
            <span>{formatTimestamp(book.fileMtime)}</span>
          </div>
          <div className="flex items-center justify-between gap-3">
            <span className="text-muted-foreground">导入时间</span>
            <span>{formatTimestamp(book.importedAt)}</span>
          </div>
        </CardContent>
      </Card>

      <Card className="bg-card/40">
        <CardHeader>
          <CardTitle className="text-sm">阅读</CardTitle>
        </CardHeader>
        <CardContent className="flex flex-wrap items-center justify-between gap-3">
          <div className="space-y-1">
            <p className="text-sm">
              {readingProgress.data
                ? `已阅读 ${(readingProgress.data.percent * 100).toFixed(1)}%`
                : "尚未开始阅读"}
            </p>
            <p className="text-xs text-muted-foreground">
              阅读进度会自动保存；书签可在阅读器内管理。
            </p>
          </div>

          {book.format === "epub" && book.fileExists ? (
            <Button asChild>
              <Link to={`/reader/${book.id}`}>
                <BookOpen className="size-4" aria-hidden="true" />
                {readingProgress.data ? "继续阅读" : "开始阅读"}
              </Link>
            </Button>
          ) : (
            <Button variant="outline" disabled>
              {book.fileExists ? "该格式阅读器待接入" : "文件丢失"}
            </Button>
          )}
        </CardContent>
      </Card>

      {editOpen ? (
        <BookFormDialog
          key={book.id}
          open
          onOpenChange={setEditOpen}
          book={book}
          pending={updateMutation.isPending}
          onSubmit={(input) =>
            updateMutation.mutate({ id: book.id, input }, { onSuccess: () => setEditOpen(false) })
          }
        />
      ) : null}

      <Dialog open={deleteOpen} onOpenChange={setDeleteOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>删除书籍？</DialogTitle>
            <DialogDescription>
              将删除《{book.title}
              》的书籍记录、封面和阅读数据。若使用了“复制到应用资料库”，应用内副本也会删除。
              原始文件永远不会被删除或修改。此操作不可撤销。
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
                deleteMutation.mutate(book.id, {
                  onSuccess: () => {
                    setDeleteOpen(false);
                    navigate("/bookshelf");
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
