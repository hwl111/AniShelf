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
