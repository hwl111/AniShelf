import { Trash2 } from "lucide-react";

import { Button } from "@/components/ui/button";

import type { Bookmark } from "../types";

interface BookmarksPanelProps {
  bookmarks: Bookmark[];
  isLoading: boolean;
  isError: boolean;
  onSelect: (bookmark: Bookmark) => void;
  onDelete: (bookmark: Bookmark) => void;
}

function formatDate(value: number): string {
  return new Date(value).toLocaleString("zh-CN", { hour12: false });
}

export function BookmarksPanel({
  bookmarks,
  isLoading,
  isError,
  onSelect,
  onDelete,
}: BookmarksPanelProps) {
  return (
    <aside className="w-64 shrink-0 overflow-y-auto border-l border-border bg-card/20 p-3">
      <h2 className="mb-2 text-xs font-medium text-muted-foreground">书签</h2>

      {isLoading ? <p className="text-xs text-muted-foreground">正在读取书签…</p> : null}
      {isError ? <p className="text-xs text-destructive">无法读取书签。</p> : null}
      {!isLoading && !isError && bookmarks.length === 0 ? (
        <p className="text-xs text-muted-foreground">还没有书签。使用工具栏的书签按钮添加。</p>
      ) : null}

      <div className="space-y-1">
        {bookmarks.map((bookmark) => (
          <div
            key={bookmark.id}
            className="flex items-start gap-1 rounded-md px-2 py-1.5 transition-colors hover:bg-muted"
          >
            <button
              type="button"
              className="min-w-0 flex-1 text-left"
              onClick={() => onSelect(bookmark)}
            >
              <p className="truncate text-sm">{bookmark.label || "未命名书签"}</p>
              <p className="text-[11px] text-muted-foreground">{formatDate(bookmark.createdAt)}</p>
            </button>
            <Button
              variant="ghost"
              size="icon-sm"
              aria-label="删除书签"
              title="删除书签"
              onClick={() => onDelete(bookmark)}
            >
              <Trash2 className="size-3.5" aria-hidden="true" />
            </Button>
          </div>
        ))}
      </div>
    </aside>
  );
}
