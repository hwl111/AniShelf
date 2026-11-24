import {
  BookOpen,
  LayoutDashboard,
  Library,
  Search,
  Settings,
  StickyNote,
  Tv,
  type LucideIcon,
} from "lucide-react";

export interface NavigationItem {
  path: string;
  label: string;
  description: string;
  icon: LucideIcon;
}

export const navigationItems: NavigationItem[] = [
  { path: "/", label: "首页", description: "追番与阅读概览", icon: LayoutDashboard },
  { path: "/anime", label: "番剧", description: "管理追番进度、评分与观看记录", icon: Tv },
  { path: "/bookshelf", label: "书架", description: "管理本地电子书与阅读进度", icon: Library },
  { path: "/reader", label: "阅读器", description: "继续阅读或选择一本书开始", icon: BookOpen },
  { path: "/notes", label: "笔记", description: "记录想法并关联作品", icon: StickyNote },
  { path: "/search", label: "搜索", description: "统一检索番剧、书籍与笔记", icon: Search },
  { path: "/settings", label: "设置", description: "主题、数据与备份设置", icon: Settings },
];

export function findNavigationItem(pathname: string): NavigationItem | undefined {
  return navigationItems.find((item) => item.path === pathname);
}
