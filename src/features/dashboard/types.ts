export interface DashboardAnime {
  id: number;
  title: string;
  totalEpisodes: number | null;
  watchedEpisodes: number;
}

export interface DashboardWatchRecord {
  id: number;
  animeId: number;
  animeTitle: string;
  episodeNumber: number;
  watchedAt: number;
  comment: string;
}

export interface DashboardBook {
  id: number;
  title: string;
  author: string;
  format: string;
  coverUrl: string | null;
  percent: number;
  lastReadAt: number;
}

export interface DashboardNote {
  id: number;
  title: string;
  updatedAt: number;
}

export interface DashboardStats {
  animeTotal: number;
  animeWatching: number;
  animeCompleted: number;
  animeFavorite: number;
  booksTotal: number;
  booksReading: number;
  booksFavorite: number;
  notesTotal: number;
  notesFavorite: number;
  watchedEpisodes: number;
  watchRecordsTotal: number;
  readingSessions: number;
  readingMs: number;
}

export interface ActivityItem {
  kind: string;
  entityId: number;
  title: string;
  subtitle: string;
  occurredAt: number;
}

export interface DashboardData {
  watchingAnime: DashboardAnime[];
  recentWatchRecords: DashboardWatchRecord[];
  recentBooks: DashboardBook[];
  recentNotes: DashboardNote[];
  stats: DashboardStats;
  timeline: ActivityItem[];
}
