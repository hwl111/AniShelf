import type { Anime } from "@/features/anime/types";
import { invokeCommand } from "@/lib/ipc";

import type {
  BangumiConnectionStatus,
  BangumiRefreshPreview,
  BangumiSearchResult,
  BangumiTokenStatus,
} from "./types";

export function searchBangumi(keyword: string): Promise<BangumiSearchResult[]> {
  return invokeCommand<BangumiSearchResult[]>("search_bangumi", { keyword });
}

export function importBangumiSubject(subjectId: number): Promise<Anime> {
  return invokeCommand<Anime>("import_bangumi_subject", { subjectId });
}

export function previewBangumiRefresh(animeId: number): Promise<BangumiRefreshPreview> {
  return invokeCommand<BangumiRefreshPreview>("preview_bangumi_refresh", { animeId });
}

export function applyBangumiRefresh(animeId: number): Promise<Anime> {
  return invokeCommand<Anime>("apply_bangumi_refresh", { animeId });
}

export function getBangumiTokenStatus(): Promise<BangumiTokenStatus> {
  return invokeCommand<BangumiTokenStatus>("get_bangumi_token_status");
}

export function setBangumiToken(token: string | null): Promise<BangumiTokenStatus> {
  return invokeCommand<BangumiTokenStatus>("set_bangumi_token", { token });
}

export function checkBangumiConnection(): Promise<BangumiConnectionStatus> {
  return invokeCommand<BangumiConnectionStatus>("check_bangumi_connection");
}
