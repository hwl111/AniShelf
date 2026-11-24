import type { AnimeSort, Season, WatchStatus } from "./types";

export const watchStatusLabels: Record<WatchStatus, string> = {
  wish: "想看",
  watching: "在看",
  completed: "已看",
  on_hold: "搁置",
  dropped: "弃番",
};

export const watchStatusOptions = (Object.keys(watchStatusLabels) as WatchStatus[]).map(
  (value) => ({ value, label: watchStatusLabels[value] }),
);

export const watchStatusBadgeClasses: Record<WatchStatus, string> = {
  wish: "border-border bg-muted text-muted-foreground",
  watching: "border-sky-500/30 bg-sky-500/15 text-sky-300",
  completed: "border-emerald-500/30 bg-emerald-500/15 text-emerald-300",
  on_hold: "border-amber-500/30 bg-amber-500/15 text-amber-300",
  dropped: "border-destructive/30 bg-destructive/15 text-destructive",
};

export const seasonLabels: Record<Season, string> = {
  winter: "冬",
  spring: "春",
  summer: "夏",
  fall: "秋",
};

export const seasonOptions = (Object.keys(seasonLabels) as Season[]).map((value) => ({
  value,
  label: seasonLabels[value],
}));

export const animeSortLabels: Record<AnimeSort, string> = {
  updated_desc: "最近更新",
  created_desc: "最近添加",
  title_asc: "标题 A-Z",
  rating_desc: "评分最高",
  year_desc: "年份最新",
};

export const animeSortOptions = (Object.keys(animeSortLabels) as AnimeSort[]).map((value) => ({
  value,
  label: animeSortLabels[value],
}));
