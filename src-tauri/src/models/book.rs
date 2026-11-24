use serde::{Deserialize, Serialize};

use super::tag::Tag;

pub const BOOK_FORMATS: [&str; 4] = ["epub", "pdf", "txt", "markdown"];
pub const BOOK_SORT_OPTIONS: [&str; 4] =
    ["updated_desc", "imported_desc", "title_asc", "size_desc"];

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct BookRow {
    pub id: i64,
    pub title: String,
    pub author: String,
    pub format: String,
    pub file_path: String,
    pub source_path: Option<String>,
    pub file_size: i64,
    pub file_mtime: Option<i64>,
    pub cover_path: Option<String>,
    pub description: String,
    pub category: String,
    pub imported_at: i64,
    pub last_opened_at: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Book {
    pub id: i64,
    pub title: String,
    pub author: String,
    pub format: String,
    pub file_path: String,
    pub source_path: Option<String>,
    pub file_size: i64,
    pub file_mtime: Option<i64>,
    pub cover_path: Option<String>,
    pub cover_url: Option<String>,
    pub description: String,
    pub category: String,
    pub imported_at: i64,
    pub last_opened_at: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
    pub tags: Vec<Tag>,
    pub is_favorite: bool,
    pub file_exists: bool,
}

#[derive(Debug, Clone)]
pub struct BookInsert {
    pub title: String,
    pub author: String,
    pub format: String,
    pub file_path: String,
    pub source_path: Option<String>,
    pub normalized_path: String,
    pub file_size: i64,
    pub file_mtime: Option<i64>,
    pub file_hash: String,
    pub cover_path: Option<String>,
    pub description: String,
    pub imported_at: i64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BookUpdateInput {
    pub title: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BookFilter {
    #[serde(default)]
    pub format: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub tag_id: Option<i64>,
    #[serde(default)]
    pub favorite: Option<bool>,
    #[serde(default)]
    pub sort: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BookImportResult {
    pub book: Book,
    pub duplicate: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BookFileStatus {
    pub exists: bool,
    pub path: String,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct BookRelocation {
    pub file_path: String,
    pub normalized_path: String,
    pub file_size: i64,
    pub file_mtime: Option<i64>,
    pub file_hash: String,
}
