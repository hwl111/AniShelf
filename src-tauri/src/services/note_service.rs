use std::collections::{HashMap, HashSet};

use sqlx::SqlitePool;

use crate::error::AppError;
use crate::models::note::{
    NOTE_RELATION_TARGETS, NOTE_SORT_OPTIONS, Note, NoteDeleteImpact, NoteFilter, NoteInput,
    NoteLinkCandidate, NoteRelation, NoteRelationCandidate, NoteRelationRow, NoteSummary,
};
use crate::models::tag::Tag;
use crate::repositories::note_repository as repository;
use crate::utils::now_ms;

const MAX_TITLE_CHARS: usize = 300;
const MAX_CONTENT_CHARS: usize = 200_000;
const MAX_QUERY_CHARS: usize = 200;
const MAX_TAGS: usize = 20;
const MAX_TAG_CHARS: usize = 50;

pub async fn list(pool: &SqlitePool, filter: &NoteFilter) -> Result<Vec<NoteSummary>, AppError> {
    let filter = normalize_filter(filter)?;
    let rows = repository::list(pool, &filter).await?;
    let ids: Vec<i64> = rows.iter().map(|row| row.id).collect();
    let all_tags = repository::tags_for_many(pool, &ids).await?;

    let mut tags_by_note: HashMap<i64, Vec<Tag>> = HashMap::new();
    for (note_id, id, name, color) in all_tags {
        tags_by_note
            .entry(note_id)
            .or_default()
            .push(Tag { id, name, color });
    }

    Ok(rows
        .into_iter()
        .map(|row| NoteSummary {
            id: row.id,
            title: row.title,
            excerpt: excerpt(&row.content_md),
            created_at: row.created_at,
            updated_at: row.updated_at,
            tags: tags_by_note.remove(&row.id).unwrap_or_default(),
        })
        .collect())
}

pub async fn get(pool: &SqlitePool, id: i64) -> Result<Note, AppError> {
    let mut connection = pool.acquire().await?;
    let row = repository::get_row(&mut connection, id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("笔记 {id} 不存在")))?;
    let relation_rows = repository::relations(&mut connection, id).await?;
    drop(connection);

    let tags = repository::tags_for(pool, id).await?;
    let links = repository::outgoing_links(pool, id).await?;
    let backlinks = repository::backlinks(pool, id).await?;
    let relations = resolve_relations(pool, relation_rows).await?;

    Ok(Note {
        id: row.id,
        title: row.title,
        content: row.content_md,
        created_at: row.created_at,
        updated_at: row.updated_at,
        tags,
        relations,
        links,
        backlinks,
    })
}

pub async fn create(pool: &SqlitePool, input: NoteInput) -> Result<Note, AppError> {
    let normalized = normalize_input(input)?;
    let now = now_ms();

    let mut transaction = pool.begin().await?;
    let id = repository::insert(
        &mut transaction,
        &normalized.title,
        &normalized.content,
        now,
    )
    .await?;
    repository::set_tags(&mut transaction, id, &normalized.tags, now).await?;
    transaction.commit().await?;

    get(pool, id).await
}

pub async fn update(pool: &SqlitePool, id: i64, input: NoteInput) -> Result<Note, AppError> {
    let normalized = normalize_input(input)?;
    let now = now_ms();

    let mut transaction = pool.begin().await?;
    let existing = repository::get_row(&mut transaction, id).await?;
    if existing.is_none() {
        return Err(AppError::NotFound(format!("笔记 {id} 不存在")));
    }

    repository::update(
        &mut transaction,
        id,
        &normalized.title,
        &normalized.content,
        now,
    )
    .await?;
    repository::set_tags(&mut transaction, id, &normalized.tags, now).await?;
    transaction.commit().await?;

    get(pool, id).await
}

pub async fn delete(pool: &SqlitePool, id: i64, force: bool) -> Result<(), AppError> {
    let existing = {
        let mut connection = pool.acquire().await?;
        repository::get_row(&mut connection, id).await?
    };
    if existing.is_none() {
        return Err(AppError::NotFound(format!("笔记 {id} 不存在")));
    }

    if !force {
        let backlinks = repository::backlinks(pool, id).await?;
        if !backlinks.is_empty() {
            return Err(AppError::Validation(format!(
                "该笔记被 {} 篇笔记引用，请确认引用影响后再删除",
                backlinks.len()
            )));
        }
    }

    let mut transaction = pool.begin().await?;
    repository::cleanup_polymorphic(&mut transaction, id).await?;
    repository::delete(&mut transaction, id).await?;
    transaction.commit().await?;

    Ok(())
}

pub async fn delete_impact(pool: &SqlitePool, id: i64) -> Result<NoteDeleteImpact, AppError> {
    let mut connection = pool.acquire().await?;
    let existing = repository::get_row(&mut connection, id).await?;
    drop(connection);
    if existing.is_none() {
        return Err(AppError::NotFound(format!("笔记 {id} 不存在")));
    }

    let backlinks = repository::backlinks(pool, id).await?;
    let relation_count = repository::relation_count(pool, id).await?;

    Ok(NoteDeleteImpact {
        note_id: id,
        backlinks,
        relation_count,
    })
}

pub async fn add_link(
    pool: &SqlitePool,
    source_note_id: i64,
    target_note_id: i64,
) -> Result<(), AppError> {
    if source_note_id == target_note_id {
        return Err(AppError::Validation("不能链接到笔记自身".to_string()));
    }

    let now = now_ms();
    let mut transaction = pool.begin().await?;
    let source = repository::get_row(&mut transaction, source_note_id).await?;
    if source.is_none() {
        return Err(AppError::NotFound(format!("笔记 {source_note_id} 不存在")));
    }
    let target = repository::get_row(&mut transaction, target_note_id).await?;
    if target.is_none() {
        return Err(AppError::NotFound(format!("笔记 {target_note_id} 不存在")));
    }

    repository::add_link(&mut transaction, source_note_id, target_note_id, now).await?;
    transaction.commit().await?;
    Ok(())
}

pub async fn remove_link(
    pool: &SqlitePool,
    source_note_id: i64,
    target_note_id: i64,
) -> Result<(), AppError> {
    let mut transaction = pool.begin().await?;
    repository::remove_link(&mut transaction, source_note_id, target_note_id).await?;
    transaction.commit().await?;
    Ok(())
}

pub async fn list_link_candidates(
    pool: &SqlitePool,
    note_id: i64,
    query: &str,
) -> Result<Vec<NoteLinkCandidate>, AppError> {
    let query = normalize_query(query)?;

    let rows = repository::link_candidates(pool, note_id, &query, 20).await?;
    Ok(rows
        .into_iter()
        .map(|row| NoteLinkCandidate {
            id: row.id,
            title: row.title,
            excerpt: excerpt(&row.content_md),
            updated_at: row.updated_at,
        })
        .collect())
}

pub async fn add_relation(
    pool: &SqlitePool,
    note_id: i64,
    target_type: &str,
    target_id: i64,
) -> Result<(), AppError> {
    validate_relation_target(target_type)?;
    if target_id <= 0 {
        return Err(AppError::Validation("关联对象 ID 无效".to_string()));
    }

    let now = now_ms();
    let mut transaction = pool.begin().await?;
    let note = repository::get_row(&mut transaction, note_id).await?;
    if note.is_none() {
        return Err(AppError::NotFound(format!("笔记 {note_id} 不存在")));
    }
    ensure_target_exists(&mut transaction, target_type, target_id).await?;
    repository::add_relation(&mut transaction, note_id, target_type, target_id, now).await?;
    transaction.commit().await?;
    Ok(())
}

pub async fn remove_relation(
    pool: &SqlitePool,
    note_id: i64,
    relation_id: i64,
) -> Result<(), AppError> {
    let mut transaction = pool.begin().await?;
    let removed = repository::remove_relation(&mut transaction, note_id, relation_id).await?;
    if !removed {
        return Err(AppError::NotFound(format!("关联 {relation_id} 不存在")));
    }
    transaction.commit().await?;
    Ok(())
}

pub async fn list_relation_candidates(
    pool: &SqlitePool,
    target_type: &str,
    query: &str,
) -> Result<Vec<NoteRelationCandidate>, AppError> {
    validate_relation_target(target_type)?;
    let query = normalize_query(query)?;

    match target_type {
        "anime" => {
            let rows = repository::anime_candidates(pool, &query, 20).await?;
            Ok(rows
                .into_iter()
                .map(|(id, title, original_title, year)| NoteRelationCandidate {
                    id,
                    title,
                    subtitle: anime_subtitle(original_title.as_deref(), year),
                })
                .collect())
        }
        "book" => {
            let rows = repository::book_candidates(pool, &query, 20).await?;
            Ok(rows
                .into_iter()
                .map(|(id, title, author, format)| NoteRelationCandidate {
                    id,
                    title,
                    subtitle: book_subtitle(&author, &format),
                })
                .collect())
        }
        _ => {
            let rows = repository::episode_candidates(pool, &query, 20).await?;
            Ok(rows
                .into_iter()
                .map(|(id, number, title, anime_title)| NoteRelationCandidate {
                    id,
                    title: episode_title(number, &title),
                    subtitle: anime_title,
                })
                .collect())
        }
    }
}

struct NormalizedNote {
    title: String,
    content: String,
    tags: Vec<String>,
}

fn normalize_input(input: NoteInput) -> Result<NormalizedNote, AppError> {
    let title = input.title.trim().to_string();
    let content = input.content.replace("\r\n", "\n").replace('\r', "\n");

    if title.is_empty() && content.trim().is_empty() {
        return Err(AppError::Validation(
            "笔记标题和正文不能同时为空".to_string(),
        ));
    }
    if title.chars().count() > MAX_TITLE_CHARS {
        return Err(AppError::Validation(format!(
            "笔记标题不能超过 {MAX_TITLE_CHARS} 个字符"
        )));
    }
    if content.chars().count() > MAX_CONTENT_CHARS {
        return Err(AppError::Validation(format!(
            "笔记正文不能超过 {MAX_CONTENT_CHARS} 个字符"
        )));
    }

    let tags = normalize_tags(input.tags)?;

    Ok(NormalizedNote {
        title,
        content,
        tags,
    })
}

fn normalize_filter(filter: &NoteFilter) -> Result<NoteFilter, AppError> {
    let mut normalized = filter.clone();

    normalized.query = match filter.query.as_deref() {
        Some(value) => {
            let trimmed = value.trim();
            if trimmed.chars().count() > MAX_QUERY_CHARS {
                return Err(AppError::Validation(format!(
                    "搜索关键词不能超过 {MAX_QUERY_CHARS} 个字符"
                )));
            }
            Some(trimmed.to_string())
        }
        None => None,
    };

    if let Some(sort) = filter.sort.as_deref()
        && !NOTE_SORT_OPTIONS.contains(&sort)
    {
        return Err(AppError::Validation("笔记排序方式无效".to_string()));
    }

    if let Some(tag_id) = filter.tag_id
        && tag_id <= 0
    {
        return Err(AppError::Validation("标签筛选无效".to_string()));
    }

    match (filter.relation_type.as_deref(), filter.relation_id) {
        (Some(target_type), Some(target_id)) => {
            validate_relation_target(target_type)?;
            if target_id <= 0 {
                return Err(AppError::Validation("关联对象 ID 无效".to_string()));
            }
        }
        (None, None) => {}
        _ => {
            return Err(AppError::Validation(
                "关联筛选需要同时提供类型和 ID".to_string(),
            ));
        }
    }

    normalized.limit = Some(filter.limit.unwrap_or(100).clamp(1, 200));
    normalized.offset = Some(filter.offset.unwrap_or(0).max(0));

    Ok(normalized)
}

fn normalize_query(query: &str) -> Result<String, AppError> {
    let trimmed = query.trim();
    if trimmed.chars().count() > MAX_QUERY_CHARS {
        return Err(AppError::Validation(format!(
            "搜索关键词不能超过 {MAX_QUERY_CHARS} 个字符"
        )));
    }
    Ok(trimmed.to_string())
}

fn normalize_tags(values: Vec<String>) -> Result<Vec<String>, AppError> {
    let mut result = Vec::new();
    let mut seen = HashSet::new();

    for value in values {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.chars().count() > MAX_TAG_CHARS {
            return Err(AppError::Validation(format!(
                "标签单个不能超过 {MAX_TAG_CHARS} 个字符"
            )));
        }

        if seen.insert(trimmed.to_lowercase()) {
            result.push(trimmed.to_string());
        }
    }

    if result.len() > MAX_TAGS {
        return Err(AppError::Validation(format!("标签最多 {MAX_TAGS} 个")));
    }

    Ok(result)
}

fn validate_relation_target(target_type: &str) -> Result<(), AppError> {
    if !NOTE_RELATION_TARGETS.contains(&target_type) {
        return Err(AppError::Validation("关联对象类型无效".to_string()));
    }
    Ok(())
}

async fn ensure_target_exists(
    connection: &mut sqlx::SqliteConnection,
    target_type: &str,
    target_id: i64,
) -> Result<(), AppError> {
    let exists: bool = match target_type {
        "anime" => {
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM anime WHERE id = ?)")
                .bind(target_id)
                .fetch_one(&mut *connection)
                .await?
        }
        "book" => {
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM books WHERE id = ?)")
                .bind(target_id)
                .fetch_one(&mut *connection)
                .await?
        }
        _ => {
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM anime_episodes WHERE id = ?)")
                .bind(target_id)
                .fetch_one(&mut *connection)
                .await?
        }
    };

    if !exists {
        return Err(AppError::NotFound("关联对象不存在".to_string()));
    }
    Ok(())
}

async fn resolve_relations(
    pool: &SqlitePool,
    rows: Vec<NoteRelationRow>,
) -> Result<Vec<NoteRelation>, AppError> {
    let mut anime_ids = Vec::new();
    let mut book_ids = Vec::new();
    let mut episode_ids = Vec::new();

    for row in &rows {
        match row.target_type.as_str() {
            "anime" => anime_ids.push(row.target_id),
            "book" => book_ids.push(row.target_id),
            _ => episode_ids.push(row.target_id),
        }
    }

    let anime_infos: HashMap<i64, (String, Option<String>, Option<i64>)> =
        repository::anime_infos(pool, &anime_ids)
            .await?
            .into_iter()
            .map(|(id, title, original_title, year)| (id, (title, original_title, year)))
            .collect();
    let book_infos: HashMap<i64, (String, String, String)> =
        repository::book_infos(pool, &book_ids)
            .await?
            .into_iter()
            .map(|(id, title, author, format)| (id, (title, author, format)))
            .collect();
    let episode_infos: HashMap<i64, (i64, String, String)> =
        repository::episode_infos(pool, &episode_ids)
            .await?
            .into_iter()
            .map(|(id, number, title, anime_title)| (id, (number, title, anime_title)))
            .collect();

    Ok(rows
        .into_iter()
        .map(|row| {
            let (title, subtitle) = match row.target_type.as_str() {
                "anime" => match anime_infos.get(&row.target_id) {
                    Some((title, original_title, year)) => (
                        title.clone(),
                        anime_subtitle(original_title.as_deref(), *year),
                    ),
                    None => ("已删除的番剧".to_string(), String::new()),
                },
                "book" => match book_infos.get(&row.target_id) {
                    Some((title, author, format)) => (title.clone(), book_subtitle(author, format)),
                    None => ("已删除的书籍".to_string(), String::new()),
                },
                _ => match episode_infos.get(&row.target_id) {
                    Some((number, title, anime_title)) => {
                        (episode_title(*number, title), anime_title.clone())
                    }
                    None => ("已删除的集数".to_string(), String::new()),
                },
            };

            NoteRelation {
                id: row.id,
                target_type: row.target_type,
                target_id: row.target_id,
                title,
                subtitle,
            }
        })
        .collect())
}

fn anime_subtitle(original_title: Option<&str>, year: Option<i64>) -> String {
    let mut parts = vec!["番剧".to_string()];
    if let Some(original) = original_title
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        parts.push(original.to_string());
    }
    if let Some(year) = year {
        parts.push(format!("{year}"));
    }
    parts.join(" · ")
}

fn book_subtitle(author: &str, format: &str) -> String {
    let author = author.trim();
    if author.is_empty() {
        format!("书籍 · {}", format.to_uppercase())
    } else {
        format!("书籍 · {author}")
    }
}

fn episode_title(number: i64, title: &str) -> String {
    let title = title.trim();
    if title.is_empty() {
        format!("第 {number} 集")
    } else {
        format!("第 {number} 集 · {title}")
    }
}

fn excerpt(content: &str) -> String {
    let flattened = content.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut chars = flattened.chars();
    let text: String = chars.by_ref().take(160).collect();
    if chars.next().is_some() {
        format!("{text}…")
    } else {
        text
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{connect, migrate};

    use sqlx::SqlitePool;
    use tempfile::{TempDir, tempdir};

    async fn migrated_pool() -> (TempDir, SqlitePool) {
        let dir = tempdir().expect("tempdir");
        let database_file = dir.path().join("note-test.sqlite");
        let backups_dir = dir.path().join("backups");
        let pool = connect(&database_file).await.expect("connect");
        migrate::run(&pool, &backups_dir).await.expect("migrate");
        (dir, pool)
    }

    async fn count(pool: &SqlitePool, table: &str) -> i64 {
        let sql = format!("SELECT COUNT(*) FROM {table}");
        sqlx::query_scalar(sqlx::AssertSqlSafe(sql))
            .fetch_one(pool)
            .await
            .expect("count")
    }

    fn input(title: &str, content: &str, tags: &[&str]) -> NoteInput {
        NoteInput {
            title: title.to_string(),
            content: content.to_string(),
            tags: tags.iter().map(|tag| (*tag).to_string()).collect(),
        }
    }

    #[tokio::test]
    async fn note_crud_and_tags_round_trip() {
        let (_dir, pool) = migrated_pool().await;

        let created = create(&pool, input("旅途记录", "第一条正文", &["记录", "旅途"]))
            .await
            .expect("create");
        assert_eq!(created.title, "旅途记录");
        assert_eq!(created.content, "第一条正文");
        assert_eq!(created.tags.len(), 2);

        let updated = update(
            &pool,
            created.id,
            input("旅途记录（二）", "更新后的正文", &["记录"]),
        )
        .await
        .expect("update");
        assert_eq!(updated.title, "旅途记录（二）");
        assert_eq!(updated.content, "更新后的正文");
        assert_eq!(updated.tags.len(), 1);
        assert_eq!(updated.tags[0].name, "记录");

        let summaries = list(&pool, &NoteFilter::default()).await.expect("list");
        assert_eq!(summaries.len(), 1);
        assert_eq!(summaries[0].id, created.id);

        delete(&pool, created.id, false).await.expect("delete");
        assert_eq!(count(&pool, "notes").await, 0);
        assert_eq!(count(&pool, "entity_tags").await, 0);
    }

    #[tokio::test]
    async fn note_validation_rejects_empty_and_oversized_values() {
        let (_dir, pool) = migrated_pool().await;

        let empty = create(&pool, input("", "   ", &[])).await;
        assert!(matches!(empty, Err(AppError::Validation(_))));

        let long_title = "字".repeat(MAX_TITLE_CHARS + 1);
        let oversized = create(&pool, input(&long_title, "正文", &[])).await;
        assert!(matches!(oversized, Err(AppError::Validation(_))));

        let too_many_tags: Vec<String> =
            (0..=MAX_TAGS).map(|index| format!("标签{index}")).collect();
        let tags_error = create(
            &pool,
            NoteInput {
                title: "标签测试".to_string(),
                content: String::new(),
                tags: too_many_tags,
            },
        )
        .await;
        assert!(matches!(tags_error, Err(AppError::Validation(_))));

        let duplicate_tags = create(&pool, input("标题", "", &["标签", "标签", " 标签 "]))
            .await
            .expect("create");
        assert_eq!(duplicate_tags.tags.len(), 1);
    }

    #[tokio::test]
    async fn note_search_uses_fts_and_like_fallback() {
        let (_dir, pool) = migrated_pool().await;

        create(&pool, input("东京的雨夜", "城市与灯火", &[]))
            .await
            .expect("first");
        let star = create(&pool, input("星星", "夜空中的光点", &[]))
            .await
            .expect("second");

        let filter = NoteFilter {
            query: Some("雨夜".to_string()),
            ..NoteFilter::default()
        };
        let two_chars = list(&pool, &filter).await.expect("two-char search");
        assert_eq!(two_chars.len(), 1);
        assert_eq!(two_chars[0].title, "东京的雨夜");

        let filter = NoteFilter {
            query: Some("东京的雨".to_string()),
            ..NoteFilter::default()
        };
        let fts = list(&pool, &filter).await.expect("fts search");
        assert_eq!(fts.len(), 1);

        let filter = NoteFilter {
            query: Some("城市".to_string()),
            ..NoteFilter::default()
        };
        let content_match = list(&pool, &filter).await.expect("content search");
        assert_eq!(content_match.len(), 1);

        let filter = NoteFilter {
            query: Some("不存在".to_string()),
            ..NoteFilter::default()
        };
        let empty = list(&pool, &filter).await.expect("empty search");
        assert!(empty.is_empty());

        update(&pool, star.id, input("月亮", "夜空", &[]))
            .await
            .expect("update");

        let filter = NoteFilter {
            query: Some("星星".to_string()),
            ..NoteFilter::default()
        };
        let stale = list(&pool, &filter).await.expect("stale search");
        assert!(stale.is_empty());

        let filter = NoteFilter {
            query: Some("月亮".to_string()),
            ..NoteFilter::default()
        };
        let updated = list(&pool, &filter).await.expect("updated search");
        assert_eq!(updated.len(), 1);

        delete(&pool, star.id, false).await.expect("delete");
        let after_delete = list(&pool, &filter).await.expect("after delete");
        assert!(after_delete.is_empty());
    }

    #[tokio::test]
    async fn links_and_backlinks_are_tracked() {
        let (_dir, pool) = migrated_pool().await;

        let first = create(&pool, input("第一篇", "正文一", &[]))
            .await
            .expect("first");
        let second = create(&pool, input("第二篇", "正文二", &[]))
            .await
            .expect("second");

        add_link(&pool, first.id, second.id).await.expect("link");
        let stored = get(&pool, first.id).await.expect("get first");
        assert_eq!(stored.links.len(), 1);
        assert_eq!(stored.links[0].note_id, second.id);

        let target = get(&pool, second.id).await.expect("get second");
        assert_eq!(target.backlinks.len(), 1);
        assert_eq!(target.backlinks[0].note_id, first.id);

        let self_link = add_link(&pool, first.id, first.id).await;
        assert!(matches!(self_link, Err(AppError::Validation(_))));

        let candidates = list_link_candidates(&pool, first.id, "第二")
            .await
            .expect("candidates");
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].id, second.id);

        remove_link(&pool, first.id, second.id)
            .await
            .expect("unlink");
        assert!(get(&pool, first.id).await.expect("get").links.is_empty());
        assert!(
            get(&pool, second.id)
                .await
                .expect("get")
                .backlinks
                .is_empty()
        );
    }

    #[tokio::test]
    async fn deleting_referenced_note_requires_confirmation() {
        let (_dir, pool) = migrated_pool().await;

        let source = create(&pool, input("引用者", "正文", &["清理"]))
            .await
            .expect("source");
        let target = create(&pool, input("被引用", "正文", &[]))
            .await
            .expect("target");
        add_link(&pool, source.id, target.id).await.expect("link");

        let impact = delete_impact(&pool, target.id).await.expect("impact");
        assert_eq!(impact.backlinks.len(), 1);
        assert_eq!(impact.backlinks[0].note_id, source.id);

        let blocked = delete(&pool, target.id, false).await;
        assert!(matches!(blocked, Err(AppError::Validation(_))));
        assert_eq!(count(&pool, "notes").await, 2);

        delete(&pool, target.id, true).await.expect("force delete");
        assert_eq!(count(&pool, "notes").await, 1);
        assert_eq!(count(&pool, "note_links").await, 0);
        assert!(
            get(&pool, source.id)
                .await
                .expect("source")
                .links
                .is_empty()
        );
    }

    async fn insert_anime(pool: &SqlitePool, title: &str) -> i64 {
        sqlx::query(
            "INSERT INTO anime (title, watch_status, created_at, updated_at)
             VALUES (?, 'wish', 1, 1)",
        )
        .bind(title)
        .execute(pool)
        .await
        .expect("anime")
        .last_insert_rowid()
    }

    async fn insert_book(pool: &SqlitePool, title: &str, author: &str) -> i64 {
        sqlx::query(
            "INSERT INTO books
             (title, author, format, file_path, normalized_path, file_size, file_hash,
              imported_at, created_at, updated_at)
             VALUES (?, ?, 'epub', '/tmp/book.epub', ?, 1, 'hash', 1, 1, 1)",
        )
        .bind(title)
        .bind(author)
        .bind(format!("/tmp/{title}.epub"))
        .execute(pool)
        .await
        .expect("book")
        .last_insert_rowid()
    }

    async fn insert_episode(pool: &SqlitePool, anime_id: i64, number: i64, title: &str) -> i64 {
        sqlx::query(
            "INSERT INTO anime_episodes (anime_id, episode_number, title, created_at, updated_at)
             VALUES (?, ?, ?, 1, 1)",
        )
        .bind(anime_id)
        .bind(number)
        .bind(title)
        .execute(pool)
        .await
        .expect("episode")
        .last_insert_rowid()
    }

    #[tokio::test]
    async fn relations_validate_targets_and_resolve_titles() {
        let (_dir, pool) = migrated_pool().await;

        let note = create(&pool, input("关联测试", "正文", &[]))
            .await
            .expect("note");
        let anime_id = insert_anime(&pool, "葬送的芙莉莲").await;
        let book_id = insert_book(&pool, "示例书籍", "示例作者").await;
        let episode_id = insert_episode(&pool, anime_id, 3, "启程").await;

        add_relation(&pool, note.id, "anime", anime_id)
            .await
            .expect("anime relation");
        add_relation(&pool, note.id, "book", book_id)
            .await
            .expect("book relation");
        add_relation(&pool, note.id, "episode", episode_id)
            .await
            .expect("episode relation");
        add_relation(&pool, note.id, "anime", anime_id)
            .await
            .expect("duplicate relation is ignored");

        let stored = get(&pool, note.id).await.expect("get");
        assert_eq!(stored.relations.len(), 3);
        let anime = stored
            .relations
            .iter()
            .find(|relation| relation.target_type == "anime")
            .expect("anime");
        assert_eq!(anime.title, "葬送的芙莉莲");
        assert!(anime.subtitle.contains("番剧"));
        let episode = stored
            .relations
            .iter()
            .find(|relation| relation.target_type == "episode")
            .expect("episode");
        assert_eq!(episode.title, "第 3 集 · 启程");
        assert_eq!(episode.subtitle, "葬送的芙莉莲");

        let filter = NoteFilter {
            relation_type: Some("anime".to_string()),
            relation_id: Some(anime_id),
            ..NoteFilter::default()
        };
        let filtered = list(&pool, &filter).await.expect("filtered");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, note.id);

        let invalid_type = add_relation(&pool, note.id, "movie", 1).await;
        assert!(matches!(invalid_type, Err(AppError::Validation(_))));

        let missing_target = add_relation(&pool, note.id, "book", 999_999).await;
        assert!(matches!(missing_target, Err(AppError::NotFound(_))));

        let relation_id = stored
            .relations
            .iter()
            .find(|relation| relation.target_type == "book")
            .expect("book relation")
            .id;
        remove_relation(&pool, note.id, relation_id)
            .await
            .expect("remove relation");
        assert_eq!(get(&pool, note.id).await.expect("get").relations.len(), 2);

        let missing_relation = remove_relation(&pool, note.id, relation_id).await;
        assert!(matches!(missing_relation, Err(AppError::NotFound(_))));
    }

    #[tokio::test]
    async fn relation_candidates_cover_anime_book_and_episode() {
        let (_dir, pool) = migrated_pool().await;

        let anime_id = insert_anime(&pool, "孤独摇滚").await;
        insert_book(&pool, "示例书", "作者").await;
        insert_episode(&pool, anime_id, 1, "第一话").await;

        let anime_candidates = list_relation_candidates(&pool, "anime", "孤独")
            .await
            .expect("anime candidates");
        assert_eq!(anime_candidates.len(), 1);
        assert_eq!(anime_candidates[0].title, "孤独摇滚");

        let book_candidates = list_relation_candidates(&pool, "book", "示例")
            .await
            .expect("book candidates");
        assert_eq!(book_candidates.len(), 1);
        assert_eq!(book_candidates[0].subtitle, "书籍 · 作者");

        let episode_candidates = list_relation_candidates(&pool, "episode", "1")
            .await
            .expect("episode candidates");
        assert_eq!(episode_candidates.len(), 1);
        assert_eq!(episode_candidates[0].title, "第 1 集 · 第一话");
        assert_eq!(episode_candidates[0].subtitle, "孤独摇滚");
    }
}
