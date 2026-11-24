use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Episode {
    pub id: i64,
    pub anime_id: i64,
    pub episode_number: i64,
    pub title: String,
    pub aired_at: Option<i64>,
    pub is_watched: bool,
    pub last_watched_at: Option<i64>,
    pub watch_count: i64,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct WatchRecord {
    pub id: i64,
    pub anime_id: i64,
    pub episode_id: Option<i64>,
    pub episode_number: i64,
    pub watched_at: i64,
    pub comment: String,
    pub created_at: i64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EpisodeInput {
    pub episode_number: i64,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub aired_at: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WatchRecordInput {
    pub episode_number: i64,
    #[serde(default)]
    pub watched_at: Option<i64>,
    #[serde(default)]
    pub comment: String,
}
