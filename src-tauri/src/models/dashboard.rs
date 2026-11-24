use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardData {
    pub watching_anime: Vec<DashboardAnime>,
    pub recent_watch_records: Vec<DashboardWatchRecord>,
    pub recent_books: Vec<DashboardBook>,
    pub recent_notes: Vec<DashboardNote>,
    pub stats: DashboardStats,
    pub timeline: Vec<ActivityItem>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardAnime {
    pub id: i64,
    pub title: String,
    pub total_episodes: Option<i64>,
    pub watched_episodes: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardWatchRecord {
    pub id: i64,
    pub anime_id: i64,
    pub anime_title: String,
    pub episode_number: i64,
    pub watched_at: i64,
    pub comment: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardBook {
    pub id: i64,
    pub title: String,
    pub author: String,
    pub format: String,
    pub cover_url: Option<String>,
    pub percent: f64,
    pub last_read_at: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardNote {
    pub id: i64,
    pub title: String,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DashboardStats {
    pub anime_total: i64,
    pub anime_watching: i64,
    pub anime_completed: i64,
    pub anime_favorite: i64,
    pub books_total: i64,
    pub books_reading: i64,
    pub books_favorite: i64,
    pub notes_total: i64,
    pub notes_favorite: i64,
    pub watched_episodes: i64,
    pub watch_records_total: i64,
    pub reading_sessions: i64,
    pub reading_ms: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityItem {
    pub kind: String,
    pub entity_id: i64,
    pub title: String,
    pub subtitle: String,
    pub occurred_at: i64,
}
