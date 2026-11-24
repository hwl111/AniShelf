use sqlx::SqlitePool;

use crate::error::AppError;
use crate::utils::escape_like;

const MAX_QUERY_CHARS: usize = 200;

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    pub entity_type: String,
    pub id: i64,
    pub title: String,
    pub subtitle: String,
    pub parent_id: Option<i64>,
}

pub async fn search(
    pool: &SqlitePool,
    query: &str,
    limit: u32,
) -> Result<Vec<SearchResult>, AppError> {
    let query = query.trim();
    if query.is_empty() {
        return Ok(Vec::new());
    }
    if query.chars().count() > MAX_QUERY_CHARS {
        return Err(AppError::Validation(format!(
            "搜索关键词不能超过 {MAX_QUERY_CHARS} 个字符"
        )));
    }

    let limit = limit.clamp(1, 50) as i64;
    let mut results = Vec::new();

    if query.chars().count() >= 3 {
        // FTS5 trigram supports substring matches of at least three characters.
        let match_query = phrase(query);
        results.extend(search_anime_fts(pool, &match_query, limit).await?);
        results.extend(search_books_fts(pool, &match_query, limit).await?);
        results.extend(search_notes_fts(pool, &match_query, limit).await?);
        results.extend(search_annotations_fts(pool, &match_query, limit).await?);
    } else {
        // Trigram cannot match one or two characters; fall back to escaped LIKE scans.
        let pattern = format!("%{}%", escape_like(query));
        results.extend(search_anime_like(pool, &pattern, limit).await?);
        results.extend(search_books_like(pool, &pattern, limit).await?);
        results.extend(search_notes_like(pool, &pattern, limit).await?);
        results.extend(search_annotations_like(pool, &pattern, limit).await?);
    }

    Ok(results)
}

fn phrase(query: &str) -> String {
    format!("\"{}\"", query.replace('"', "\"\""))
}

async fn search_anime_fts(
    pool: &SqlitePool,
    match_query: &str,
    limit: i64,
) -> Result<Vec<SearchResult>, AppError> {
    let rows: Vec<(i64, String, String)> = sqlx::query_as(
        "SELECT rowid, title,
                COALESCE(NULLIF(original_title, ''), substr(summary, 1, 80), '')
         FROM anime_fts
         WHERE anime_fts MATCH ?
         ORDER BY bm25(anime_fts)
         LIMIT ?",
    )
    .bind(match_query)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|(id, title, subtitle)| SearchResult {
            entity_type: "anime".to_string(),
            id,
            title,
            subtitle,
            parent_id: None,
        })
        .collect())
}

async fn search_books_fts(
    pool: &SqlitePool,
    match_query: &str,
    limit: i64,
) -> Result<Vec<SearchResult>, AppError> {
    let rows: Vec<(i64, String, String)> = sqlx::query_as(
        "SELECT rowid, title,
                COALESCE(NULLIF(author, ''), substr(description, 1, 80), '')
         FROM books_fts
         WHERE books_fts MATCH ?
         ORDER BY bm25(books_fts)
         LIMIT ?",
    )
    .bind(match_query)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|(id, title, subtitle)| SearchResult {
            entity_type: "book".to_string(),
            id,
            title,
            subtitle,
            parent_id: None,
        })
        .collect())
}

async fn search_notes_fts(
    pool: &SqlitePool,
    match_query: &str,
    limit: i64,
) -> Result<Vec<SearchResult>, AppError> {
    let rows: Vec<(i64, String, String)> = sqlx::query_as(
        "SELECT rowid,
                COALESCE(NULLIF(title, ''), substr(content_md, 1, 40), '无标题笔记'),
                COALESCE(substr(replace(content_md, char(10), ' '), 1, 80), '')
         FROM notes_fts
         WHERE notes_fts MATCH ?
         ORDER BY bm25(notes_fts)
         LIMIT ?",
    )
    .bind(match_query)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|(id, title, subtitle)| SearchResult {
            entity_type: "note".to_string(),
            id,
            title,
            subtitle,
            parent_id: None,
        })
        .collect())
}

async fn search_annotations_fts(
    pool: &SqlitePool,
    match_query: &str,
    limit: i64,
) -> Result<Vec<SearchResult>, AppError> {
    let rows: Vec<(i64, String, String, i64)> = sqlx::query_as(
        "SELECT annotations_fts.rowid,
                COALESCE(NULLIF(annotations_fts.selected_text, ''),
                         NULLIF(annotations_fts.note, ''), '批注'),
                COALESCE(b.title, ''),
                a.book_id
         FROM annotations_fts
         JOIN annotations a ON a.id = annotations_fts.rowid
         JOIN books b ON b.id = a.book_id
         WHERE annotations_fts MATCH ?
         ORDER BY bm25(annotations_fts)
         LIMIT ?",
    )
    .bind(match_query)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|(id, title, subtitle, book_id)| SearchResult {
            entity_type: "annotation".to_string(),
            id,
            title,
            subtitle,
            parent_id: Some(book_id),
        })
        .collect())
}

async fn search_anime_like(
    pool: &SqlitePool,
    pattern: &str,
    limit: i64,
) -> Result<Vec<SearchResult>, AppError> {
    let rows: Vec<(i64, String, String)> = sqlx::query_as(
        "SELECT id, title, COALESCE(NULLIF(original_title, ''), substr(summary, 1, 80), '')
         FROM anime
         WHERE title LIKE ? ESCAPE '\\'
            OR original_title LIKE ? ESCAPE '\\'
            OR aliases_json LIKE ? ESCAPE '\\'
            OR summary LIKE ? ESCAPE '\\'
         ORDER BY updated_at DESC
         LIMIT ?",
    )
    .bind(pattern)
    .bind(pattern)
    .bind(pattern)
    .bind(pattern)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|(id, title, subtitle)| SearchResult {
            entity_type: "anime".to_string(),
            id,
            title,
            subtitle,
            parent_id: None,
        })
        .collect())
}

async fn search_books_like(
    pool: &SqlitePool,
    pattern: &str,
    limit: i64,
) -> Result<Vec<SearchResult>, AppError> {
    let rows: Vec<(i64, String, String)> = sqlx::query_as(
        "SELECT id, title, COALESCE(NULLIF(author, ''), substr(description, 1, 80), '')
         FROM books
         WHERE title LIKE ? ESCAPE '\\'
            OR author LIKE ? ESCAPE '\\'
            OR description LIKE ? ESCAPE '\\'
         ORDER BY updated_at DESC
         LIMIT ?",
    )
    .bind(pattern)
    .bind(pattern)
    .bind(pattern)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|(id, title, subtitle)| SearchResult {
            entity_type: "book".to_string(),
            id,
            title,
            subtitle,
            parent_id: None,
        })
        .collect())
}

async fn search_notes_like(
    pool: &SqlitePool,
    pattern: &str,
    limit: i64,
) -> Result<Vec<SearchResult>, AppError> {
    let rows: Vec<(i64, String, String)> = sqlx::query_as(
        "SELECT id,
                COALESCE(NULLIF(title, ''), substr(content_md, 1, 40), '无标题笔记'),
                COALESCE(substr(replace(content_md, char(10), ' '), 1, 80), '')
         FROM notes
         WHERE title LIKE ? ESCAPE '\\'
            OR content_md LIKE ? ESCAPE '\\'
         ORDER BY updated_at DESC
         LIMIT ?",
    )
    .bind(pattern)
    .bind(pattern)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|(id, title, subtitle)| SearchResult {
            entity_type: "note".to_string(),
            id,
            title,
            subtitle,
            parent_id: None,
        })
        .collect())
}

async fn search_annotations_like(
    pool: &SqlitePool,
    pattern: &str,
    limit: i64,
) -> Result<Vec<SearchResult>, AppError> {
    let rows: Vec<(i64, String, String, i64)> = sqlx::query_as(
        "SELECT a.id,
                COALESCE(NULLIF(a.selected_text, ''), NULLIF(a.note, ''), '批注'),
                COALESCE(b.title, ''),
                a.book_id
         FROM annotations a
         JOIN books b ON b.id = a.book_id
         WHERE a.selected_text LIKE ? ESCAPE '\\'
            OR a.note LIKE ? ESCAPE '\\'
         ORDER BY a.updated_at DESC
         LIMIT ?",
    )
    .bind(pattern)
    .bind(pattern)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|(id, title, subtitle, book_id)| SearchResult {
            entity_type: "annotation".to_string(),
            id,
            title,
            subtitle,
            parent_id: Some(book_id),
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{connect, migrate};

    use tempfile::{TempDir, tempdir};

    async fn migrated_pool() -> (TempDir, SqlitePool) {
        let dir = tempdir().expect("tempdir");
        let database_file = dir.path().join("search-test.sqlite");
        let backups_dir = dir.path().join("backups");
        let pool = connect(&database_file).await.expect("connect");
        migrate::run(&pool, &backups_dir).await.expect("migrate");
        (dir, pool)
    }

    async fn insert_anime(pool: &SqlitePool, title: &str) -> i64 {
        let result =
            sqlx::query("INSERT INTO anime (title, created_at, updated_at) VALUES (?, 1, 1)")
                .bind(title)
                .execute(pool)
                .await
                .expect("insert anime");
        result.last_insert_rowid()
    }

    async fn insert_book(pool: &SqlitePool, title: &str, author: &str) -> i64 {
        let result = sqlx::query(
            "INSERT INTO books
             (title, author, format, file_path, normalized_path, file_size, imported_at, created_at, updated_at)
             VALUES (?, ?, 'epub', ?, ?, 10, 1, 1, 1)",
        )
        .bind(title)
        .bind(author)
        .bind(format!("/library/{title}.epub"))
        .bind(format!("/library/{title}.epub"))
        .execute(pool)
        .await
        .expect("insert book");
        result.last_insert_rowid()
    }

    async fn insert_note(pool: &SqlitePool, title: &str, content: &str) -> i64 {
        let result = sqlx::query(
            "INSERT INTO notes (title, content_md, created_at, updated_at) VALUES (?, ?, 1, 1)",
        )
        .bind(title)
        .bind(content)
        .execute(pool)
        .await
        .expect("insert note");
        result.last_insert_rowid()
    }

    async fn insert_annotation(pool: &SqlitePool, book_id: i64, selected: &str, note: &str) -> i64 {
        let result = sqlx::query(
            "INSERT INTO annotations
             (book_id, kind, selected_text, location_type, location_json, note, created_at, updated_at)
             VALUES (?, 'highlight', ?, 'pdf_page', '{}', ?, 1, 1)",
        )
        .bind(book_id)
        .bind(selected)
        .bind(note)
        .execute(pool)
        .await
        .expect("insert annotation");
        result.last_insert_rowid()
    }

    fn has(results: &[SearchResult], entity_type: &str, id: i64) -> bool {
        results
            .iter()
            .any(|result| result.entity_type == entity_type && result.id == id)
    }

    #[tokio::test]
    async fn chinese_trigram_and_like_fallback() {
        let (_dir, pool) = migrated_pool().await;
        let anime_id = insert_anime(&pool, "葬送的芙莉莲").await;

        let full_match = search(&pool, "芙莉莲", 20).await.expect("trigram search");
        assert!(has(&full_match, "anime", anime_id));

        let short_match = search(&pool, "芙莉", 20).await.expect("like fallback");
        assert!(has(&short_match, "anime", anime_id));

        let miss = search(&pool, "不存在", 20).await.expect("miss search");
        assert!(miss.is_empty());
    }

    #[tokio::test]
    async fn fts_tracks_updates_and_deletes() {
        let (_dir, pool) = migrated_pool().await;
        let anime_id = insert_anime(&pool, "旧标题测试").await;

        let before = search(&pool, "旧标题", 20).await.expect("before update");
        assert!(has(&before, "anime", anime_id));

        sqlx::query("UPDATE anime SET title = '新标题测试', updated_at = 2 WHERE id = ?")
            .bind(anime_id)
            .execute(&pool)
            .await
            .expect("update anime");

        let old_term = search(&pool, "旧标题", 20).await.expect("old term");
        assert!(!has(&old_term, "anime", anime_id));

        let new_term = search(&pool, "新标题", 20).await.expect("new term");
        assert!(has(&new_term, "anime", anime_id));

        sqlx::query("DELETE FROM anime WHERE id = ?")
            .bind(anime_id)
            .execute(&pool)
            .await
            .expect("delete anime");

        let deleted = search(&pool, "新标题", 20).await.expect("after delete");
        assert!(!has(&deleted, "anime", anime_id));
    }

    #[tokio::test]
    async fn books_notes_and_annotations_are_searchable() {
        let (_dir, pool) = migrated_pool().await;
        let book_id = insert_book(&pool, "深入理解计算机系统", "Randal E. Bryant").await;
        let note_id = insert_note(&pool, "读书笔记", "内存管理机制").await;
        let annotation_id = insert_annotation(&pool, book_id, "虚拟内存", "页表").await;

        let book_hit = search(&pool, "计算机系统", 20).await.expect("book search");
        assert!(has(&book_hit, "book", book_id));
        let book_result = book_hit
            .iter()
            .find(|result| result.entity_type == "book" && result.id == book_id)
            .expect("book result");
        assert_eq!(book_result.subtitle, "Randal E. Bryant");

        let note_hit = search(&pool, "内存管理", 20).await.expect("note search");
        assert!(has(&note_hit, "note", note_id));

        let annotation_hit = search(&pool, "虚拟内存", 20)
            .await
            .expect("annotation search");
        assert!(has(&annotation_hit, "annotation", annotation_id));
        let annotation_result = annotation_hit
            .iter()
            .find(|result| result.entity_type == "annotation" && result.id == annotation_id)
            .expect("annotation result");
        assert_eq!(annotation_result.parent_id, Some(book_id));
        assert_eq!(annotation_result.subtitle, "深入理解计算机系统");

        let author_hit = search(&pool, "randal", 20).await.expect("author search");
        assert!(has(&author_hit, "book", book_id));
    }

    #[tokio::test]
    async fn short_queries_escape_like_wildcards() {
        let (_dir, pool) = migrated_pool().await;
        let percent_id = insert_anime(&pool, "100%完成").await;
        insert_anime(&pool, "普通标题").await;

        let percent = search(&pool, "%", 20).await.expect("percent query");
        assert!(has(&percent, "anime", percent_id));
        assert_eq!(percent.len(), 1);

        let underscore = search(&pool, "_", 20).await.expect("underscore query");
        assert!(underscore.is_empty());
    }

    #[tokio::test]
    async fn query_length_is_validated() {
        let (_dir, pool) = migrated_pool().await;
        let long_query = "字".repeat(MAX_QUERY_CHARS + 1);
        let result = search(&pool, &long_query, 20).await;
        assert!(matches!(result, Err(AppError::Validation(_))));
    }

    #[tokio::test]
    async fn fts_integrity_is_valid_after_mutations() {
        let (_dir, pool) = migrated_pool().await;
        let anime_id = insert_anime(&pool, "完整性测试").await;
        sqlx::query("UPDATE anime SET title = '完整性更新', updated_at = 2 WHERE id = ?")
            .bind(anime_id)
            .execute(&pool)
            .await
            .expect("update anime");
        sqlx::query("DELETE FROM anime WHERE id = ?")
            .bind(anime_id)
            .execute(&pool)
            .await
            .expect("delete anime");

        for statement in [
            "INSERT INTO anime_fts (anime_fts) VALUES ('integrity-check')",
            "INSERT INTO books_fts (books_fts) VALUES ('integrity-check')",
            "INSERT INTO notes_fts (notes_fts) VALUES ('integrity-check')",
            "INSERT INTO annotations_fts (annotations_fts) VALUES ('integrity-check')",
        ] {
            sqlx::query(statement)
                .execute(&pool)
                .await
                .expect("fts integrity check");
        }
    }
}
