import { invokeCommand } from "@/lib/ipc";

import type {
  Anime,
  AnimeFilter,
  AnimeInput,
  Episode,
  EpisodeInput,
  Tag,
  WatchRecord,
  WatchRecordInput,
} from "./types";

export function listAnime(filter: AnimeFilter = {}): Promise<Anime[]> {
  return invokeCommand<Anime[]>("list_anime", { filter });
}

export function getAnime(id: number): Promise<Anime> {
  return invokeCommand<Anime>("get_anime", { id });
}

export function createAnime(input: AnimeInput): Promise<Anime> {
  return invokeCommand<Anime>("create_anime", { input });
}

export function updateAnime(id: number, input: AnimeInput): Promise<Anime> {
  return invokeCommand<Anime>("update_anime", { id, input });
}

export function deleteAnime(id: number): Promise<void> {
  return invokeCommand<void>("delete_anime", { id });
}

export function setAnimeFavorite(id: number, favorite: boolean): Promise<Anime> {
  return invokeCommand<Anime>("set_anime_favorite", { id, favorite });
}

export function listTags(): Promise<Tag[]> {
  return invokeCommand<Tag[]>("list_tags");
}

export function listEpisodes(animeId: number): Promise<Episode[]> {
  return invokeCommand<Episode[]>("list_episodes", { animeId });
}

export function createEpisode(animeId: number, input: EpisodeInput): Promise<Episode> {
  return invokeCommand<Episode>("create_episode", { animeId, input });
}

export function updateEpisode(id: number, input: EpisodeInput): Promise<Episode> {
  return invokeCommand<Episode>("update_episode", { id, input });
}

export function deleteEpisode(id: number): Promise<void> {
  return invokeCommand<void>("delete_episode", { id });
}

export function setEpisodeWatched(
  id: number,
  watched: boolean,
  watchedAt: number | null,
  comment: string | null,
): Promise<Episode> {
  return invokeCommand<Episode>("set_episode_watched", { id, watched, watchedAt, comment });
}

export function listWatchRecords(animeId: number): Promise<WatchRecord[]> {
  return invokeCommand<WatchRecord[]>("list_watch_records", { animeId });
}

export function addWatchRecord(animeId: number, input: WatchRecordInput): Promise<WatchRecord> {
  return invokeCommand<WatchRecord>("add_watch_record", { animeId, input });
}

export function updateWatchRecord(
  id: number,
  watchedAt: number | null,
  comment: string,
): Promise<WatchRecord> {
  return invokeCommand<WatchRecord>("update_watch_record", { id, watchedAt, comment });
}

export function deleteWatchRecord(id: number): Promise<void> {
  return invokeCommand<void>("delete_watch_record", { id });
}
