import { invokeCommand } from "@/lib/ipc";

import type { Anime, AnimeFilter, AnimeInput, Tag } from "./types";

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
