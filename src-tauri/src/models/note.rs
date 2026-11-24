use serde::{Deserialize, Serialize};

use super::tag::Tag;

pub const NOTE_SORT_OPTIONS: [&str; 2] = ["updated_desc", "created_desc"];
pub const NOTE_RELATION_TARGETS: [&str; 3] = ["anime", "book", "episode"];

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct NoteRow {
    pub id: i64,
    pub title: String,
    pub content_md: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteSummary {
    pub id: i64,
    pub title: String,
    pub excerpt: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub tags: Vec<Tag>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Note {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub tags: Vec<Tag>,
    pub relations: Vec<NoteRelation>,
    pub links: Vec<NoteLink>,
    pub backlinks: Vec<NoteLink>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteRelation {
    pub id: i64,
    pub target_type: String,
    pub target_id: i64,
    pub title: String,
    pub subtitle: String,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct NoteLink {
    pub note_id: i64,
    pub title: String,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteDeleteImpact {
    pub note_id: i64,
    pub backlinks: Vec<NoteLink>,
    pub relation_count: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteLinkCandidate {
    pub id: i64,
    pub title: String,
    pub excerpt: String,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteRelationCandidate {
    pub id: i64,
    pub title: String,
    pub subtitle: String,
}

#[derive(Debug, Clone)]
pub struct NoteRelationRow {
    pub id: i64,
    pub target_type: String,
    pub target_id: i64,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteInput {
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub content: String,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteFilter {
    #[serde(default)]
    pub query: Option<String>,
    #[serde(default)]
    pub tag_id: Option<i64>,
    #[serde(default)]
    pub relation_type: Option<String>,
    #[serde(default)]
    pub relation_id: Option<i64>,
    #[serde(default)]
    pub sort: Option<String>,
    #[serde(default)]
    pub limit: Option<i64>,
    #[serde(default)]
    pub offset: Option<i64>,
}
