use serde::{Deserialize, Serialize};

use super::tag::Tag;

pub const WATCH_STATUSES: [&str; 5] = ["wish", "watching", "completed", "on_hold", "dropped"];
pub const SEASONS: [&str; 4] = ["winter", "spring", "summer", "fall"];
pub const SORT_OPTIONS: [&str; 5] = [
    "updated_desc",
    "created_desc",
    "title_asc",
    "rating_desc",
    "year_desc",
];

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AnimeRow {
    pub id: i64,
    pub bangumi_subject_id: Option<i64>,
    pub title: String,
    pub original_title: Option<String>,
    pub aliases_json: String,
    pub summary: String,
    pub cover_path: Option<String>,
    pub cover_source_url: Option<String>,
    pub year: Option<i64>,
    pub season: Option<String>,
    pub total_episodes: Option<i64>,
    pub external_updated_at: Option<i64>,
    pub watch_status: String,
    pub rating: Option<i64>,
    pub review: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub watched_episodes: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Anime {
    pub id: i64,
    pub bangumi_subject_id: Option<i64>,
    pub title: String,
    pub original_title: Option<String>,
    pub aliases: Vec<String>,
    pub summary: String,
    pub cover_path: Option<String>,
    pub cover_source_url: Option<String>,
    pub year: Option<i64>,
    pub season: Option<String>,
    pub total_episodes: Option<i64>,
    pub external_updated_at: Option<i64>,
    pub watch_status: String,
    pub rating: Option<i64>,
    pub review: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub watched_episodes: i64,
    pub tags: Vec<Tag>,
    pub is_favorite: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnimeInput {
    pub title: String,
    #[serde(default)]
    pub original_title: Option<String>,
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(default)]
    pub summary: String,
    #[serde(default)]
    pub year: Option<i64>,
    #[serde(default)]
    pub season: Option<String>,
    #[serde(default)]
    pub total_episodes: Option<i64>,
    pub watch_status: String,
    #[serde(default)]
    pub rating: Option<i64>,
    #[serde(default)]
    pub review: String,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnimeFilter {
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub year: Option<i64>,
    #[serde(default)]
    pub tag_id: Option<i64>,
    #[serde(default)]
    pub sort: Option<String>,
}
