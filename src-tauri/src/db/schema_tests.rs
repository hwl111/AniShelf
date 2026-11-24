use sqlx::SqlitePool;
use tempfile::{TempDir, tempdir};

use super::{connect, migrate};

async fn migrated_pool() -> (TempDir, SqlitePool) {
    let dir = tempdir().expect("tempdir");
    let database_file = dir.path().join("schema-test.sqlite");
    let backups_dir = dir.path().join("backups");
    let pool = connect(&database_file).await.expect("connect");
    migrate::run(&pool, &backups_dir).await.expect("migrate");
    (dir, pool)
}

async fn insert_anime(pool: &SqlitePool, title: &str) -> i64 {
    let result = sqlx::query("INSERT INTO anime (title, created_at, updated_at) VALUES (?, 1, 1)")
        .bind(title)
        .execute(pool)
        .await
        .expect("insert anime");
    result.last_insert_rowid()
}

async fn insert_episode(pool: &SqlitePool, anime_id: i64, episode_number: i64) -> i64 {
    let result = sqlx::query(
        "INSERT INTO anime_episodes (anime_id, episode_number, created_at, updated_at)
         VALUES (?, ?, 1, 1)",
    )
    .bind(anime_id)
    .bind(episode_number)
    .execute(pool)
    .await
    .expect("insert episode");
    result.last_insert_rowid()
}

async fn insert_watch_record(
    pool: &SqlitePool,
    anime_id: i64,
    episode_id: i64,
    episode_number: i64,
) -> i64 {
    let result = sqlx::query(
        "INSERT INTO watch_records (anime_id, episode_id, episode_number, watched_at, created_at)
         VALUES (?, ?, ?, 1, 1)",
    )
    .bind(anime_id)
    .bind(episode_id)
    .bind(episode_number)
    .execute(pool)
    .await
    .expect("insert watch record");
    result.last_insert_rowid()
}

async fn insert_book(pool: &SqlitePool, normalized_path: &str) -> i64 {
    let result = sqlx::query(
        "INSERT INTO books
         (title, format, file_path, normalized_path, file_size, imported_at, created_at, updated_at)
         VALUES ('测试书', 'epub', ?, ?, 10, 1, 1, 1)",
    )
    .bind(normalized_path)
    .bind(normalized_path)
    .execute(pool)
    .await
    .expect("insert book");
    result.last_insert_rowid()
}

async fn count(pool: &SqlitePool, table: &str) -> i64 {
    let sql = format!("SELECT COUNT(*) FROM {table}");
    sqlx::query_scalar(sqlx::AssertSqlSafe(sql))
        .fetch_one(pool)
        .await
        .expect("count")
}

#[tokio::test]
async fn deleting_anime_cascades_episodes_and_watch_records() {
    let (_dir, pool) = migrated_pool().await;
    let anime_id = insert_anime(&pool, "葬送的芙莉莲").await;
    let episode_id = insert_episode(&pool, anime_id, 1).await;
    insert_watch_record(&pool, anime_id, episode_id, 1).await;

    sqlx::query("DELETE FROM anime WHERE id = ?")
        .bind(anime_id)
        .execute(&pool)
        .await
        .expect("delete anime");

    assert_eq!(count(&pool, "anime_episodes").await, 0);
    assert_eq!(count(&pool, "watch_records").await, 0);
}

#[tokio::test]
async fn deleting_episode_keeps_watch_record_snapshot() {
    let (_dir, pool) = migrated_pool().await;
    let anime_id = insert_anime(&pool, "测试番剧").await;
    let episode_id = insert_episode(&pool, anime_id, 3).await;
    let watch_record_id = insert_watch_record(&pool, anime_id, episode_id, 3).await;

    sqlx::query("DELETE FROM anime_episodes WHERE id = ?")
        .bind(episode_id)
        .execute(&pool)
        .await
        .expect("delete episode");

    let (episode_id, episode_number): (Option<i64>, i64) =
        sqlx::query_as("SELECT episode_id, episode_number FROM watch_records WHERE id = ?")
            .bind(watch_record_id)
            .fetch_one(&pool)
            .await
            .expect("watch record");

    assert!(episode_id.is_none());
    assert_eq!(episode_number, 3);
}

#[tokio::test]
async fn deleting_book_cascades_reading_data() {
    let (_dir, pool) = migrated_pool().await;
    let book_id = insert_book(&pool, "/library/test.epub").await;

    sqlx::query(
        "INSERT INTO reading_progress (book_id, location_type, location_json, percent, updated_at)
         VALUES (?, 'epub_cfi', '{\"cfi\":\"/6/2\"}', 0.5, 1)",
    )
    .bind(book_id)
    .execute(&pool)
    .await
    .expect("insert progress");

    sqlx::query(
        "INSERT INTO annotations
         (book_id, kind, selected_text, location_type, location_json, created_at, updated_at)
         VALUES (?, 'highlight', '摘录', 'epub_cfi', '{}', 1, 1)",
    )
    .bind(book_id)
    .execute(&pool)
    .await
    .expect("insert annotation");

    sqlx::query("DELETE FROM books WHERE id = ?")
        .bind(book_id)
        .execute(&pool)
        .await
        .expect("delete book");

    assert_eq!(count(&pool, "reading_progress").await, 0);
    assert_eq!(count(&pool, "annotations").await, 0);
}

#[tokio::test]
async fn duplicate_episode_and_book_path_are_rejected() {
    let (_dir, pool) = migrated_pool().await;
    let anime_id = insert_anime(&pool, "测试番剧").await;
    insert_episode(&pool, anime_id, 1).await;

    let duplicate_episode = sqlx::query(
        "INSERT INTO anime_episodes (anime_id, episode_number, created_at, updated_at)
         VALUES (?, 1, 1, 1)",
    )
    .bind(anime_id)
    .execute(&pool)
    .await;
    assert!(duplicate_episode.is_err());

    insert_book(&pool, "/library/duplicate.epub").await;
    let duplicate_book = sqlx::query(
        "INSERT INTO books
         (title, format, file_path, normalized_path, file_size, imported_at, created_at, updated_at)
         VALUES ('重复', 'pdf', '/library/duplicate.epub', '/library/duplicate.epub', 10, 1, 1, 1)",
    )
    .execute(&pool)
    .await;
    assert!(duplicate_book.is_err());
}

#[tokio::test]
async fn check_constraints_reject_invalid_values() {
    let (_dir, pool) = migrated_pool().await;

    let invalid_status = sqlx::query(
        "INSERT INTO anime (title, watch_status, created_at, updated_at)
         VALUES ('测试', 'unknown', 1, 1)",
    )
    .execute(&pool)
    .await;
    assert!(invalid_status.is_err());

    let invalid_rating = sqlx::query(
        "INSERT INTO anime (title, rating, created_at, updated_at) VALUES ('测试', 11, 1, 1)",
    )
    .execute(&pool)
    .await;
    assert!(invalid_rating.is_err());

    let invalid_format = sqlx::query(
        "INSERT INTO books
         (title, format, file_path, normalized_path, file_size, imported_at, created_at, updated_at)
         VALUES ('测试', 'docx', '/library/a.docx', '/library/a.docx', 10, 1, 1, 1)",
    )
    .execute(&pool)
    .await;
    assert!(invalid_format.is_err());

    let book_id = insert_book(&pool, "/library/percent.epub").await;
    let invalid_percent = sqlx::query(
        "INSERT INTO reading_progress (book_id, location_type, location_json, percent, updated_at)
         VALUES (?, 'epub_cfi', '{}', 1.5, 1)",
    )
    .bind(book_id)
    .execute(&pool)
    .await;
    assert!(invalid_percent.is_err());
}

#[tokio::test]
async fn domain_foreign_keys_reject_orphans() {
    let (_dir, pool) = migrated_pool().await;

    let orphan_episode = sqlx::query(
        "INSERT INTO anime_episodes (anime_id, episode_number, created_at, updated_at)
         VALUES (999, 1, 1, 1)",
    )
    .execute(&pool)
    .await;
    assert!(orphan_episode.is_err());

    let orphan_progress = sqlx::query(
        "INSERT INTO reading_progress (book_id, location_type, location_json, percent, updated_at)
         VALUES (999, 'pdf_page', '{}', 0.1, 1)",
    )
    .execute(&pool)
    .await;
    assert!(orphan_progress.is_err());
}

async fn insert_note(pool: &SqlitePool, title: &str) -> i64 {
    let result = sqlx::query(
        "INSERT INTO notes (title, content_md, created_at, updated_at) VALUES (?, '', 1, 1)",
    )
    .bind(title)
    .execute(pool)
    .await
    .expect("insert note");
    result.last_insert_rowid()
}

async fn insert_tag(pool: &SqlitePool, name: &str) -> i64 {
    let result = sqlx::query("INSERT INTO tags (name, created_at) VALUES (?, 1)")
        .bind(name)
        .execute(pool)
        .await
        .expect("insert tag");
    result.last_insert_rowid()
}

async fn insert_annotation(pool: &SqlitePool, book_id: i64) -> i64 {
    let result = sqlx::query(
        "INSERT INTO annotations
         (book_id, kind, selected_text, location_type, location_json, created_at, updated_at)
         VALUES (?, 'highlight', '摘录', 'pdf_page', '{}', 1, 1)",
    )
    .bind(book_id)
    .execute(pool)
    .await
    .expect("insert annotation");
    result.last_insert_rowid()
}

#[tokio::test]
async fn note_links_require_distinct_notes_and_cascade() {
    let (_dir, pool) = migrated_pool().await;
    let first = insert_note(&pool, "第一条").await;
    let second = insert_note(&pool, "第二条").await;

    let self_link = sqlx::query(
        "INSERT INTO note_links (source_note_id, target_note_id, created_at) VALUES (?, ?, 1)",
    )
    .bind(first)
    .bind(first)
    .execute(&pool)
    .await;
    assert!(self_link.is_err());

    sqlx::query(
        "INSERT INTO note_links (source_note_id, target_note_id, created_at) VALUES (?, ?, 1)",
    )
    .bind(first)
    .bind(second)
    .execute(&pool)
    .await
    .expect("insert note link");

    sqlx::query("DELETE FROM notes WHERE id = ?")
        .bind(first)
        .execute(&pool)
        .await
        .expect("delete source note");

    assert_eq!(count(&pool, "note_links").await, 0);
}

#[tokio::test]
async fn deleting_target_note_removes_incoming_links() {
    let (_dir, pool) = migrated_pool().await;
    let first = insert_note(&pool, "第一条").await;
    let second = insert_note(&pool, "第二条").await;

    sqlx::query(
        "INSERT INTO note_links (source_note_id, target_note_id, created_at) VALUES (?, ?, 1)",
    )
    .bind(first)
    .bind(second)
    .execute(&pool)
    .await
    .expect("insert note link");

    sqlx::query("DELETE FROM notes WHERE id = ?")
        .bind(second)
        .execute(&pool)
        .await
        .expect("delete target note");

    assert_eq!(count(&pool, "note_links").await, 0);
}

#[tokio::test]
async fn notes_reject_completely_empty_content() {
    let (_dir, pool) = migrated_pool().await;

    let empty_note = sqlx::query(
        "INSERT INTO notes (title, content_md, created_at, updated_at) VALUES ('', '', 1, 1)",
    )
    .execute(&pool)
    .await;
    assert!(empty_note.is_err());

    let body_only = sqlx::query(
        "INSERT INTO notes (title, content_md, created_at, updated_at) VALUES ('', '正文', 1, 1)",
    )
    .execute(&pool)
    .await;
    assert!(body_only.is_ok());
}

#[tokio::test]
async fn tag_names_are_case_insensitive_and_tags_cascade() {
    let (_dir, pool) = migrated_pool().await;
    let note_id = insert_note(&pool, "笔记").await;
    let tag_id = insert_tag(&pool, "Rust").await;

    let duplicate = sqlx::query("INSERT INTO tags (name, created_at) VALUES ('rust', 1)")
        .execute(&pool)
        .await;
    assert!(duplicate.is_err());

    sqlx::query(
        "INSERT INTO entity_tags (entity_type, entity_id, tag_id, created_at)
         VALUES ('note', ?, ?, 1)",
    )
    .bind(note_id)
    .bind(tag_id)
    .execute(&pool)
    .await
    .expect("insert entity tag");

    sqlx::query("DELETE FROM tags WHERE id = ?")
        .bind(tag_id)
        .execute(&pool)
        .await
        .expect("delete tag");

    assert_eq!(count(&pool, "entity_tags").await, 0);
}

#[tokio::test]
async fn annotation_notes_cascade_from_notes_and_annotations() {
    let (_dir, pool) = migrated_pool().await;
    let book_id = insert_book(&pool, "/library/annotation.epub").await;
    let annotation_id = insert_annotation(&pool, book_id).await;
    let note_id = insert_note(&pool, "批注笔记").await;

    sqlx::query(
        "INSERT INTO annotation_notes (annotation_id, note_id, created_at) VALUES (?, ?, 1)",
    )
    .bind(annotation_id)
    .bind(note_id)
    .execute(&pool)
    .await
    .expect("insert annotation note");

    sqlx::query("DELETE FROM notes WHERE id = ?")
        .bind(note_id)
        .execute(&pool)
        .await
        .expect("delete note");
    assert_eq!(count(&pool, "annotation_notes").await, 0);

    let second_note_id = insert_note(&pool, "第二条批注笔记").await;
    sqlx::query(
        "INSERT INTO annotation_notes (annotation_id, note_id, created_at) VALUES (?, ?, 1)",
    )
    .bind(annotation_id)
    .bind(second_note_id)
    .execute(&pool)
    .await
    .expect("insert annotation note again");

    sqlx::query("DELETE FROM annotations WHERE id = ?")
        .bind(annotation_id)
        .execute(&pool)
        .await
        .expect("delete annotation");
    assert_eq!(count(&pool, "annotation_notes").await, 0);
}

#[tokio::test]
async fn note_relations_are_unique_and_cascade() {
    let (_dir, pool) = migrated_pool().await;
    let note_id = insert_note(&pool, "关联笔记").await;

    sqlx::query(
        "INSERT INTO note_relations (note_id, target_type, target_id, created_at)
         VALUES (?, 'anime', 1, 1)",
    )
    .bind(note_id)
    .execute(&pool)
    .await
    .expect("insert relation");

    let duplicate = sqlx::query(
        "INSERT INTO note_relations (note_id, target_type, target_id, created_at)
         VALUES (?, 'anime', 1, 1)",
    )
    .bind(note_id)
    .execute(&pool)
    .await;
    assert!(duplicate.is_err());

    sqlx::query("DELETE FROM notes WHERE id = ?")
        .bind(note_id)
        .execute(&pool)
        .await
        .expect("delete note");

    assert_eq!(count(&pool, "note_relations").await, 0);
}

#[tokio::test]
async fn favorites_reject_duplicates() {
    let (_dir, pool) = migrated_pool().await;
    let anime_id = insert_anime(&pool, "收藏测试").await;

    sqlx::query(
        "INSERT INTO favorites (entity_type, entity_id, created_at) VALUES ('anime', ?, 1)",
    )
    .bind(anime_id)
    .execute(&pool)
    .await
    .expect("insert favorite");

    let duplicate = sqlx::query(
        "INSERT INTO favorites (entity_type, entity_id, created_at) VALUES ('anime', ?, 1)",
    )
    .bind(anime_id)
    .execute(&pool)
    .await;
    assert!(duplicate.is_err());
}

#[tokio::test]
async fn sync_log_survives_anime_deletion() {
    let (_dir, pool) = migrated_pool().await;
    let anime_id = insert_anime(&pool, "同步日志测试").await;

    sqlx::query(
        "INSERT INTO anime_metadata_sync_logs
         (anime_id, bangumi_subject_id, status, started_at)
         VALUES (?, 42, 'success', 1)",
    )
    .bind(anime_id)
    .execute(&pool)
    .await
    .expect("insert sync log");

    sqlx::query("DELETE FROM anime WHERE id = ?")
        .bind(anime_id)
        .execute(&pool)
        .await
        .expect("delete anime");

    let stored_anime_id: Option<i64> =
        sqlx::query_scalar("SELECT anime_id FROM anime_metadata_sync_logs LIMIT 1")
            .fetch_one(&pool)
            .await
            .expect("sync log");

    assert!(stored_anime_id.is_none());
}
