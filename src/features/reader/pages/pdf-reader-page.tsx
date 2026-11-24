import type { PDFDocumentProxy, RenderTask } from "pdfjs-dist";
import {
  ArrowLeft,
  Bookmark as BookmarkIcon,
  BookOpen,
  ChevronLeft,
  ChevronRight,
  List,
  Minus,
  Plus,
} from "lucide-react";
import { useCallback, useEffect, useRef, useState } from "react";
import type { WheelEvent as ReactWheelEvent } from "react";
import { Link, useParams } from "react-router";
import { toast } from "sonner";

import { EmptyState } from "@/components/common/empty-state";
import { ErrorState } from "@/components/common/error-state";
import { LoadingState } from "@/components/common/loading-state";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { useBook } from "@/features/books/queries";

import * as readerApi from "../api";
import { BookmarksPanel } from "../components/bookmarks-panel";
import {
  loadPdfDocument,
  parsePdfLocation,
  pdfLocationJson,
  renderPdfPage,
  type PdfZoomMode,
} from "../pdf/pdf-renderer";
import { useBookmarks, useCreateBookmark, useDeleteBookmark, useReadingProgress } from "../queries";
import type { Bookmark } from "../types";
import { createWheelPager } from "../wheel-paging";

function errorText(error: unknown, fallback: string): string {
  return error instanceof Error ? error.message : fallback;
}

function isRenderCancelled(error: unknown): boolean {
  return (
    typeof error === "object" &&
    error !== null &&
    (error as { name?: string }).name === "RenderingCancelledException"
  );
}

export function PdfReaderPage() {
  const params = useParams();
  const bookId = Number(params.bookId);
  const validId = Number.isInteger(bookId) && bookId > 0;

  const bookQuery = useBook(validId ? bookId : undefined);
  const progressQuery = useReadingProgress(bookId);
  const bookmarksQuery = useBookmarks(bookId);
  const createBookmark = useCreateBookmark(bookId);
  const deleteBookmark = useDeleteBookmark(bookId);

  const canvasRef = useRef<HTMLCanvasElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const documentRef = useRef<PDFDocumentProxy | null>(null);
  const renderTaskRef = useRef<RenderTask | null>(null);
  const sessionIdRef = useRef<number | null>(null);
  const saveTimeoutRef = useRef<number | null>(null);
  const pageRef = useRef(1);
  const zoomModeRef = useRef<PdfZoomMode>("fit-width");
  const customScaleRef = useRef(1);
  const effectiveScaleRef = useRef(1);
  const totalPagesRef = useRef(0);

  const [pdfDocument, setPdfDocument] = useState<PDFDocumentProxy | null>(null);
  const [totalPages, setTotalPages] = useState(0);
  const [pageNumber, setPageNumber] = useState(1);
  const [zoomMode, setZoomMode] = useState<PdfZoomMode>("fit-width");
  const [customScale, setCustomScale] = useState(1);
  const [effectiveScale, setEffectiveScale] = useState(1);
  const [percent, setPercent] = useState(0);
  const [containerSize, setContainerSize] = useState({ width: 0, height: 0 });
  const [documentLoading, setDocumentLoading] = useState(true);
  const [documentError, setDocumentError] = useState<string | null>(null);
  const [renderError, setRenderError] = useState<string | null>(null);
  const [retryKey, setRetryKey] = useState(0);
  const [bookmarksOpen, setBookmarksOpen] = useState(false);
  const [bookmarkDialogOpen, setBookmarkDialogOpen] = useState(false);
  const [bookmarkLabel, setBookmarkLabel] = useState("");

  const loadedBookId = bookQuery.data?.id ?? null;
  const bookFormat = bookQuery.data?.format ?? null;
  const savedLocationJson = progressQuery.data?.locationJson ?? null;
  const progressFetched = progressQuery.isFetched;

  useEffect(() => {
    pageRef.current = pageNumber;
  }, [pageNumber]);

  useEffect(() => {
    zoomModeRef.current = zoomMode;
  }, [zoomMode]);

  useEffect(() => {
    customScaleRef.current = customScale;
  }, [customScale]);

  useEffect(() => {
    effectiveScaleRef.current = effectiveScale;
  }, [effectiveScale]);

  useEffect(() => {
    totalPagesRef.current = totalPages;
  }, [totalPages]);

  const persistProgress = useCallback(() => {
    const total = totalPagesRef.current;
    if (!total) {
      return;
    }

    const page = Math.min(total, Math.max(1, pageRef.current));

    void readerApi
      .saveReadingProgress(bookId, {
        locationType: "pdf_page",
        locationJson: pdfLocationJson(
          page,
          zoomModeRef.current,
          effectiveScaleRef.current || customScaleRef.current,
        ),
        percent: page / total,
      })
      .catch((error: unknown) => {
        toast.error(errorText(error, "保存阅读进度失败"));
      });
  }, [bookId]);

  const scheduleProgressSave = useCallback(() => {
    if (saveTimeoutRef.current !== null) {
      window.clearTimeout(saveTimeoutRef.current);
    }

    saveTimeoutRef.current = window.setTimeout(() => {
      persistProgress();
    }, 500);
  }, [persistProgress]);

  const goToPage = useCallback((nextPage: number) => {
    const total = totalPagesRef.current || 1;
    const page = Math.min(total, Math.max(1, nextPage));
    pageRef.current = page;
    setPageNumber(page);
    setRenderError(null);
  }, []);

  const zoomBy = useCallback((factor: number) => {
    const base = effectiveScaleRef.current || customScaleRef.current || 1;
    const scale = Math.min(5, Math.max(0.2, base * factor));
    customScaleRef.current = scale;
    zoomModeRef.current = "custom";
    setCustomScale(scale);
    setZoomMode("custom");
    setRenderError(null);
  }, []);

  const wheelPagerRef = useRef<ReturnType<typeof createWheelPager> | null>(null);

  useEffect(() => {
    if (!validId || !loadedBookId || !progressFetched || bookFormat !== "pdf") {
      return;
    }

    let cancelled = false;

    async function load() {
      try {
        const data = await readerApi.readBookBytes(bookId);
        if (cancelled) {
          return;
        }

        const document = await loadPdfDocument(data);
        if (cancelled) {
          void document.loadingTask.destroy();
          return;
        }

        documentRef.current = document;
        setPdfDocument(document);
        setTotalPages(document.numPages);
        totalPagesRef.current = document.numPages;

        const saved = parsePdfLocation(savedLocationJson);
        const page = Math.min(document.numPages, Math.max(1, saved?.page ?? 1));
        const mode = saved?.mode ?? "fit-width";
        const scale = saved?.scale ?? 1;

        pageRef.current = page;
        zoomModeRef.current = mode;
        customScaleRef.current = scale;
        setPageNumber(page);
        setZoomMode(mode);
        setCustomScale(scale);
        setDocumentLoading(false);

        void readerApi
          .startReadingSession(bookId)
          .then((sessionId) => {
            if (cancelled) {
              void readerApi.endReadingSession(sessionId, null);
              return;
            }
            sessionIdRef.current = sessionId;
          })
          .catch((error: unknown) => {
            console.error("[AniShelf] 无法开始阅读会话", error);
          });
      } catch (error) {
        if (!cancelled) {
          setDocumentError(errorText(error, "无法打开 PDF"));
          setDocumentLoading(false);
        }
      }
    }

    void load();

    return () => {
      cancelled = true;
      renderTaskRef.current?.cancel();
      renderTaskRef.current = null;

      if (saveTimeoutRef.current !== null) {
        window.clearTimeout(saveTimeoutRef.current);
        saveTimeoutRef.current = null;
      }

      persistProgress();

      const sessionId = sessionIdRef.current;
      sessionIdRef.current = null;
      if (sessionId !== null) {
        const total = totalPagesRef.current;
        const location = total
          ? pdfLocationJson(
              Math.min(total, Math.max(1, pageRef.current)),
              zoomModeRef.current,
              effectiveScaleRef.current || customScaleRef.current,
            )
          : null;
        void readerApi.endReadingSession(sessionId, location).catch(() => undefined);
      }

      const document = documentRef.current;
      documentRef.current = null;
      if (document) {
        void document.loadingTask.destroy();
      }
    };
  }, [
    bookFormat,
    bookId,
    loadedBookId,
    persistProgress,
    progressFetched,
    retryKey,
    savedLocationJson,
    validId,
  ]);

  useEffect(() => {
    const element = containerRef.current;
    if (!element) {
      return;
    }

    let timer = 0;
    const observer = new ResizeObserver(() => {
      window.clearTimeout(timer);
      timer = window.setTimeout(() => {
        setContainerSize({
          width: element.clientWidth,
          height: element.clientHeight,
        });
      }, 120);
    });

    observer.observe(element);
    return () => {
      observer.disconnect();
      window.clearTimeout(timer);
    };
  }, []);

  useEffect(() => {
    if (
      !pdfDocument ||
      !canvasRef.current ||
      !containerRef.current ||
      containerSize.width === 0 ||
      containerSize.height === 0
    ) {
      return;
    }

    let cancelled = false;

    async function render() {
      try {
        renderTaskRef.current?.cancel();
        const result = await renderPdfPage(
          pdfDocument as PDFDocumentProxy,
          pageRef.current,
          zoomModeRef.current,
          customScaleRef.current,
          containerRef.current as HTMLDivElement,
          canvasRef.current as HTMLCanvasElement,
          (task) => {
            renderTaskRef.current = task;
          },
        );

        if (cancelled) {
          return;
        }

        effectiveScaleRef.current = result.scale;
        setEffectiveScale(result.scale);
        setRenderError(null);
        setPercent(pageRef.current / (totalPagesRef.current || 1));
        scheduleProgressSave();
      } catch (error) {
        if (!cancelled && !isRenderCancelled(error)) {
          setRenderError(errorText(error, "渲染 PDF 页面失败"));
        }
      }
    }

    void render();

    return () => {
      cancelled = true;
      renderTaskRef.current?.cancel();
      renderTaskRef.current = null;
    };
  }, [containerSize, customScale, pageNumber, pdfDocument, scheduleProgressSave, zoomMode]);

  useEffect(() => {
    function handleKeyDown(event: KeyboardEvent) {
      if (document.querySelector('[role="dialog"]')) {
        return;
      }

      const target = event.target;
      if (
        target instanceof HTMLElement &&
        ["INPUT", "TEXTAREA", "SELECT"].includes(target.tagName)
      ) {
        return;
      }

      if (event.key === "ArrowLeft" || event.key === "PageUp") {
        event.preventDefault();
        goToPage(pageRef.current - 1);
      } else if (event.key === "ArrowRight" || event.key === "PageDown" || event.key === " ") {
        event.preventDefault();
        goToPage(pageRef.current + 1);
      } else if (event.key === "+" || event.key === "=") {
        event.preventDefault();
        zoomBy(1.15);
      } else if (event.key === "-") {
        event.preventDefault();
        zoomBy(1 / 1.15);
      }
    }

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [goToPage, zoomBy]);

  function handleWheel(event: ReactWheelEvent<HTMLDivElement>) {
    if (document.querySelector('[role="dialog"]')) {
      return;
    }
    if (event.ctrlKey || event.metaKey || event.deltaY === 0) {
      return;
    }
    if (totalPagesRef.current === 0) {
      return;
    }

    const container = containerRef.current;
    if (!container) {
      return;
    }

    const canScrollDown = container.scrollTop + container.clientHeight < container.scrollHeight - 1;
    const canScrollUp = container.scrollTop > 0;
    if ((event.deltaY > 0 && canScrollDown) || (event.deltaY < 0 && canScrollUp)) {
      return;
    }

    wheelPagerRef.current ??= createWheelPager({
      onNext: () => goToPage(pageRef.current + 1),
      onPrevious: () => goToPage(pageRef.current - 1),
    });

    if (wheelPagerRef.current(event.nativeEvent)) {
      container.scrollTo({ top: 0, left: 0 });
    }
  }

  function changeZoomMode(mode: PdfZoomMode) {
    zoomModeRef.current = mode;
    setZoomMode(mode);
    setRenderError(null);
  }

  function retry() {
    setDocumentError(null);
    setDocumentLoading(true);
    setPdfDocument(null);
    setRetryKey((value) => value + 1);
  }

  function openBookmarkDialog() {
    setBookmarkLabel(`第 ${pageRef.current} 页`);
    setBookmarkDialogOpen(true);
  }

  function saveBookmark() {
    createBookmark.mutate(
      {
        locationType: "pdf_page",
        locationJson: pdfLocationJson(
          pageRef.current,
          zoomModeRef.current,
          effectiveScaleRef.current || customScaleRef.current,
        ),
        label: bookmarkLabel.trim() || `第 ${pageRef.current} 页`,
      },
      { onSuccess: () => setBookmarkDialogOpen(false) },
    );
  }

  function jumpToBookmark(bookmark: Bookmark) {
    const location = parsePdfLocation(bookmark.locationJson);
    if (!location) {
      toast.error("书签位置无效");
      return;
    }

    const total = totalPagesRef.current || 1;
    const page = Math.min(total, Math.max(1, location.page));
    pageRef.current = page;
    zoomModeRef.current = location.mode;
    customScaleRef.current = location.scale;
    setPageNumber(page);
    setZoomMode(location.mode);
    setCustomScale(location.scale);
  }

  if (!validId) {
    return (
      <div className="flex h-[calc(100vh-3.5rem)] items-center justify-center px-6">
        <div className="w-full max-w-lg">
          <ErrorState title="阅读器无法打开" description="无效的书籍 ID。" />
        </div>
      </div>
    );
  }

  if (bookQuery.isPending) {
    return (
      <div className="flex h-[calc(100vh-3.5rem)] items-center justify-center">
        <LoadingState label="正在读取书籍信息…" />
      </div>
    );
  }

  if (bookQuery.isError || !bookQuery.data) {
    return (
      <div className="flex h-[calc(100vh-3.5rem)] items-center justify-center px-6">
        <div className="w-full max-w-lg">
          <ErrorState
            title="无法读取书籍"
            description={bookQuery.error?.message ?? "书籍不存在或已被删除。"}
            onRetry={() => void bookQuery.refetch()}
          />
        </div>
      </div>
    );
  }

  if (bookQuery.data.format !== "pdf") {
    return (
      <div className="flex h-[calc(100vh-3.5rem)] items-center justify-center px-6">
        <div className="w-full max-w-lg">
          <ErrorState title="格式不匹配" description="该书籍不是 PDF。" />
        </div>
      </div>
    );
  }

  if (!bookQuery.data.fileExists) {
    return (
      <div className="flex h-[calc(100vh-3.5rem)] items-center justify-center px-6">
        <div className="w-full max-w-lg">
          <EmptyState
            icon={BookOpen}
            title="书籍文件不存在"
            description="原始文件可能已被移动或删除，请返回书籍详情重新定位。"
            action={
              <Button asChild>
                <Link to={`/bookshelf/${bookId}`}>返回书籍详情</Link>
              </Button>
            }
          />
        </div>
      </div>
    );
  }

  const book = bookQuery.data;

  return (
    <div className="flex h-[calc(100vh-3.5rem)] flex-col bg-background">
      <header className="flex h-12 shrink-0 items-center justify-between gap-2 border-b border-border px-3">
        <div className="flex min-w-0 items-center gap-1">
          <Button asChild variant="ghost" size="icon-sm" title="返回书籍详情">
            <Link to={`/bookshelf/${book.id}`} aria-label="返回书籍详情">
              <ArrowLeft className="size-4" aria-hidden="true" />
            </Link>
          </Button>
          <div className="min-w-0 px-1">
            <p className="truncate text-sm font-medium">{book.title}</p>
            <p className="truncate text-[11px] text-muted-foreground">
              第 {pageNumber} / {totalPages || "—"} 页
            </p>
          </div>
        </div>

        <div className="flex items-center gap-0.5">
          <Button
            variant="ghost"
            size="icon-sm"
            title="上一页"
            aria-label="上一页"
            disabled={documentLoading || pageNumber <= 1}
            onClick={() => goToPage(pageNumber - 1)}
          >
            <ChevronLeft className="size-4" aria-hidden="true" />
          </Button>
          <Input
            type="number"
            min={1}
            max={totalPages || 1}
            value={pageNumber}
            aria-label="页码"
            className="h-8 w-16 text-center"
            onChange={(event) => {
              const value = Number(event.target.value);
              if (Number.isInteger(value) && value >= 1 && value <= (totalPagesRef.current || 1)) {
                goToPage(value);
              }
            }}
          />
          <Button
            variant="ghost"
            size="icon-sm"
            title="下一页"
            aria-label="下一页"
            disabled={documentLoading || pageNumber >= totalPages}
            onClick={() => goToPage(pageNumber + 1)}
          >
            <ChevronRight className="size-4" aria-hidden="true" />
          </Button>

          <Button
            variant="ghost"
            size="icon-sm"
            title="缩小"
            aria-label="缩小"
            disabled={documentLoading}
            onClick={() => zoomBy(1 / 1.15)}
          >
            <Minus className="size-4" aria-hidden="true" />
          </Button>
          <span className="w-11 text-center text-[11px] text-muted-foreground">
            {Math.round(effectiveScale * 100)}%
          </span>
          <Button
            variant="ghost"
            size="icon-sm"
            title="放大"
            aria-label="放大"
            disabled={documentLoading}
            onClick={() => zoomBy(1.15)}
          >
            <Plus className="size-4" aria-hidden="true" />
          </Button>

          <Button
            variant={zoomMode === "fit-width" ? "secondary" : "ghost"}
            size="sm"
            title="适合宽度"
            onClick={() => changeZoomMode("fit-width")}
          >
            适宽
          </Button>
          <Button
            variant={zoomMode === "fit-page" ? "secondary" : "ghost"}
            size="sm"
            title="适合页面"
            onClick={() => changeZoomMode("fit-page")}
          >
            适页
          </Button>

          <Button
            variant="ghost"
            size="icon-sm"
            title="添加书签"
            aria-label="添加书签"
            disabled={documentLoading || totalPages === 0}
            onClick={openBookmarkDialog}
          >
            <BookmarkIcon className="size-4" aria-hidden="true" />
          </Button>
          <Button
            variant={bookmarksOpen ? "secondary" : "ghost"}
            size="icon-sm"
            title="书签列表"
            aria-label="书签列表"
            onClick={() => setBookmarksOpen((value) => !value)}
          >
            <List className="size-4" aria-hidden="true" />
          </Button>
        </div>
      </header>

      <div className="flex min-h-0 flex-1">
        <div
          ref={containerRef}
          className="relative min-w-0 flex-1 overflow-auto bg-muted/20 p-3"
          onWheel={handleWheel}
        >
          <div className="flex min-h-full items-start justify-center">
            <canvas ref={canvasRef} className="shadow-xl" />
          </div>

          {documentLoading ? (
            <div className="absolute inset-0 flex items-center justify-center bg-background/95">
              <LoadingState label="正在打开 PDF…" />
            </div>
          ) : null}

          {documentError ? (
            <div className="absolute inset-0 flex items-center justify-center bg-background px-6">
              <div className="w-full max-w-lg">
                <ErrorState title="无法打开 PDF" description={documentError} onRetry={retry} />
              </div>
            </div>
          ) : null}

          {renderError ? (
            <div className="absolute inset-0 flex items-center justify-center bg-background px-6">
              <div className="w-full max-w-lg">
                <ErrorState title="无法渲染页面" description={renderError} onRetry={retry} />
              </div>
            </div>
          ) : null}
        </div>

        {bookmarksOpen ? (
          <BookmarksPanel
            bookmarks={bookmarksQuery.data ?? []}
            isLoading={bookmarksQuery.isPending}
            isError={bookmarksQuery.isError}
            onSelect={jumpToBookmark}
            onDelete={(bookmark) => deleteBookmark.mutate(bookmark.id)}
          />
        ) : null}
      </div>

      <footer className="flex h-9 shrink-0 items-center justify-between border-t border-border px-3 text-xs text-muted-foreground">
        <span>
          第 {pageNumber} / {totalPages || "—"} 页
        </span>
        <span>全书 {(percent * 100).toFixed(1)}%</span>
      </footer>

      <Dialog open={bookmarkDialogOpen} onOpenChange={setBookmarkDialogOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>添加书签</DialogTitle>
            <DialogDescription>保存当前页码，便于之后快速返回。</DialogDescription>
          </DialogHeader>

          <div className="space-y-1">
            <Label htmlFor="pdf-bookmark-label">书签名称</Label>
            <Input
              id="pdf-bookmark-label"
              maxLength={200}
              value={bookmarkLabel}
              onChange={(event) => setBookmarkLabel(event.target.value)}
            />
          </div>

          <DialogFooter>
            <Button variant="outline" onClick={() => setBookmarkDialogOpen(false)}>
              取消
            </Button>
            <Button disabled={createBookmark.isPending} onClick={saveBookmark}>
              {createBookmark.isPending ? "保存中…" : "保存书签"}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
