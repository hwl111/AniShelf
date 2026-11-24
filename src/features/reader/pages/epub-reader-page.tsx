import type { Book, Location, Rendition } from "epubjs";
import {
  ArrowLeft,
  Bookmark as BookmarkIcon,
  BookOpen,
  ChevronLeft,
  ChevronRight,
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
import { TocPanel } from "../components/toc-panel";
import {
  applyReaderPreferences,
  findTocLabel,
  flattenToc,
  openEpub,
  type TocEntry,
} from "../epub/adapter";
import { parseCfi, parseReaderPreferences } from "../preferences";
import {
  useBookmarks,
  useCreateBookmark,
  useDeleteBookmark,
  useReadingProgress,
  useReaderSettings,
} from "../queries";
import type { Bookmark, ReaderPreferences } from "../types";
import { createWheelPager } from "../wheel-paging";

function errorText(error: unknown, fallback: string): string {
  return error instanceof Error ? error.message : fallback;
}

export function EpubReaderPage() {
  const params = useParams();
  const bookId = Number(params.bookId);
  const validId = Number.isInteger(bookId) && bookId > 0;

  const bookQuery = useBook(validId ? bookId : undefined);
  const progressQuery = useReadingProgress(bookId);
  const settingsQuery = useReaderSettings("epub");
  const bookmarksQuery = useBookmarks(bookId);
  const createBookmark = useCreateBookmark(bookId);
  const deleteBookmark = useDeleteBookmark(bookId);

  const viewerRef = useRef<HTMLDivElement>(null);
  const bookRef = useRef<Book | null>(null);
  const renditionRef = useRef<Rendition | null>(null);
  const sessionIdRef = useRef<number | null>(null);
  const saveTimeoutRef = useRef<number | null>(null);
  const latestLocationRef = useRef<{ cfi: string; href: string; percent: number } | null>(null);
  const locationsReadyRef = useRef(false);

  const [localPreferences, setLocalPreferences] = useState<ReaderPreferences | null>(null);
  const parsedPreferences = useMemo(
    () => parseReaderPreferences(settingsQuery.data?.preferencesJson),
    [settingsQuery.data?.preferencesJson],
  );
  const preferences = localPreferences ?? parsedPreferences;
  const preferencesRef = useRef(preferences);
  const settingsSaveTimeoutRef = useRef<number | null>(null);

  const [isLoading, setIsLoading] = useState(true);
  const [readerError, setReaderError] = useState<string | null>(null);
  const [retryKey, setRetryKey] = useState(0);
  const [toc, setToc] = useState<TocEntry[]>([]);
  const [tocOpen, setTocOpen] = useState(false);
  const [bookmarksOpen, setBookmarksOpen] = useState(false);
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [bookmarkDialogOpen, setBookmarkDialogOpen] = useState(false);
  const [bookmarkLabel, setBookmarkLabel] = useState("");
  const [currentCfi, setCurrentCfi] = useState<string | null>(null);
  const [currentLabel, setCurrentLabel] = useState("");
  const [percent, setPercent] = useState(0);
  const [chapterPage, setChapterPage] = useState<{ page: number; total: number } | null>(null);

  const updateChapterPage = useCallback(
    (displayed: { page?: number; total?: number } | null | undefined) => {
      const page = Number(displayed?.page);
      const total = Number(displayed?.total);
      if (!Number.isFinite(page) || !Number.isFinite(total) || page <= 0 || total <= 0) {
        setChapterPage(null);
        return;
      }

      setChapterPage({ page: Math.min(page, total), total });
    },
    [],
  );

  const loadedBookId = bookQuery.data?.id ?? null;
  const savedLocationJson = progressQuery.data?.locationJson ?? null;
  const progressFetched = progressQuery.isFetched;
  const settingsFetched = settingsQuery.isFetched;

  useEffect(() => {
    preferencesRef.current = preferences;
    const rendition = renditionRef.current;
    if (rendition) {
      applyReaderPreferences(rendition, preferences);
    }
  }, [preferences]);

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
        void renditionRef.current?.prev();
      } else if (event.key === "ArrowRight" || event.key === "PageDown" || event.key === " ") {
        event.preventDefault();
        void renditionRef.current?.next();
      }
    }

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, []);

  useEffect(() => {
    const element = viewerRef.current;
    if (!element) {
      return;
    }

    const observer = new ResizeObserver(() => {
      const rect = element.getBoundingClientRect();
      if (rect.width > 0 && rect.height > 0) {
        renditionRef.current?.resize(rect.width, rect.height);
      }
    });

    observer.observe(element);
    return () => observer.disconnect();
  }, []);

  useEffect(
    () => () => {
      if (settingsSaveTimeoutRef.current !== null) {
        window.clearTimeout(settingsSaveTimeoutRef.current);
      }
    },
    [],
  );

  useEffect(() => {
    if (!validId || !loadedBookId || !viewerRef.current || !progressFetched || !settingsFetched) {
      return;
    }

    let cancelled = false;
    let wheelHandler: ((event: WheelEvent) => void) | null = null;
    const viewer = viewerRef.current;

    async function initialize() {
      try {
        const data = await readerApi.readBookBytes(bookId);
        if (cancelled) {
          return;
        }

        const opened = await openEpub(viewer, data);
        if (cancelled) {
          opened.rendition.destroy();
          opened.book.destroy();
          return;
        }

        bookRef.current = opened.book;
        renditionRef.current = opened.rendition;
        locationsReadyRef.current = false;
        applyReaderPreferences(opened.rendition, preferencesRef.current);

        const entries = flattenToc(opened.book.navigation.toc ?? []);
        setToc(entries);

        function scheduleSave() {
          if (saveTimeoutRef.current !== null) {
            window.clearTimeout(saveTimeoutRef.current);
          }

          saveTimeoutRef.current = window.setTimeout(() => {
            const latest = latestLocationRef.current;
            if (!latest || cancelled) {
              return;
            }

            void readerApi
              .saveReadingProgress(bookId, {
                locationType: "epub_cfi",
                locationJson: JSON.stringify({
                  cfi: latest.cfi,
                  href: latest.href,
                  label: findTocLabel(entries, latest.href) ?? "",
                }),
                percent: latest.percent,
              })
              .catch((error: unknown) => {
                toast.error(errorText(error, "保存阅读进度失败"));
              });
          }, 800);
        }

        function handleRelocated(location: Location) {
          if (cancelled) {
            return;
          }

          const cfi = location.start.cfi;
          const href = location.start.href;
          const displayed = location.start.displayed;
          const displayedPage =
            displayed && Number.isFinite(displayed.page) && displayed.page > 0 ? displayed.page : 1;
          const displayedTotal =
            displayed && Number.isFinite(displayed.total) && displayed.total > 0
              ? displayed.total
              : 1;
          const pageFraction = Math.min(1, Math.max(0, displayedPage / displayedTotal));
          const spineLength =
            (opened.book.spine as unknown as { spineItems?: unknown[] }).spineItems?.length ?? 0;
          const spineIndex = Number.isFinite(location.start.index) ? location.start.index : 0;
          const spinePercent =
            spineLength > 0 ? Math.min(1, (spineIndex + pageFraction) / spineLength) : pageFraction;

          let nextPercent = Math.max(
            spinePercent,
            typeof location.start.percentage === "number" &&
              Number.isFinite(location.start.percentage)
              ? location.start.percentage
              : 0,
          );

          if (locationsReadyRef.current) {
            try {
              const value = opened.book.locations.percentageFromCfi(cfi);
              if (typeof value === "number" && Number.isFinite(value)) {
                nextPercent = Math.max(nextPercent, value);
              }
            } catch {
              // Keep the spine-based estimate.
            }
          }

          nextPercent = Math.min(1, Math.max(0, nextPercent));

          latestLocationRef.current = { cfi, href, percent: nextPercent };
          setCurrentCfi(cfi);
          setCurrentLabel(findTocLabel(entries, href) ?? "");
          setPercent(nextPercent);
          updateChapterPage(location.start.displayed);
          scheduleSave();
        }

        function handleLinkClicked(href: string) {
          if (/^https?:\/\//i.test(href)) {
            void readerApi
              .openExternalUrl(href)
              .catch((error: unknown) => toast.error(errorText(error, "无法打开外部链接")));
          } else {
            void opened.rendition.display(href).catch((error: unknown) => {
              toast.error(errorText(error, "无法打开章节链接"));
            });
          }
        }

        opened.rendition.on("relocated", handleRelocated);
        opened.rendition.on("linkClicked", handleLinkClicked);

        const wheelPager = createWheelPager({
          onNext: () => {
            void opened.rendition.next();
          },
          onPrevious: () => {
            void opened.rendition.prev();
          },
        });
        wheelHandler = (event: WheelEvent) => {
          if (document.querySelector('[role="dialog"]')) {
            return;
          }
          if (wheelPager(event)) {
            event.preventDefault();
          }
        };
        viewer.addEventListener("wheel", wheelHandler, { passive: false });
        opened.rendition.hooks.content.register((contents: { document: Document }) => {
          contents.document.addEventListener("wheel", (event) => wheelHandler?.(event), {
            passive: false,
          });
        });

        const savedCfi = parseCfi(savedLocationJson);
        try {
          await opened.rendition.display(savedCfi ?? undefined);
        } catch {
          await opened.rendition.display();
        }

        void opened.book.locations
          .generate(1600)
          .then(() => {
            if (cancelled) {
              return;
            }

            locationsReadyRef.current = true;
            const latest = latestLocationRef.current;
            if (!latest) {
              return;
            }

            try {
              const value = opened.book.locations.percentageFromCfi(latest.cfi);
              if (typeof value === "number" && Number.isFinite(value)) {
                const nextPercent = Math.min(1, Math.max(latest.percent, value));
                latestLocationRef.current = { ...latest, percent: nextPercent };
                setPercent(nextPercent);
                scheduleSave();
              }
            } catch {
              // Keep the estimated percentage if the CFI cannot be mapped.
            }
          })
          .catch((error: unknown) => {
            console.error("[AniShelf] 生成 EPUB 位置索引失败", error);
          });

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

        setIsLoading(false);
      } catch (error) {
        if (!cancelled) {
          setReaderError(errorText(error, "无法打开 EPUB"));
          setIsLoading(false);
        }
      }
    }

    void initialize();

    return () => {
      cancelled = true;

      if (wheelHandler) {
        viewer.removeEventListener("wheel", wheelHandler);
        wheelHandler = null;
      }

      if (saveTimeoutRef.current !== null) {
        window.clearTimeout(saveTimeoutRef.current);
        saveTimeoutRef.current = null;
      }

      const latest = latestLocationRef.current;
      if (latest) {
        void readerApi
          .saveReadingProgress(bookId, {
            locationType: "epub_cfi",
            locationJson: JSON.stringify({ cfi: latest.cfi, href: latest.href }),
            percent: latest.percent,
          })
          .catch(() => undefined);
      }

      const sessionId = sessionIdRef.current;
      sessionIdRef.current = null;
      if (sessionId !== null) {
        void readerApi
          .endReadingSession(
            sessionId,
            latest ? JSON.stringify({ cfi: latest.cfi, href: latest.href }) : null,
          )
          .catch(() => undefined);
      }

      renditionRef.current?.destroy();
      renditionRef.current = null;
      bookRef.current?.destroy();
      bookRef.current = null;
      latestLocationRef.current = null;
      locationsReadyRef.current = false;
    };
  }, [
    bookId,
    loadedBookId,
    progressFetched,
    savedLocationJson,
    settingsFetched,
    retryKey,
    validId,
    updateChapterPage,
  ]);

  function retry() {
    setReaderError(null);
    setIsLoading(true);
    setRetryKey((value) => value + 1);
  }

  function updatePreferences(next: ReaderPreferences) {
    setLocalPreferences(next);

    if (settingsSaveTimeoutRef.current !== null) {
      window.clearTimeout(settingsSaveTimeoutRef.current);
    }

    settingsSaveTimeoutRef.current = window.setTimeout(() => {
      void readerApi
        .saveReaderSettings("epub", JSON.stringify(next))
        .catch((error: unknown) => toast.error(errorText(error, "保存阅读设置失败")));
    }, 400);
  }

  function refreshChapterPage(rendition: Rendition) {
    window.setTimeout(() => {
      void Promise.resolve(rendition.currentLocation())
        .then((location) => {
          const displayed = (
            location as {
              start?: { displayed?: { page?: number; total?: number } };
            }
          ).start?.displayed;
          updateChapterPage(displayed);
        })
        .catch(() => undefined);
    }, 120);
  }

  function displayReaderTarget(target: string, errorMessage: string) {
    const rendition = renditionRef.current;
    if (!rendition) {
      return;
    }

    void rendition
      .display(target)
      .then(() => refreshChapterPage(rendition))
      .catch((error: unknown) => {
        toast.error(errorText(error, errorMessage));
      });
  }

  function openBookmarkDialog() {
    setBookmarkLabel(currentLabel);
    setBookmarkDialogOpen(true);
  }

  function saveBookmark() {
    const latest = latestLocationRef.current;
    if (!latest) {
      return;
    }

    createBookmark.mutate(
      {
        locationType: "epub_cfi",
        locationJson: JSON.stringify({
          cfi: latest.cfi,
          href: latest.href,
          label: findTocLabel(toc, latest.href) ?? "",
        }),
        label: bookmarkLabel.trim() || currentLabel || "未命名书签",
      },
      { onSuccess: () => setBookmarkDialogOpen(false) },
    );
  }

  function jumpToBookmark(bookmark: Bookmark) {
    const cfi = parseCfi(bookmark.locationJson);
    if (!cfi) {
      toast.error("书签位置无效");
      return;
    }

    displayReaderTarget(cfi, "无法跳转到书签");
  }

  function resolveTocHref(href: string): string {
    if (!href || href.startsWith("#") || href.startsWith("/")) {
      return href;
    }
    if (/^[a-z][a-z0-9+.-]*:/i.test(href)) {
      return href;
    }

    const book = bookRef.current;
    const packaging = (
      book as unknown as {
        packaging?: { navPath?: string | false; ncxPath?: string | false };
      } | null
    )?.packaging;
    const navigationPath = packaging?.navPath || packaging?.ncxPath;
    if (!navigationPath || !navigationPath.includes("/")) {
      return href;
    }

    const baseDir = navigationPath.slice(0, navigationPath.lastIndexOf("/"));
    const [pathPart, fragment] = href.split("#");
    const parts: string[] = [];

    for (const part of [...baseDir.split("/"), ...pathPart.split("/")]) {
      if (!part || part === ".") {
        continue;
      }
      if (part === "..") {
        parts.pop();
        continue;
      }
      parts.push(part);
    }

    const resolved = parts.join("/");
    return fragment ? `${resolved}#${fragment}` : resolved;
  }

  function jumpToToc(href: string) {
    setTocOpen(false);
    const rendition = renditionRef.current;
    if (!rendition) {
      return;
    }

    // Navigation hrefs are relative to the nav/NCX document, while the
    // spine uses paths relative to the OPF. Resolve the former first.
    const resolved = resolveTocHref(href);
    const spine = bookRef.current?.spine;
    const target = spine?.get(resolved) ? resolved : spine?.get(href) ? href : resolved;

    // Closing the TOC changes the viewer width; epub.js resizes and
    // re-displays the previous location. Delay the jump so it is queued
    // after that resize, otherwise the jump is immediately overridden.
    window.setTimeout(() => {
      void rendition
        .display(target)
        .then(() => refreshChapterPage(rendition))
        .catch((error: unknown) => {
          toast.error(errorText(error, "无法打开章节"));
        });
    }, 80);
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
          <Button
            variant={tocOpen ? "secondary" : "ghost"}
            size="icon-sm"
            title="目录"
            aria-label="目录"
            onClick={() => {
              setTocOpen((value) => !value);
              setBookmarksOpen(false);
            }}
          >
            <List className="size-4" aria-hidden="true" />
          </Button>
          <div className="min-w-0 px-1">
            <p className="truncate text-sm font-medium">{book.title}</p>
            <p className="truncate text-[11px] text-muted-foreground">
              {currentLabel || "正在定位…"}
            </p>
          </div>
        </div>

        <div className="flex items-center gap-0.5">
          <Button
            variant="ghost"
            size="icon-sm"
            title="上一页"
            aria-label="上一页"
            disabled={isLoading || Boolean(readerError)}
            onClick={() => void renditionRef.current?.prev()}
          >
            <ChevronLeft className="size-4" aria-hidden="true" />
          </Button>
          <Button
            variant="ghost"
            size="icon-sm"
            title="下一页"
            aria-label="下一页"
            disabled={isLoading || Boolean(readerError)}
            onClick={() => void renditionRef.current?.next()}
          >
            <ChevronRight className="size-4" aria-hidden="true" />
          </Button>
          <Button
            variant="ghost"
            size="icon-sm"
            title="添加书签"
            aria-label="添加书签"
            disabled={!currentCfi || isLoading}
            onClick={openBookmarkDialog}
          >
            <BookmarkIcon className="size-4" aria-hidden="true" />
          </Button>
          <Button
            variant={bookmarksOpen ? "secondary" : "ghost"}
            size="icon-sm"
            title="书签列表"
            aria-label="书签列表"
            onClick={() => {
              setBookmarksOpen((value) => !value);
              setTocOpen(false);
            }}
          >
            <BookmarkIcon className="size-4" aria-hidden="true" />
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

      <div className="flex min-h-0 flex-1">
        {tocOpen ? <TocPanel entries={toc} onSelect={jumpToToc} /> : null}

        <div className="relative min-w-0 flex-1">
          <div ref={viewerRef} className="h-full w-full" />

          {isLoading ? (
            <div className="absolute inset-0 flex items-center justify-center bg-background/95">
              <LoadingState label="正在打开 EPUB…" />
            </div>
          ) : null}

          {readerError ? (
            <div className="absolute inset-0 flex items-center justify-center bg-background px-6">
              <div className="w-full max-w-lg">
                <ErrorState title="无法打开 EPUB" description={readerError} onRetry={retry} />
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
        <span className="truncate">{currentLabel || "未定位章节"}</span>
        <span className="shrink-0">
          {chapterPage && chapterPage.total > 0
            ? `本章 ${chapterPage.page}/${chapterPage.total} 页 · `
            : ""}
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
            <DialogDescription>保存当前位置，便于之后快速返回。</DialogDescription>
          </DialogHeader>

          <div className="space-y-1">
            <Label htmlFor="bookmark-label">书签名称</Label>
            <Input
              id="bookmark-label"
              maxLength={200}
              placeholder={currentLabel || "当前位置"}
              value={bookmarkLabel}
              onChange={(event) => setBookmarkLabel(event.target.value)}
            />
          </div>

          <DialogFooter>
            <Button variant="outline" onClick={() => setBookmarkDialogOpen(false)}>
              取消
            </Button>
            <Button disabled={createBookmark.isPending || !currentCfi} onClick={saveBookmark}>
              {createBookmark.isPending ? "保存中…" : "保存书签"}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
