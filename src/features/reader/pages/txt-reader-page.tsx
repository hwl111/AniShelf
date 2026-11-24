import {
  ArrowDownToLine,
  ArrowLeft,
  ArrowUpToLine,
  Bookmark as BookmarkIcon,
  BookOpen,
  List,
  Settings,
} from "lucide-react";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
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
import { ReaderSettingsDialog } from "../components/reader-settings-dialog";
import { parseReaderPreferences, readerThemeColors } from "../preferences";
import {
  useBookmarks,
  useCreateBookmark,
  useDeleteBookmark,
  useReadingProgress,
  useReaderSettings,
  useTextBookInfo,
} from "../queries";
import type { Bookmark, ReaderPreferences, TextChunk } from "../types";

const CHUNK_BYTES = 64 * 1024;
const OVERSCAN_BYTES = 400;

function errorText(error: unknown, fallback: string): string {
  return error instanceof Error ? error.message : fallback;
}

function parseTextOffset(locationJson: string | null | undefined): number | null {
  if (!locationJson) {
    return null;
  }

  try {
    const value = JSON.parse(locationJson) as { offset?: unknown };
    return typeof value.offset === "number" && Number.isFinite(value.offset)
      ? Math.max(0, Math.floor(value.offset))
      : null;
  } catch {
    return null;
  }
}

function textLocationJson(offset: number, encoding: string): string {
  return JSON.stringify({ offset, encoding });
}

function formatBytes(value: number): string {
  if (value < 1024) {
    return `${value} B`;
  }

  if (value < 1024 * 1024) {
    return `${(value / 1024).toFixed(1)} KB`;
  }

  return `${(value / (1024 * 1024)).toFixed(1)} MB`;
}

export function TxtReaderPage() {
  const params = useParams();
  const bookId = Number(params.bookId);
  const validId = Number.isInteger(bookId) && bookId > 0;

  const bookQuery = useBook(validId ? bookId : undefined);
  const infoQuery = useTextBookInfo(bookId);
  const progressQuery = useReadingProgress(bookId);
  const settingsQuery = useReaderSettings("txt");
  const bookmarksQuery = useBookmarks(bookId);
  const createBookmark = useCreateBookmark(bookId);
  const deleteBookmark = useDeleteBookmark(bookId);

  const scrollRef = useRef<HTMLDivElement>(null);
  const sentinelRef = useRef<HTMLDivElement>(null);
  const anchorOffsetRef = useRef(0);
  const totalBytesRef = useRef(0);
  const encodingRef = useRef("utf-8");
  const sessionIdRef = useRef<number | null>(null);
  const saveTimeoutRef = useRef<number | null>(null);
  const settingsSaveTimeoutRef = useRef<number | null>(null);
  const scrollTimerRef = useRef<number | null>(null);
  const loadingMoreRef = useRef(false);
  const jumpingRef = useRef(false);
  const generationRef = useRef(0);
  const initializedRef = useRef(false);
  const initialLocationRef = useRef<string | null | undefined>(undefined);
  const initialLocationBookRef = useRef<number | null>(null);

  const [localPreferences, setLocalPreferences] = useState<ReaderPreferences | null>(null);
  const parsedPreferences = useMemo(
    () => parseReaderPreferences(settingsQuery.data?.preferencesJson),
    [settingsQuery.data?.preferencesJson],
  );
  const preferences = localPreferences ?? parsedPreferences;

  const [chunks, setChunks] = useState<TextChunk[]>([]);
  const [percent, setPercent] = useState(0);
  const [initialLoading, setInitialLoading] = useState(true);
  const [loadingMore, setLoadingMore] = useState(false);
  const [readerError, setReaderError] = useState<string | null>(null);
  const [retryKey, setRetryKey] = useState(0);
  const [bookmarksOpen, setBookmarksOpen] = useState(false);
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [bookmarkDialogOpen, setBookmarkDialogOpen] = useState(false);
  const [bookmarkLabel, setBookmarkLabel] = useState("");

  const info = infoQuery.data ?? null;
  const book = bookQuery.data ?? null;
  const loadedBookId = book?.id ?? null;
  const bookFormat = book?.format ?? null;
  const progressFetched = progressQuery.isFetched;
  const settingsFetched = settingsQuery.isFetched;
  const totalBytes = info?.totalBytes ?? 0;
  const infoEncoding = info?.encoding ?? null;
  const infoTotalBytes = info?.totalBytes ?? 0;

  const colors = readerThemeColors[preferences.theme];
  const decodeWarning =
    info?.warning ??
    (chunks.some((chunk) => chunk.hadErrors)
      ? "部分文本无法按检测到的编码解码，可能出现乱码。"
      : null);

  useEffect(() => {
    if (progressFetched && initialLocationBookRef.current !== bookId) {
      initialLocationRef.current = progressQuery.data?.locationJson ?? null;
      initialLocationBookRef.current = bookId;
    }
  }, [bookId, progressFetched, progressQuery.data?.locationJson]);

  const persistProgress = useCallback(() => {
    const total = totalBytesRef.current;
    if (total <= 0) {
      return;
    }

    const offset = Math.min(total, Math.max(0, Math.round(anchorOffsetRef.current)));

    void readerApi
      .saveReadingProgress(bookId, {
        locationType: "txt_offset",
        locationJson: textLocationJson(offset, encodingRef.current),
        percent: total > 0 ? offset / total : 0,
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
      saveTimeoutRef.current = null;
      persistProgress();
    }, 800);
  }, [persistProgress]);

  const updateAnchorFromScroll = useCallback(() => {
    const root = scrollRef.current;
    if (!root) {
      return;
    }

    const rootTop = root.getBoundingClientRect().top;
    const sections = root.querySelectorAll<HTMLElement>("[data-offset]");
    let nextOffset = anchorOffsetRef.current;

    for (const section of Array.from(sections)) {
      const rect = section.getBoundingClientRect();
      if (rect.bottom <= rootTop + 16) {
        continue;
      }

      const start = Number(section.dataset.offset);
      const end = Number(section.dataset.end);
      if (!Number.isFinite(start) || !Number.isFinite(end) || end <= start) {
        break;
      }

      const fraction = Math.min(1, Math.max(0, (rootTop - rect.top) / Math.max(1, rect.height)));
      nextOffset = Math.round(start + fraction * (end - start));
      break;
    }

    if (nextOffset === anchorOffsetRef.current) {
      return;
    }

    anchorOffsetRef.current = nextOffset;
    const total = totalBytesRef.current;
    setPercent(total > 0 ? Math.min(1, nextOffset / total) : 0);
    scheduleProgressSave();
  }, [scheduleProgressSave]);

  const handleScroll = useCallback(() => {
    if (scrollTimerRef.current !== null) {
      return;
    }

    scrollTimerRef.current = window.setTimeout(() => {
      scrollTimerRef.current = null;
      updateAnchorFromScroll();
    }, 120);
  }, [updateAnchorFromScroll]);

  const loadMore = useCallback(async () => {
    const last = chunks[chunks.length - 1];
    if (!info || !last || last.eof || loadingMoreRef.current || jumpingRef.current) {
      return;
    }

    const generation = generationRef.current;
    loadingMoreRef.current = true;
    setLoadingMore(true);

    try {
      const chunk = await readerApi.readTextChunk(
        bookId,
        last.nextOffset,
        CHUNK_BYTES,
        info.encoding,
      );

      if (generation !== generationRef.current) {
        return;
      }

      setChunks((previous) => {
        if (previous.some((item) => item.startOffset === chunk.startOffset)) {
          return previous;
        }

        return [...previous, chunk];
      });
    } catch (error) {
      if (generation === generationRef.current) {
        toast.error(errorText(error, "加载后续文本失败"));
      }
    } finally {
      if (generation === generationRef.current) {
        loadingMoreRef.current = false;
        setLoadingMore(false);
      }
    }
  }, [bookId, chunks, info]);

  const jumpToOffset = useCallback(
    async (offset: number) => {
      if (!info || info.totalBytes <= 0 || initialLoading || jumpingRef.current) {
        return;
      }

      jumpingRef.current = true;
      generationRef.current += 1;
      loadingMoreRef.current = false;
      setLoadingMore(false);
      setInitialLoading(true);
      setReaderError(null);

      const generation = generationRef.current;
      const target = Math.max(0, Math.min(info.totalBytes - 1, Math.round(offset)));

      try {
        const chunk = await readerApi.readTextChunk(bookId, target, CHUNK_BYTES, info.encoding);
        if (generation !== generationRef.current) {
          return;
        }

        setChunks([chunk]);
        initializedRef.current = true;
        anchorOffsetRef.current = chunk.startOffset;
        setPercent(info.totalBytes > 0 ? Math.min(1, chunk.startOffset / info.totalBytes) : 0);
        setInitialLoading(false);
        scrollRef.current?.scrollTo({ top: 0 });
        scheduleProgressSave();
      } catch (error) {
        if (generation === generationRef.current) {
          setReaderError(errorText(error, "无法跳转到指定位置"));
          setInitialLoading(false);
        }
      } finally {
        if (generation === generationRef.current) {
          jumpingRef.current = false;
        }
      }
    },
    [bookId, info, initialLoading, scheduleProgressSave],
  );

  useEffect(() => {
    if (
      !validId ||
      !loadedBookId ||
      bookFormat !== "txt" ||
      !infoEncoding ||
      infoTotalBytes <= 0 ||
      !progressFetched ||
      !settingsFetched
    ) {
      return;
    }

    let cancelled = false;
    generationRef.current += 1;
    const generation = generationRef.current;
    jumpingRef.current = false;
    loadingMoreRef.current = false;
    initializedRef.current = false;
    totalBytesRef.current = infoTotalBytes;
    encodingRef.current = infoEncoding;
    const encoding = infoEncoding;
    const total = infoTotalBytes;

    async function initialize() {
      const saved = parseTextOffset(initialLocationRef.current);
      const target = Math.min(Math.max(0, total - 1), Math.max(0, saved ?? 0));

      try {
        const chunk = await readerApi.readTextChunk(bookId, target, CHUNK_BYTES, encoding);

        if (cancelled || generation !== generationRef.current) {
          return;
        }

        setChunks([chunk]);
        setLoadingMore(false);
        initializedRef.current = true;
        anchorOffsetRef.current = chunk.startOffset;
        setPercent(total > 0 ? Math.min(1, chunk.startOffset / total) : 0);
        setReaderError(null);
        setInitialLoading(false);
        scrollRef.current?.scrollTo({ top: 0 });

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
          setReaderError(errorText(error, "无法打开文本"));
          setInitialLoading(false);
        }
      }
    }

    void initialize();

    return () => {
      cancelled = true;
      generationRef.current += 1;

      if (saveTimeoutRef.current !== null) {
        window.clearTimeout(saveTimeoutRef.current);
        saveTimeoutRef.current = null;
      }

      if (initializedRef.current) {
        persistProgress();
      }
      initializedRef.current = false;

      const sessionId = sessionIdRef.current;
      sessionIdRef.current = null;
      if (sessionId !== null) {
        const location = textLocationJson(anchorOffsetRef.current, encodingRef.current);
        void readerApi.endReadingSession(sessionId, location).catch(() => undefined);
      }
    };
  }, [
    bookFormat,
    bookId,
    infoEncoding,
    infoTotalBytes,
    loadedBookId,
    persistProgress,
    progressFetched,
    retryKey,
    settingsFetched,
    validId,
  ]);

  useEffect(() => {
    const sentinel = sentinelRef.current;
    const root = scrollRef.current;
    if (!sentinel || !root || !info) {
      return;
    }

    const observer = new IntersectionObserver(
      (entries) => {
        if (entries.some((entry) => entry.isIntersecting)) {
          void loadMore();
        }
      },
      { root, rootMargin: `${OVERSCAN_BYTES}px 0px` },
    );

    observer.observe(sentinel);
    return () => observer.disconnect();
  }, [info, loadMore]);

  useEffect(() => {
    if (initialLoading) {
      return;
    }

    const frame = window.requestAnimationFrame(() => {
      const root = scrollRef.current;
      const last = chunks[chunks.length - 1];
      if (!root || !last?.eof || totalBytesRef.current <= 0) {
        return;
      }

      if (root.scrollHeight <= root.clientHeight + 8) {
        anchorOffsetRef.current = totalBytesRef.current;
        setPercent(1);
        scheduleProgressSave();
      }
    });

    return () => window.cancelAnimationFrame(frame);
  }, [chunks, initialLoading, scheduleProgressSave]);

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

      const root = scrollRef.current;
      if (!root) {
        return;
      }

      if (event.key === "PageDown" || event.key === " ") {
        event.preventDefault();
        root.scrollBy({ top: root.clientHeight * 0.85, behavior: "smooth" });
      } else if (event.key === "PageUp") {
        event.preventDefault();
        root.scrollBy({ top: -root.clientHeight * 0.85, behavior: "smooth" });
      } else if (event.key === "Home") {
        event.preventDefault();
        void jumpToOffset(0);
      } else if (event.key === "End") {
        event.preventDefault();
        if (info) {
          void jumpToOffset(Math.max(0, info.totalBytes - CHUNK_BYTES));
        }
      }
    }

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [info, jumpToOffset]);

  useEffect(
    () => () => {
      if (settingsSaveTimeoutRef.current !== null) {
        window.clearTimeout(settingsSaveTimeoutRef.current);
      }

      if (scrollTimerRef.current !== null) {
        window.clearTimeout(scrollTimerRef.current);
      }
    },
    [],
  );

  function retry() {
    setReaderError(null);
    setInitialLoading(true);
    setChunks([]);
    setRetryKey((value) => value + 1);
  }

  function updatePreferences(next: ReaderPreferences) {
    setLocalPreferences(next);

    if (settingsSaveTimeoutRef.current !== null) {
      window.clearTimeout(settingsSaveTimeoutRef.current);
    }

    settingsSaveTimeoutRef.current = window.setTimeout(() => {
      void readerApi
        .saveReaderSettings("txt", JSON.stringify(next))
        .catch((error: unknown) => toast.error(errorText(error, "保存阅读设置失败")));
    }, 400);
  }

  function openBookmarkDialog() {
    setBookmarkLabel(`阅读位置 ${(percent * 100).toFixed(1)}%`);
    setBookmarkDialogOpen(true);
  }

  function saveBookmark() {
    createBookmark.mutate(
      {
        locationType: "txt_offset",
        locationJson: textLocationJson(anchorOffsetRef.current, encodingRef.current),
        label: bookmarkLabel.trim() || `阅读位置 ${(percent * 100).toFixed(1)}%`,
      },
      { onSuccess: () => setBookmarkDialogOpen(false) },
    );
  }

  function jumpToBookmark(bookmark: Bookmark) {
    const offset = parseTextOffset(bookmark.locationJson);
    if (offset === null) {
      toast.error("书签位置无效");
      return;
    }

    void jumpToOffset(offset);
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

  if (bookQuery.isError || !book) {
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

  if (book.format !== "txt") {
    return (
      <div className="flex h-[calc(100vh-3.5rem)] items-center justify-center px-6">
        <div className="w-full max-w-lg">
          <ErrorState title="格式不匹配" description="该书籍不是 TXT 文本。" />
        </div>
      </div>
    );
  }

  if (!book.fileExists) {
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

  if (infoQuery.isPending) {
    return (
      <div className="flex h-[calc(100vh-3.5rem)] items-center justify-center">
        <LoadingState label="正在检测文本编码…" />
      </div>
    );
  }

  if (infoQuery.isError || !info) {
    return (
      <div className="flex h-[calc(100vh-3.5rem)] items-center justify-center px-6">
        <div className="w-full max-w-lg">
          <ErrorState
            title="无法打开文本"
            description={infoQuery.error?.message ?? "无法读取文本文件信息。"}
            onRetry={() => void infoQuery.refetch()}
          />
        </div>
      </div>
    );
  }

  if (info.totalBytes === 0) {
    return (
      <div className="flex h-[calc(100vh-3.5rem)] items-center justify-center px-6">
        <div className="w-full max-w-lg">
          <EmptyState
            icon={BookOpen}
            title="文本文件为空"
            description="这个 TXT 文件没有可阅读的内容，请检查原始文件。"
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
              {info.encoding.toUpperCase()}
              {info.hasBom ? " · 含 BOM" : ""} · 共 {formatBytes(info.totalBytes)}
            </p>
          </div>
        </div>

        <div className="flex items-center gap-0.5">
          <Button
            variant="ghost"
            size="icon-sm"
            title="跳到开头"
            aria-label="跳到开头"
            disabled={initialLoading || Boolean(readerError)}
            onClick={() => void jumpToOffset(0)}
          >
            <ArrowUpToLine className="size-4" aria-hidden="true" />
          </Button>
          <Button
            variant="ghost"
            size="icon-sm"
            title="跳到结尾"
            aria-label="跳到结尾"
            disabled={initialLoading || Boolean(readerError)}
            onClick={() => void jumpToOffset(Math.max(0, info.totalBytes - CHUNK_BYTES))}
          >
            <ArrowDownToLine className="size-4" aria-hidden="true" />
          </Button>
          <Button
            variant="ghost"
            size="icon-sm"
            title="添加书签"
            aria-label="添加书签"
            disabled={initialLoading || Boolean(readerError)}
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
          <Button
            variant="ghost"
            size="icon-sm"
            title="阅读设置"
            aria-label="阅读设置"
            onClick={() => setSettingsOpen(true)}
          >
            <Settings className="size-4" aria-hidden="true" />
          </Button>
        </div>
      </header>

      {decodeWarning ? (
        <div className="shrink-0 border-b border-amber-500/30 bg-amber-500/10 px-4 py-2 text-xs text-amber-600 dark:text-amber-300">
          {decodeWarning}
        </div>
      ) : null}

      <div className="flex min-h-0 flex-1">
        <div
          ref={scrollRef}
          className="relative min-w-0 flex-1 overflow-y-auto"
          style={{ backgroundColor: colors.background }}
          onScroll={handleScroll}
        >
          <article
            className="mx-auto max-w-3xl px-8 py-8 break-words whitespace-pre-wrap"
            style={{
              color: colors.text,
              fontSize: `${preferences.fontSize}px`,
              lineHeight: preferences.lineHeight,
            }}
          >
            {chunks.map((chunk) => (
              <section
                key={chunk.startOffset}
                data-offset={chunk.startOffset}
                data-end={chunk.nextOffset}
                className="min-h-px"
              >
                {chunk.text}
              </section>
            ))}

            {loadingMore ? (
              <p className="py-4 text-center text-xs opacity-60">正在加载后续内容…</p>
            ) : null}
            <div ref={sentinelRef} aria-hidden="true" className="h-px" />
          </article>

          {initialLoading ? (
            <div className="absolute inset-0 flex items-center justify-center bg-background/95">
              <LoadingState label="正在打开文本…" />
            </div>
          ) : null}

          {readerError ? (
            <div className="absolute inset-0 flex items-center justify-center bg-background px-6">
              <div className="w-full max-w-lg">
                <ErrorState title="无法打开文本" description={readerError} onRetry={retry} />
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
        <span className="truncate">
          {info.encoding.toUpperCase()} · {formatBytes(totalBytes)}
        </span>
        <span className="shrink-0">
          {chunks.some((chunk) => chunk.eof) ? "" : "后续内容滚动加载 · "}
          全书 {(percent * 100).toFixed(1)}%
        </span>
      </footer>

      {settingsOpen ? (
        <ReaderSettingsDialog
          open
          onOpenChange={setSettingsOpen}
          preferences={preferences}
          onChange={updatePreferences}
        />
      ) : null}

      <Dialog open={bookmarkDialogOpen} onOpenChange={setBookmarkDialogOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>添加书签</DialogTitle>
            <DialogDescription>保存当前阅读位置，便于之后快速返回。</DialogDescription>
          </DialogHeader>

          <div className="space-y-1">
            <Label htmlFor="txt-bookmark-label">书签名称</Label>
            <Input
              id="txt-bookmark-label"
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
