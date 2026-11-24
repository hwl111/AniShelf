use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ReadingProgress {
    pub book_id: i64,
    pub location_type: String,
    pub location_json: String,
    pub percent: f64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Bookmark {
    pub id: i64,
    pub book_id: i64,
    pub location_type: String,
    pub location_json: String,
    pub label: String,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ReaderSettings {
    pub scope: String,
    pub preferences_json: String,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProgressInput {
    pub location_type: String,
    pub location_json: String,
    pub percent: f64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BookmarkInput {
    pub location_type: String,
    pub location_json: String,
    #[serde(default)]
    pub label: String,
}
