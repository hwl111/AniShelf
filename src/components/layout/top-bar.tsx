import { ChevronRight } from "lucide-react";
import { useLocation } from "react-router";

import { findNavigationItem } from "@/app/navigation";

export function TopBar() {
  const { pathname } = useLocation();
  const current = findNavigationItem(pathname);
  const title = current?.label ?? "页面未找到";

  return (
    <header className="flex h-14 shrink-0 items-center justify-between border-b border-border bg-background/95 px-6">
      <nav aria-label="面包屑" className="flex items-center gap-2 text-sm">
        <span className="text-muted-foreground">AniShelf</span>
        <ChevronRight className="size-3.5 text-muted-foreground" aria-hidden="true" />
        <span className="font-medium">{title}</span>
      </nav>
      <span className="hidden text-xs text-muted-foreground sm:inline">本地优先 · 离线可用</span>
    </header>
  );
}
