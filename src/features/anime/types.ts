export type WatchStatus = "wish" | "watching" | "completed" | "on_hold" | "dropped";
export type Season = "winter" | "spring" | "summer" | "fall";
export type AnimeSort = "updated_desc" | "created_desc" | "title_asc" | "rating_desc" | "year_desc";

export interface Tag {
  id: number;
  name: string;
  color: string | null;
}

export interface Anime {
  id: number;
  bangumiSubjectId: number | null;
  title: string;
  originalTitle: string | null;
  aliases: string[];
  summary: string;
  coverPath: string | null;
  coverSourceUrl: string | null;
  year: number | null;
  season: Season | null;
  totalEpisodes: number | null;
  externalUpdatedAt: number | null;
  watchStatus: WatchStatus;
  rating: number | null;
  review: string;
  createdAt: number;
  updatedAt: number;
  watchedEpisodes: number;
  tags: Tag[];
  isFavorite: boolean;
}

export interface AnimeInput {
  title: string;
  originalTitle: string | null;
  aliases: string[];
  summary: string;
  year: number | null;
  season: Season | null;
  totalEpisodes: number | null;
  watchStatus: WatchStatus;
  rating: number | null;
  review: string;
  tags: string[];
}

export interface AnimeFilter {
  status?: WatchStatus;
  year?: number;
  tagId?: number;
  sort?: AnimeSort;
}

export interface Episode {
  id: number;
  animeId: number;
  episodeNumber: number;
  title: string;
  airedAt: number | null;
  isWatched: boolean;
  lastWatchedAt: number | null;
  watchCount: number;
}

export interface EpisodeInput {
  episodeNumber: number;
  title: string;
  airedAt: number | null;
}

export interface WatchRecord {
  id: number;
  animeId: number;
  episodeId: number | null;
  episodeNumber: number;
  watchedAt: number;
  comment: string;
  createdAt: number;
}

export interface WatchRecordInput {
  episodeNumber: number;
  watchedAt: number | null;
  comment: string;
}
