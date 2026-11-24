import type { Season } from "@/features/anime/types";

export interface BangumiSearchResult {
  subjectId: number;
  title: string;
  originalTitle: string | null;
  summary: string;
  year: number | null;
  season: Season | null;
  totalEpisodes: number | null;
  coverUrl: string | null;
  airDate: string | null;
}

export interface BangumiFieldChange {
  field: string;
  label: string;
  oldValue: string;
  newValue: string;
}

export interface BangumiRefreshPreview {
  subjectId: number;
  title: string;
  changes: BangumiFieldChange[];
}

export interface BangumiTokenStatus {
  hasToken: boolean;
}

export interface BangumiConnectionStatus {
  ok: boolean;
  message: string;
}
