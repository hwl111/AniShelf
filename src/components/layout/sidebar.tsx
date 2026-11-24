import { PanelLeftClose, PanelLeftOpen } from "lucide-react";
import { NavLink } from "react-router";

import { navigationItems } from "@/app/navigation";
import { Button } from "@/components/ui/button";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import { cn } from "@/lib/utils";
import { useUiStore } from "@/stores/ui-store";

export function Sidebar() {
  const collapsed = useUiStore((state) => state.sidebarCollapsed);
  const toggleSidebar = useUiStore((state) => state.toggleSidebar);

  return (
    <aside
      aria-label="主导航"
      className={cn(
        "flex h-full shrink-0 flex-col border-r border-sidebar-border bg-sidebar text-sidebar-foreground transition-[width] duration-200",
        collapsed ? "w-16" : "w-60",
      )}
    >
      <div
        className={cn(
          "flex h-14 items-center gap-2 border-b border-sidebar-border px-3",
          collapsed && "justify-center px-0",
        )}
      >
        <div className="flex size-8 shrink-0 items-center justify-center rounded-lg bg-sidebar-primary text-sm font-semibold text-sidebar-primary-foreground">
          A
        </div>
        {collapsed ? null : (
          <span className="truncate text-sm font-semibold tracking-wide">AniShelf</span>
        )}
      </div>

      <nav className="flex-1 space-y-1 overflow-y-auto p-2">
        {navigationItems.map((item) => {
          const link = (
            <NavLink
              key={item.path}
              to={item.path}
              end={item.path === "/"}
              className={({ isActive }) =>
                cn(
                  "flex items-center gap-3 rounded-md px-3 py-2 text-sm transition-colors",
                  collapsed && "justify-center px-0",
                  isActive
                    ? "bg-sidebar-accent font-medium text-sidebar-accent-foreground"
                    : "text-sidebar-foreground/80 hover:bg-sidebar-accent/60 hover:text-sidebar-accent-foreground",
                )
              }
            >
              <item.icon className="size-4 shrink-0" aria-hidden="true" />
              {collapsed ? (
                <span className="sr-only">{item.label}</span>
              ) : (
                <span className="truncate">{item.label}</span>
              )}
            </NavLink>
          );

          if (!collapsed) {
            return link;
          }

          return (
            <Tooltip key={item.path}>
              <TooltipTrigger asChild>{link}</TooltipTrigger>
              <TooltipContent side="right">{item.label}</TooltipContent>
            </Tooltip>
          );
        })}
      </nav>

      <div className="border-t border-sidebar-border p-2">
        <Button
          variant="ghost"
          size="sm"
          aria-label={collapsed ? "展开侧边栏" : "收起侧边栏"}
          className={cn(
            "w-full justify-start gap-2 text-muted-foreground",
            collapsed && "justify-center px-0",
          )}
          onClick={toggleSidebar}
        >
          {collapsed ? (
            <PanelLeftOpen className="size-4" aria-hidden="true" />
          ) : (
            <PanelLeftClose className="size-4" aria-hidden="true" />
          )}
          {collapsed ? null : <span>收起侧边栏</span>}
        </Button>
      </div>
    </aside>
  );
}
