import { Outlet, useLocation } from "react-router";

import { findNavigationItem } from "@/app/navigation";
import { useDocumentTitle } from "@/hooks/use-document-title";

import { Sidebar } from "./sidebar";
import { TopBar } from "./top-bar";

export function AppShell() {
  const { pathname } = useLocation();
  const title = findNavigationItem(pathname)?.label ?? "页面未找到";
  useDocumentTitle(`${title} · AniShelf`);

  return (
    <div className="flex h-screen overflow-hidden bg-background text-foreground">
      <Sidebar />
      <div className="flex min-w-0 flex-1 flex-col">
        <TopBar />
        <main className="min-h-0 flex-1 overflow-y-auto">
          <Outlet />
        </main>
      </div>
    </div>
  );
}
